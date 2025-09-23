//! 终端日志处理器 - 高性能异步架构

use std::io::{self, Write, BufWriter};
use std::any::Any;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::producer_consumer::LogProcessor;
use crate::config::{Record, FormatConfig, ColorConfig, Level};

/// 终端输出配置
#[derive(Debug, Clone)]
pub struct TermConfig {
    /// 是否启用颜色输出
    pub enable_color: bool,
    /// 格式配置
    pub format: Option<FormatConfig>,
    /// 颜色配置
    pub color: Option<ColorConfig>,
}

impl TermConfig {
    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), String> {
        // 验证颜色配置一致性
        if !self.enable_color && self.color.is_some() {
            return Err(format!("配置冲突: 颜色配置被提供但 enable_color 为 false。如果要启用颜色，请设置 enable_color = true；如果要禁用颜色，请移除 color 配置。"));
        }

        // 验证格式配置（如果提供）
        if let Some(format_config) = &self.format {
            if format_config.format_template.is_empty() {
                return Err("配置错误: 格式模板不能为空".to_string());
            }
            if format_config.timestamp_format.is_empty() {
                return Err("配置错误: 时间戳格式不能为空".to_string());
            }
        }

        Ok(())
    }
}

impl Default for TermConfig {
    fn default() -> Self {
        Self {
            enable_color: true,
            format: None,
            color: None,
        }
    }
}

/// 终端日志处理器 - 实现LogProcessor trait
pub struct TermProcessor {
    config: TermConfig,
    formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync>,
    stdout: Arc<Mutex<BufWriter<io::Stdout>>>,
}

impl TermProcessor {
    /// 创建新的终端处理器
    pub fn new() -> Self {
        let config = TermConfig::default();
        Self::with_config(config)
    }

    /// 使用配置创建终端处理器
    pub fn with_config(config: TermConfig) -> Self {
        // 验证配置，如果失败则直接panic，让用户明确知道配置问题
        if let Err(e) = config.validate() {
            panic!("TermConfig 验证失败: {}\n请检查您的配置并修复上述问题后再重试。", e);
        }

        let formatter: Box<dyn Fn(&mut dyn Write, &Record) -> io::Result<()> + Send + Sync> = {
            // 检查是否启用颜色且有颜色配置
            let use_color = config.enable_color && config.color.is_some();

            match (&config.format, use_color) {
                (Some(format_config), true) => {
                    // 有格式配置且启用颜色
                    let format_config = format_config.clone();
                    let color_config = config.color.as_ref().unwrap().clone();
                    Box::new(move |buf, record| {
                        format_with_color(buf, record, &format_config, &color_config)
                    })
                }
                (Some(format_config), false) => {
                    // 有格式配置但不启用颜色
                    let format_config = format_config.clone();
                    Box::new(move |buf, record| {
                        format_with_config(buf, record, &format_config)
                    })
                }
                (None, true) => {
                    // 无格式配置但启用颜色
                    let default_format_config = FormatConfig::default();
                    let color_config = config.color.as_ref().unwrap().clone();
                    Box::new(move |buf, record| {
                        format_with_color(buf, record, &default_format_config, &color_config)
                    })
                }
                (None, false) => Box::new(default_format),
            }
        };

        let processor = Self {
            config,
            formatter,
            stdout: Arc::new(Mutex::new(BufWriter::new(io::stdout()))),
        };

        processor
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
        self.formatter = Box::new(move |buf, record| format_with_config(buf, record, &format_config));
        self
    }

    /// 使用格式配置和颜色配置
    pub fn with_format_and_color(mut self, format_config: FormatConfig, color_config: ColorConfig) -> Self {
        let format_config = format_config.clone();
        let color_config = color_config.clone();
        self.formatter = Box::new(move |buf, record| format_with_color(buf, record, &format_config, &color_config));
        self
    }

    /// 格式化日志记录
    fn format_record(&self, record: &Record) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        (self.formatter)(&mut buf, record)
            .map_err(|e| format!("格式化失败: {}", e))?;
        Ok(buf)
    }

    /// 写入到终端
    fn write_to_terminal(&self, data: &[u8]) -> Result<(), String> {
        let mut stdout_guard = self.stdout.lock();
        stdout_guard.write_all(data)
            .map_err(|e| format!("终端写入失败: {}", e))?;
        stdout_guard.flush()
            .map_err(|e| format!("终端刷新失败: {}", e))?;
        Ok(())
    }
}

