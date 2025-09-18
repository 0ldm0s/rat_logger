//! 文件日志处理器

use std::io::{self, Write};
use std::any::Any;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::Mutex;

use crate::handler::{LogHandler, HandlerType};
use crate::config::{Record, FileConfig};

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
    current_file: Option<File>,
    current_path: PathBuf,
    max_size: usize,
    current_size: usize,
}

/// 日志轮转器
struct LogRotator {
    base_path: PathBuf,
    max_files: usize,
}

/// 文件日志处理器
pub struct FileHandler {
    config: FileConfig,
    writer: Arc<Mutex<LogWriter>>,
    rotator: LogRotator,
    rotate_lock: Arc<Mutex<()>>,
    formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync>,
}

impl FileHandler {
    /// 创建新的文件处理器
    pub fn new(config: FileConfig) -> Self {
        let writer = Arc::new(Mutex::new(
            LogWriter::new(&config.log_dir, config.max_file_size as usize)
                .unwrap_or_else(|_| {
                    LogWriter::create_default(&config.log_dir, config.max_file_size as usize)
                })
        ));

        Self {
            config: config.clone(),
            writer,
            rotator: LogRotator::new(config.log_dir.clone(), config.max_compressed_files),
            rotate_lock: Arc::new(Mutex::new(())),
            formatter: Box::new(default_format),
        }
    }

    /// 设置自定义格式化函数
    pub fn with_formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync + 'static,
    {
        self.formatter = Box::new(formatter);
        self
    }

    /// 强制压缩当前日志文件
    pub fn force_compress(&self) -> io::Result<()> {
        let mut writer = self.writer.lock();
        if writer.current_size > 0 {
            self.rotate(&mut writer)?;
        }
        Ok(())
    }

    /// 轮转日志文件
    fn rotate(&self, writer: &mut LogWriter) -> io::Result<()> {
        let _lock = self.rotate_lock.lock();

        let old_path = writer.current_path.clone();
        if !old_path.as_os_str().is_empty() {
            writer.current_path = PathBuf::new();
            let new_path = self.rotator.next_path();

            let new_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&new_path)?;

            writer.current_file = Some(new_file);
            writer.current_path = new_path;
            writer.current_size = 0;

            drop(_lock);
            if old_path.exists() {
                self.compress_async(old_path);
            }

            self.rotator.cleanup_old_files();
        }
        Ok(())
    }

    /// 异步压缩文件
    fn compress_async(&self, path: PathBuf) {
        let max_files = self.config.max_compressed_files;
        let base_path = self.config.log_dir.clone();

        COMPRESSION_POOL.execute(move || {
            if let Err(e) = Self::compress_file(&path, &base_path, max_files) {
                eprintln!("Failed to compress {}: {}", path.display(), e);
            } else {
                // 重试删除原文件
                for _ in 0..3 {
                    match std::fs::remove_file(&path) {
                        Ok(_) => break,
                        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            continue;
                        }
                        Err(e) => {
                            eprintln!("Failed to remove {}: {}", path.display(), e);
                            break;
                        }
                    }
                }
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

impl LogHandler for FileHandler {
    fn handle(&self, record: &Record) {
        // 根据配置决定是否跳过服务端自身日志
        if self.config.skip_server_logs && record.metadata.app_id.is_none() {
            return;
        }

        let mut buf = Vec::new();
        if let Err(e) = (self.formatter)(&mut buf, record) {
            eprintln!("File format error: {}", e);
            return;
        }

        let mut writer = self.writer.lock();
        if let Err(e) = writer.write(&buf) {
            eprintln!("File write error: {}", e);
            return;
        }

        if writer.current_size >= writer.max_size {
            if let Err(e) = self.rotate(&mut writer) {
                eprintln!("Log rotate error: {}", e);
            }
        }
    }

    fn flush(&self) {
        let mut writer = self.writer.lock();
        if let Some(file) = &mut writer.current_file {
            let _ = file.flush();
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
        if self.config.compress_on_drop {
            let _ = self.force_compress();
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
            current_file: Some(file),
            current_path: path,
            max_size,
            current_size: 0,
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
            current_file: Some(file),
            current_path: path,
            max_size,
            current_size: 0,
        }
    }

    fn write(&mut self, buf: &[u8]) -> io::Result<()> {
        if let Some(file) = &mut self.current_file {
            file.write_all(buf)?;
            self.current_size += buf.len();
            file.sync_all()?;
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
