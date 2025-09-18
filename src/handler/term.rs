//! 终端日志处理器

use std::io::{self, Write};
use std::any::Any;

use crate::handler::{LogHandler, HandlerType};
use crate::config::Record;

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

/// 终端日志处理器
pub struct TermHandler {
    formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync>,
}

impl TermHandler {
    /// 创建新的终端处理器
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