impl LogProcessor for TermProcessor {
    fn name(&self) -> &'static str {
        "term_processor"
    }

    fn process(&mut self, data: &[u8]) -> Result<(), String> {
        eprintln!("DEBUG: TermProcessor::process 被调用，数据长度: {}", data.len());
        // 反序列化日志记录
        let record = bincode::decode_from_slice::<Record, _>(data, bincode::config::standard())
            .map_err(|e| format!("反序列化失败: {}", e))?.0;
        eprintln!("DEBUG: TermProcessor 反序列化成功: {:?}", record.args);

        // 格式化日志记录
        let formatted_data = self.format_record(&record)?;
        eprintln!("DEBUG: TermProcessor 格式化成功，数据长度: {}", formatted_data.len());

        // 写入到终端
        let result = self.write_to_terminal(&formatted_data);
        eprintln!("DEBUG: TermProcessor 写入结果: {:?}", result);
        result
    }

    fn process_batch(&mut self, batch: &[Vec<u8>]) -> Result<(), String> {
        let mut all_data = Vec::new();

        // 批量反序列化和格式化
        for data in batch {
            let record = bincode::decode_from_slice::<Record, _>(data, bincode::config::standard())
                .map_err(|e| format!("批量反序列化失败: {}", e))?.0;

            let formatted_data = self.format_record(&record)?;
            all_data.extend_from_slice(&formatted_data);
        }

        // 批量写入
        self.write_to_terminal(&all_data)
    }

    fn flush(&mut self) -> Result<(), String> {
        // 直接刷新终端
        let mut stdout_guard = self.stdout.lock();
        stdout_guard.flush()
            .map_err(|e| format!("终端刷新失败: {}", e))?;
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), String> {
        // 刷新所有剩余数据
        self.flush()
    }
}

impl Drop for TermProcessor {
    fn drop(&mut self) {
        // 清理时会自动调用cleanup
        let _ = self.cleanup();
    }
}

/// 默认格式化函数
pub fn default_format(buf: &mut dyn Write, record: &Record) -> io::Result<()> {
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

/// 格式化函数
pub fn format_with_config(buf: &mut dyn Write, record: &Record, format_config: &FormatConfig) -> io::Result<()> {
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

/// 带颜色的格式化函数
pub fn format_with_color(buf: &mut dyn Write, record: &Record, format_config: &FormatConfig, color_config: &ColorConfig) -> io::Result<()> {
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

    // 获取级别颜色
    let level_color = match record.metadata.level {
        Level::Error => &color_config.error,
        Level::Warn => &color_config.warn,
        Level::Info => &color_config.info,
        Level::Debug => &color_config.debug,
        Level::Trace => &color_config.trace,
    };

    // 重置颜色
    let reset_color = "\x1b[0m";

    // 使用格式模板并应用颜色
    let colored_timestamp = format!("{}{}{}", color_config.timestamp, timestamp, reset_color);
    let colored_level = format!("{}{}{}", level_color, level_text, reset_color);
    let colored_target = format!("{}{}{}", color_config.target, record.metadata.target, reset_color);
    let colored_file = format!("{}{}{}", color_config.file, record.file.as_deref().unwrap_or("unknown"), reset_color);
    let colored_line = format!("{}{}{}", color_config.file, record.line.unwrap_or(0), reset_color);
    let colored_message = format!("{}{}{}", color_config.message, record.args, reset_color);

    // 使用格式模板进行格式化
    let mut formatted = format_config.format_template
        .replace("{timestamp}", &colored_timestamp)
        .replace("{level}", &colored_level)
        .replace("{target}", &colored_target)
        .replace("{file}", &colored_file)
        .replace("{line}", &colored_line)
        .replace("{message}", &colored_message);

    // 处理格式模板中可能包含的冒号和分隔符
    formatted = formatted.replace("}:", format!("{}:{}", reset_color, color_config.file).as_str());

    writeln!(buf, "{}", formatted)
}