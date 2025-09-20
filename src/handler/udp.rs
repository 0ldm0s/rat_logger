//! UDP日志处理器 - 高性能异步架构

use std::any::Any;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use dashmap::DashMap;
use parking_lot::Mutex;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;

use crate::producer_consumer::LogProcessor;
use crate::config::{Record, NetworkConfig};
use crate::udp_helper::UdpPacketHelper;

/// UDP连接池
pub struct UdpConnectionPool {
    connections: DashMap<String, Arc<UdpSocket>>,
    runtime: Arc<Runtime>,
}

impl UdpConnectionPool {
    /// 创建新的连接池
    pub fn new() -> Self {
        let runtime = match Runtime::new() {
            Ok(rt) => Arc::new(rt),
            Err(e) => {
                panic!("Failed to create tokio runtime: {}", e);
            }
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

/// UDP处理器配置
#[derive(Debug, Clone)]
pub struct UdpConfig {
    /// 网络配置
    pub network_config: NetworkConfig,
    /// 重试次数
    pub retry_count: u32,
    /// 批量大小
    pub batch_size: usize,
    /// 刷新间隔（毫秒）
    pub flush_interval_ms: u64,
    /// 连接池大小
    pub pool_size: usize,
}

impl UdpConfig {
    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), String> {
        // 验证重试次数
        if self.retry_count == 0 {
            return Err("配置错误: 重试次数不能为 0".to_string());
        }
        if self.retry_count > 10 {
            return Err("配置错误: 重试次数过多 (最大 10次)".to_string());
        }

        // 验证批量大小
        if self.batch_size == 0 {
            return Err("配置错误: 批量大小不能为 0".to_string());
        }
        if self.batch_size > 1024 * 1024 {
            return Err("配置错误: 批量大小过大 (最大 1MB)".to_string());
        }

        // 验证刷新间隔
        if self.flush_interval_ms == 0 {
            return Err("配置错误: 刷新间隔不能为 0".to_string());
        }
        if self.flush_interval_ms > 60000 {
            return Err("配置错误: 刷新间隔过长 (最大 60秒)".to_string());
        }

        // 验证连接池大小
        if self.pool_size == 0 {
            return Err("配置错误: 连接池大小不能为 0".to_string());
        }
        if self.pool_size > 100 {
            return Err("配置错误: 连接池大小过大 (最大 100)".to_string());
        }

        Ok(())
    }
}

impl Default for UdpConfig {
    fn default() -> Self {
        Self {
            network_config: NetworkConfig::default(),
            retry_count: 3,
            batch_size: 8192,  // 8KB批量发送
            flush_interval_ms: 100, // 100ms刷新间隔
            pool_size: 10,
        }
    }
}

/// UDP日志处理器 - 实现LogProcessor trait
pub struct UdpProcessor {
    config: UdpConfig,
    pool: Arc<UdpConnectionPool>,
    buffer: Arc<Mutex<Vec<Vec<u8>>>>,
    current_batch_size: Arc<Mutex<usize>>,
    last_flush: Arc<Mutex<Instant>>,
}

impl UdpProcessor {
    /// 创建新的UDP处理器
    pub fn new(config: NetworkConfig) -> Self {
        let udp_config = UdpConfig {
            network_config: config,
            ..Default::default()
        };
        Self::with_config(udp_config)
    }

    /// 使用UDP配置创建处理器
    pub fn with_config(config: UdpConfig) -> Self {
        // 验证配置，如果失败则直接panic，让用户明确知道配置问题
        if let Err(e) = config.validate() {
            panic!("UdpConfig 验证失败: {}\n请检查您的配置并修复上述问题后再重试。", e);
        }

        Self {
            config,
            pool: Arc::new(UdpConnectionPool::new()),
            buffer: Arc::new(Mutex::new(Vec::with_capacity(8192))),
            current_batch_size: Arc::new(Mutex::new(0)),
            last_flush: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// 设置重试次数
    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.config.retry_count = retry_count;
        self
    }

    /// 设置批量大小
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.config.batch_size = batch_size;
        self
    }

    /// 设置刷新间隔
    pub fn with_flush_interval(mut self, flush_interval_ms: u64) -> Self {
        self.config.flush_interval_ms = flush_interval_ms;
        self
    }

    /// 编码日志记录
    fn encode_record(&self, record: &Record) -> Result<Vec<u8>, String> {
        UdpPacketHelper::encode_record(
            record,
            Some(self.config.network_config.auth_token.clone()),
            Some(self.config.network_config.app_id.clone()),
        ).map_err(|e| format!("UDP编码失败: {}", e))
    }

    /// 批量发送数据
    fn send_batch(&self) -> Result<(), String> {
        let mut buffer_guard = self.buffer.lock();
        let mut current_batch_size_guard = self.current_batch_size.lock();

        if buffer_guard.is_empty() {
            return Ok(());
        }

        let addr = format!("{}:{}", self.config.network_config.server_addr, self.config.network_config.server_port);

        // 批量发送所有数据
        for data in buffer_guard.drain(..) {
            let pool = Arc::clone(&self.pool);
            let addr = addr.clone();
            let retry_count = self.config.retry_count;

            // 在当前线程的运行时中异步发送
            pool.runtime.block_on(async {
                for attempt in 0..retry_count {
                    match pool.send_data(&addr, &data).await {
                        Ok(_) => break,
                        Err(e) => {
                            if attempt == retry_count - 1 {
                                eprintln!("[udp] 批量发送失败，重试{}次后放弃: {}", retry_count, e);
                            } else {
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }
                        }
                    }
                }
            });
        }

        // 重置批处理大小计数器
        *current_batch_size_guard = 0;

        // 更新最后刷新时间
        let mut last_flush_guard = self.last_flush.lock();
        *last_flush_guard = Instant::now();

        Ok(())
    }

    /// 添加数据到缓冲区
    fn add_to_buffer(&self, data: Vec<u8>) -> Result<(), String> {
        let mut buffer_guard = self.buffer.lock();
        let mut current_batch_size_guard = self.current_batch_size.lock();
        let data_size = data.len();

        buffer_guard.push(data);
        *current_batch_size_guard += data_size;

        // 检查是否需要发送
        let should_send = *current_batch_size_guard >= self.config.batch_size ||
                          {
                              let last_flush_guard = self.last_flush.lock();
                              last_flush_guard.elapsed() >= std::time::Duration::from_millis(self.config.flush_interval_ms)
                          };

        if should_send {
            drop(buffer_guard);
            drop(current_batch_size_guard);
            self.send_batch()?;
        }

        Ok(())
    }
}

impl LogProcessor for UdpProcessor {
    fn name(&self) -> &'static str {
        "udp_processor"
    }

    fn process(&mut self, data: &[u8]) -> Result<(), String> {
        // 反序列化日志记录
        let record = bincode::decode_from_slice::<Record, _>(data, bincode::config::standard())
            .map_err(|e| format!("反序列化失败: {}", e))?.0;

        // 编码为UDP包
        let encoded_data = self.encode_record(&record)?;

        // 添加到缓冲区
        self.add_to_buffer(encoded_data)
    }

    fn process_batch(&mut self, batch: &[Vec<u8>]) -> Result<(), String> {
        for data in batch {
            // 反序列化日志记录
            let record = bincode::decode_from_slice::<Record, _>(data, bincode::config::standard())
                .map_err(|e| format!("批量反序列化失败: {}", e))?.0;

            // 编码为UDP包
            let encoded_data = self.encode_record(&record)?;

            // 添加到缓冲区
            self.add_to_buffer(encoded_data)?;
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        // 发送缓冲区中的所有数据
        self.send_batch()
    }

    fn cleanup(&mut self) -> Result<(), String> {
        // 先刷新剩余数据
        self.flush()?;

        // 清理连接池
        self.pool.cleanup();

        Ok(())
    }
}