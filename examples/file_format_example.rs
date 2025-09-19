//! rat_logger 文件格式配置示例
//!
//! 专门演示如何配置文件输出的格式
//!
//! ⚠️  重要提醒：
//! - 本示例启用开发模式以确保日志立即输出，方便演示和学习
//! - 在生产环境中，请禁用开发模式以获得最佳性能
//! - 生产环境推荐：LoggerBuilder::new().add_file(config).build()

use rat_logger::{LoggerBuilder, LevelFilter, Level, FileConfig, Logger};
use rat_logger::config::{Record, Metadata};
use std::sync::Arc;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_logger 文件格式配置示例 ===\n");

    // 1. 创建简洁格式配置
    println!("1. 创建简洁格式配置:");
    let simple_format = rat_logger::FormatConfig {
        timestamp_format: "%H:%M:%S".to_string(),
        level_style: rat_logger::LevelStyle {
            error: "E".to_string(),
            warn: "W".to_string(),
            info: "I".to_string(),
            debug: "D".to_string(),
            trace: "T".to_string(),
        },
        format_template: "{level} {timestamp} {message}".to_string(),
    };

    // 2. 创建详细格式配置
    println!("2. 创建详细格式配置:");
    let detailed_format = rat_logger::FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle {
            error: "ERROR".to_string(),
            warn: "WARN ".to_string(),
            info: "INFO ".to_string(),
            debug: "DEBUG".to_string(),
            trace: "TRACE".to_string(),
        },
        format_template: "[{level}] {timestamp} {target}:{line} - {message}".to_string(),
    };

    // 3. 创建JSON格式配置
    println!("3. 创建JSON格式配置:");
    let json_format = rat_logger::FormatConfig {
        timestamp_format: "%Y-%m-%dT%H:%M:%S%.3fZ".to_string(),
        level_style: rat_logger::LevelStyle {
            error: "error".to_string(),
            warn: "warn".to_string(),
            info: "info".to_string(),
            debug: "debug".to_string(),
            trace: "trace".to_string(),
        },
        format_template: "{{\"timestamp\":\"{timestamp}\",\"level\":\"{level}\",\"target\":\"{target}\",\"message\":\"{message}\"}}".to_string(),
    };

    println!("   ✓ 已创建配置\n");

    // 4. 测试不同文件格式配置
    println!("4. 测试不同文件格式配置:");

    // 4.1 默认格式文件
    println!("   4.1 默认格式文件:");
    let mut file_config1 = FileConfig {
        log_dir: PathBuf::from("./default_format_logs"),
        max_file_size: 1024 * 1024,
        max_compressed_files: 2,
        compression_level: 6,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        format: None, // 使用默认格式
    };

    let logger1 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_file(file_config1)
        .build();

    logger1.log(&create_test_record(Level::Error, "default_test", "错误消息"));
    logger1.log(&create_test_record(Level::Info, "default_test", "信息消息"));

    // 4.2 简洁格式文件
    println!("   4.2 简洁格式文件:");
    let mut file_config2 = FileConfig {
        log_dir: PathBuf::from("./simple_format_logs"),
        max_file_size: 1024 * 1024,
        max_compressed_files: 2,
        compression_level: 6,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        format: Some(simple_format.clone()),
    };

    let logger2 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_file(file_config2)
        .build();

    logger2.log(&create_test_record(Level::Error, "simple_test", "错误消息"));
    logger2.log(&create_test_record(Level::Info, "simple_test", "信息消息"));

    // 4.3 详细格式文件
    println!("   4.3 详细格式文件:");
    let mut file_config3 = FileConfig {
        log_dir: PathBuf::from("./detailed_format_logs"),
        max_file_size: 1024 * 1024,
        max_compressed_files: 2,
        compression_level: 6,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        format: Some(detailed_format.clone()),
    };

    let logger3 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_file(file_config3)
        .build();

    logger3.log(&create_test_record(Level::Error, "detailed_test", "错误消息"));
    logger3.log(&create_test_record(Level::Info, "detailed_test", "信息消息"));

    // 4.4 JSON格式文件
    println!("   4.4 JSON格式文件:");
    let mut file_config4 = FileConfig {
        log_dir: PathBuf::from("./json_format_logs"),
        max_file_size: 1024 * 1024,
        max_compressed_files: 2,
        compression_level: 6,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        format: Some(json_format.clone()),
    };

    let logger4 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_file(file_config4)
        .build();

    logger4.log(&create_test_record(Level::Error, "json_test", "错误消息"));
    logger4.log(&create_test_record(Level::Info, "json_test", "信息消息"));

    // 等待文件写入完成
    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("\n5. 查看生成的日志文件内容:");

    // 5.1 显示默认格式文件内容
    println!("   5.1 默认格式文件内容:");
    if let Ok(entries) = std::fs::read_dir("./default_format_logs") {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("log") {
                println!("     文件: {}", entry.path().display());
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for line in content.lines().take(2) {
                        println!("       {}", line);
                    }
                }
            }
        }
    }

    // 5.2 显示简洁格式文件内容
    println!("\n   5.2 简洁格式文件内容:");
    if let Ok(entries) = std::fs::read_dir("./simple_format_logs") {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("log") {
                println!("     文件: {}", entry.path().display());
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for line in content.lines().take(2) {
                        println!("       {}", line);
                    }
                }
            }
        }
    }

    // 5.3 显示详细格式文件内容
    println!("\n   5.3 详细格式文件内容:");
    if let Ok(entries) = std::fs::read_dir("./detailed_format_logs") {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("log") {
                println!("     文件: {}", entry.path().display());
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for line in content.lines().take(2) {
                        println!("       {}", line);
                    }
                }
            }
        }
    }

    // 5.4 显示JSON格式文件内容
    println!("\n   5.4 JSON格式文件内容:");
    if let Ok(entries) = std::fs::read_dir("./json_format_logs") {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("log") {
                println!("     文件: {}", entry.path().display());
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for line in content.lines().take(2) {
                        println!("       {}", line);
                    }
                }
            }
        }
    }

    println!("\n=== 示例完成 ===");
    println!("文件格式配置说明:");
    println!("- FileConfig.format: 控制文件输出的格式模板");
    println!("- 格式模板支持: {{timestamp}}, {{level}}, {{target}}, {{file}}, {{line}}, {{message}}");
    println!("- 适合不同的日志处理需求：人类阅读、机器处理、结构化日志等");
    println!("- 可以自定义时间戳格式和级别显示文本");

    Ok(())
}

fn create_test_record(
    level: Level,
    target: &str,
    message: &str,
) -> Record {
    Record {
        metadata: Arc::new(Metadata {
            level,
            target: target.to_string(),
            auth_token: None,
            app_id: Some("file_format_example".to_string()),
        }),
        args: message.to_string(),
        module_path: Some("file_format_example".to_string()),
        file: Some("file_format_example.rs".to_string()),
        line: Some(42),
    }
}