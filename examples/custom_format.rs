//! 自定义格式示例

use rat_logger::*;
use std::io::{self, Write};

fn custom_format(buf: &mut dyn Write, record: &Record) -> io::Result<()> {
    use chrono::Local;

    let now = Local::now();
    let timestamp = now.format("%H:%M:%S%.3f");

    // 自定义彩色输出
    let level_color = match record.metadata.level {
        Level::Error => "\x1b[31m",    // 红色
        Level::Warn => "\x1b[33m",     // 黄色
        Level::Info => "\x1b[32m",     // 绿色
        Level::Debug => "\x1b[36m",    // 青色
        Level::Trace => "\x1b[35m",    // 紫色
    };

    let reset = "\x1b[0m";

    writeln!(
        buf,
        "{}[{}]{} {}:{} - {}{}",
        level_color,
        record.metadata.level,
        reset,
        timestamp,
        record.file.as_deref().unwrap_or("unknown"),
        record.args,
        reset
    )
}

fn main() {
    let term_handler = crate::handler::term::TermHandler::with_formatter(custom_format);

    let logger = LoggerCore::new(LevelFilter::Trace);
    logger.add_handler(Arc::new(term_handler));

    core::set_logger(Arc::new(logger)).unwrap();

    error!("彩色错误日志");
    warn!("彩色警告日志");
    info!("彩色信息日志");
    debug!("彩色调试日志");
    trace!("彩色跟踪日志");
}