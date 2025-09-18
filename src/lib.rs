//! rat_logger - 高性能日志库
//!
//! 基于 zerg_creep 重新设计的高性能日志库，支持多处理器、异步IO和批处理优化

pub mod core;
pub mod handler;
pub mod config;

use core::LoggerCore;
use handler::{LogHandler, HandlerType};
use config::{Level, LevelFilter, Record, Metadata, AppId};

// 重新导出主要类型
pub use core::{Logger, LoggerBuilder};
pub use handler::{composite::CompositeHandler, term::TermHandler, file::FileHandler, udp::UdpHandler};
pub use config::{FileConfig, NetworkConfig};

// 日志宏
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ($crate::__private_log!($crate::Level::Error, $($arg)*));
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ($crate::__private_log!($crate::Level::Warn, $($arg)*));
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ($crate::__private_log!($crate::Level::Info, $($arg)*));
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ($crate::__private_log!($crate::Level::Debug, $($arg)*));
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => ($crate::__private_log!($crate::Level::Trace, $($arg)*));
}

#[macro_export]
#[doc(hidden)]
macro_rules! __private_log {
    ($level:expr, $($arg:tt)*) => {
        $crate::__private_log_impl($level, format_args!($($arg)*))
    };
}

#[doc(hidden)]
pub fn __private_log_impl(level: Level, args: std::fmt::Arguments<'_>) {
    if let Some(logger) = core::LOGGER.get() {
        let record = Record {
            metadata: std::sync::Arc::new(Metadata {
                level,
                target: module_path!().to_string(),
                auth_token: None,
                app_id: None,
            }),
            args: args.to_string(),
            module_path: Some(module_path!().to_string()),
            file: Some(file!().to_string()),
            line: Some(line!()),
        };
        logger.log(&record);
    }
}

// 便捷初始化函数
pub fn init() -> Result<(), core::SetLoggerError> {
    LoggerBuilder::new()
        .add_terminal()
        .init()
}

pub fn init_with_level(level: LevelFilter) -> Result<(), core::SetLoggerError> {
    LoggerBuilder::new()
        .add_terminal()
        .with_level(level)
        .init()
}
