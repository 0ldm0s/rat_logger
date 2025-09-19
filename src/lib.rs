//! rat_logger - 高性能日志库
//!
//! 基于 zerg_creep 重新设计的高性能日志库，支持多处理器、异步IO和批处理优化

pub mod core;
pub mod handler;
pub mod config;
pub mod udp_helper;
pub mod producer_consumer;

use core::LoggerCore;
use handler::{LogHandler, HandlerType};
use config::{Record, Metadata, AppId};

// 重新导出主要类型
pub use core::{Logger, LoggerBuilder};
pub use handler::{composite::CompositeHandler, term::TermProcessor, file::FileProcessor, udp::UdpProcessor};
pub use config::{Level, LevelFilter, FileConfig, NetworkConfig, FormatConfig, LevelStyle, ColorConfig};

// 日志宏
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::__private_log_impl(
            $crate::Level::Error,
            format_args!($($arg)*),
            module_path!(),
            file!(),
            line!(),
        )
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::__private_log_impl(
            $crate::Level::Warn,
            format_args!($($arg)*),
            module_path!(),
            file!(),
            line!(),
        )
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::__private_log_impl(
            $crate::Level::Info,
            format_args!($($arg)*),
            module_path!(),
            file!(),
            line!(),
        )
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::__private_log_impl(
            $crate::Level::Debug,
            format_args!($($arg)*),
            module_path!(),
            file!(),
            line!(),
        )
    };
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        $crate::__private_log_impl(
            $crate::Level::Trace,
            format_args!($($arg)*),
            module_path!(),
            file!(),
            line!(),
        )
    };
}

#[doc(hidden)]
pub fn __private_log_impl(
    level: Level,
    args: std::fmt::Arguments<'_>,
    module_path: &'static str,
    file: &'static str,
    line: u32,
) {
    if let Some(logger) = core::LOGGER.get() {
        let record = Record {
            metadata: std::sync::Arc::new(Metadata {
                level,
                target: module_path.to_string(),
                auth_token: None,
                app_id: None,
            }),
            args: args.to_string(),
            module_path: Some(module_path.to_string()),
            file: Some(file.to_string()),
            line: Some(line),
        };
        logger.log(&record);
    }
}

// 注意：以下便捷初始化函数已弃用，将在0.3.0版本中彻底移除
// 请改用LoggerBuilder进行初始化，以便获得更灵活的配置选项
#[deprecated(since = "0.2.0", note = "请使用LoggerBuilder::new().add_terminal().init()")]
pub fn init() -> Result<(), core::SetLoggerError> {
    LoggerBuilder::new()
        .add_terminal()
        .init()
}

#[deprecated(since = "0.2.0", note = "请使用LoggerBuilder::new().add_terminal().with_level(level).init()")]
pub fn init_with_level(level: LevelFilter) -> Result<(), core::SetLoggerError> {
    LoggerBuilder::new()
        .add_terminal()
        .with_level(level)
        .init()
}
