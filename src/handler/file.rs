//! 文件日志处理器 - 高性能异步架构

use std::io::{self, Write, BufWriter};
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::{Duration, Instant};
use crossbeam_channel::{Sender, Receiver, unbounded};
use std::thread;

use crate::producer_consumer::LogProcessor;
use crate::config::{Record, FileConfig, FormatConfig, Level};

/// 全局压缩线程池
lazy_static::lazy_static! {
    static ref COMPRESSION_POOL: threadpool::ThreadPool = {
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        threadpool::ThreadPool::new(num_threads.max(1))
    };
}

/// 日志文件写入器
struct LogWriter {
    current_file: Option<BufWriter<File>>,
    current_path: PathBuf,
    max_size: usize,
    current_size: usize,
    last_flush: Instant,
    flush_interval: Duration,
    aggressive_sync: bool,
}

/// 日志轮转器
struct LogRotator {
    base_path: PathBuf,
    max_files: usize,
}

/// 文件处理器配置
#[derive(Debug, Clone)]
pub struct FileProcessorConfig {
    /// 文件配置
    pub file_config: FileConfig,
    /// 批量大小
    pub batch_size: usize,
    /// 刷新间隔（毫秒）
    pub flush_interval_ms: u64,
}

impl Default for FileProcessorConfig {
    fn default() -> Self {
        Self {
            file_config: FileConfig::default(),
            batch_size: 8192,  // 8KB批量写入
            flush_interval_ms: 100, // 100ms刷新间隔
        }
    }
}

/// 文件日志处理器 - 实现LogProcessor trait
pub struct FileProcessor {
    config: FileProcessorConfig,
    writer: Arc<Mutex<LogWriter>>,
    rotator: Arc<LogRotator>,
    buffer: Arc<Mutex<Vec<u8>>>,
    last_flush: Arc<Mutex<Instant>>,
    command_sender: Sender<crate::producer_consumer::LogCommand>,
    writer_thread: Option<thread::JoinHandle<()>>,
    formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync>,
}

impl FileProcessor {
    /// 创建新的文件处理器
    pub fn new(config: FileConfig) -> Self {
        let processor_config = FileProcessorConfig {
            file_config: config,
            ..Default::default()
        };
        Self::with_config(processor_config)
    }

