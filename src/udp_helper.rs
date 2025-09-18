//! UDP封包解包辅助工具

use crate::config::{Level, Record, NetRecord, Metadata};
use bincode;
use std::io;

/// UDP封包解包工具
pub struct UdpPacketHelper;

impl UdpPacketHelper {
    /// 将Record编码为UDP数据包
    pub fn encode_record(record: &Record, auth_token: Option<String>, app_id: Option<String>) -> io::Result<Vec<u8>> {
        let mut net_record = NetRecord::from(record);
        net_record.auth_token = auth_token;
        net_record.app_id = app_id;

        bincode::encode_to_vec(&net_record, bincode::config::standard())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// 将UDP数据包解码为NetRecord
    pub fn decode_packet(data: &[u8]) -> io::Result<NetRecord> {
        bincode::decode_from_slice(data, bincode::config::standard())
            .map(|(record, _)| record)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// 将NetRecord转换为Record
    pub fn net_record_to_record(net_record: &NetRecord) -> Record {
        let metadata = Metadata {
            level: net_record.level,
            target: net_record.target.clone(),
            auth_token: net_record.auth_token.clone(),
            app_id: net_record.app_id.clone(),
        };

        Record {
            metadata: std::sync::Arc::new(metadata),
            args: net_record.message.clone(),
            module_path: net_record.module_path.clone(),
            file: net_record.file.clone(),
            line: net_record.line,
        }
    }

    /// 创建一个带有认证和App ID的编码器
    pub fn create_encoder(auth_token: String, app_id: String) -> impl Fn(&Record) -> io::Result<Vec<u8>> + Send + Sync {
        move |record| {
            Self::encode_record(record, Some(auth_token.clone()), Some(app_id.clone()))
        }
    }

    /// 创建一个解码器
    pub fn create_decoder() -> impl Fn(&[u8]) -> io::Result<Record> + Send + Sync {
        |data| {
            let net_record = Self::decode_packet(data)?;
            Ok(Self::net_record_to_record(&net_record))
        }
    }

    /// 验证UDP数据包的有效性
    pub fn validate_packet(data: &[u8]) -> bool {
        Self::decode_packet(data).is_ok()
    }

    /// 获取数据包的元数据（不解码整个包）
    pub fn get_packet_metadata(data: &[u8]) -> Option<PacketMetadata> {
        if let Ok(net_record) = Self::decode_packet(data) {
            Some(PacketMetadata {
                level: net_record.level,
                target: net_record.target,
                app_id: net_record.app_id,
                timestamp: net_record.timestamp,
                message_length: net_record.message.len(),
            })
        } else {
            None
        }
    }
}

/// UDP数据包的元数据信息
#[derive(Debug, Clone)]
pub struct PacketMetadata {
    pub level: Level,
    pub target: String,
    pub app_id: Option<String>,
    pub timestamp: u64,
    pub message_length: usize,
}

impl PacketMetadata {
    /// 检查数据包是否来自指定的应用
    pub fn is_from_app(&self, app_id: &str) -> bool {
        self.app_id.as_deref() == Some(app_id)
    }

    /// 获取数据包的年龄（秒）
    pub fn age_seconds(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(self.timestamp)
    }
}

/// UDP数据包批处理器
pub struct UdpBatchProcessor {
    batch_size: usize,
    max_wait_time_ms: u64,
}

impl UdpBatchProcessor {
    /// 创建新的批处理器
    pub fn new(batch_size: usize, max_wait_time_ms: u64) -> Self {
        Self {
            batch_size,
            max_wait_time_ms,
        }
    }

    /// 处理一批UDP数据包
    pub fn process_batch(&self, packets: &[Vec<u8>]) -> Vec<Record> {
        let mut records = Vec::new();

        for packet in packets {
            if let Ok(record) = UdpPacketHelper::create_decoder()(packet) {
                records.push(record);
            }
        }

        records
    }

    /// 过滤数据包
    pub fn filter_packets(&self, packets: &[Vec<u8>], filter: &dyn Fn(&PacketMetadata) -> bool) -> Vec<Vec<u8>> {
        let mut filtered = Vec::new();

        for packet in packets {
            if let Some(metadata) = UdpPacketHelper::get_packet_metadata(packet) {
                if filter(&metadata) {
                    filtered.push(packet.clone());
                }
            }
        }

        filtered
    }
}

impl Default for UdpBatchProcessor {
    fn default() -> Self {
        Self::new(100, 1000) // 默认100个包一批，最多等待1秒
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Level;

    #[test]
    fn test_encode_decode_roundtrip() {
        let record = Record {
            metadata: std::sync::Arc::new(Metadata {
                level: Level::Info,
                target: "test".to_string(),
                auth_token: None,
                app_id: None,
            }),
            args: "test message".to_string(),
            module_path: Some("test::module".to_string()),
            file: Some("test.rs".to_string()),
            line: Some(42),
        };

        let encoded = UdpPacketHelper::encode_record(&record, Some("token".to_string()), Some("app".to_string())).unwrap();
        let decoded = UdpPacketHelper::decode_packet(&encoded).unwrap();
        let restored = UdpPacketHelper::net_record_to_record(&decoded);

        assert_eq!(restored.metadata.level, record.metadata.level);
        assert_eq!(restored.metadata.target, record.metadata.target);
        assert_eq!(restored.args, record.args);
        assert_eq!(restored.line, record.line);
    }

    #[test]
    fn test_packet_metadata() {
        let record = Record {
            metadata: std::sync::Arc::new(Metadata {
                level: Level::Error,
                target: "test".to_string(),
                auth_token: None,
                app_id: Some("my_app".to_string()),
            }),
            args: "error message".to_string(),
            module_path: None,
            file: None,
            line: None,
        };

        let encoded = UdpPacketHelper::encode_record(&record, None, Some("my_app".to_string())).unwrap();
        let metadata = UdpPacketHelper::get_packet_metadata(&encoded).unwrap();

        assert_eq!(metadata.level, Level::Error);
        assert_eq!(metadata.target, "test");
        assert_eq!(metadata.app_id, Some("my_app".to_string()));
        assert!(metadata.is_from_app("my_app"));
        assert!(!metadata.is_from_app("other_app"));
        assert!(metadata.level.should_log_at_level(Level::Error));  // Error日志在Error级别下应该发送
        assert!(metadata.level.should_log_at_level(Level::Warn));   // Error日志在Warn级别下应该发送
        assert!(metadata.level.should_log_at_level(Level::Info));   // Error日志在Info级别下应该发送
        assert!(metadata.level.should_log_at_level(Level::Debug));  // Error日志在Debug级别下应该发送
        assert!(metadata.level.should_log_at_level(Level::Trace));  // Error日志在Trace级别下应该发送
    }

    #[test]
    fn test_level_filtering() {
        // 测试Debug级别的日志
        let debug_record = Record {
            metadata: std::sync::Arc::new(Metadata {
                level: Level::Debug,
                target: "test".to_string(),
                auth_token: None,
                app_id: None,
            }),
            args: "debug message".to_string(),
            module_path: None,
            file: None,
            line: None,
        };

        let encoded = UdpPacketHelper::encode_record(&debug_record, None, None).unwrap();
        let metadata = UdpPacketHelper::get_packet_metadata(&encoded).unwrap();

        assert!(!metadata.level.should_log_at_level(Level::Error));  // Debug日志不应该在Error级别下发送
        assert!(!metadata.level.should_log_at_level(Level::Warn));   // Debug日志不应该在Warn级别下发送
        assert!(!metadata.level.should_log_at_level(Level::Info));   // Debug日志不应该在Info级别下发送
        assert!(metadata.level.should_log_at_level(Level::Debug));  // Debug日志应该在Debug级别下发送
        assert!(metadata.level.should_log_at_level(Level::Trace));  // Debug日志应该在Trace级别下发送
    }
}