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
    file_config: FileConfig,
    writer: Arc<Mutex<LogWriter>>,
    rotator: Arc<LogRotator>,
    formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync>,
}

impl FileProcessor {
    /// 创建新的文件处理器
    pub fn new(config: FileConfig) -> Self {
        // 验证配置，如果失败则直接panic，让用户明确知道配置问题
        if let Err(e) = config.validate() {
            panic!("FileConfig 验证失败: {}\n请检查您的配置并修复上述问题后再重试。", e);
        }

        let writer = Arc::new(Mutex::new(
            LogWriter::new(&config.log_dir, config.max_file_size as usize, config.force_sync)
                .unwrap_or_else(|_| LogWriter::create_default(&config.log_dir, config.max_file_size as usize, config.force_sync))
        ));

        let rotator = Arc::new(LogRotator::new(config.log_dir.clone(), config.max_compressed_files));

        // 根据配置设置格式化器，原始模式下使用原始格式
        let formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync> =
            if config.is_raw {
                Box::new(Self::raw_format)
            } else if let Some(format_config) = &config.format {
                let format_config = format_config.clone();
                Box::new(move |buf, record| {
                    Self::format_with_config(buf, record, &format_config)
                })
            } else {
                Box::new(Self::default_format)
            };

        Self {
            file_config: config,
            writer,
            rotator,
            formatter,
        }
    }

    
    
    
    
    /// 执行日志轮转
    fn perform_rotation(&self) -> Result<(), String> {
        let old_path = {
            let writer_guard = self.writer.lock();
            writer_guard.current_path.clone()
        };

        if !old_path.as_os_str().is_empty() {
            // Flush并关闭当前文件
            {
                let mut writer_guard = self.writer.lock();
                if let Some(mut file) = writer_guard.current_file.take() {
                    if let Err(e) = file.flush() {
                        eprintln!("[file] 轮转前刷新失败: {}", e);
                    }
                    drop(file);
                }
            }

            let new_path = self.rotator.next_path();
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
                let mut writer_guard = self.writer.lock();
                writer_guard.current_file = Some(BufWriter::new(new_file));
                writer_guard.current_path = new_path;
                writer_guard.current_size = 0;
            }

            // 异步压缩旧文件
            if old_path.exists() {
                let log_dir = self.file_config.log_dir.clone();
                let max_compressed_files = self.file_config.max_compressed_files;
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

            self.rotator.cleanup_old_files();
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

impl LogProcessor for FileProcessor {
    fn name(&self) -> &'static str {
        "file_processor"
    }

    fn process(&mut self, data: &[u8]) -> Result<(), String> {
        // 反序列化日志记录
        let record = bincode::decode_from_slice::<Record, _>(data, bincode::config::standard())
            .map_err(|e| format!("反序列化失败: {}", e))?.0;

  
        // 根据配置决定是否跳过服务端自身日志
        if self.file_config.skip_server_logs && record.metadata.app_id.is_none() {
            return Ok(());
        }

        // 格式化日志记录
        let formatted_data = self.format_record(&record)?;

        // 直接写入文件
        {
            let mut writer_guard = self.writer.lock();
            if let Err(e) = writer_guard.write_direct(&formatted_data) {
                return Err(format!("文件写入失败: {}", e));
            }
        }

        // 检查是否需要轮转
        {
            let writer_guard = self.writer.lock();
            if writer_guard.current_size >= writer_guard.max_size {
                drop(writer_guard);
                self.perform_rotation()?;
            }
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
            if self.file_config.skip_server_logs && record.metadata.app_id.is_none() {
                continue;
            }

            let formatted_data = self.format_record(&record)?;
            all_data.extend_from_slice(&formatted_data);
        }

        if all_data.is_empty() {
            return Ok(());
        }

        // 批量写入文件
        {
            let mut writer_guard = self.writer.lock();
            if let Err(e) = writer_guard.write_direct(&all_data) {
                return Err(format!("批量写入文件失败: {}", e));
            }

            // 检查是否需要轮转
            if writer_guard.current_size >= writer_guard.max_size {
                drop(writer_guard);
                self.perform_rotation()?;
            }
        }

        Ok(())
    }

    fn handle_rotate(&mut self) -> Result<(), String> {
        self.perform_rotation()
    }

    fn handle_compress(&mut self, path: &Path) -> Result<(), String> {
        // 直接执行压缩
        let path = path.to_path_buf();
        let log_dir = self.file_config.log_dir.clone();
        let max_compressed_files = self.file_config.max_compressed_files;
        COMPRESSION_POOL.execute(move || {
            if let Err(e) = Self::compress_file(&path, &log_dir, max_compressed_files) {
                eprintln!("[file] 压缩失败 {}: {}", path.display(), e);
            }
        });
        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        let mut writer_guard = self.writer.lock();
        if let Err(e) = writer_guard.sync_all() {
            return Err(format!("文件同步失败: {}", e));
        }
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), String> {
        // 先刷新剩余数据
        self.flush()?;
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
    fn new(base_path: &Path, max_size: usize, force_sync: bool) -> io::Result<Self> {
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
            aggressive_sync: force_sync || !cfg!(windows), // 优先使用用户配置
        })
    }

    fn create_default(base_path: &Path, max_size: usize, force_sync: bool) -> Self {
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
            aggressive_sync: force_sync || !cfg!(windows), // 优先使用用户配置
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

    /// 直接写入数据（不批量处理）
    fn write_direct(&mut self, data: &[u8]) -> io::Result<()> {
        if let Some(file) = &mut self.current_file {
            file.write_all(data)?;
            self.current_size += data.len();
            file.flush()?; // 直接刷新确保数据写入

            // 如果配置为强制同步，则立即同步到磁盘
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