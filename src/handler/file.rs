//! 文件日志处理器

use std::io::{self, Write, BufWriter};
use std::any::Any;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::{Duration, Instant};
use crossbeam_channel::{Sender, Receiver, unbounded};
use std::thread;

use crate::handler::{LogHandler, HandlerType};
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

/// 指令枚举 - 生产者消费者模式
enum FileCommand {
    /// 写入日志数据
    Write(Vec<u8>),
    /// 强制刷新到磁盘
    Flush,
    /// 轮转日志文件
    Rotate,
    /// 压缩指定文件
    Compress(PathBuf),
    /// 停止工作线程
    Shutdown,
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

/// 文件日志处理器
pub struct FileHandler {
    config: FileConfig,
    command_sender: Sender<FileCommand>,
    writer_thread: Option<thread::JoinHandle<()>>,
    formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync>,
}

impl FileHandler {
    /// 创建新的文件处理器
    pub fn new(config: FileConfig) -> Self {
        let (command_sender, command_receiver) = unbounded();

        let config_clone = config.clone();
        let writer_thread = thread::spawn(move || {
            Self::worker_thread(config_clone, command_receiver);
        });

        Self {
            config,
            command_sender,
            writer_thread: Some(writer_thread),
            formatter: Box::new(default_format),
        }
    }

    /// 工作线程 - 处理所有文件操作
    fn worker_thread(config: FileConfig, receiver: Receiver<FileCommand>) {
        let mut writer = LogWriter::new(&config.log_dir, config.max_file_size as usize)
            .unwrap_or_else(|_| LogWriter::create_default(&config.log_dir, config.max_file_size as usize));

        let rotator = LogRotator::new(config.log_dir.clone(), config.max_compressed_files);
        let mut batch_buffer = Vec::with_capacity(64 * 1024); // 64KB批量缓冲区
        let mut last_flush = Instant::now();
        let flush_interval = Duration::from_millis(100);

        while let Ok(command) = receiver.recv() {
            match command {
                FileCommand::Write(data) => {
                    batch_buffer.extend_from_slice(&data);

                    // 批量写入条件：达到8KB或100ms间隔
                    if batch_buffer.len() >= 8192 || last_flush.elapsed() >= flush_interval {
                        if let Err(e) = writer.write_batch(&batch_buffer) {
                            eprintln!("批量写入失败: {}", e);
                        }
                        batch_buffer.clear();
                        last_flush = Instant::now();
                    }

                    // 检查是否需要轮转
                    if writer.current_size >= writer.max_size {
                        Self::handle_rotation(&mut writer, &rotator, &config);
                    }
                }

                FileCommand::Flush => {
                    // 写入剩余数据
                    if !batch_buffer.is_empty() {
                        if let Err(e) = writer.write_batch(&batch_buffer) {
                            eprintln!("最终批量写入失败: {}", e);
                        }
                        batch_buffer.clear();
                    }

                    if let Err(e) = writer.sync_all() {
                        eprintln!("同步失败: {}", e);
                    }
                }

                FileCommand::Rotate => {
                    Self::handle_rotation(&mut writer, &rotator, &config);
                }

                FileCommand::Compress(path) => {
                    Self::handle_compression(path, &config);
                }

                FileCommand::Shutdown => {
                    // 处理剩余数据并退出
                    if !batch_buffer.is_empty() {
                        if let Err(e) = writer.write_batch(&batch_buffer) {
                            eprintln!("关闭时批量写入失败: {}", e);
                        }
                    }
                    if let Err(e) = writer.sync_all() {
                        eprintln!("关闭时同步失败: {}", e);
                    }
                    break;
                }
            }
        }
    }

    /// 处理日志轮转
    fn handle_rotation(writer: &mut LogWriter, rotator: &LogRotator, config: &FileConfig) {
        let old_path = writer.current_path.clone();
        if !old_path.as_os_str().is_empty() {
            // Flush并关闭当前文件
            if let Some(mut file) = writer.current_file.take() {
                if let Err(e) = file.flush() {
                    eprintln!("轮转前刷新失败: {}", e);
                }
                drop(file);
            }

            let new_path = rotator.next_path();
            let new_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&new_path)
                .unwrap_or_else(|_| {
                    eprintln!("无法创建新日志文件: {}", new_path.display());
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&new_path)
                        .expect("无法恢复日志文件创建")
                });

            writer.current_file = Some(BufWriter::new(new_file));
            writer.current_path = new_path;
            writer.current_size = 0;

            // 异步压缩旧文件
            if old_path.exists() {
                let log_dir = config.log_dir.clone();
                let max_compressed_files = config.max_compressed_files;
                COMPRESSION_POOL.execute(move || {
                    if let Err(e) = Self::compress_file(&old_path, &log_dir, max_compressed_files) {
                        eprintln!("压缩失败 {}: {}", old_path.display(), e);
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
                                    eprintln!("删除原文件失败 {}: {}", old_path.display(), e);
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
                eprintln!("压缩失败 {}: {}", path.display(), e);
            }
        });
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
        self.formatter = Box::new(move |buf, record| file_format_with_config(buf, record, &format_config));
        self
    }

    /// 强制压缩当前日志文件
    pub fn force_compress(&self) -> io::Result<()> {
        // 发送轮转指令到工作线程
        let command = FileCommand::Rotate;
        if let Err(e) = self.command_sender.send(command) {
            eprintln!("发送轮转指令失败: {}", e);
        }
        Ok(())
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

impl LogHandler for FileHandler {
    fn handle(&self, record: &Record) {
        // 根据配置决定是否跳过服务端自身日志
        if self.config.skip_server_logs && record.metadata.app_id.is_none() {
            return;
        }

        let mut buf = Vec::new();
        if let Err(e) = (self.formatter)(&mut buf, record) {
            eprintln!("文件格式化错误: {}", e);
            return;
        }

        // 发送写入指令到工作线程
        let command = FileCommand::Write(buf);
        if let Err(e) = self.command_sender.send(command) {
            eprintln!("发送写入指令失败: {}", e);
        }
    }

    fn flush(&self) {
        // 发送刷新指令到工作线程
        let command = FileCommand::Flush;
        if let Err(e) = self.command_sender.send(command) {
            eprintln!("发送刷新指令失败: {}", e);
        }
    }

    fn handler_type(&self) -> HandlerType {
        HandlerType::File
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for FileHandler {
    fn drop(&mut self) {
        // 发送停止指令到工作线程
        let _ = self.command_sender.send(FileCommand::Shutdown);

        // 等待工作线程结束
        if let Some(thread) = self.writer_thread.take() {
            let _ = thread.join();
        }
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
                        eprintln!("Failed to remove old log file: {}", e);
                    }
                    files.remove(0);
                }
            }
        }
    }
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

/// 文件格式化函数
fn file_format_with_config(buf: &mut dyn Write, record: &Record, format_config: &FormatConfig) -> io::Result<()> {
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
        .replace("{level}", level_text)
        .replace("{timestamp}", &timestamp.to_string())
        .replace("{target}", &record.metadata.target)
        .replace("{file}", record.file.as_deref().unwrap_or("unknown"))
        .replace("{line}", &record.line.unwrap_or(0).to_string())
        .replace("{message}", &record.args);

    writeln!(buf, "{}", formatted)
}
