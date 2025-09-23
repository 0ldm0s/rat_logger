//! 日志核心模块 - 完全异步的生产者消费者架构

use std::sync::Arc;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use crossbeam_channel::Sender;

use crate::config::{LevelFilter, Record};
use crate::producer_consumer::{ProcessorManager, BatchConfig};

/// 全局日志器实例
pub static LOGGER: Lazy<Mutex<Option<Arc<dyn Logger>>>> = Lazy::new(|| Mutex::new(None));

/// 全局日志器锁（用于开发模式重新初始化）
static LOGGER_LOCK: std::sync::RwLock<()> = std::sync::RwLock::new(());

/// 全局最大日志级别
static MAX_LEVEL: AtomicUsize = AtomicUsize::new(LevelFilter::Info as usize);

/// 处理器类型名称常量
pub mod processor_types {
    /// 终端处理器类型名称
    pub const TERMINAL: &str = "term_processor";
    /// 文件处理器类型名称
    pub const FILE: &str = "file_processor";
    /// UDP处理器类型名称
    pub const UDP: &str = "udp_processor";
}

/// 统一的日志命令枚举
#[derive(Debug, Clone)]
pub enum LogCommand {
    /// 写入日志数据
    Write(Vec<u8>),
    /// 强制写入日志数据（忽略批量限制）
    WriteForce(Vec<u8>),
    /// 文件轮转
    Rotate,
    /// 文件压缩
    Compress(std::path::PathBuf),
    /// 强制刷新
    Flush,
    /// 停止工作线程
    Shutdown(&'static str),
    /// 健康检查（用于初始化时验证工作线程状态）
    HealthCheck(Sender<bool>),
}

/// 日志器 trait - 极简接口
pub trait Logger: Send + Sync {
    fn log(&self, record: &Record);
    fn flush(&self);
    fn set_level(&self, level: LevelFilter);
    fn level(&self) -> LevelFilter;

    /// 临时强制刷新 - 立即输出所有缓冲的日志，无视批量配置
    fn force_flush(&self);

    /// 紧急日志 - 无视所有限制立即输出，适用于启动日志和关键错误
    fn emergency_log(&self, record: &Record);
}

/// 日志核心实现 - 极简设计
#[derive(Clone)]
pub struct LoggerCore {
    level: LevelFilter,
    processor_manager: Arc<ProcessorManager>,
    dev_mode: bool, // 开发模式：同步等待日志处理完成
    /// 需要等待的处理器类型集合
    expected_processor_types: Arc<std::sync::Mutex<std::collections::HashSet<String>>>,
}

impl LoggerCore {
    /// 创建新的日志核心
    pub fn new(level: LevelFilter, processor_manager: ProcessorManager, batch_config: BatchConfig, dev_mode: bool) -> Self {
        Self {
            level,
            processor_manager: Arc::new(processor_manager),
            dev_mode,
            expected_processor_types: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
        }
    }

    /// 创建新的日志核心（带预期的处理器类型）
    pub fn with_expected_types(
        level: LevelFilter,
        processor_manager: ProcessorManager,
        batch_config: BatchConfig,
        dev_mode: bool,
        expected_types: std::collections::HashSet<String>
    ) -> Self {
        Self {
            level,
            processor_manager: Arc::new(processor_manager),
            dev_mode,
            expected_processor_types: Arc::new(std::sync::Mutex::new(expected_types)),
        }
    }

    /// 获取当前日志级别
    pub fn level(&self) -> LevelFilter {
        self.level
    }

    /// 检查是否应该记录该级别的日志
    pub fn should_log(&self, level: &crate::config::Level) -> bool {
        (level.to_level_filter() as u8) <= (self.level as u8)
    }

    /// 获取ProcessorManager的引用
    pub fn processor_manager(&self) -> &Arc<ProcessorManager> {
        &self.processor_manager
    }

