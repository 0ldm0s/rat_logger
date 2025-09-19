//! rat_logger 基础使用示例
//!
//! 展示rat_logger的基本功能和使用方法
//!
//! ⚠️  重要提醒：
//! - 本示例启用开发模式以确保日志立即输出，方便演示和学习
//! - 在生产环境中，请禁用开发模式以获得最佳性能
//! - 生产环境推荐：LoggerBuilder::new().add_terminal().build()

use rat_logger::{LoggerBuilder, LevelFilter, Level, FileConfig, NetworkConfig, config::Record, Logger};
use rat_logger::config::Metadata;
use std::sync::Arc;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_logger 基础使用示例 ===\n");

    // 1. 基本终端日志
    println!("1. 基本终端日志:");
    let terminal_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true) // 示例启用开发模式，确保日志立即输出
        .add_terminal()
        .build();

    let record = Record {
        metadata: Arc::new(Metadata {
            level: Level::Info,
            target: "basic_example".to_string(),
            auth_token: None,
            app_id: Some("main".to_string()),
        }),
        args: "应用程序启动".to_string(),
        module_path: Some("basic_usage".to_string()),
        file: Some("basic_usage.rs".to_string()),
        line: Some(29),
    };
    terminal_logger.log(&record);

    // 2. 文件日志
    println!("\n2. 文件日志:");
    let file_config = FileConfig {
        log_dir: PathBuf::from("./example_logs"),
        max_file_size: 1024 * 1024 * 10, // 10MB
        max_compressed_files: 3,
        compression_level: 6,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        format: None,
    };

    let file_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .with_dev_mode(true) // 示例启用开发模式，确保日志立即输出
        .add_file(file_config)
        .build();

    let file_record = Record {
        metadata: Arc::new(Metadata {
            level: Level::Warn,
            target: "file_example".to_string(),
            auth_token: None,
            app_id: Some("file_app".to_string()),
        }),
        args: "这是一条文件日志".to_string(),
        module_path: Some("basic_usage".to_string()),
        file: Some("basic_usage.rs".to_string()),
        line: Some(60),
    };
    file_logger.log(&file_record);

    // 3. 网络日志
    println!("\n3. 网络日志:");
    let network_config = NetworkConfig {
        server_addr: "127.0.0.1".to_string(),
        server_port: 54321,
        auth_token: "example_token".to_string(),
        app_id: "network_app".to_string(),
    };

    let network_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .with_dev_mode(true) // 示例启用开发模式，确保日志立即输出
        .add_udp(network_config)
        .build();

    let network_record = Record {
        metadata: Arc::new(Metadata {
            level: Level::Error,
            target: "network_example".to_string(),
            auth_token: Some("example_token".to_string()),
            app_id: Some("network_app".to_string()),
        }),
        args: "这是一条网络日志".to_string(),
        module_path: Some("basic_usage".to_string()),
        file: Some("basic_usage.rs".to_string()),
        line: Some(90),
    };
    network_logger.log(&network_record);

    // 4. 多输出日志
    println!("\n4. 多输出日志 (终端+文件):");
    let multi_config = FileConfig {
        log_dir: PathBuf::from("./example_logs"),
        max_file_size: 1024 * 1024 * 5, // 5MB
        max_compressed_files: 2,
        compression_level: 6,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        format: None,
    };

    let multi_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true) // 示例启用开发模式，确保日志立即输出
        .add_terminal()
        .add_file(multi_config)
        .build();

    let multi_record = Record {
        metadata: Arc::new(Metadata {
            level: Level::Info,
            target: "multi_example".to_string(),
            auth_token: None,
            app_id: Some("multi_app".to_string()),
        }),
        args: "这是一条多输出日志".to_string(),
        module_path: Some("basic_usage".to_string()),
        file: Some("basic_usage.rs".to_string()),
        line: Some(126),
    };
    multi_logger.log(&multi_record);

    // 5. 不同级别的日志
    println!("\n5. 不同级别的日志:");
    let level_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Trace)
        .with_dev_mode(true) // 示例启用开发模式，确保日志立即输出
        .add_terminal()
        .build();

    let levels = vec![
        (Level::Error, "错误日志"),
        (Level::Warn, "警告日志"),
        (Level::Info, "信息日志"),
        (Level::Debug, "调试日志"),
        (Level::Trace, "跟踪日志"),
    ];

    for (level, message) in levels {
        let record = Record {
            metadata: Arc::new(Metadata {
                level,
                target: "level_example".to_string(),
                auth_token: None,
                app_id: Some("level_app".to_string()),
            }),
            args: message.to_string(),
            module_path: Some("basic_usage".to_string()),
            file: Some("basic_usage.rs".to_string()),
            line: Some(160),
        };
        level_logger.log(&record);
    }

    println!("\n=== 示例完成 ===");
    println!("日志文件已保存到 ./example_logs 目录");
    println!("网络日志已发送到 127.0.0.1:54321");

    Ok(())
}