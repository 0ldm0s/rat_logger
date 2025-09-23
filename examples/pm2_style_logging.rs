//! PM2风格多文件日志管理示例
//!
//! 展示如何像PM2一样管理多个独立的日志文件，不同类型的日志输出到不同的文件
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

fn main() {
    // 创建PM2风格的多文件日志管理

    // 1. 主应用日志文件 - 程序结束时压缩
    let main_app_config = FileConfig {
        log_dir: PathBuf::from("./app_logs"),
        max_file_size: 10 * 1024 * 1024, // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: true, // 程序结束时强制压缩
        force_sync: false, // 异步写入，性能更好
        format: None,
    };

    // 2. 错误日志文件 - 程序结束时压缩
    let error_log_config = FileConfig {
        log_dir: PathBuf::from("./error_logs"),
        max_file_size: 5 * 1024 * 1024,  // 5MB
        max_compressed_files: 10,
        compression_level: 6, // 更高压缩率
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: true, // 程序结束时强制压缩
        force_sync: true, // 错误日志同步写入，确保不丢失
        format: None,
    };

    // 3. 访问日志文件 - 不在程序结束时压缩
    let access_log_config = FileConfig {
        log_dir: PathBuf::from("./access_logs"),
        max_file_size: 50 * 1024 * 1024, // 50MB
        max_compressed_files: 3,
        compression_level: 1, // 快速压缩
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false, // 不在程序结束时压缩
        force_sync: false, // 异步写入，性能更好
        format: None,
    };

    // 4. 性能监控日志文件 - 不在程序结束时压缩
    let perf_log_config = FileConfig {
        log_dir: PathBuf::from("./perf_logs"),
        max_file_size: 20 * 1024 * 1024, // 20MB
        max_compressed_files: 7,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false, // 不在程序结束时压缩
        force_sync: false, // 异步写入，性能更好
        format: None,
    };

    // 创建主日志器（包含终端输出）
    let main_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        // .with_dev_mode(true) // 注释掉开发模式，使用正常的批量处理模式
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())  // 终端输出
        .add_file(main_app_config.clone())
        .build();

    // 创建错误专用日志器
    let error_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Error)  // 只记录错误级别
        // .with_dev_mode(true) // 注释掉开发模式，使用force_sync=true保证数据安全
        .add_file(error_log_config.clone())
        .build();

    // 创建访问日志专用日志器
    let access_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        // .with_dev_mode(true) // 注释掉开发模式，使用批量处理提高性能
        .add_file(access_log_config.clone())
        .build();

    // 创建性能监控专用日志器
    let perf_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        // .with_dev_mode(true) // 注释掉开发模式，使用批量处理提高性能
        .add_file(perf_log_config.clone())
        .build();

    // 模拟应用运行
    println!("PM2风格日志系统启动...");

    // 模拟不同类型的日志记录
    for i in 0..10 {
        // 应用日志（输出到终端和主应用日志文件）
        let app_record = create_app_record(format!("应用运行中 - 第{}次循环", i));
        main_logger.log(&app_record);

        // 错误日志（输出到终端和错误日志文件）
        if i % 3 == 0 {
            let error_record = create_error_record(format!("发生错误 - 第{}次循环", i));
            main_logger.log(&error_record);  // 终端 + 主文件
            error_logger.log(&error_record); // 错误专用文件
        }

        // 访问日志（只输出到访问日志文件）
        let access_record = create_access_record(format!("GET /api/users - 第{}次请求", i));
        access_logger.log(&access_record);

        // 性能日志（只输出到性能日志文件）
        if i % 2 == 0 {
            let perf_record = create_perf_record(format!("API响应时间: {}ms", i * 10));
            perf_logger.log(&perf_record);
        }
    }

    // 刷新所有日志器
    main_logger.flush();
    error_logger.flush();
    access_logger.flush();
    perf_logger.flush();

    println!("PM2风格日志系统运行完成");
    println!("检查以下目录中的日志文件：");
    println!("- ./app_logs/    (主应用日志 - 程序结束时压缩)");
    println!("- ./error_logs/  (错误日志 - 程序结束时压缩)");
    println!("- ./access_logs/ (访问日志 - 保持未压缩状态)");
    println!("- ./perf_logs/   (性能日志 - 保持未压缩状态)");
    println!("");
    println!("compress_on_drop配置说明：");
    println!("- true: 程序结束时强制压缩当前日志文件");
    println!("- false: 程序结束时保持日志文件未压缩，等待下次轮转时压缩");
    println!("");
    println!("注意：所有*_logs目录都被git忽略，不会被提交到版本控制");
}

// 创建应用日志记录
fn create_app_record(message: String) -> Record {
    Record {
        metadata: Arc::new(Metadata {
            level: rat_logger::config::Level::Info,
            target: "main_app".to_string(),
            auth_token: None,
            app_id: Some("my_app".to_string()),
        }),
        args: message,
        module_path: Some("main".to_string()),
        file: Some("main.rs".to_string()),
        line: Some(42),
    }
}

// 创建错误日志记录
fn create_error_record(message: String) -> Record {
    Record {
        metadata: Arc::new(Metadata {
            level: rat_logger::config::Level::Error,
            target: "main_app".to_string(),
            auth_token: None,
            app_id: Some("my_app".to_string()),
        }),
        args: message,
        module_path: Some("main".to_string()),
        file: Some("main.rs".to_string()),
        line: Some(85),
    }
}

// 创建访问日志记录
fn create_access_record(message: String) -> Record {
    Record {
        metadata: Arc::new(Metadata {
            level: rat_logger::config::Level::Info,
            target: "access_log".to_string(),
            auth_token: None,
            app_id: Some("my_app".to_string()),
        }),
        args: message,
        module_path: Some("middleware".to_string()),
        file: Some("access.rs".to_string()),
        line: Some(120),
    }
}

// 创建性能日志记录
fn create_perf_record(message: String) -> Record {
    Record {
        metadata: Arc::new(Metadata {
            level: rat_logger::config::Level::Debug,
            target: "perf_monitor".to_string(),
            auth_token: None,
            app_id: Some("my_app".to_string()),
        }),
        args: message,
        module_path: Some("monitor".to_string()),
        file: Some("perf.rs".to_string()),
        line: Some(35),
    }
}