    /// 智能等待所有工作线程启动就绪
    pub fn wait_for_workers_ready(&self, timeout_ms: u64) -> Result<(), String> {
        // 获取预期的处理器类型
        let expected_types: Vec<String> = {
            let guard = self.expected_processor_types.lock().unwrap();
            guard.iter().cloned().collect()
        };

        if expected_types.is_empty() {
            // 如果没有指定预期的处理器类型，直接检查所有处理器
            self.processor_manager.smart_health_check(timeout_ms)?;
        } else {
            // 检查指定的处理器类型
            self.processor_manager.check_specific_types(&expected_types, timeout_ms)?;
        }

        Ok(())
    }

    /// 添加预期的处理器类型
    pub fn add_expected_type(&self, processor_type: String) {
        let mut guard = self.expected_processor_types.lock().unwrap();
        guard.insert(processor_type);
    }
}

impl Logger for LoggerCore {
    fn log(&self, record: &Record) {
        if self.should_log(&record.metadata.level) {
            // Error级别日志自动使用紧急模式
            if record.metadata.level == crate::config::Level::Error {
                self.emergency_log(record);
                return;
            }

            // 序列化日志数据并发送给所有处理器
            if let Ok(data) = bincode::encode_to_vec(record, bincode::config::standard()) {
                let _ = self.processor_manager.broadcast_write(data);

                // 开发模式：同步等待日志处理完成
                if self.dev_mode {
                    self.flush();
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        }
    }

    fn flush(&self) {
        // 广播刷新命令给所有处理器
        let _ = self.processor_manager.broadcast_flush();
    }

    fn set_level(&self, level: LevelFilter) {
        // 更新全局最大级别
        MAX_LEVEL.store(level as usize, Ordering::Relaxed);
    }

    fn level(&self) -> LevelFilter {
        self.level
    }

    fn force_flush(&self) {
        // 强制刷新所有处理器，无视批量配置
        let _ = self.processor_manager.broadcast_flush();
        // 给处理器一些时间来完成刷新
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    fn emergency_log(&self, record: &Record) {
        // 紧急日志：直接发送并立即刷新，无视级别检查和批量配置
        if let Ok(data) = bincode::encode_to_vec(record, bincode::config::standard()) {
            // 直接发送给所有处理器，使用强制写入命令（忽略批量限制）
            let _ = self.processor_manager.broadcast_write_force(data);
        }
    }
}

/// 日志构建器 - 极简设计
pub struct LoggerBuilder {
    level: LevelFilter,
    processor_manager: ProcessorManager,
    batch_config: Option<BatchConfig>,
    dev_mode: bool, // 开发模式：同步等待日志处理完成
    /// 是否启用异步模式
    enable_async: bool,
    /// 预期的处理器类型集合
    expected_processor_types: std::collections::HashSet<String>,
}

impl LoggerBuilder {
    /// 创建新的日志构建器
    pub fn new() -> Self {
        Self {
            level: LevelFilter::Info,
            processor_manager: ProcessorManager::new(),
            batch_config: None,
            dev_mode: false,
            enable_async: false,
            expected_processor_types: std::collections::HashSet::new(),
        }
    }

    /// 设置是否启用异步模式
    pub fn with_async_mode(mut self, enable_async: bool) -> Self {
        self.enable_async = enable_async;
        self
    }

    /// 设置日志级别
    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    /// 设置批量配置
    pub fn with_batch_config(mut self, config: BatchConfig) -> Self {
        self.batch_config = Some(config);
        self
    }

    /// 启用开发模式（同步等待日志处理完成）
    pub fn with_dev_mode(mut self, enabled: bool) -> Self {
        self.dev_mode = enabled;
        self
    }

    
    /// 添加带配置的终端处理器
    pub fn add_terminal_with_config(mut self, config: crate::handler::term::TermConfig) -> Self {
        use crate::handler::term::TermProcessor;
        let processor = TermProcessor::with_config(config);

        // 如果还没有设置batch_config，使用默认的同步配置
        let batch_config = self.batch_config.clone().unwrap_or_else(|| {
            if self.enable_async {
                panic!("配置错误: 异步模式必须先配置BatchConfig，请使用with_batch_config()方法设置。");
            } else {
                BatchConfig {
                    batch_size: 1,
                    batch_interval_ms: 1,
                    buffer_size: 1024,
                }
            }
        });

        if let Err(e) = self.processor_manager.add_processor(processor, batch_config) {
            eprintln!("添加终端处理器失败: {}", e);
        } else {
            self.expected_processor_types.insert(processor_types::TERMINAL.to_string());
        }
        self
    }

    /// 添加文件处理器
    pub fn add_file(mut self, config: crate::config::FileConfig) -> Self {
        use crate::handler::file::FileProcessor;
        let processor = FileProcessor::new(config);

        // 如果还没有设置batch_config，使用默认的同步配置
        let batch_config = self.batch_config.clone().unwrap_or_else(|| {
            if self.enable_async {
                panic!("配置错误: 异步模式必须先配置BatchConfig，请使用with_batch_config()方法设置。");
            } else {
                BatchConfig {
                    batch_size: 1,
                    batch_interval_ms: 1,
                    buffer_size: 1024,
                }
            }
        });

        if let Err(e) = self.processor_manager.add_processor(processor, batch_config) {
            eprintln!("添加文件处理器失败: {}", e);
        } else {
            self.expected_processor_types.insert(processor_types::FILE.to_string());
        }
        self
    }

    /// 添加UDP处理器
    pub fn add_udp(mut self, config: crate::config::NetworkConfig) -> Self {
        use crate::handler::udp::UdpProcessor;
        let processor = UdpProcessor::new(config);

        // 如果还没有设置batch_config，使用默认的同步配置
        let batch_config = self.batch_config.clone().unwrap_or_else(|| {
            if self.enable_async {
                panic!("配置错误: 异步模式必须先配置BatchConfig，请使用with_batch_config()方法设置。");
            } else {
                BatchConfig {
                    batch_size: 1,
                    batch_interval_ms: 1,
                    buffer_size: 1024,
                }
            }
        });

        if let Err(e) = self.processor_manager.add_processor(processor, batch_config) {
            eprintln!("添加UDP处理器失败: {}", e);
        } else {
            self.expected_processor_types.insert(processor_types::UDP.to_string());
        }
        self
    }

    /// 构建日志器
    pub fn build(self) -> LoggerCore {
        // 验证批量配置
        let batch_config = match self.batch_config {
            Some(config) => config,
            None => {
                if self.enable_async {
                    panic!("配置错误: 异步模式必须配置BatchConfig，请使用with_batch_config()方法设置。");
                } else {
                    // 同步模式使用默认配置
                    BatchConfig {
                        batch_size: 1,
                        batch_interval_ms: 1,
                        buffer_size: 1024,
                    }
                }
            }
        };

        // 验证批量配置
        if let Err(e) = batch_config.validate() {
            panic!("LoggerBuilder 批量配置验证失败: {}\n请检查您的批量配置并修复上述问题后再重试。", e);
        }

        // 验证是否有处理器
        if self.processor_manager.is_empty() {
            panic!("配置错误: 必须至少添加一个处理器（终端、文件或UDP）");
        }

        LoggerCore::with_expected_types(
            self.level,
            self.processor_manager,
            batch_config,
            self.dev_mode,
            self.expected_processor_types
        )
    }

    /// 构建并初始化全局日志器
    pub fn init_global_logger(self) -> Result<(), SetLoggerError> {
        let level = self.level;
        let is_dev_mode = self.dev_mode;
        let logger = Arc::new(self.build());

        // 开发模式下允许重新初始化
        if is_dev_mode && cfg!(debug_assertions) {
            set_logger_dev(logger)?;
        } else {
            // 生产模式：允许重新初始化以应对程序多次运行的情况
            let _lock = LOGGER_LOCK.write().unwrap();
            let mut guard = LOGGER.lock().unwrap();

            // 检查是否已经初始化过
            if guard.is_some() {
                // 如果已经初始化过，直接使用现有的日志器
                // 注意：这里我们放弃新创建的logger，保持现有配置
                eprintln!("⚠️  警告：全局日志器已经初始化，跳过重复初始化");
                eprintln!("⚠️  这将导致新创建的LoggerCore被丢弃，ProcessorWorker的Drop trait会被调用！");
            } else {
                // 如果没有初始化过，正常设置
                *guard = Some(logger);
            }

            // 智能等待所有工作线程启动就绪
            // 替换原来的固定延时，提供更可靠的等待机制
            if let Some(logger) = guard.as_ref() {
                // 使用更安全的方式检查类型
                let logger_ptr = logger.as_ref() as *const dyn Logger;
                let logger_core_ptr = logger_ptr as *const LoggerCore;

                // 检查是否确实是LoggerCore类型
                if !logger_ptr.is_null() && !logger_core_ptr.is_null() {
                    // 安全转换，因为我们已经检查了类型
                    let logger_core = unsafe { &*logger_core_ptr };

                    // 智能等待所有工作线程启动就绪，超时时间5秒
                    if let Err(e) = logger_core.wait_for_workers_ready(5000) {
                        panic!("❌ 日志器初始化失败：工作线程健康检查失败: {}\n请检查处理器配置或系统资源", e);
                    }
                }
            }
        }

        set_max_level(level);
        Ok(())
    }

    /// 构建并初始化全局日志器（已弃用，请使用init_global_logger）
    #[deprecated(since = "0.2.7", note = "请使用init_global_logger方法")]
    pub fn init(self) -> Result<(), SetLoggerError> {
        self.init_global_logger()
    }
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 设置全局日志器
pub fn set_logger(logger: Arc<dyn Logger>) -> Result<(), SetLoggerError> {
    let mut guard = LOGGER.lock().unwrap();
    if guard.is_some() {
        return Err(SetLoggerError(()));
    }
    *guard = Some(logger);
    Ok(())
}

/// 开发模式友好的日志器设置（允许重新初始化）
pub fn set_logger_dev(logger: Arc<dyn Logger>) -> Result<(), SetLoggerError> {
    // 开发模式下：使用写锁来保证安全
    let _lock = LOGGER_LOCK.write().unwrap();

    let mut guard = LOGGER.lock().unwrap();
    if guard.is_some() {
        eprintln!("⚠️  警告：重新初始化全局日志器（开发模式）");
        eprintln!("⚠️  此功能仅供开发使用，生产环境请确保只初始化一次日志器");

        // 先清理旧的日志器，确保资源正确释放
        if let Some(old_logger) = guard.take() {
            drop(old_logger);
            // 给旧日志器一些时间来清理资源
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
    *guard = Some(logger);
    Ok(())
}

/// 空日志器实现（用于开发模式重新初始化）
struct NullLogger;
impl Logger for NullLogger {
    fn log(&self, _record: &Record) {}
    fn flush(&self) {}
    fn set_level(&self, _level: LevelFilter) {}
    fn level(&self) -> LevelFilter { LevelFilter::Off }
    fn force_flush(&self) {}
    fn emergency_log(&self, _record: &Record) {}
}

/// 设置全局最大日志级别
pub fn set_max_level(level: LevelFilter) {
    MAX_LEVEL.store(level as usize, Ordering::Relaxed);
}

/// 获取全局最大日志级别
pub fn max_level() -> LevelFilter {
    match MAX_LEVEL.load(Ordering::Relaxed) {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}

/// 日志器设置错误
#[derive(Debug)]
pub struct SetLoggerError(());

impl std::fmt::Display for SetLoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("failed to set logger")
    }
}

impl std::error::Error for SetLoggerError {}