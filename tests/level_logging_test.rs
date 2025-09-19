//! rat_logger 按级别输出日志示例
//!
//! 展示如何使用LoggerBuilder初始化日志系统并输出不同级别的日志

use rat_logger::{LoggerBuilder, LevelFilter, Level, Logger};
use rat_logger::config::{Record, Metadata};
use std::sync::Arc;

#[test]
fn test_level_logging() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_logger 按级别输出日志示例 ===\n");

    // 首先测试直接使用Logger是否正常工作
    println!("0. 测试直接使用Logger：");
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Trace)
        .with_dev_mode(true) // 启用开发模式确保日志立即输出
        .add_terminal()
        .build();

    let record = Record {
        metadata: Arc::new(Metadata {
            level: Level::Info,
            target: "test".to_string(),
            auth_token: None,
            app_id: Some("test".to_string()),
        }),
        args: "直接使用Logger的测试日志".to_string(),
        module_path: Some("test".to_string()),
        file: Some("test.rs".to_string()),
        line: Some(1),
    };
    logger.log(&record);
    println!("直接调用Logger.log()完成\n");

    // 测试宏是否能工作 - 只初始化一次
    println!("1. 测试宏输出（Trace级别）：");
    let _ = LoggerBuilder::new()
        .with_level(LevelFilter::Trace)
        .with_dev_mode(true) // 启用开发模式确保日志立即输出
        .add_terminal()
        .init();

    println!("现在测试宏输出：");
    rat_logger::error!("宏 - Error日志");
    rat_logger::warn!("宏 - Warn日志");
    rat_logger::info!("宏 - Info日志");
    rat_logger::debug!("宏 - Debug日志");
    rat_logger::trace!("宏 - Trace日志");

    // 测试不同的级别设置 - 使用独立的logger而不是全局初始化
    println!("\n2. 测试不同级别（使用独立Logger）：");

    // Debug级别
    println!("Debug级别：");
    let debug_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true) // 启用开发模式确保日志立即输出
        .add_terminal()
        .build();

    log_with_logger(&debug_logger, Level::Error, "Debug级别 - Error");
    log_with_logger(&debug_logger, Level::Warn, "Debug级别 - Warn");
    log_with_logger(&debug_logger, Level::Info, "Debug级别 - Info");
    log_with_logger(&debug_logger, Level::Debug, "Debug级别 - Debug");
    log_with_logger(&debug_logger, Level::Trace, "Debug级别 - Trace（不应该显示）");

    // Info级别
    println!("\nInfo级别：");
    let info_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .with_dev_mode(true) // 启用开发模式确保日志立即输出
        .add_terminal()
        .build();

    log_with_logger(&info_logger, Level::Error, "Info级别 - Error");
    log_with_logger(&info_logger, Level::Warn, "Info级别 - Warn");
    log_with_logger(&info_logger, Level::Info, "Info级别 - Info");
    log_with_logger(&info_logger, Level::Debug, "Info级别 - Debug（不应该显示）");
    log_with_logger(&info_logger, Level::Trace, "Info级别 - Trace（不应该显示）");

    // Warn级别
    println!("\nWarn级别：");
    let warn_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Warn)
        .with_dev_mode(true) // 启用开发模式确保日志立即输出
        .add_terminal()
        .build();

    log_with_logger(&warn_logger, Level::Error, "Warn级别 - Error");
    log_with_logger(&warn_logger, Level::Warn, "Warn级别 - Warn");
    log_with_logger(&warn_logger, Level::Info, "Warn级别 - Info（不应该显示）");
    log_with_logger(&warn_logger, Level::Debug, "Warn级别 - Debug（不应该显示）");
    log_with_logger(&warn_logger, Level::Trace, "Warn级别 - Trace（不应该显示）");

    // Error级别
    println!("\nError级别：");
    let error_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Error)
        .with_dev_mode(true) // 启用开发模式确保日志立即输出
        .add_terminal()
        .build();

    log_with_logger(&error_logger, Level::Error, "Error级别 - Error");
    log_with_logger(&error_logger, Level::Warn, "Error级别 - Warn（不应该显示）");
    log_with_logger(&error_logger, Level::Info, "Error级别 - Info（不应该显示）");
    log_with_logger(&error_logger, Level::Debug, "Error级别 - Debug（不应该显示）");
    log_with_logger(&error_logger, Level::Trace, "Error级别 - Trace（不应该显示）");

    // Off级别
    println!("\nOff级别：");
    let off_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Off)
        .with_dev_mode(true) // 启用开发模式确保日志立即输出
        .add_terminal()
        .build();

    log_with_logger(&off_logger, Level::Error, "Off级别 - Error（不应该显示）");
    log_with_logger(&off_logger, Level::Warn, "Off级别 - Warn（不应该显示）");
    log_with_logger(&off_logger, Level::Info, "Off级别 - Info（不应该显示）");
    log_with_logger(&off_logger, Level::Debug, "Off级别 - Debug（不应该显示）");
    log_with_logger(&off_logger, Level::Trace, "Off级别 - Trace（不应该显示）");

    println!("\n=== 示例完成 ===");
    println!("推荐使用LoggerBuilder::new().with_level(level).add_terminal().init()");
    println!("而不是使用已弃用的init()或init_with_level()函数");

    Ok(())
}

fn log_with_logger(logger: &dyn Logger, level: Level, message: &str) {
    let record = Record {
        metadata: Arc::new(Metadata {
            level,
            target: "level_example".to_string(),
            auth_token: None,
            app_id: Some("level_app".to_string()),
        }),
        args: message.to_string(),
        module_path: Some("level_logging_example".to_string()),
        file: Some("level_logging_example.rs".to_string()),
        line: Some(140),
    };
    logger.log(&record);
}