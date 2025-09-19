//! 配置模块

use serde::{Serialize, Deserialize};
use bincode::{Encode, Decode};
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

    /// 检查在给定的过滤级别下，该日志是否应该被记录
    /// 返回true表示该级别的日志应该被发送
    pub fn should_log_at(&self, filter_level: LevelFilter) -> bool {
        self.to_level_filter() as u8 <= filter_level as u8
    }

    /// 检查在给定的日志级别下，该日志是否应该被记录
    /// 返回true表示该级别的日志应该被发送
    pub fn should_log_at_level(&self, filter_level: Level) -> bool {
        *self as u8 <= filter_level as u8
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

impl bincode::Decode<()> for Level {
    fn decode<D: bincode::de::Decoder<Context = ()>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let level_str: String = bincode::Decode::decode(decoder)?;
        match level_str.as_str() {
            "ERROR" => Ok(Level::Error),
            "WARN" => Ok(Level::Warn),
            "INFO" => Ok(Level::Info),
            "DEBUG" => Ok(Level::Debug),
            "TRACE" => Ok(Level::Trace),
            _ => Err(bincode::error::DecodeError::OtherString("Invalid level string".to_string())),
        }
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

impl bincode::Encode for Metadata {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.level, encoder)?;
        bincode::Encode::encode(&self.target, encoder)?;
        bincode::Encode::encode(&self.auth_token, encoder)?;
        bincode::Encode::encode(&self.app_id, encoder)
    }
}

impl bincode::Decode<()> for Metadata {
    fn decode<D: bincode::de::Decoder<Context = ()>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let level = bincode::Decode::decode(decoder)?;
        let target = bincode::Decode::decode(decoder)?;
        let auth_token = bincode::Decode::decode(decoder)?;
        let app_id = bincode::Decode::decode(decoder)?;
        Ok(Metadata {
            level,
            target,
            auth_token,
            app_id,
        })
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

impl bincode::Encode for Record {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&(*self.metadata).clone(), encoder)?;
        bincode::Encode::encode(&self.args, encoder)?;
        bincode::Encode::encode(&self.module_path, encoder)?;
        bincode::Encode::encode(&self.file, encoder)?;
        bincode::Encode::encode(&self.line, encoder)
    }
}

impl bincode::Decode<()> for Record {
    fn decode<D: bincode::de::Decoder<Context = ()>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let metadata = bincode::Decode::decode(decoder)?;
        let args = bincode::Decode::decode(decoder)?;
        let module_path = bincode::Decode::decode(decoder)?;
        let file = bincode::Decode::decode(decoder)?;
        let line = bincode::Decode::decode(decoder)?;
        Ok(Record {
            metadata: std::sync::Arc::new(metadata),
            args,
            module_path,
            file,
            line,
        })
    }
}

/// 文件日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub log_dir: PathBuf,
    pub max_file_size: u64,
    pub max_compressed_files: usize,
    pub compression_level: u8,
    pub min_compress_threads: usize,
    pub skip_server_logs: bool,
    pub is_raw: bool,
    pub compress_on_drop: bool, // 是否在Drop时强制压缩
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
            compress_on_drop: false, // 默认不在Drop时压缩
        }
    }
}

/// 日志格式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatConfig {
    /// 时间戳格式
    pub timestamp_format: String,
    /// 日志级别显示样式
    pub level_style: LevelStyle,
    /// 输出格式模板
    pub format_template: String,
}

/// 日志级别样式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelStyle {
    /// 错误级别显示
    pub error: String,
    /// 警告级别显示
    pub warn: String,
    /// 信息级别显示
    pub info: String,
    /// 调试级别显示
    pub debug: String,
    /// 跟踪级别显示
    pub trace: String,
}

/// 终端颜色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    /// 错误级别颜色 (ANSI颜色代码)
    pub error: String,
    /// 警告级别颜色
    pub warn: String,
    /// 信息级别颜色
    pub info: String,
    /// 调试级别颜色
    pub debug: String,
    /// 跟踪级别颜色
    pub trace: String,
    /// 时间戳颜色
    pub timestamp: String,
    /// 目标颜色
    pub target: String,
    /// 文件名颜色
    pub file: String,
    /// 消息颜色
    pub message: String,
}

/// 网络日志配置
#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
            level_style: LevelStyle::default(),
            format_template: "{timestamp} [{level}] {target}:{line} - {message}".to_string(),
        }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            error: "\x1b[31m".to_string(),      // 红色
            warn: "\x1b[33m".to_string(),       // 黄色
            info: "\x1b[32m".to_string(),       // 绿色
            debug: "\x1b[36m".to_string(),      // 青色
            trace: "\x1b[37m".to_string(),      // 白色
            timestamp: "\x1b[90m".to_string(),   // 深灰色
            target: "\x1b[34m".to_string(),      // 蓝色
            file: "\x1b[35m".to_string(),       // 紫色
            message: "\x1b[0m".to_string(),      // 重置颜色
        }
    }
}

impl Default for LevelStyle {
    fn default() -> Self {
        Self {
            error: "ERROR".to_string(),
            warn: "WARN".to_string(),
            info: "INFO".to_string(),
            debug: "DEBUG".to_string(),
            trace: "TRACE".to_string(),
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

impl bincode::Decode<()> for NetRecord {
    fn decode<D: bincode::de::Decoder<Context = ()>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            level: bincode::Decode::decode(decoder)?,
            target: bincode::Decode::decode(decoder)?,
            message: bincode::Decode::decode(decoder)?,
            module_path: bincode::Decode::decode(decoder)?,
            file: bincode::Decode::decode(decoder)?,
            line: bincode::Decode::decode(decoder)?,
            timestamp: bincode::Decode::decode(decoder)?,
            auth_token: bincode::Decode::decode(decoder)?,
            app_id: bincode::Decode::decode(decoder)?,
        })
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
