//! 多输出处理器示例
//!
//! 展示如何使用LoggerBuilder同时添加终端和文件输出，实现类似组合处理器的效果
//!
//! ⚠️  重要提醒：
//! - 本示例启用开发模式以确保日志立即输出，方便演示和学习
//! - 在生产环境中，请禁用开发模式以获得最佳性能
//! - 生产环境推荐：LoggerBuilder::new().add_terminal().add_file(config).build()

use rat_logger::{LoggerBuilder, LevelFilter, FileConfig, Logger};
use rat_logger::config::Record;
use rat_logger::config::Metadata;
use std::sync::Arc;
use std::path::PathBuf;

fn main() {
    let file_config = FileConfig {
        log_dir: PathBuf::from("./composite_logs"),
        max_file_size: 512 * 1024, // 512KB
        max_compressed_files: 3,
        compression_level: 4,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        format: None,
    };

    // 使用LoggerBuilder创建多输出日志器（终端 + 文件）
    // 这实际上是内部使用了CompositeHandler
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .with_dev_mode(true) // 示例启用开发模式，确保日志立即输出
        .add_terminal()  // 添加终端输出
        .add_file(file_config)  // 添加文件输出
        .build();

    // 创建日志记录
    let record = Record {
        metadata: Arc::new(Metadata {
            level: rat_logger::config::Level::Info,
            target: "composite_example".to_string(),
            auth_token: None,
            app_id: Some("main".to_string()),
        }),
        args: "使用多输出处理器，日志将同时输出到终端和文件".to_string(),
        module_path: Some("composite_handler".to_string()),
        file: Some("composite_handler.rs".to_string()),
        line: Some(38),
    };

    logger.log(&record);

    let warn_record = Record {
        metadata: Arc::new(Metadata {
            level: rat_logger::config::Level::Warn,
            target: "composite_example".to_string(),
            auth_token: None,
            app_id: Some("main".to_string()),
        }),
        args: "多输出处理器测试".to_string(),
        module_path: Some("composite_handler".to_string()),
        file: Some("composite_handler.rs".to_string()),
        line: Some(53),
    };

    logger.log(&warn_record);

    let error_record = Record {
        metadata: Arc::new(Metadata {
            level: rat_logger::config::Level::Error,
            target: "composite_example".to_string(),
            auth_token: None,
            app_id: Some("main".to_string()),
        }),
        args: "错误日志将通过多个处理器并行处理".to_string(),
        module_path: Some("composite_handler".to_string()),
        file: Some("composite_handler.rs".to_string()),
        line: Some(66),
    };

    logger.log(&error_record);

    println!("多输出处理器示例运行完成");
    println!("日志已同时输出到终端和 ./composite_logs 目录");
    println!("LoggerBuilder内部自动使用CompositeHandler来处理多个输出目标");
}