    /// 使用配置创建文件处理器
    pub fn with_config(config: FileProcessorConfig) -> Self {
        // 验证配置，如果失败则直接panic，让用户明确知道配置问题
        if let Err(e) = config.file_config.validate() {
            panic!("FileConfig 验证失败: {}\n请检查您的配置并修复上述问题后再重试。", e);
        }

        let writer = Arc::new(Mutex::new(
            LogWriter::new(&config.file_config.log_dir, config.file_config.max_file_size as usize)
                .unwrap_or_else(|_| LogWriter::create_default(&config.file_config.log_dir, config.file_config.max_file_size as usize))
        ));

        let rotator = Arc::new(LogRotator::new(config.file_config.log_dir.clone(), config.file_config.max_compressed_files));

        let (sender, receiver) = unbounded();
        let writer_clone = Arc::clone(&writer);
        let rotator_clone = Arc::clone(&rotator);
        let config_clone = config.clone();

        let writer_thread = thread::spawn(move || {
            Self::worker_thread(writer_clone, rotator_clone, receiver, config_clone);
        });

        // 根据配置设置格式化器，原始模式下使用原始格式
        let formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync> =
            if config.file_config.is_raw {
                Box::new(Self::raw_format)
            } else if let Some(format_config) = &config.file_config.format {
                let format_config = format_config.clone();
                Box::new(move |buf, record| {
                    Self::format_with_config(buf, record, &format_config)
                })
            } else {
                Box::new(Self::default_format)
            };

        Self {
            config,
            writer,
            rotator,
            buffer: Arc::new(Mutex::new(Vec::with_capacity(8192))),
            last_flush: Arc::new(Mutex::new(Instant::now())),
            command_sender: sender,
            writer_thread: Some(writer_thread),
            formatter,
        }
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

    /// 工作线程 - 处理所有文件操作
    fn worker_thread(
        writer: Arc<Mutex<LogWriter>>,
        rotator: Arc<LogRotator>,
        receiver: Receiver<crate::producer_consumer::LogCommand>,
        config: FileProcessorConfig,
    ) {
        let mut batch_buffer = Vec::with_capacity(config.batch_size);
        let mut last_flush = Instant::now();
        let flush_interval = Duration::from_millis(config.flush_interval_ms);

        while let Ok(command) = receiver.recv() {
            match command {
                crate::producer_consumer::LogCommand::Write(data) => {
                    batch_buffer.extend_from_slice(&data);

                    // 批量写入条件：达到8KB或100ms间隔
                    if batch_buffer.len() >= config.batch_size || last_flush.elapsed() >= flush_interval {
                        {
                            let mut writer_guard = writer.lock();
                            if let Err(e) = writer_guard.write_batch(&batch_buffer) {
                                eprintln!("[file] 批量写入失败: {}", e);
                            }
                        }
                        batch_buffer.clear();
                        last_flush = Instant::now();
                    }

                    // 检查是否需要轮转
                    {
                        let writer_guard = writer.lock();
                        if writer_guard.current_size >= writer_guard.max_size {
                            drop(writer_guard);
                            Self::handle_rotation(&writer, &rotator, &config.file_config);
                        }
                    }
                }

                crate::producer_consumer::LogCommand::Rotate => {
                    // 先处理缓冲区中的数据
                    if !batch_buffer.is_empty() {
                        {
                            let mut writer_guard = writer.lock();
                            if let Err(e) = writer_guard.write_batch(&batch_buffer) {
                                eprintln!("[file] 轮转前批量写入失败: {}", e);
                            }
                        }
                        batch_buffer.clear();
                    }

                    Self::handle_rotation(&writer, &rotator, &config.file_config);
                    last_flush = Instant::now();
                }

                crate::producer_consumer::LogCommand::Compress(path) => {
                    // 先处理缓冲区中的数据
                    if !batch_buffer.is_empty() {
                        {
                            let mut writer_guard = writer.lock();
                            if let Err(e) = writer_guard.write_batch(&batch_buffer) {
                                eprintln!("[file] 压缩前批量写入失败: {}", e);
                            }
                        }
                        batch_buffer.clear();
                    }

                    Self::handle_compression(path, &config.file_config);
                    last_flush = Instant::now();
                }

                crate::producer_consumer::LogCommand::Flush => {
                    // 写入剩余数据
                    if !batch_buffer.is_empty() {
                        {
                            let mut writer_guard = writer.lock();
                            if let Err(e) = writer_guard.write_batch(&batch_buffer) {
                                eprintln!("[file] 刷新时批量写入失败: {}", e);
                            }
                        }
                        batch_buffer.clear();
                    }

                    {
                        let mut writer_guard = writer.lock();
                        if let Err(e) = writer_guard.sync_all() {
                            eprintln!("[file] 同步失败: {}", e);
                        }
                    }
                    last_flush = Instant::now();
                }

                crate::producer_consumer::LogCommand::Shutdown => {
                    // 处理剩余数据并退出
                    if !batch_buffer.is_empty() {
                        {
                            let mut writer_guard = writer.lock();
                            if let Err(e) = writer_guard.write_batch(&batch_buffer) {
                                eprintln!("[file] 关闭时批量写入失败: {}", e);
                            }
                        }
                    }
                    {
                        let mut writer_guard = writer.lock();
                        if let Err(e) = writer_guard.sync_all() {
                            eprintln!("[file] 关闭时同步失败: {}", e);
                        }
                    }
                    break;
                }

                crate::producer_consumer::LogCommand::HealthCheck(response_sender) => {
                    // 健康检查：立即响应，表示工作线程正常运行
                    let _ = response_sender.send(true);
                }
            }
        }
    }

    /// 处理日志轮转
    fn handle_rotation(writer: &Arc<Mutex<LogWriter>>, rotator: &Arc<LogRotator>, config: &FileConfig) {
        let old_path = {
            let mut writer_guard = writer.lock();
            writer_guard.current_path.clone()
        };

        if !old_path.as_os_str().is_empty() {
            // Flush并关闭当前文件
            {
                let mut writer_guard = writer.lock();
                if let Some(mut file) = writer_guard.current_file.take() {
                    if let Err(e) = file.flush() {
                        eprintln!("[file] 轮转前刷新失败: {}", e);
                    }
                    drop(file);
                }
            }

            let new_path = rotator.next_path();
            let new_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&new_path)
                .unwrap_or_else(|_| {
                    eprintln!("[file] 无法创建新日志文件: {}", new_path.display());
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&new_path)
                        .expect("无法恢复日志文件创建")
                });

            {
                let mut writer_guard = writer.lock();
                writer_guard.current_file = Some(BufWriter::new(new_file));
                writer_guard.current_path = new_path;
                writer_guard.current_size = 0;
            }

            // 异步压缩旧文件
            if old_path.exists() {
                let log_dir = config.log_dir.clone();
                let max_compressed_files = config.max_compressed_files;
                COMPRESSION_POOL.execute(move || {
                    if let Err(e) = Self::compress_file(&old_path, &log_dir, max_compressed_files) {
                        eprintln!("[file] 压缩失败 {}: {}", old_path.display(), e);
                    } else {
                        // 重试删除原文件
                        for attempt in 0..5 {
                            match std::fs::remove_file(&old_path) {
                                Ok(_) => break,
                                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                                    let delay = if cfg!(windows) { 200 } else { 100 };
                                    thread::sleep(Duration::from_millis(delay * (attempt + 1)));
                                    continue;
                                }
                                Err(e) => {
                                    eprintln!("[file] 删除原文件失败 {}: {}", old_path.display(), e);
                                    break;
                                }
                            }
                        }
                    }
                });
            }

