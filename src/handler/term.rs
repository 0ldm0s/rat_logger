//! 终端日志处理器

use std::io::{self, Write};
use std::any::Any;

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

/// 终端日志处理器
pub struct TermHandler {
    formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync>,
}

impl TermHandler {
    /// 创建新的终端处理器（默认无主题）
    pub fn new() -> Self {
        Self {
            formatter: Box::new(default_format),
        }
    }

    /// 使用自定义格式化函数
    pub fn with_formatter<F>(formatter: F) -> Self
    where
        F: Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync + 'static,
    {
        Self {
            formatter: Box::new(formatter),
        }
    }

    /// 使用格式配置
    pub fn with_format(format_config: FormatConfig) -> Self {
        let format_config = format_config.clone();
        Self {
            formatter: Box::new(move |buf, record| format_with_config(buf, record, &format_config)),
        }
    }

    /// 使用格式配置和颜色配置
    pub fn with_format_and_color(format_config: FormatConfig, color_config: ColorConfig) -> Self {
        let format_config = format_config.clone();
        let color_config = color_config.clone();
        Self {
            formatter: Box::new(move |buf, record| format_with_color(buf, record, &format_config, &color_config)),
        }
    }
}

impl LogHandler for TermHandler {
    fn handle(&self, record: &Record) {
        let mut stdout = io::stdout();
        if let Err(e) = (self.formatter)(&mut stdout, record) {
            eprintln!("Terminal write error: {}", e);
        }
    }

    fn flush(&self) {
        let _ = io::stdout().flush();
    }

    fn handler_type(&self) -> HandlerType {
        HandlerType::Terminal
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Default for TermHandler {
    fn default() -> Self {
        Self::new()
    }
}
