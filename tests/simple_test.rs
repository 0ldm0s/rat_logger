//! 简单的架构测试
//! 测试新的广播模式是否正常工作

use rat_logger::{LoggerBuilder, LevelFilter, Level, Logger};
use rat_logger::config::{Record, Metadata};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_basic_logging() {
    println!("=== 基础日志测试 ===");

    // 创建一个简单的logger
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal()
        .build();

    // 创建测试记录
    let record = Record {
        metadata: Arc::new(Metadata {
            level: Level::Info,
            target: "test".to_string(),
            auth_token: None,
            app_id: Some("test_app".to_string()),
        }),
        args: "测试消息".to_string(),
        module_path: Some("test".to_string()),
        file: Some("test.rs".to_string()),
        line: Some(42),
    };

    // 记录日志
    logger.log(&record);
    println!("日志记录完成");

    // 等待异步处理完成
    thread::sleep(Duration::from_millis(500));
    println!("测试完成");
}

#[test]
fn test_level_filtering() {
    println!("=== 级别过滤测试 ===");

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Warn)
        .add_terminal()
        .build();

    // 只应该显示 Warn 和 Error
    let levels = [Level::Error, Level::Warn, Level::Info, Level::Debug, Level::Trace];

    for level in levels.iter() {
        let record = Record {
            metadata: Arc::new(Metadata {
                level: *level,
                target: "filter_test".to_string(),
                auth_token: None,
                app_id: Some("filter_app".to_string()),
            }),
            args: format!("{} 级别消息", level),
            module_path: Some("filter_test.rs".to_string()),
            file: Some("filter_test.rs".to_string()),
            line: Some(1),
        };

        println!("发送 {} 级别日志", level);
        logger.log(&record);
    }

    // 等待异步处理完成
    thread::sleep(Duration::from_millis(500));
    println!("过滤测试完成");
}