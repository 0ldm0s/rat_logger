//! rat_logger 同步异步模式演示示例
//!
//! 专门演示force_sync配置对日志写入行为的影响
//!
//! 对比测试：
//! - 异步模式：force_sync = false (高性能，适合大多数场景)
//! - 同步模式：force_sync = true (数据安全，适合关键业务)

use rat_logger::{LoggerBuilder, LevelFilter, Level, FileConfig, Logger};
use rat_logger::config::Record;
use rat_logger::config::Metadata;
use std::sync::Arc;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_logger 同步异步模式对比演示 ===\n");

    // 测试1: 异步模式 (force_sync = false)
    println!("1. 异步模式测试 (force_sync = false):");
    println!("   特点：高性能，批量写入，适合大多数场景");

    let async_config = FileConfig {
        log_dir: PathBuf::from("./async_logs"),
        max_file_size: 1024 * 1024, // 1MB
        max_compressed_files: 3,
        compression_level: 6,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: false, // 异步写入，性能更好
        format: None,
    };

    let async_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        // .with_dev_mode(true) // 注释掉开发模式，使用正常的批量处理
        .add_file(async_config)
        .build();

    let start_time = std::time::Instant::now();

    // 快速写入100条日志
    for i in 0..100 {
        let record = Record {
            metadata: Arc::new(Metadata {
                level: Level::Info,
                target: "async_test".to_string(),
                auth_token: None,
                app_id: Some("async_demo".to_string()),
            }),
            args: format!("异步日志消息 #{}", i),
            module_path: Some("sync_async_demo".to_string()),
            file: Some("sync_async_demo.rs".to_string()),
            line: Some(42),
        };
        async_logger.log(&record);
    }

    let async_duration = start_time.elapsed();
    println!("   ✓ 异步模式写入100条日志耗时: {:?}", async_duration);

    // 等待异步写入完成
    thread::sleep(Duration::from_millis(100));

    // 测试2: 同步模式 (force_sync = true)
    println!("\n2. 同步模式测试 (force_sync = true):");
    println!("   特点：数据安全，立即写入磁盘，适合关键业务");

    let sync_config = FileConfig {
        log_dir: PathBuf::from("./sync_logs"),
        max_file_size: 1024 * 1024, // 1MB
        max_compressed_files: 3,
        compression_level: 6,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: true, // 同步写入，确保数据安全
        format: None,
    };

    let sync_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        // .with_dev_mode(true) // 注释掉开发模式
        .add_file(sync_config)
        .build();

    let start_time = std::time::Instant::now();

    // 快速写入100条日志
    for i in 0..100 {
        let record = Record {
            metadata: Arc::new(Metadata {
                level: Level::Info,
                target: "sync_test".to_string(),
                auth_token: None,
                app_id: Some("sync_demo".to_string()),
            }),
            args: format!("同步日志消息 #{}", i),
            module_path: Some("sync_async_demo".to_string()),
            file: Some("sync_async_demo.rs".to_string()),
            line: Some(42),
        };
        sync_logger.log(&record);
    }

    let sync_duration = start_time.elapsed();
    println!("   ✓ 同步模式写入100条日志耗时: {:?}", sync_duration);

    // 等待同步写入完成
    thread::sleep(Duration::from_millis(100));

    // 性能对比
    println!("\n3. 性能对比:");
    println!("   异步模式耗时: {:?}", async_duration);
    println!("   同步模式耗时: {:?}", sync_duration);

    if sync_duration > async_duration {
        let speedup = sync_duration.as_nanos() as f64 / async_duration.as_nanos() as f64;
        println!("   异步模式比同步模式快 {:.2} 倍", speedup);
    }

    // 测试3: 混合场景演示
    println!("\n4. 混合场景演示 (不同类型日志使用不同模式):");

    // 普通业务日志 - 异步模式
    let business_config = FileConfig {
        log_dir: PathBuf::from("./business_logs"),
        max_file_size: 1024 * 1024,
        max_compressed_files: 5,
        compression_level: 6,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: false, // 业务日志异步写入，追求性能
        format: None,
    };

    // 关键错误日志 - 同步模式
    let error_config = FileConfig {
        log_dir: PathBuf::from("./critical_error_logs"),
        max_file_size: 1024 * 1024,
        max_compressed_files: 10,
        compression_level: 6,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: true, // 错误日志同步写入，确保不丢失
        format: None,
    };

    println!("   业务日志 (异步) 和 错误日志 (同步) 同时写入测试...");

    // 模拟混合日志写入
    for i in 0..50 {
        // 业务日志
        let business_record = Record {
            metadata: Arc::new(Metadata {
                level: Level::Info,
                target: "business".to_string(),
                auth_token: None,
                app_id: Some("business_app".to_string()),
            }),
            args: format!("用户操作日志 #{}", i),
            module_path: Some("sync_async_demo".to_string()),
            file: Some("sync_async_demo.rs".to_string()),
            line: Some(42),
        };

        // 错误日志 (每10条业务日志产生1条错误日志)
        if i % 10 == 0 {
            let error_record = Record {
                metadata: Arc::new(Metadata {
                    level: Level::Error,
                    target: "critical_error".to_string(),
                    auth_token: None,
                    app_id: Some("error_app".to_string()),
                }),
                args: format!("严重错误！处理失败，ID: {}", i),
                module_path: Some("sync_async_demo.rs".to_string()),
                file: Some("sync_async_demo.rs".to_string()),
                line: Some(42),
            };

            // 使用不同的日志器
            let error_logger = LoggerBuilder::new()
                .with_level(LevelFilter::Error)
                .add_file(error_config.clone())
                .build();
            error_logger.log(&error_record);
        }

        let business_logger = LoggerBuilder::new()
            .with_level(LevelFilter::Info)
            .add_file(business_config.clone())
            .build();
        business_logger.log(&business_record);
    }

    println!("   ✓ 混合场景测试完成");

    // 验证日志文件
    println!("\n5. 验证生成的日志文件:");

    let log_dirs = ["./async_logs", "./sync_logs", "./business_logs", "./critical_error_logs"];

    for dir in &log_dirs {
        if std::path::Path::new(dir).exists() {
            println!("   📁 {}:", dir);
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("log") {
                        println!("      📄 {}", path.display());
                        println!("         大小: {} bytes", entry.metadata().unwrap().len());

                        // 显示最后一条日志内容
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let lines: Vec<&str> = content.lines().collect();
                            if let Some(last_line) = lines.last() {
                                println!("         最后一条: {}", last_line);
                            }
                        }
                    }
                }
            }
        } else {
            println!("   ❌ {} 目录不存在", dir);
        }
    }

    println!("\n=== 演示完成 ===");
    println!("配置建议:");
    println!("- 普通业务日志：使用 force_sync = false，获得更好的性能");
    println!("- 关键错误日志：使用 force_sync = true，确保数据安全");
    println!("- 访问日志：使用 force_sync = false，适合高并发场景");
    println!("- 审计日志：使用 force_sync = true，确保合规性要求");

    Ok(())
}