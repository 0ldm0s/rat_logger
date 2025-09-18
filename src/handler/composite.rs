//! 组合日志处理器

use std::any::Any;
use std::sync::Arc;

use crate::handler::{LogHandler, HandlerType};
use crate::config::Record;

/// 组合多个日志处理器的实现
pub struct CompositeHandler {
    handlers: Vec<Arc<dyn LogHandler>>,
    parallel: bool,
}

impl CompositeHandler {
    /// 创建新的组合处理器
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            parallel: false,
        }
    }

    /// 启用并行处理
    pub fn with_parallel(mut self) -> Self {
        self.parallel = true;
        self
    }

    /// 添加日志处理器
    pub fn add_handler(&mut self, handler: Arc<dyn LogHandler>) {
        self.handlers.push(handler);
    }
}

impl Default for CompositeHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl LogHandler for CompositeHandler {
    fn handle(&self, record: &Record) {
        if self.parallel && self.handlers.len() > 1 {
            // 并行处理：为每个处理器创建独立的任务
            let handlers: Vec<Arc<dyn LogHandler>> = self.handlers.iter().cloned().collect();
            let record = record.clone();

            // 使用tokio进行并行处理
            tokio::spawn(async move {
                let join_handles: Vec<_> = handlers
                    .into_iter()
                    .map(|handler| {
                        let record = record.clone();
                        tokio::spawn(async move {
                            handler.handle(&record);
                        })
                    })
                    .collect();

                // 等待所有任务完成
                for handle in join_handles {
                    let _ = handle.await;
                }
            });
        } else {
            // 串行处理
            for handler in &self.handlers {
                handler.handle(record);
            }
        }
    }

    fn flush(&self) {
        for handler in &self.handlers {
            handler.flush();
        }
    }

    fn handler_type(&self) -> HandlerType {
        HandlerType::Composite
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
