use rat_logger::{init_with_level, LevelFilter, LoggerBuilder, FileConfig, Logger};

fn main() {
    // 初始化日志器
    init_with_level(LevelFilter::Debug).unwrap();

    // 基本日志测试
    rat_logger::error!("这是一条错误日志");
    rat_logger::warn!("这是一条警告日志");
    rat_logger::info!("这是一条信息日志");
    rat_logger::debug!("这是一条调试日志");

    println!("基本日志测试完成");

    // 文件日志测试
    let file_config = FileConfig {
        log_dir: "./test_logs".into(),
        max_file_size: 1024 * 1024, // 1MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
    };

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Trace)
        .add_file(file_config.clone())
        .build();

    let record = rat_logger::config::Record {
        metadata: std::sync::Arc::new(rat_logger::config::Metadata {
            level: rat_logger::config::Level::Info,
            target: "test".to_string(),
            auth_token: None,
            app_id: None,
        }),
        args: "文件日志测试".to_string(),
        module_path: Some("test".to_string()),
        file: Some("test.rs".to_string()),
        line: Some(1),
    };

    logger.log(&record);
    logger.flush();

    println!("文件日志测试完成");

    // 组合处理器测试
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Trace)
        .add_terminal()
        .add_file(file_config.clone())
        .build();

    logger.log(&record);
    logger.flush();

    println!("组合处理器测试完成");
}