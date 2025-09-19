//! 文件轮转示例
//!
//! 展示rat_logger的文件轮转和压缩功能
//!
//! ⚠️  重要提醒：
//! - 本示例启用开发模式以确保日志立即输出，方便演示和学习
//! - 在生产环境中，请禁用开发模式以获得最佳性能
//! - 生产环境推荐：LoggerBuilder::new().add_file(config).build()

use rat_logger::{LoggerBuilder, LevelFilter, FileConfig, Logger};
use rat_logger::config::Record;
use rat_logger::config::Metadata;
use std::sync::Arc;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn main() {
    let file_config = FileConfig {
        log_dir: PathBuf::from("./rotation_logs"),
        max_file_size: 1024, // 1KB - 很小以测试轮转
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        format: None,
    };

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .with_dev_mode(true) // 示例启用开发模式，确保日志立即输出
        .add_file(file_config)
        .build();

    println!("开始测试文件轮转...");

    // 生成大量日志以触发轮转
    for i in 0..100 {
        let record = Record {
            metadata: Arc::new(Metadata {
                level: rat_logger::config::Level::Info,
                target: "rotation_example".to_string(),
                auth_token: None,
                app_id: Some("main".to_string()),
            }),
            args: format!("这是第 {} 条日志，用于测试文件轮转功能", i),
            module_path: Some("file_rotation".to_string()),
            file: Some("file_rotation.rs".to_string()),
            line: Some(42),
        };
        logger.log(&record);

        let warn_record = Record {
            metadata: Arc::new(Metadata {
                level: rat_logger::config::Level::Warn,
                target: "rotation_example".to_string(),
                auth_token: None,
                app_id: Some("main".to_string()),
            }),
            args: format!("警告日志 {}", i),
            module_path: Some("file_rotation".to_string()),
            file: Some("file_rotation.rs".to_string()),
            line: Some(58),
        };
        logger.log(&warn_record);

        let error_record = Record {
            metadata: Arc::new(Metadata {
                level: rat_logger::config::Level::Error,
                target: "rotation_example".to_string(),
                auth_token: None,
                app_id: Some("main".to_string()),
            }),
            args: format!("错误日志 {}", i),
            module_path: Some("file_rotation".to_string()),
            file: Some("file_rotation.rs".to_string()),
            line: Some(73),
        };
        logger.log(&error_record);

        // 小延迟确保时间戳不同
        thread::sleep(Duration::from_millis(10));
    }

    println!("日志生成完成，请检查 ./rotation_logs 目录");
    println!("由于设置了max_file_size为1KB，应该会触发多次文件轮转");
}