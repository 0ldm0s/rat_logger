//! 广播式生产者消费者模式实现
//! 主线程广播消息给所有处理器，每个处理器自己决定是否处理

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use crossbeam_channel::{Sender, Receiver, unbounded};
use std::sync::Mutex;
use once_cell::sync::Lazy;

// 重新导出core模块中的LogCommand
pub use crate::core::LogCommand;

/// 全局工作线程就绪计数器
static WORKER_READY_COUNT: Lazy<std::sync::atomic::AtomicUsize> = Lazy::new(|| {
    std::sync::atomic::AtomicUsize::new(0)
});

/// 全局预期工作线程数量
static EXPECTED_WORKER_COUNT: Lazy<std::sync::atomic::AtomicUsize> = Lazy::new(|| {
    std::sync::atomic::AtomicUsize::new(0)
});

/// 增加就绪计数器
pub fn increment_ready_count() {
    WORKER_READY_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

/// 设置预期工作线程数量
pub fn set_expected_worker_count(count: usize) {
    EXPECTED_WORKER_COUNT.store(count, std::sync::atomic::Ordering::SeqCst);
}

/// 获取就绪工作线程数量
pub fn get_ready_count() -> usize {
    WORKER_READY_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

/// 重置就绪计数器
pub fn reset_ready_count() {
    WORKER_READY_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
}

/// 等待所有工作线程就绪
pub fn wait_for_all_ready(timeout_ms: u64) -> Result<(), String> {
    let expected = EXPECTED_WORKER_COUNT.load(std::sync::atomic::Ordering::SeqCst);
    if expected == 0 {
        return Ok(());
    }

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_ms);

    while start.elapsed() < timeout {
        let ready = get_ready_count();
        if ready >= expected {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let ready = get_ready_count();
    Err(format!("工作线程就绪超时（{}/{}个）", ready, expected))
}

/// 批量处理配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 批量大小阈值（字节）
    pub batch_size: usize,
    /// 批量时间间隔（毫秒）
    pub batch_interval_ms: u64,
    /// 缓冲区大小
    pub buffer_size: usize,
}

impl BatchConfig {
    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), String> {
        // 验证批量大小
        if self.batch_size == 0 {
            return Err("配置错误: 批量大小不能为 0".to_string());
        }
        if self.batch_size > 1024 * 1024 {
            return Err("配置错误: 批量大小过大 (最大 1MB)".to_string());
        }

        // 验证批量间隔
        if self.batch_interval_ms == 0 {
            return Err("配置错误: 批量间隔不能为 0".to_string());
        }
        if self.batch_interval_ms > 60000 {
            return Err("配置错误: 批量间隔过长 (最大 60秒)".to_string());
        }

        // 验证缓冲区大小
        if self.buffer_size == 0 {
            return Err("配置错误: 缓冲区大小不能为 0".to_string());
        }
        if self.buffer_size > 10 * 1024 * 1024 {
            return Err("配置错误: 缓冲区大小过大 (最大 10MB)".to_string());
        }

        // 验证缓冲区大小与批量大小的关系
        if self.buffer_size < self.batch_size {
            return Err(format!("配置错误: 缓冲区大小 ({}) 必须大于等于批量大小 ({})", self.buffer_size, self.batch_size));
        }

        Ok(())
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 2048,           // 2KB - 更保守的批量大小确保可靠输出
            batch_interval_ms: 25,       // 25ms - 更短的间隔确保及时输出
            buffer_size: 16 * 1024,     // 16KB - 相应减小缓冲区大小
        }
    }
}

