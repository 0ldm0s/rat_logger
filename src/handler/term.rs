//! 终端日志处理器 - 高性能异步架构

use std::io::{self, Write, BufWriter};
use std::any::Any;
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::producer_consumer::LogProcessor;
use crate::config::{Record, FormatConfig, ColorConfig, Level};

/// 终端输出配置
#[derive(Debug, Clone)]
pub struct TermConfig {
    /// 是否启用颜色输出
    pub enable_color: bool,
    /// 是否启用异步输出
    pub enable_async: bool,
    /// 批量大小
    pub batch_size: usize,
    /// 刷新间隔（毫秒）
    pub flush_interval_ms: u64,
    /// 格式配置
    pub format: Option<FormatConfig>,
    /// 颜色配置
    pub color: Option<ColorConfig>,
}

impl TermConfig {
    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), String> {
        // 验证颜色配置一致性
        if !self.enable_color && self.color.is_some() {
            return Err(format!("配置冲突: 颜色配置被提供但 enable_color 为 false。如果要启用颜色，请设置 enable_color = true；如果要禁用颜色，请移除 color 配置。"));
        }

        // 验证批量大小合理性
        if self.batch_size == 0 {
            return Err("配置错误: 批量大小不能为 0".to_string());
        }
        if self.batch_size > 1024 * 1024 {
            return Err("配置错误: 批量大小过大 (最大 1MB)".to_string());
        }

        // 验证刷新间隔
        if self.flush_interval_ms == 0 {
            return Err("配置错误: 刷新间隔不能为 0".to_string());
        }
        if self.flush_interval_ms > 60000 {
            return Err("配置错误: 刷新间隔过长 (最大 60秒)".to_string());
        }

        // 验证格式配置（如果提供）
        if let Some(format_config) = &self.format {
            if format_config.format_template.is_empty() {
                return Err("配置错误: 格式模板不能为空".to_string());
            }
            if format_config.timestamp_format.is_empty() {
                return Err("配置错误: 时间戳格式不能为空".to_string());
            }
        }

        Ok(())
    }
}

impl Default for TermConfig {
    fn default() -> Self {
        Self {
            enable_color: true,
            enable_async: true,
            batch_size: 8192,
            flush_interval_ms: 100,
            format: None,
            color: None,
        }
    }
}

/// 终端日志处理器 - 实现LogProcessor trait
pub struct TermProcessor {
    config: TermConfig,
    formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync>,
    buffer: Arc<Mutex<Vec<u8>>>,
    last_flush: Arc<Mutex<Instant>>,
    is_running: Arc<AtomicBool>,
    stdout: Arc<Mutex<BufWriter<io::Stdout>>>,
}

impl TermProcessor {
    /// 创建新的终端处理器
    pub fn new() -> Self {
        let config = TermConfig::default();
        Self::with_config(config)
    }

