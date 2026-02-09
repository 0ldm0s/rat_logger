//! 快速初始化模块 - 提供类似 tracing_subscriber::fmt().init() 的简洁 API
//!
//! # 使用示例
//!
//! ```rust
//! use rat_logger::{error, info};
//!
//! // 最简单的用法 - 使用默认配置
//! rat_logger::fmt().init();
//!
//! // 自定义日志级别
//! rat_logger::fmt()
//!     .with_max_level(rat_logger::LevelFilter::Debug)
//!     .init();
//!
//! error!("这是一条错误日志");
//! info!("这是一条信息日志");
//! ```

use crate::{LevelFilter, LoggerBuilder};

/// 快速初始化器 - 类似 tracing_subscriber::fmt()
///
/// # 示例
///
/// ```rust
/// // 默认配置
/// rat_logger::fmt().init();
///
/// // 自定义日志级别
/// rat_logger::fmt()
///     .with_max_level(rat_logger::LevelFilter::Debug)
///     .init();
/// ```
#[derive(Debug, Clone)]
pub struct FmtInitializer {
    max_level: LevelFilter,
}

impl Default for FmtInitializer {
    fn default() -> Self {
        Self {
            max_level: LevelFilter::Info,  // 默认 Info 级别
        }
    }
}

impl FmtInitializer {
    /// 创建新的格式化初始化器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大日志级别
    ///
    /// # 示例
    ///
    /// ```rust
    /// rat_logger::fmt()
    ///     .with_max_level(rat_logger::LevelFilter::Debug)
    ///     .init();
    /// ```
    pub fn with_max_level(mut self, level: LevelFilter) -> Self {
        self.max_level = level;
        self
    }

    /// 初始化全局日志器
    ///
    /// # 示例
    ///
    /// ```rust
    /// rat_logger::fmt().init();
    /// ```
    pub fn init(self) -> Result<(), crate::core::SetLoggerError> {
        LoggerBuilder::new()
            .add_terminal_with_config(crate::handler::term::TermConfig::default())
            .with_level(self.max_level)
            .init()
    }
}

/// 创建格式化初始化器 - 类似 tracing_subscriber::fmt()
///
/// # 示例
///
/// ```rust
/// // 最简单的用法
/// rat_logger::fmt().init();
///
/// // 自定义日志级别
/// rat_logger::fmt()
///     .with_max_level(rat_logger::LevelFilter::Debug)
///     .init();
/// ```
pub fn fmt() -> FmtInitializer {
    FmtInitializer::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_initializer_default() {
        let initializer = FmtInitializer::default();
        assert_eq!(initializer.max_level, LevelFilter::Info);
    }

    #[test]
    fn test_fmt_initializer_builder() {
        let initializer = fmt()
            .with_max_level(LevelFilter::Debug);

        assert_eq!(initializer.max_level, LevelFilter::Debug);
    }
}