/// 处理器 trait - 各个处理器实现具体的处理逻辑
pub trait LogProcessor: Send + 'static {
    /// 处理器名称
    fn name(&self) -> &'static str;

    /// 处理单个日志数据
    fn process(&mut self, data: &[u8]) -> Result<(), String>;

    /// 批量处理日志数据 - 保持原有优化逻辑
    fn process_batch(&mut self, batch: &[Vec<u8>]) -> Result<(), String> {
        // 默认实现：逐个处理
        for data in batch {
            if let Err(e) = self.process(data) {
                return Err(e);
            }
        }
        Ok(())
    }

    /// 处理文件轮转命令 - 默认忽略（只有文件处理器需要处理）
    fn handle_rotate(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// 处理文件压缩命令 - 默认忽略（只有文件处理器需要处理）
    fn handle_compress(&mut self, _path: &std::path::Path) -> Result<(), String> {
        Ok(())
    }

    /// 刷新操作
    fn flush(&mut self) -> Result<(), String>;

    /// 清理资源
    fn cleanup(&mut self) -> Result<(), String>;
}

/// 单个处理器的工作线程
pub struct ProcessorWorker {
    sender: Sender<LogCommand>,
    worker_thread: Option<thread::JoinHandle<()>>,
    config: BatchConfig,
    /// 处理器类型名称
    processor_type: String,
}

impl ProcessorWorker {
    /// 创建新的处理器工作线程
    pub fn new<P>(mut processor: P, config: BatchConfig) -> Self
    where
        P: LogProcessor + Send + 'static,
    {
        // 验证配置，如果失败则直接panic，让用户明确知道配置问题
        if let Err(e) = config.validate() {
            panic!("BatchConfig 验证失败: {}\n请检查您的配置并修复上述问题后再重试。", e);
        }

        let (sender, receiver) = unbounded();
        let config_clone = config.clone();
        let processor_name = processor.name();

        let worker_thread = thread::spawn(move || {
            Self::worker_thread(processor, receiver, config_clone, processor_name);
        });

        Self {
            sender,
            worker_thread: Some(worker_thread),
            config,
            processor_type: processor_name.to_string(),
        }
    }

