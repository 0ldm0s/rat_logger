//! 终端日志处理器

use std::io::{self, Write, BufWriter};
use std::any::Any;
use std::time::Instant;
use std::sync::Arc;
use parking_lot::Mutex;
use crossbeam_channel::{Sender, Receiver, unbounded};
use std::thread;

use crate::handler::{LogHandler, HandlerType};
use crate::config::{Record, FormatConfig, ColorConfig, Level};

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

    writeln!(
        buf,
        "{}{}{} {}{}{} {}{}{} {}{}{}:{} - {}",
        color_config.timestamp, timestamp, reset_color,
        level_color, level_text, reset_color,
        color_config.target, record.metadata.target, reset_color,
        color_config.file, record.file.as_deref().unwrap_or("unknown"), reset_color,
        record.line.unwrap_or(0),
        record.args
    )
}

/// 终端指令枚举 - 生产者消费者模式
enum TermCommand {
    /// 写入日志数据
    Write(Vec<u8>),
    /// 强制刷新到终端
    Flush,
    /// 停止工作线程
    Shutdown,
}

/// 终端日志处理器
pub struct TermHandler {
    config: TermHandlerConfig,
    command_sender: Sender<TermCommand>,
    worker_thread: Option<thread::JoinHandle<()>>,
}

/// 终端处理器配置
#[derive(Clone)]
struct TermHandlerConfig {
    formatter: Arc<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync>,
    batch_size: usize,
    flush_interval_ms: u64,
}

impl TermHandler {
    /// 创建新的终端处理器（默认无主题）- 使用异步批量写入
    pub fn new() -> Self {
        Self::with_config(TermHandlerConfig {
            formatter: Arc::new(default_format),
            batch_size: 8192, // 8KB批量写入
            flush_interval_ms: 100, // 100ms刷新间隔
        })
    }

    /// 使用自定义格式化函数
    pub fn with_formatter<F>(formatter: F) -> Self
    where
        F: Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync + 'static,
    {
        Self::with_config(TermHandlerConfig {
            formatter: Arc::new(formatter),
            batch_size: 8192,
            flush_interval_ms: 100,
        })
    }

    /// 使用格式配置
    pub fn with_format(format_config: FormatConfig) -> Self {
        let format_config = format_config.clone();
        Self::with_config(TermHandlerConfig {
            formatter: Arc::new(move |buf, record| format_with_config(buf, record, &format_config)),
            batch_size: 8192,
            flush_interval_ms: 100,
        })
    }

    /// 使用格式配置和颜色配置
    pub fn with_format_and_color(format_config: FormatConfig, color_config: ColorConfig) -> Self {
        let format_config = format_config.clone();
        let color_config = color_config.clone();
        Self::with_config(TermHandlerConfig {
            formatter: Arc::new(move |buf, record| format_with_color(buf, record, &format_config, &color_config)),
            batch_size: 8192,
            flush_interval_ms: 100,
        })
    }

    /// 使用配置创建终端处理器
    fn with_config(config: TermHandlerConfig) -> Self {
        let (command_sender, command_receiver) = unbounded();
        let config_clone = config.clone();

        let worker_thread = thread::spawn(move || {
            Self::worker_thread(config_clone, command_receiver);
        });

        Self {
            config,
            command_sender,
            worker_thread: Some(worker_thread),
        }
    }

    /// 工作线程 - 处理所有终端输出
    fn worker_thread(config: TermHandlerConfig, receiver: Receiver<TermCommand>) {
        let mut stdout = BufWriter::new(io::stdout());
        let mut batch_buffer = Vec::with_capacity(config.batch_size);
        let mut last_flush = Instant::now();
        let flush_interval = std::time::Duration::from_millis(config.flush_interval_ms);

        while let Ok(command) = receiver.recv() {
            match command {
                TermCommand::Write(data) => {
                    batch_buffer.extend_from_slice(&data);

                    // 批量写入条件：达到batch_size或flush_interval间隔
                    if batch_buffer.len() >= config.batch_size || last_flush.elapsed() >= flush_interval {
                        if let Err(e) = stdout.write_all(&batch_buffer) {
                            eprintln!("终端批量写入失败: {}", e);
                        }
                        batch_buffer.clear();
                        last_flush = Instant::now();
                    }
                }

                TermCommand::Flush => {
                    // 写入剩余数据
                    if !batch_buffer.is_empty() {
                        if let Err(e) = stdout.write_all(&batch_buffer) {
                            eprintln!("终端最终写入失败: {}", e);
                        }
                        batch_buffer.clear();
                    }

                    if let Err(e) = stdout.flush() {
                        eprintln!("终端刷新失败: {}", e);
                    }
                }

                TermCommand::Shutdown => {
                    // 处理剩余数据并退出
                    if !batch_buffer.is_empty() {
                        if let Err(e) = stdout.write_all(&batch_buffer) {
                            eprintln!("终端关闭时写入失败: {}", e);
                        }
                    }
                    if let Err(e) = stdout.flush() {
                        eprintln!("终端关闭时刷新失败: {}", e);
                    }
                    break;
                }
            }
        }
    }
}

impl LogHandler for TermHandler {
    fn handle(&self, record: &Record) {
        // 格式化日志消息到缓冲区
        let mut buffer = Vec::with_capacity(512); // 预分配512字节缓冲区
        {
            let mut writer = &mut buffer as &mut dyn std::io::Write;
            if let Err(e) = (self.config.formatter)(&mut writer, record) {
                eprintln!("终端格式化失败: {}", e);
                return;
            }
        }

        // 发送到工作线程进行异步批量写入
        if let Err(e) = self.command_sender.send(TermCommand::Write(buffer.clone())) {
            // 如果发送失败（比如工作线程已退出），直接写入
            let mut stdout = io::stdout();
            if let Err(e) = stdout.write_all(&buffer) {
                eprintln!("终端直接写入失败: {}", e);
            }
            if let Err(e) = stdout.flush() {
                eprintln!("终端直接刷新失败: {}", e);
            }
        }
    }

    fn flush(&self) {
        // 发送刷新命令
        if let Err(e) = self.command_sender.send(TermCommand::Flush) {
            // 如果发送失败，直接刷新
            let _ = io::stdout().flush();
        }
    }

    fn handler_type(&self) -> HandlerType {
        HandlerType::Terminal
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for TermHandler {
    fn drop(&mut self) {
        // 发送关闭命令
        let _ = self.command_sender.send(TermCommand::Shutdown);
        // 等待工作线程结束
        if let Some(thread) = self.worker_thread.take() {
            let _ = thread.join();
        }
    }
}

impl Default for TermHandler {
    fn default() -> Self {
        Self::new()
    }
}
