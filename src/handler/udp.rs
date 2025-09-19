//! UDP日志处理器

use std::any::Any;
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use dashmap::DashMap;
use crossbeam_channel::{Sender, Receiver, unbounded};
use parking_lot::Mutex;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;

use crate::handler::{LogHandler, HandlerType};
use crate::config::{Record, NetworkConfig};
use crate::udp_helper::UdpPacketHelper;

/// UDP指令枚举 - 生产者消费者模式
enum UdpCommand {
    /// 发送日志数据
    Send(Vec<u8>),
    /// 强制发送缓冲区数据
    Flush,
    /// 停止工作线程
    Shutdown,
}

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
#[derive(Clone)]
struct UdpHandlerConfig {
    config: NetworkConfig,
    retry_count: u32,
    batch_size: usize,
    flush_interval_ms: u64,
}

/// UDP日志处理器
pub struct UdpHandler {
    config: UdpHandlerConfig,
    command_sender: Sender<UdpCommand>,
    worker_thread: Option<thread::JoinHandle<()>>,
}

impl UdpHandler {
    /// 创建新的UDP处理器（使用异步批量发送）
    pub fn new(config: NetworkConfig) -> Self {
        Self::with_config(UdpHandlerConfig {
            config,
            retry_count: 3,
            batch_size: 8192,  // 8KB批量发送
            flush_interval_ms: 100, // 100ms刷新间隔
        })
    }

    /// 设置重试次数
    pub fn with_retry_count(mut self, retry_count: u32) -> Self {
        self.config.retry_count = retry_count;
        self
    }

    /// 使用配置创建UDP处理器
    fn with_config(config: UdpHandlerConfig) -> Self {
        let (command_sender, command_receiver) = unbounded();
        let config_clone = config.clone();

        let worker_thread = thread::spawn(move || {
            Self::worker_thread(config_clone, command_receiver);
        });

        Self {
            config,
            command_sender,
            worker_thread: Some(worker_thread),
        }
    }

    /// 工作线程 - 处理所有UDP发送
    fn worker_thread(config: UdpHandlerConfig, receiver: Receiver<UdpCommand>) {
        let pool = UdpConnectionPool::new();
        let mut batch_buffer = Vec::with_capacity(config.batch_size);
        let mut current_batch_size = 0;
        let mut last_flush = Instant::now();
        let flush_interval = std::time::Duration::from_millis(config.flush_interval_ms);
        let addr = format!("{}:{}", config.config.server_addr, config.config.server_port);

        while let Ok(command) = receiver.recv() {
            match command {
                UdpCommand::Send(data) => {
                    let data_size = data.len();
                    batch_buffer.push(data);
                    current_batch_size += data_size;

                    // 批量发送条件：达到batch_size字节或flush_interval间隔
                    if current_batch_size >= config.batch_size || last_flush.elapsed() >= flush_interval {
                        Self::send_batch(&pool, &addr, &mut batch_buffer, &mut current_batch_size, &config);
                        last_flush = Instant::now();
                    }
                }

                UdpCommand::Flush => {
                    // 发送剩余数据
                    if !batch_buffer.is_empty() {
                        Self::send_batch(&pool, &addr, &mut batch_buffer, &mut current_batch_size, &config);
                    }
                }

                UdpCommand::Shutdown => {
                    // 处理剩余数据并退出
                    if !batch_buffer.is_empty() {
                        Self::send_batch(&pool, &addr, &mut batch_buffer, &mut current_batch_size, &config);
                    }
                    break;
                }
            }
        }
    }

    /// 批量发送数据
    fn send_batch(pool: &UdpConnectionPool, addr: &str, batch_buffer: &mut Vec<Vec<u8>>, current_batch_size: &mut usize, config: &UdpHandlerConfig) {
        if batch_buffer.is_empty() {
            return;
        }

        // 批量发送所有数据，重用连接池的运行时
        for data in batch_buffer.drain(..) {
            let pool = pool.clone();
            let addr = addr.to_string();
            let retry_count = config.retry_count;

            pool.runtime.block_on(async {
                for attempt in 0..retry_count {
                    match pool.send_data(&addr, &data).await {
                        Ok(_) => break,
                        Err(e) => {
                            if attempt == retry_count - 1 {
                                eprintln!("UDP batch send failed after {} attempts: {}", retry_count, e);
                            } else {
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }
                        }
                    }
                }
            });
        }

        // 重置批处理大小计数器
        *current_batch_size = 0;
    }

    /// 发送单个日志记录（编码后发送到工作线程）
    fn send_to_worker(&self, record: &Record) {
        match UdpPacketHelper::encode_record(
            record,
            Some(self.config.config.auth_token.clone()),
            Some(self.config.config.app_id.clone())
        ) {
            Ok(data) => {
                if let Err(e) = self.command_sender.send(UdpCommand::Send(data)) {
                    eprintln!("UDP command send failed: {}", e);
                }
            }
            Err(e) => {
                eprintln!("UDP encode failed: {}", e);
            }
        }
    }
}

impl LogHandler for UdpHandler {
    fn handle(&self, record: &Record) {
        self.send_to_worker(record);
    }

    fn flush(&self) {
        // 发送刷新命令
        if let Err(e) = self.command_sender.send(UdpCommand::Flush) {
            eprintln!("UDP flush command send failed: {}", e);
        }
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
        // 发送关闭命令
        let _ = self.command_sender.send(UdpCommand::Shutdown);
        // 等待工作线程结束
        if let Some(thread) = self.worker_thread.take() {
            let _ = thread.join();
        }
    }
}