            rotator.cleanup_old_files();
        }
    }

    /// 处理文件压缩
    fn handle_compression(path: PathBuf, config: &FileConfig) {
        let log_dir = config.log_dir.clone();
        let max_compressed_files = config.max_compressed_files;
        COMPRESSION_POOL.execute(move || {
            if let Err(e) = Self::compress_file(&path, &log_dir, max_compressed_files) {
                eprintln!("[file] 压缩失败 {}: {}", path.display(), e);
            }
        });
    }

    /// 压缩文件
    fn compress_file(src: &Path, base_path: &Path, max_files: usize) -> io::Result<()> {
        let mut input = std::fs::File::open(src)?;
        let compressed_path = src.with_extension("log.lz4");
        let output = std::fs::File::create(&compressed_path)?;

        let mut encoder = lz4::EncoderBuilder::new()
            .build(output)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        std::io::copy(&mut input, &mut encoder)?;
        encoder.finish().1?;

        // 清理旧文件
        let rotator = LogRotator {
            base_path: base_path.to_path_buf(),
            max_files,
        };
        rotator.cleanup_old_files();

        Ok(())
    }
}

impl LogProcessor for FileProcessor {
    fn name(&self) -> &'static str {
        "file_processor"
    }

    fn process(&mut self, data: &[u8]) -> Result<(), String> {
  
        // 反序列化日志记录
        let record = bincode::decode_from_slice::<Record, _>(data, bincode::config::standard())
            .map_err(|e| format!("反序列化失败: {}", e))?.0;

  
        // 根据配置决定是否跳过服务端自身日志
        if self.config.file_config.skip_server_logs && record.metadata.app_id.is_none() {
            return Ok(());
        }

        // 格式化日志记录
        let formatted_data = self.format_record(&record)?;

        // 写入缓冲区
        let mut buffer_guard = self.buffer.lock();
        buffer_guard.extend_from_slice(&formatted_data);

        // 检查是否需要发送
        let should_send = buffer_guard.len() >= self.config.batch_size ||
                          {
                              let last_flush_guard = self.last_flush.lock();
                              last_flush_guard.elapsed() >= Duration::from_millis(self.config.flush_interval_ms)
                          };


        if should_send {
            let data_to_send = buffer_guard.clone();
            drop(buffer_guard);
            self.command_sender.send(crate::producer_consumer::LogCommand::Write(data_to_send))
                .map_err(|e| format!("发送写入命令失败: {}", e))?;

            // 清空缓冲区
            let mut buffer_guard = self.buffer.lock();
            buffer_guard.clear();

            // 更新最后刷新时间
            let mut last_flush_guard = self.last_flush.lock();
            *last_flush_guard = Instant::now();
        }

        Ok(())
    }

    fn process_batch(&mut self, batch: &[Vec<u8>]) -> Result<(), String> {
        let mut all_data = Vec::new();

        // 批量反序列化和格式化
        for data in batch {
            let record = bincode::decode_from_slice::<Record, _>(data, bincode::config::standard())
                .map_err(|e| format!("批量反序列化失败: {}", e))?.0;

            // 根据配置决定是否跳过服务端自身日志
            if self.config.file_config.skip_server_logs && record.metadata.app_id.is_none() {
                continue;
            }

            let formatted_data = self.format_record(&record)?;
            all_data.extend_from_slice(&formatted_data);
        }

        if all_data.is_empty() {
            return Ok(());
        }

        // 批量写入
        self.command_sender.send(crate::producer_consumer::LogCommand::Write(all_data))
            .map_err(|e| format!("发送批量写入命令失败: {}", e))?;

        // 更新最后刷新时间
        let mut last_flush_guard = self.last_flush.lock();
        *last_flush_guard = Instant::now();

        Ok(())
    }

    fn handle_rotate(&mut self) -> Result<(), String> {
        self.command_sender.send(crate::producer_consumer::LogCommand::Rotate)
            .map_err(|e| format!("发送轮转命令失败: {}", e))?;
        Ok(())
    }

    fn handle_compress(&mut self, path: &Path) -> Result<(), String> {
        self.command_sender.send(crate::producer_consumer::LogCommand::Compress(path.to_path_buf()))
            .map_err(|e| format!("发送压缩命令失败: {}", e))?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        // 发送缓冲区中的所有数据
        {
            let buffer_guard = self.buffer.lock();
            if !buffer_guard.is_empty() {
                let data_to_send = buffer_guard.clone();
                drop(buffer_guard);
                self.command_sender.send(crate::producer_consumer::LogCommand::Write(data_to_send))
                    .map_err(|e| format!("发送刷新写入命令失败: {}", e))?;
            }
        }

        // 发送刷新命令
        self.command_sender.send(crate::producer_consumer::LogCommand::Flush)
            .map_err(|e| format!("发送刷新命令失败: {}", e))?;

        // 更新最后刷新时间
        let mut last_flush_guard = self.last_flush.lock();
        *last_flush_guard = Instant::now();

        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), String> {
        // 先刷新剩余数据
        self.flush()?;

        // 发送停止命令
        let _ = self.command_sender.send(crate::producer_consumer::LogCommand::Shutdown);

        // 等待工作线程结束
        if let Some(thread) = self.writer_thread.take() {
            let _ = thread.join();
        }

        Ok(())
    }
}