    /// 工作线程实现 - 保持与原有文件处理器相同的批量处理逻辑
    fn worker_thread<P>(
        mut processor: P,
        receiver: Receiver<LogCommand>,
        config: BatchConfig,
        processor_name: &'static str,
    ) where
        P: LogProcessor + Send + 'static,
    {
        eprintln!("DEBUG: [{}] 工作线程启动，配置: batch_size={}, batch_interval_ms={}ms",
                 processor_name, config.batch_size, config.batch_interval_ms);

        // 发送就绪通知
        increment_ready_count();
        eprintln!("DEBUG: [{}] 已发送就绪通知，当前就绪数量: {}", processor_name, get_ready_count());
        let mut batch_buffer = Vec::with_capacity(config.buffer_size);
        let mut last_flush = Instant::now();
        let flush_interval = Duration::from_millis(config.batch_interval_ms);

        while let Ok(command) = receiver.recv() {
            eprintln!("DEBUG: [{}] 收到命令: {:?}", processor_name, command);
            if let LogCommand::Write(ref data) = command {
                eprintln!("DEBUG: [{}] 收到Write命令，数据长度: {}", processor_name, data.len());
            }
            match command {
                LogCommand::Write(data) => {
                    batch_buffer.push(data);

                    // 批量写入条件：达到batch_size或时间间隔
                    if batch_buffer.len() >= config.batch_size ||
                       last_flush.elapsed() >= flush_interval {
                        if let Err(e) = Self::process_batch(&mut processor, &mut batch_buffer) {
                            eprintln!("[{}] 批量处理失败: {}", processor_name, e);
                        }
                        last_flush = Instant::now();
                    }
                }

                LogCommand::Rotate => {
                    // 先处理缓冲区中的数据 - 保持原有逻辑
                    if !batch_buffer.is_empty() {
                        if let Err(e) = Self::process_batch(&mut processor, &mut batch_buffer) {
                            eprintln!("[{}] 轮转前批量处理失败: {}", processor_name, e);
                        }
                        last_flush = Instant::now();
                    }

                    // 处理轮转命令（只有文件处理器会真正处理）
                    if let Err(e) = processor.handle_rotate() {
                        eprintln!("[{}] 处理轮转失败: {}", processor_name, e);
                    }
                }

                LogCommand::Compress(path) => {
                    // 先处理缓冲区中的数据 - 保持原有逻辑
                    if !batch_buffer.is_empty() {
                        if let Err(e) = Self::process_batch(&mut processor, &mut batch_buffer) {
                            eprintln!("[{}] 压缩前批量处理失败: {}", processor_name, e);
                        }
                        last_flush = Instant::now();
                    }

                    // 处理压缩命令（只有文件处理器会真正处理）
                    if let Err(e) = processor.handle_compress(&path) {
                        eprintln!("[{}] 处理压缩失败: {}", processor_name, e);
                    }
                }

                LogCommand::Flush => {
                    // 写入剩余数据 - 保持原有逻辑
                    if !batch_buffer.is_empty() {
                        if let Err(e) = Self::process_batch(&mut processor, &mut batch_buffer) {
                            eprintln!("[{}] 刷新时批量处理失败: {}", processor_name, e);
                        }
                        batch_buffer.clear();
                    }

                    // 调用处理器刷新
                    if let Err(e) = processor.flush() {
                        eprintln!("[{}] 处理器刷新失败: {}", processor_name, e);
                    }
                    last_flush = Instant::now();
                }

                LogCommand::Shutdown(source) => {
                    // 显示Shutdown命令的来源
                    eprintln!("DEBUG: [{}] 收到Shutdown命令，来源: {}", processor_name, source);
                    // 处理剩余数据并退出 - 保持原有逻辑
                    if !batch_buffer.is_empty() {
                        if let Err(e) = Self::process_batch(&mut processor, &mut batch_buffer) {
                            eprintln!("[{}] 关闭时批量处理失败: {}", processor_name, e);
                        }
                    }

                    // 刷新并清理
                    if let Err(e) = processor.flush() {
                        eprintln!("[{}] 关闭时处理器刷新失败: {}", processor_name, e);
                    }
                    if let Err(e) = processor.cleanup() {
                        eprintln!("[{}] 处理器清理失败: {}", processor_name, e);
                    }
                    break;
                }

                LogCommand::HealthCheck(response_sender) => {
                    // 健康检查：立即响应，表示工作线程正常运行
                    let _ = response_sender.send(true);
                }
            }
        }
    }

    /// 处理批量数据
    fn process_batch<P>(processor: &mut P, batch: &mut Vec<Vec<u8>>) -> Result<(), String>
    where
        P: LogProcessor,
    {
        if batch.is_empty() {
            return Ok(());
        }

        let result = processor.process_batch(batch);
        batch.clear(); // 确保缓冲区被清空
        result
    }

    /// 发送写入命令
    pub fn send_write(&self, data: Vec<u8>) -> Result<(), String> {
        let command = LogCommand::Write(data);
        self.sender.send(command)
            .map_err(|e| format!("发送写入命令失败: {}", e))?;
        Ok(())
    }

    /// 发送轮转命令
    pub fn send_rotate(&self) -> Result<(), String> {
        let command = LogCommand::Rotate;
        self.sender.send(command)
            .map_err(|e| format!("发送轮转命令失败: {}", e))?;
        Ok(())
    }

    /// 发送压缩命令
    pub fn send_compress(&self, path: std::path::PathBuf) -> Result<(), String> {
        let command = LogCommand::Compress(path);
        self.sender.send(command)
            .map_err(|e| format!("发送压缩命令失败: {}", e))?;
        Ok(())
    }

    /// 发送刷新命令
    pub fn send_flush(&self) -> Result<(), String> {
        let command = LogCommand::Flush;
        self.sender.send(command)
            .map_err(|e| format!("发送刷新命令失败: {}", e))?;
        Ok(())
    }

