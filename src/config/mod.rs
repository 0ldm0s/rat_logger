//! 配置模块

use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Level {
    pub fn to_level_filter(&self) -> LevelFilter {
        match self {
            Level::Error => LevelFilter::Error,
            Level::Warn => LevelFilter::Warn,
            Level::Info => LevelFilter::Info,
            Level::Debug => LevelFilter::Debug,
            Level::Trace => LevelFilter::Trace,
        }
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Error => write!(f, "ERROR"),
            Level::Warn => write!(f, "WARN"),
            Level::Info => write!(f, "INFO"),
            Level::Debug => write!(f, "DEBUG"),
            Level::Trace => write!(f, "TRACE"),
        }
    }
}

impl bincode::Encode for Level {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.to_string(), encoder)
    }
}

/// 日志级别过滤器
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LevelFilter {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// 应用ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppId(String);

impl AppId {
    pub fn new<S: Into<String>>(s: S) -> Self {
        AppId(s.into())
    }
}

impl From<String> for AppId {
    fn from(s: String) -> Self {
        AppId(s)
    }
}

impl From<&str> for AppId {
    fn from(s: &str) -> Self {
        AppId(s.to_string())
    }
}

impl std::fmt::Display for AppId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 日志元数据
#[derive(Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub level: Level,
    pub target: String,
    pub auth_token: Option<String>,
    pub app_id: Option<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            level: Level::Info,
            target: String::new(),
            auth_token: None,
            app_id: None,
        }
    }
}

/// 日志记录
#[derive(Clone)]
pub struct Record {
    pub metadata: std::sync::Arc<Metadata>,
    pub args: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

impl Serialize for Record {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Record", 6)?;
        state.serialize_field("metadata", &*self.metadata)?;
        state.serialize_field("args", &self.args)?;
        state.serialize_field("module_path", &self.module_path)?;
        state.serialize_field("file", &self.file)?;
        state.serialize_field("line", &self.line)?;
        state.end()
    }
}

/// 文件日志配置
#[derive(Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub log_dir: PathBuf,
    pub max_file_size: u64,
    pub max_compressed_files: usize,
    pub compression_level: u8,
    pub min_compress_threads: usize,
    pub skip_server_logs: bool,
    pub is_raw: bool,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("./logs"),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_compressed_files: 10,
            compression_level: 4,
            min_compress_threads: 2,
            skip_server_logs: false,
            is_raw: false,
        }
    }
}

/// 网络日志配置
#[derive(Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub auth_token: String,
    pub app_id: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1".to_string(),
            server_port: 5140,
            auth_token: "default_token".to_string(),
            app_id: "default_app".to_string(),
        }
    }
}

/// 用于网络传输的日志记录
#[derive(Serialize, Deserialize)]
pub struct NetRecord {
    pub level: Level,
    pub target: String,
    pub message: String,
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub timestamp: u64,
    pub auth_token: Option<String>,
    pub app_id: Option<String>,
}

impl bincode::Encode for NetRecord {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.level, encoder)?;
        bincode::Encode::encode(&self.target, encoder)?;
        bincode::Encode::encode(&self.message, encoder)?;
        bincode::Encode::encode(&self.module_path, encoder)?;
        bincode::Encode::encode(&self.file, encoder)?;
        bincode::Encode::encode(&self.line, encoder)?;
        bincode::Encode::encode(&self.timestamp, encoder)?;
        bincode::Encode::encode(&self.auth_token, encoder)?;
        bincode::Encode::encode(&self.app_id, encoder)?;
        Ok(())
    }
}

impl From<&Record> for NetRecord {
    fn from(record: &Record) -> Self {
        NetRecord {
            level: record.metadata.level,
            target: record.metadata.target.clone(),
            message: record.args.clone(),
            module_path: record.module_path.clone(),
            file: record.file.clone(),
            line: record.line,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            auth_token: record.metadata.auth_token.clone(),
            app_id: record.metadata.app_id.clone(),
        }
    }
}