impl Drop for FileProcessor {
    fn drop(&mut self) {
        // 清理时会自动调用cleanup
        let _ = self.cleanup();
    }
}

impl LogWriter {
    fn new(base_path: &Path, max_size: usize) -> io::Result<Self> {
        if let Some(parent) = base_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let path = LogRotator::new_path(base_path);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Self {
            current_file: Some(BufWriter::new(file)),
            current_path: path,
            max_size,
            current_size: 0,
            last_flush: Instant::now(),
            flush_interval: Duration::from_millis(100),
            aggressive_sync: !cfg!(windows), // Windows默认不使用强同步
        })
    }

    fn create_default(base_path: &Path, max_size: usize) -> Self {
        let path = LogRotator::new_path(base_path);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap_or_else(|_| {
                std::fs::create_dir_all(base_path.parent().unwrap_or(Path::new("."))).unwrap();
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .unwrap()
            });

        Self {
            current_file: Some(BufWriter::new(file)),
            current_path: path,
            max_size,
            current_size: 0,
            last_flush: Instant::now(),
            flush_interval: Duration::from_millis(100),
            aggressive_sync: !cfg!(windows), // Windows默认不使用强同步
        }
    }

    /// 批量写入数据
    fn write_batch(&mut self, data: &[u8]) -> io::Result<()> {
        if let Some(file) = &mut self.current_file {
            file.write_all(data)?;
            self.current_size += data.len();

            // 定期flush到操作系统缓冲区，避免频繁sync到磁盘
            if self.last_flush.elapsed() >= self.flush_interval {
                file.flush()?;
                self.last_flush = Instant::now();
            }
        }
        Ok(())
    }

    /// 立即刷新并同步到磁盘
    fn sync_all(&mut self) -> io::Result<()> {
        if let Some(file) = &mut self.current_file {
            file.flush()?;

            // 根据配置和平台选择同步策略
            if self.aggressive_sync {
                #[cfg(windows)]
                {
                    // Windows上使用更轻量的同步方式
                    file.get_mut().sync_data()?;
                }
                #[cfg(not(windows))]
                {
                    file.get_mut().sync_all()?;
                }
            } else {
                // 只flush到操作系统缓冲区，让系统决定何时写入磁盘
                // 这样在Windows上有更好的性能
            }
        }
        Ok(())
    }
}