    /// 发送停止命令
    pub fn send_shutdown(&self) -> Result<(), String> {
        let command = LogCommand::Shutdown("ProcessorWorker::send_shutdown");
        self.sender.send(command)
            .map_err(|e| format!("发送停止命令失败: {}", e))?;
        Ok(())
    }

    /// 获取发送者（用于高级用法）
    pub fn sender(&self) -> &Sender<LogCommand> {
        &self.sender
    }

    /// 获取批量配置
    pub fn config(&self) -> &BatchConfig {
        &self.config
    }

    /// 获取处理器类型
    pub fn get_processor_type(&self) -> &str {
        &self.processor_type
    }

    /// 执行健康检查，验证工作线程是否正常运行
    pub fn health_check(&self, timeout_ms: u64) -> Result<(), String> {
        let (response_sender, response_receiver) = unbounded();

        // 发送健康检查命令
        let command = LogCommand::HealthCheck(response_sender);
        self.sender.send(command)
            .map_err(|e| format!("发送健康检查命令失败: {}", e))?;

        // 等待响应
        let timeout = Duration::from_millis(timeout_ms);
        match response_receiver.recv_timeout(timeout) {
            Ok(true) => Ok(()),
            Ok(false) => Err("工作线程响应异常".to_string()),
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                Err(format!("健康检查超时（{}ms）", timeout_ms))
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                Err("工作线程已断开连接".to_string())
            }
        }
    }
}

// ProcessorWorker 不应该实现 Clone，因为每个实例代表一个真实的工作线程
// impl Clone for ProcessorWorker {
//     fn clone(&self) -> Self {
//         eprintln!("DEBUG: ProcessorWorker::clone 被调用！这将导致worker_thread被设置为None！");
//         Self {
//             sender: self.sender.clone(),
//             worker_thread: None, // 不克隆工作线程，只克隆发送者
//             config: self.config.clone(),
//             processor_type: self.processor_type.clone(),
//         }
//     }
// }

impl Drop for ProcessorWorker {
    fn drop(&mut self) {
        // 发送停止命令
        let _ = self.sender.send(LogCommand::Shutdown("ProcessorWorker::drop"));

        // 等待工作线程结束
        if let Some(thread) = self.worker_thread.take() {
            let _ = thread.join();
        }
    }
}

/// 处理器管理器 - 管理所有处理器的工作线程
pub struct ProcessorManager {
    workers: Vec<ProcessorWorker>,
    /// 已验证的处理器类型集合
    verified_types: std::collections::HashSet<String>,
}

impl ProcessorManager {
    /// 创建新的处理器管理器
    pub fn new() -> Self {
        Self {
            workers: Vec::new(),
            verified_types: std::collections::HashSet::new(),
        }
    }

    /// 添加处理器
    pub fn add_processor<P>(&mut self, processor: P, config: BatchConfig) -> Result<(), String>
    where
        P: LogProcessor + Send + 'static,
    {
        let processor_type = processor.name().to_string();
        let worker = ProcessorWorker::new(processor, config);
        self.workers.push(worker);

        // 新增处理器类型，需要重新验证
        self.verified_types.remove(&processor_type);

        Ok(())
    }

    /// 广播写入命令给所有处理器
    pub fn broadcast_write(&self, data: Vec<u8>) -> Result<(), String> {
        eprintln!("DEBUG: broadcast_write 被调用，workers数量: {}, 数据长度: {}", self.workers.len(), data.len());
        for (i, worker) in self.workers.iter().enumerate() {
            eprintln!("DEBUG: 发送Write命令给worker {}", i);
            if let Err(e) = worker.send_write(data.clone()) {
                eprintln!("DEBUG: 发送Write命令给worker {} 失败: {}", i, e);
                return Err(e);
            }
            eprintln!("DEBUG: 发送Write命令给worker {} 成功", i);
        }
        eprintln!("DEBUG: broadcast_write 完成");
        Ok(())
    }

