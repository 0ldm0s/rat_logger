//! UDP日志处理器

use std::any::Any;
use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::Mutex;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;

use crate::handler::{LogHandler, HandlerType};
use crate::config::{Record, NetworkConfig, NetRecord};

/// UDP连接池
pub struct UdpConnectionPool {
    connections: DashMap<String, Arc<UdpSocket>>,
    runtime: Option<Arc<Runtime>>,
}

impl UdpConnectionPool {
    /// 创建新的连接池
    pub fn new() -> Self {
        let runtime = match Runtime::new() {
            Ok(rt) => Some(Arc::new(rt)),
            Err(_) => None,
        };

        Self {
            connections: DashMap::new(),
            runtime,
        }
    }

    /// 获取或创建UDP连接
    async fn get_connection(&self, addr: &str) -> Option<Arc<UdpSocket>> {
        if let Some(socket) = self.connections.get(addr) {
            return Some(socket.clone());
        }

        match UdpSocket::bind("0.0.0.0:0").await {
            Ok(socket) => {
                if let Ok(()) = socket.connect(addr).await {
                    let socket = Arc::new(socket);
                    self.connections.insert(addr.to_string(), socket.clone());
                    Some(socket)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// 发送数据
    async fn send_data(&self, addr: &str, data: &[u8]) -> std::io::Result<()> {
        if let Some(socket) = self.get_connection(addr).await {
            socket.send(data).await?;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Failed to establish UDP connection",
            ))
        }
    }

    /// 清理连接
    fn cleanup(&self) {
        self.connections.clear();
    }
}

impl Default for UdpConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for UdpConnectionPool {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// UDP日志处理器
pub struct UdpHandler {
    config: NetworkConfig,
    pool: Arc<UdpConnectionPool>,
    retry_count: u32,
}

impl UdpHandler {
    /// 创建新的UDP处理器
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            pool: Arc::new(UdpConnectionPool::new()),
            retry_count: 3,
        }
    }

    /// 设置重试次数
    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }

    /// 异步发送日志记录
    async fn send_record(&self, record: &Record) {
        let mut net_record = NetRecord::from(record);
        net_record.auth_token = Some(self.config.auth_token.clone());
        net_record.app_id = Some(self.config.app_id.clone());

        if let Ok(data) = bincode::encode_to_vec(&net_record, bincode::config::standard()) {
            let addr = format!("{}:{}", self.config.server_addr, self.config.server_port);

            for attempt in 0..self.retry_count {
                match self.pool.send_data(&addr, &data).await {
                    Ok(_) => return,
                    Err(e) => {
                        if attempt == self.retry_count - 1 {
                            eprintln!("UDP send failed after {} attempts: {}", self.retry_count, e);
                        } else {
                            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }
    }

    /// 同步发送（兼容性接口）
    fn send_record_sync(&self, record: &Record) {
        if let Some(runtime) = &self.pool.runtime {
            let _ = runtime.block_on(self.send_record(record));
        } else {
            // 如果没有运行时，使用tokio::spawn
            let pool = Arc::clone(&self.pool);
            let addr = format!("{}:{}", self.config.server_addr, self.config.server_port);
            let mut net_record = NetRecord::from(record);
            net_record.auth_token = Some(self.config.auth_token.clone());
            net_record.app_id = Some(self.config.app_id.clone());

            if let Ok(data) = bincode::encode_to_vec(&net_record, bincode::config::standard()) {
                tokio::spawn(async move {
                    for attempt in 0..3 {
                        match pool.send_data(&addr, &data).await {
                            Ok(_) => return,
                            Err(e) => {
                                if attempt == 2 {
                                    eprintln!("UDP send failed: {}", e);
                                } else {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                }
                            }
                        }
                    }
                });
            }
        }
    }
}

impl LogHandler for UdpHandler {
    fn handle(&self, record: &Record) {
        self.send_record_sync(record);
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

impl Drop for UdpHandler {
    fn drop(&mut self) {
        self.pool.cleanup();
    }
}