    /// 使用配置创建终端处理器
    pub fn with_config(config: TermConfig) -> Self {
        // 验证配置，如果失败则直接panic，让用户明确知道配置问题
        if let Err(e) = config.validate() {
            panic!("TermConfig 验证失败: {}\n请检查您的配置并修复上述问题后再重试。", e);
        }

        let formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync> = {
            // 检查是否启用颜色且有颜色配置
            let use_color = config.enable_color && config.color.is_some();

            match (&config.format, use_color) {
                (Some(format_config), true) => {
                    // 有格式配置且启用颜色
                    let format_config = format_config.clone();
                    let color_config = config.color.as_ref().unwrap().clone();
                    Box::new(move |buf, record| {
                        format_with_color(buf, record, &format_config, &color_config)
                    })
                }
                (Some(format_config), false) => {
                    // 有格式配置但不启用颜色
                    let format_config = format_config.clone();
                    Box::new(move |buf, record| {
                        format_with_config(buf, record, &format_config)
                    })
                }
                (None, true) => {
                    // 无格式配置但启用颜色
                    let default_format_config = FormatConfig::default();
                    let color_config = config.color.as_ref().unwrap().clone();
                    Box::new(move |buf, record| {
                        format_with_color(buf, record, &default_format_config, &color_config)
                    })
                }
                (None, false) => Box::new(default_format),
            }
        };

        let processor = Self {
            config,
            formatter,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(64 * 1024))),
            last_flush: Arc::new(Mutex::new(Instant::now())),
            is_running: Arc::new(AtomicBool::new(true)),
            stdout: Arc::new(Mutex::new(BufWriter::new(io::stdout()))),
        };

        // 如果启用异步输出，启动刷新线程
        if processor.config.enable_async {
            processor.start_flush_thread();
        }

        processor
    }

    /// 启动异步刷新线程
    fn start_flush_thread(&self) {
        let buffer = Arc::clone(&self.buffer);
        let last_flush = Arc::clone(&self.last_flush);
        let stdout = Arc::clone(&self.stdout);
        let is_running = Arc::clone(&self.is_running);
        let flush_interval = Duration::from_millis(self.config.flush_interval_ms);

        std::thread::spawn(move || {
            while is_running.load(Ordering::Relaxed) {
                std::thread::sleep(flush_interval);

                let should_flush = {
                    let last_flush_guard = last_flush.lock();
                    last_flush_guard.elapsed() >= flush_interval
                };

                if should_flush {
                    let mut buffer_guard = buffer.lock();
                    if !buffer_guard.is_empty() {
                        let mut stdout_guard = stdout.lock();
                        if let Err(e) = stdout_guard.write_all(buffer_guard.as_slice()) {
                            eprintln!("[term] 异步写入失败: {}", e);
                        }
                        if let Err(e) = stdout_guard.flush() {
                            eprintln!("[term] 异步刷新失败: {}", e);
                        }
                        buffer_guard.clear();
                    }
                    drop(buffer_guard);

                    let mut last_flush_guard = last_flush.lock();
                    *last_flush_guard = Instant::now();
                }
            }

            // 线程结束时，刷新剩余数据
            let mut buffer_guard = buffer.lock();
            if !buffer_guard.is_empty() {
                let mut stdout_guard = stdout.lock();
                if let Err(e) = stdout_guard.write_all(buffer_guard.as_slice()) {
                    eprintln!("[term] 最终写入失败: {}", e);
                }
                if let Err(e) = stdout_guard.flush() {
                    eprintln!("[term] 最终刷新失败: {}", e);
                }
                buffer_guard.clear();
            }
        });
    }

    /// 设置自定义格式化函数
    pub fn with_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync + 'static,
    {
        self.formatter = Box::new(formatter);
        self
    }

    /// 使用格式配置
    pub fn with_format(mut self, format_config: FormatConfig) -> Self {
        let format_config = format_config.clone();
        self.formatter = Box::new(move |buf, record| format_with_config(buf, record, &format_config));
        self
    }

    /// 使用格式配置和颜色配置
    pub fn with_format_and_color(mut self, format_config: FormatConfig, color_config: ColorConfig) -> Self {
        let format_config = format_config.clone();
        let color_config = color_config.clone();
        self.formatter = Box::new(move |buf, record| format_with_color(buf, record, &format_config, &color_config));
        self
    }

    /// 格式化日志记录
    fn format_record(&self, record: &Record) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        (self.formatter)(&mut buf, record)
            .map_err(|e| format!("格式化失败: {}", e))?;
        Ok(buf)
    }

    /// 写入到缓冲区
    fn write_to_buffer(&self, data: &[u8]) -> Result<(), String> {
        if self.config.enable_async {
            // 异步模式：写入缓冲区
            let mut buffer_guard = self.buffer.lock();
            buffer_guard.extend_from_slice(data);

            // 检查是否需要立即刷新
            if buffer_guard.len() >= self.config.batch_size {
                let mut stdout_guard = self.stdout.lock();
                stdout_guard.write_all(buffer_guard.as_slice())
                    .map_err(|e| format!("批量写入失败: {}", e))?;
                stdout_guard.flush()
                    .map_err(|e| format!("批量刷新失败: {}", e))?;
                buffer_guard.clear();
            }

            // 更新最后刷新时间
            let mut last_flush_guard = self.last_flush.lock();
            *last_flush_guard = Instant::now();
        } else {
            // 同步模式：直接写入
            let mut stdout_guard = self.stdout.lock();
            stdout_guard.write_all(data)
                .map_err(|e| format!("同步写入失败: {}", e))?;
            stdout_guard.flush()
                .map_err(|e| format!("同步刷新失败: {}", e))?;
        }

        Ok(())
    }
}

impl LogProcessor for TermProcessor {
    fn name(&self) -> &'static str {
        "term_processor"
    }

    fn process(&mut self, data: &[u8]) -> Result<(), String> {
        // 反序列化日志记录
        let record = bincode::decode_from_slice::<Record, _>(data, bincode::config::standard())
            .map_err(|e| format!("反序列化失败: {}", e))?.0;

        // 格式化日志记录
        let formatted_data = self.format_record(&record)?;

        // 写入到终端
        self.write_to_buffer(&formatted_data)
    }

    fn process_batch(&mut self, batch: &[Vec<u8>]) -> Result<(), String> {
        let mut all_data = Vec::new();

        // 批量反序列化和格式化
        for data in batch {
            let record = bincode::decode_from_slice::<Record, _>(data, bincode::config::standard())
                .map_err(|e| format!("批量反序列化失败: {}", e))?.0;

            let formatted_data = self.format_record(&record)?;
            all_data.extend_from_slice(&formatted_data);
        }

        // 批量写入
        self.write_to_buffer(&all_data)
    }

    fn flush(&mut self) -> Result<(), String> {
        if self.config.enable_async {
            // 异步模式：刷新缓冲区
            let mut buffer_guard = self.buffer.lock();
            if !buffer_guard.is_empty() {
                let mut stdout_guard = self.stdout.lock();
                stdout_guard.write_all(buffer_guard.as_slice())
                    .map_err(|e| format!("刷新写入失败: {}", e))?;
                stdout_guard.flush()
                    .map_err(|e| format!("刷新失败: {}", e))?;
                buffer_guard.clear();
            }

            // 更新最后刷新时间
            let mut last_flush_guard = self.last_flush.lock();
            *last_flush_guard = Instant::now();
        } else {
            // 同步模式：直接刷新
            let mut stdout_guard = self.stdout.lock();
            stdout_guard.flush()
                .map_err(|e| format!("同步刷新失败: {}", e))?;
        }

        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), String> {
        // 停止异步刷新线程
        self.is_running.store(false, Ordering::Relaxed);

        // 刷新所有剩余数据
        self.flush()
    }
}