    /// 广播轮转命令给所有处理器
    pub fn broadcast_rotate(&self) -> Result<(), String> {
        for worker in &self.workers {
            if let Err(e) = worker.send_rotate() {
                return Err(e);
            }
        }
        Ok(())
    }

    /// 广播压缩命令给所有处理器
    pub fn broadcast_compress(&self, path: std::path::PathBuf) -> Result<(), String> {
        for worker in &self.workers {
            if let Err(e) = worker.send_compress(path.clone()) {
                return Err(e);
            }
        }
        Ok(())
    }

    /// 广播刷新命令给所有处理器
    pub fn broadcast_flush(&self) -> Result<(), String> {
        for worker in &self.workers {
            if let Err(e) = worker.send_flush() {
                return Err(e);
            }
        }
        Ok(())
    }

    /// 广播停止命令给所有处理器
    pub fn broadcast_shutdown(&self, source: &'static str) -> Result<(), String> {
        for worker in &self.workers {
            if let Err(e) = worker.send_shutdown() {
                return Err(e);
            }
        }
        Ok(())
    }

    /// 获取处理器数量
    pub fn len(&self) -> usize {
        self.workers.len()
    }

    /// 智能健康检查：被动等待工作线程就绪通知
    pub fn smart_health_check(&self, timeout_ms: u64) -> Result<Vec<String>, String> {
        // 设置预期的工作线程数量（未验证的处理器类型）
        let mut unverified_count = 0;
        let mut newly_verified = Vec::new();

        for worker in &self.workers {
            let worker_type = worker.get_processor_type();
            if !self.verified_types.contains(worker_type) {
                unverified_count += 1;
                newly_verified.push(worker_type.to_string());
            }
        }

        if unverified_count == 0 {
            return Ok(vec![]); // 没有需要验证的处理器
        }

        // 重置计数器并设置预期数量
        reset_ready_count();
        set_expected_worker_count(unverified_count);

        // 等待所有工作线程就绪
        wait_for_all_ready(timeout_ms)?;

        Ok(newly_verified)
    }

    /// 标记处理器类型为已验证
    pub fn mark_as_verified(&mut self, processor_types: &[String]) {
        for processor_type in processor_types {
            self.verified_types.insert(processor_type.clone());
        }
    }

    /// 检查特定处理器类型的健康状态（简化版，使用智能健康检查）
    pub fn check_specific_types(&self, expected_types: &[String], timeout_ms: u64) -> Result<(), String> {
        eprintln!("DEBUG: check_specific_types 被调用，expected_types: {:?}", expected_types);
        eprintln!("DEBUG: 当前已验证的类型: {:?}", self.verified_types);
        eprintln!("DEBUG: 当前工作线程数量: {}", self.workers.len());

        // 过滤出预期的处理器类型
        let expected_set: std::collections::HashSet<&str> = expected_types.iter().map(|s| s.as_str()).collect();
        let mut expected_workers = Vec::new();

        for worker in &self.workers {
            let worker_type = worker.get_processor_type();
            let is_expected = expected_set.contains(worker_type);
            let is_verified = self.verified_types.contains(worker_type);
            eprintln!("DEBUG: 工作线程类型: {}, 预期: {}, 已验证: {}", worker_type, is_expected, is_verified);

            if is_expected && !is_verified {
                expected_workers.push(worker_type.to_string());
            }
        }

        eprintln!("DEBUG: 预期但未验证的工作线程数量: {}", expected_workers.len());

        if expected_workers.is_empty() {
            eprintln!("DEBUG: 所有预期的处理器都已验证，跳过等待");
            return Ok(()); // 所有预期的处理器都已验证
        }

        // 计算还需要等待的工作线程数量
        let current_ready = get_ready_count();
        let remaining = expected_workers.len().saturating_sub(current_ready);
        eprintln!("DEBUG: 当前已就绪数量: {}, 预期总数: {}, 还需要等待: {}", current_ready, expected_workers.len(), remaining);

        if remaining == 0 {
            eprintln!("DEBUG: 所有工作线程已经就绪，无需等待");
            return Ok(());
        }

        // 设置预期数量并等待剩余的工作线程
        eprintln!("DEBUG: 设置剩余预期数量为: {}", remaining);
        set_expected_worker_count(remaining);

        // 等待所有预期工作线程就绪
        eprintln!("DEBUG: 开始等待剩余工作线程就绪...");
        wait_for_all_ready(timeout_ms)?;

        Ok(())
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }
}

