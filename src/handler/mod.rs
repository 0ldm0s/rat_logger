//! 日志处理器模块

use std::any::Any;
use crate::config::Record;

/// 日志处理器 trait
pub trait LogHandler: Send + Sync + Any {
    /// 处理日志记录
    fn handle(&self, record: &Record);

    /// 刷新处理器
    fn flush(&self);

    /// 获取处理器类型
    fn handler_type(&self) -> HandlerType;

    /// 类型安全的向下转型
    fn as_any(&self) -> &dyn Any;
}

/// 处理器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerType {
    Terminal,
    File,
    Udp,
    Composite,
}

pub mod term;
pub mod file;
pub mod udp;
pub mod composite;

pub use term::TermHandler;
pub use file::FileHandler;
pub use udp::UdpHandler;
pub use composite::CompositeHandler;
