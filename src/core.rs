//! 日志核心模块 - 完全异步的生产者消费者架构

use std::sync::Arc;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::config::{LevelFilter, Record};
use crate::producer_consumer::{ProcessorManager, BatchConfig};

/// 全局日志器实例
pub static LOGGER: Lazy<Mutex<Option<Arc<dyn Logger>>>> = Lazy::new(|| Mutex::new(None));

/// 全局日志器锁（用于开发模式重新初始化）
static LOGGER_LOCK: std::sync::RwLock<()> = std::sync::RwLock::new(());

/// 全局最大日志级别
static MAX_LEVEL: AtomicUsize = AtomicUsize::new(LevelFilter::Info as usize);

/// 统一的日志命令枚举
#[derive(Debug, Clone)]
pub enum LogCommand {
    /// 写入日志数据
    Write(Vec<u8>),
    /// 文件轮转
    Rotate,
    /// 文件压缩
    Compress(std::path::PathBuf),
    /// 强制刷新
    Flush,
    /// 停止工作线程
    Shutdown,
}

/// 日志器 trait - 极简接口
pub trait Logger: Send + Sync {
    fn log(&self, record: &Record);
    fn flush(&self);
    fn set_level(&self, level: LevelFilter);
    fn level(&self) -> LevelFilter;
}

/// 日志核心实现 - 极简设计
#[derive(Clone)]
pub struct LoggerCore {
    level: LevelFilter,
    processor_manager: Arc<ProcessorManager>,
    dev_mode: bool, // 开发模式：同步等待日志处理完成
}

impl LoggerCore {
    /// 创建新的日志核心
    pub fn new(level: LevelFilter, processor_manager: ProcessorManager, dev_mode: bool) -> Self {
        Self {
            level,
            processor_manager: Arc::new(processor_manager),
            dev_mode,
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
}

impl Logger for LoggerCore {
    fn log(&self, record: &Record) {
        if self.should_log(&record.metadata.level) {
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
}

/// 日志构建器 - 极简设计
pub struct LoggerBuilder {
    level: LevelFilter,
    processor_manager: ProcessorManager,
    batch_config: BatchConfig,
    dev_mode: bool, // 开发模式：同步等待日志处理完成
}

impl LoggerBuilder {
    /// 创建新的日志构建器
    pub fn new() -> Self {
        Self {
            level: LevelFilter::Info,
            processor_manager: ProcessorManager::new(),
            batch_config: BatchConfig::default(),
            dev_mode: false,
        }
    }

    /// 设置日志级别
    pub fn with_level(mut self, level: LevelFilter) -> Self {
        self.level = level;
        self
    }

    /// 设置批量配置
    pub fn with_batch_config(mut self, config: BatchConfig) -> Self {
        self.batch_config = config;
        self
    }

    /// 启用开发模式（同步等待日志处理完成）
    pub fn with_dev_mode(mut self, enabled: bool) -> Self {
        self.dev_mode = enabled;
        self
    }

    /// 添加终端处理器
    pub fn add_terminal(mut self) -> Self {
        self.add_terminal_with_config(crate::handler::term::TermConfig::default())
    }

    /// 添加带配置的终端处理器
    pub fn add_terminal_with_config(mut self, config: crate::handler::term::TermConfig) -> Self {
        use crate::handler::term::TermProcessor;
        let processor = TermProcessor::with_config(config);
        if let Err(e) = self.processor_manager.add_processor(processor, self.batch_config.clone()) {
            eprintln!("添加终端处理器失败: {}", e);
        }
        self
    }

    /// 添加文件处理器
    pub fn add_file(mut self, config: crate::config::FileConfig) -> Self {
        use crate::handler::file::FileProcessor;
        let processor = FileProcessor::new(config);
        if let Err(e) = self.processor_manager.add_processor(processor, self.batch_config.clone()) {
            eprintln!("添加文件处理器失败: {}", e);
        }
        self
    }

    /// 添加UDP处理器
    pub fn add_udp(mut self, config: crate::config::NetworkConfig) -> Self {
        use crate::handler::udp::UdpProcessor;
        let processor = UdpProcessor::new(config);
        if let Err(e) = self.processor_manager.add_processor(processor, self.batch_config.clone()) {
            eprintln!("添加UDP处理器失败: {}", e);
        }
        self
    }

    /// 构建日志器
    pub fn build(self) -> LoggerCore {
        // 验证批量配置
        if let Err(e) = self.batch_config.validate() {
            panic!("LoggerBuilder 批量配置验证失败: {}\n请检查您的批量配置并修复上述问题后再重试。", e);
        }

        // 验证是否有处理器
        if self.processor_manager.is_empty() {
            panic!("配置错误: 必须至少添加一个处理器（终端、文件或UDP）");
        }

        LoggerCore::new(self.level, self.processor_manager, self.dev_mode)
    }

    /// 构建并初始化全局日志器
    pub fn init(self) -> Result<(), SetLoggerError> {
        let level = self.level;
        let is_dev_mode = self.dev_mode;
        let logger = Arc::new(self.build());

        // 开发模式下允许重新初始化
        if is_dev_mode || cfg!(debug_assertions) {
            set_logger_dev(logger)?;
        } else {
            set_logger(logger)?;
        }

        set_max_level(level);
        Ok(())
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