impl Default for ProcessorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ProcessorManager {
    fn drop(&mut self) {
        // 优雅地关闭所有工作线程
        let _ = self.broadcast_shutdown("ProcessorManager::drop");

        // 给每个工作线程一些时间来清理资源
        std::thread::sleep(std::time::Duration::from_millis(100));

        // 清理工作线程
        self.workers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试处理器
    struct TestProcessor {
        name: &'static str,
        processed_data: Vec<Vec<u8>>,
        rotate_count: usize,
        compress_count: usize,
        flush_count: usize,
    }

    impl TestProcessor {
        fn new(name: &'static str) -> Self {
            Self {
                name,
                processed_data: Vec::new(),
                rotate_count: 0,
                compress_count: 0,
                flush_count: 0,
            }
        }
    }

    impl LogProcessor for TestProcessor {
        fn name(&self) -> &'static str {
            self.name
        }

        fn process(&mut self, data: &[u8]) -> Result<(), String> {
            self.processed_data.push(data.to_vec());
            Ok(())
        }

        fn process_batch(&mut self, batch: &[Vec<u8>]) -> Result<(), String> {
            self.processed_data.extend(batch.iter().cloned());
            Ok(())
        }

        fn handle_rotate(&mut self) -> Result<(), String> {
            self.rotate_count += 1;
            Ok(())
        }

        fn handle_compress(&mut self, _path: &std::path::Path) -> Result<(), String> {
            self.compress_count += 1;
            Ok(())
        }

        fn flush(&mut self) -> Result<(), String> {
            self.flush_count += 1;
            Ok(())
        }

        fn cleanup(&mut self) -> Result<(), String> {
            self.processed_data.clear();
            Ok(())
        }
    }

    #[test]
    fn test_processor_worker() {
        let processor = TestProcessor::new("test_worker");
        let config = BatchConfig {
            batch_size: 2,
            batch_interval_ms: 10,
            buffer_size: 10,
        };

        let worker = ProcessorWorker::new(processor, config);

        // 发送数据
        worker.send_write(b"test1".to_vec()).unwrap();
        worker.send_write(b"test2".to_vec()).unwrap();

        // 发送轮转命令
        worker.send_rotate().unwrap();

        // 发送压缩命令
        worker.send_compress(std::path::PathBuf::from("test.log")).unwrap();

        // 发送刷新命令
        worker.send_flush().unwrap();

        std::thread::sleep(Duration::from_millis(50));

        // 注意：由于是异步处理，实际测试中需要其他方式验证
    }

    #[test]
    fn test_processor_manager() {
        let mut manager = ProcessorManager::new();

        // 添加多个处理器
        let config = BatchConfig::default();
        manager.add_processor(TestProcessor::new("processor1"), config.clone()).unwrap();
        manager.add_processor(TestProcessor::new("processor2"), config).unwrap();

        // 广播写入命令
        manager.broadcast_write(b"test_data".to_vec()).unwrap();

        // 广播轮转命令
        manager.broadcast_rotate().unwrap();

        // 广播刷新命令
        manager.broadcast_flush().unwrap();

        std::thread::sleep(Duration::from_millis(50));

        assert_eq!(manager.len(), 2);
    }
}