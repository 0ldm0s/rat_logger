//! 测试全局日志器 vs 实例日志器

use rat_logger::{LoggerBuilder, LevelFilter, info, debug, warn, error, Logger, config::{Record, Metadata}};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 对比测试：全局日志器 vs 实例日志器 ===");

    // 第一部分：测试实例方式（basic_usage_no_dev.rs的方式）
    println!("\n=== 第一部分：实例方式 ===");
    println!("1. 创建实例日志器:");
    let instance_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()
        .build();
    println!("   实例创建完成");

    println!("2. 测试实例日志器:");
    let record1 = Record {
        metadata: Arc::new(Metadata {
            level: rat_logger::Level::Info,
            target: "instance_test".to_string(),
            auth_token: None,
            app_id: Some("instance_app".to_string()),
        }),
        args: "这是实例日志器的测试日志".to_string(),
        module_path: Some("test_global_logger".to_string()),
        file: Some("test_global_logger.rs".to_string()),
        line: Some(35),
    };
    instance_logger.log(&record1);
    println!("   实例日志记录完成");

    // 等待一下
    std::thread::sleep(std::time::Duration::from_millis(200));

    // 第二部分：测试全局方式
    println!("\n=== 第二部分：全局方式 ===");
    println!("1. 初始化全局日志器:");
    LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()
        .init()?;
    println!("   全局初始化完成");

    println!("2. 测试全局日志宏:");
    println!("   2.1 测试info宏:");
    info!("这是一条全局日志器的测试日志（宏）");
    println!("   2.2 测试debug宏:");
    debug!("这是一条debug级别的测试日志");
    println!("   2.3 测试warn宏:");
    warn!("这是一条warn级别的测试日志");
    println!("   2.4 测试error宏:");
    error!("这是一条error级别的测试日志");
    println!("   2.5 宏调用完成");

    println!("3. 测试全局日志器手动调用:");
    let record2 = Record {
        metadata: Arc::new(Metadata {
            level: rat_logger::Level::Info,
            target: "global_test".to_string(),
            auth_token: None,
            app_id: Some("global_app".to_string()),
        }),
        args: "这是全局日志器的手动测试日志".to_string(),
        module_path: Some("test_global_logger".to_string()),
        file: Some("test_global_logger.rs".to_string()),
        line: Some(65),
    };

    // 方法1：直接访问全局LOGGER变量
    println!("   方法1：直接访问全局LOGGER变量");
    if let Some(logger) = rat_logger::core::LOGGER.lock().unwrap().as_ref() {
        println!("   ✓ 找到全局日志器，开始记录...");
        logger.log(&record2);
        println!("   ✓ 全局手动日志记录完成");
    } else {
        println!("   ✗ 错误：未找到全局日志器！");
    }

    // 方法2：使用rat_logger::log函数（如果有）
    println!("   方法2：验证日志器内部状态");
    // 尝试不同的调用方式
    if let Some(logger) = rat_logger::core::LOGGER.lock().unwrap().as_ref() {
        println!("   ✓ 日志器对象存在");
        // 检查日志器的内部状态
        println!("   ✓ 日志器地址：{:p}", Arc::as_ptr(logger));
    } else {
        println!("   ✗ 无法获取日志器引用");
    }

    // 方法3：验证日志器状态和功能
    println!("   方法3：测试日志器基本功能");
    let record3 = Record {
        metadata: Arc::new(Metadata {
            level: rat_logger::Level::Debug,
            target: "global_test_debug".to_string(),
            auth_token: None,
            app_id: Some("global_app_debug".to_string()),
        }),
        args: "这是全局日志器的Debug级别测试日志".to_string(),
        module_path: Some("test_global_logger".to_string()),
        file: Some("test_global_logger.rs".to_string()),
        line: Some(95),
    };

    if let Some(logger) = rat_logger::core::LOGGER.lock().unwrap().as_ref() {
        println!("   ✓ 测试Debug级别日志...");
        logger.log(&record3);
        println!("   ✓ Debug级别日志记录完成");
    } else {
        println!("   ✗ 无法获取日志器进行Debug测试");
    }

    // 等待一下
    std::thread::sleep(std::time::Duration::from_millis(500));

    println!("\n=== 测试完成 ===");
    Ok(())
}