impl LogRotator {
    fn new(base_path: PathBuf, max_files: usize) -> Self {
        Self { base_path, max_files }
    }

    fn next_path(&self) -> PathBuf {
        Self::new_path(&self.base_path)
    }

    fn new_path(base_path: &Path) -> PathBuf {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let dir = base_path;
        std::fs::create_dir_all(dir).unwrap_or(());
        dir.join(format!("app_{}.log", timestamp))
    }

    fn cleanup_old_files(&self) {
        let dir_path = self.base_path.parent().unwrap_or_else(|| Path::new("."));
        if !dir_path.exists() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(dir_path) {
            let mut files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    path.extension().map_or(false, |ext|
                        ext == "log" || ext == "lz4"
                    )
                })
                .collect();

            files.sort_by(|a, b| {
                let a_time = a.metadata().ok()
                    .and_then(|m| m.modified().ok());
                let b_time = b.metadata().ok()
                    .and_then(|m| m.modified().ok());
                a_time.cmp(&b_time)
            });

            while files.len() > self.max_files {
                if let Some(oldest) = files.first() {
                    if let Err(e) = std::fs::remove_file(oldest.path()) {
                        eprintln!("[file] 删除旧日志文件失败: {}", e);
                    }
                    files.remove(0);
                }
            }
        }
    }
}

impl FileProcessor {
    /// 格式化日志记录
    fn format_record(&self, record: &Record) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        (self.formatter)(&mut buf, record)
            .map_err(|e| format!("格式化失败: {}", e))?;
        Ok(buf)
    }

    /// 默认格式化函数
    fn default_format(buf: &mut dyn Write, record: &Record) -> io::Result<()> {
        use chrono::Local;

        let now = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S%.3f");

        writeln!(
            buf,
            "{} [{}] {} {}:{} - {}",
            timestamp,
            record.metadata.level,
            record.metadata.target,
            record.file.as_deref().unwrap_or("unknown"),
            record.line.unwrap_or(0),
            record.args
        )
    }

    /// 原始格式化函数 - 直接输出日志消息，不添加任何格式
    fn raw_format(buf: &mut dyn Write, record: &Record) -> io::Result<()> {
        writeln!(buf, "{}", record.args)
    }

    /// 设置自定义格式化函数
    pub fn with_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync + 'static,
    {
        self.formatter = Box::new(formatter);
        self
    }

    /// 使用格式配置
    pub fn with_format(mut self, format_config: FormatConfig) -> Self {
        let format_config = format_config.clone();
        self.formatter = Box::new(move |buf, record| Self::format_with_config(buf, record, &format_config));
        self
    }

    /// 使用格式配置的格式化函数
    fn format_with_config(buf: &mut dyn Write, record: &Record, format_config: &FormatConfig) -> io::Result<()> {
        use chrono::Local;

        let now = Local::now();
        let timestamp = now.format(&format_config.timestamp_format);

        // 获取级别显示文本
        let level_text = match record.metadata.level {
            Level::Error => &format_config.level_style.error,
            Level::Warn => &format_config.level_style.warn,
            Level::Info => &format_config.level_style.info,
            Level::Debug => &format_config.level_style.debug,
            Level::Trace => &format_config.level_style.trace,
        };

        // 使用格式模板
        let formatted = format_config.format_template
            .replace("{timestamp}", &timestamp.to_string())
            .replace("{level}", level_text)
            .replace("{target}", &record.metadata.target)
            .replace("{file}", record.file.as_deref().unwrap_or("unknown"))
            .replace("{line}", &record.line.unwrap_or(0).to_string())
            .replace("{message}", &record.args);

        writeln!(buf, "{}", formatted)
    }
}