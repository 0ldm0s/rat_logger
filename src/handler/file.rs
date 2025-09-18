//! 文件日志处理器

use std::io::{self, Write};
use std::any::Any;
use std::sync::Arc;
use parking_lot::Mutex;

use crate::handler::{LogHandler, HandlerType};
use crate::config::{Record, FileConfig};

/// 文件日志处理器
pub struct FileHandler {
    config: FileConfig,
    formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync>,
}

impl FileHandler {
    /// 创建新的文件处理器
    pub fn new(config: FileConfig) -> Self {
        Self {
            config,
            formatter: Box::new(default_format),
        }
    }

    /// 设置自定义格式化函数
    pub fn with_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync + 'static,
    {
        self.formatter = Box::new(formatter);
        self
    }
}

impl LogHandler for FileHandler {
    fn handle(&self, record: &Record) {
        // 简化实现，实际应该写入文件
        let mut buf = Vec::new();
        if let Err(e) = (self.formatter)(&mut buf, record) {
            eprintln!("File format error: {}", e);
        }
    }

    fn flush(&self) {
        // 简化实现
    }

    fn handler_type(&self) -> HandlerType {
        HandlerType::File
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// 默认格式化函数
fn default_format(buf: &mut dyn Write, record: &Record) -> io::Result<()> {
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