impl Drop for TermProcessor {
    fn drop(&mut self) {
        // 清理时会自动调用cleanup
        let _ = self.cleanup();
    }
}

/// 默认格式化函数
pub fn default_format(buf: &mut dyn Write, record: &Record) -> io::Result<()> {
    use chrono::Local;

    let now = Local::now();
    let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");

    writeln!(
        buf,
        "{} [{}] {} {}:{} - {}",
        timestamp,
        record.metadata.level,
        record.metadata.target,
        record.file.as_deref().unwrap_or("unknown"),
        record.line.unwrap_or(0),
        record.args
    )
}

/// 格式化函数
pub fn format_with_config(buf: &mut dyn Write, record: &Record, format_config: &FormatConfig) -> io::Result<()> {
    use chrono::Local;

    let now = Local::now();
    let timestamp = now.format(&format_config.timestamp_format);

    // 获取级别显示文本
    let level_text = match record.metadata.level {
        Level::Error => &format_config.level_style.error,
        Level::Warn => &format_config.level_style.warn,
        Level::Info => &format_config.level_style.info,
        Level::Debug => &format_config.level_style.debug,
        Level::Trace => &format_config.level_style.trace,
    };

    // 使用格式模板
    let formatted = format_config.format_template
        .replace("{timestamp}", &timestamp.to_string())
        .replace("{level}", level_text)
        .replace("{target}", &record.metadata.target)
        .replace("{file}", record.file.as_deref().unwrap_or("unknown"))
        .replace("{line}", &record.line.unwrap_or(0).to_string())
        .replace("{message}", &record.args);

    writeln!(buf, "{}", formatted)
}

/// 带颜色的格式化函数
pub fn format_with_color(buf: &mut dyn Write, record: &Record, format_config: &FormatConfig, color_config: &ColorConfig) -> io::Result<()> {
    use chrono::Local;

    let now = Local::now();
    let timestamp = now.format(&format_config.timestamp_format);

    // 获取级别显示文本
    let level_text = match record.metadata.level {
        Level::Error => &format_config.level_style.error,
        Level::Warn => &format_config.level_style.warn,
        Level::Info => &format_config.level_style.info,
        Level::Debug => &format_config.level_style.debug,
        Level::Trace => &format_config.level_style.trace,
    };

    // 获取级别颜色
    let level_color = match record.metadata.level {
        Level::Error => &color_config.error,
        Level::Warn => &color_config.warn,
        Level::Info => &color_config.info,
        Level::Debug => &color_config.debug,
        Level::Trace => &color_config.trace,
    };

    // 重置颜色
    let reset_color = "\x1b[0m";

    // 使用格式模板并应用颜色
    let colored_timestamp = format!("{}{}{}", color_config.timestamp, timestamp, reset_color);
    let colored_level = format!("{}{}{}", level_color, level_text, reset_color);
    let colored_target = format!("{}{}{}", color_config.target, record.metadata.target, reset_color);
    let colored_file = format!("{}{}{}", color_config.file, record.file.as_deref().unwrap_or("unknown"), reset_color);
    let colored_line = format!("{}{}{}", color_config.file, record.line.unwrap_or(0), reset_color);
    let colored_message = format!("{}{}{}", color_config.message, record.args, reset_color);

    // 使用格式模板进行格式化
    let mut formatted = format_config.format_template
        .replace("{timestamp}", &colored_timestamp)
        .replace("{level}", &colored_level)
        .replace("{target}", &colored_target)
        .replace("{file}", &colored_file)
        .replace("{line}", &colored_line)
        .replace("{message}", &colored_message);

    // 处理格式模板中可能包含的冒号和分隔符
    formatted = formatted.replace("}:", format!("{}:{}", reset_color, color_config.file).as_str());

    writeln!(buf, "{}", formatted)
}