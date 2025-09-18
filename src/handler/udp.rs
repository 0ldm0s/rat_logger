//! UDP日志处理器

use std::any::Any;
use std::sync::Arc;

use crate::handler::{LogHandler, HandlerType};
use crate::config::{Record, NetworkConfig, NetRecord};

/// UDP日志处理器
pub struct UdpHandler {
    config: NetworkConfig,
}

impl UdpHandler {
    /// 创建新的UDP处理器
    pub fn new(config: NetworkConfig) -> Self {
        Self { config }
    }
}

impl LogHandler for UdpHandler {
    fn handle(&self, record: &Record) {
        // 简化实现，实际应该发送UDP包
        let mut net_record = NetRecord::from(record);
        net_record.auth_token = Some(self.config.auth_token.clone());
        net_record.app_id = Some(self.config.app_id.clone());

        if let Ok(data) = bincode::encode_to_vec(&net_record, bincode::config::standard()) {
            // 这里应该发送UDP数据，暂时简化
            println!("UDP log: {} bytes", data.len());
        }
    }

    fn flush(&self) {
        // UDP是无连接的，不需要flush
    }

    fn handler_type(&self) -> HandlerType {
        HandlerType::Udp
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
