//! rat_logger 终端格式配置示例
//!
//! 专门演示如何配置终端输出的格式和颜色
//!
//! ⚠️  重要提醒：
//! - 本示例启用开发模式以确保日志立即输出，方便演示和学习
//! - 在生产环境中，请禁用开发模式以获得最佳性能
//! - 生产环境推荐：LoggerBuilder::new().add_terminal_with_config(config).build()

use rat_logger::{LoggerBuilder, LevelFilter, Level, Logger};
use rat_logger::config::{Record, Metadata};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_logger 终端格式配置示例 ===\n");

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
        level_templates: None,
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
        level_templates: None,
    };

    // 3. 创建颜色配置
    println!("3. 创建颜色配置:");
    let color_config = rat_logger::ColorConfig {
        error: "\x1b[91m".to_string(),      // 亮红色
        warn: "\x1b[93m".to_string(),       // 亮黄色
        info: "\x1b[92m".to_string(),       // 亮绿色
        debug: "\x1b[96m".to_string(),      // 亮青色
        trace: "\x1b[95m".to_string(),      // 亮紫色
        timestamp: "\x1b[90m".to_string(),   // 深灰色
        target: "\x1b[94m".to_string(),      // 亮蓝色
        file: "\x1b[95m".to_string(),       // 亮紫色
        message: "\x1b[97m".to_string(),      // 亮白色
    };

    println!("   ✓ 已创建配置\n");

    // 4. 测试不同配置组合
    println!("4. 测试不同配置组合:");

    // 4.1 默认格式
    println!("   4.1 默认格式（无颜色）:");
    let logger1 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .build();

    logger1.log(&create_test_record(Level::Error, "default_test", "错误消息"));
    logger1.log(&create_test_record(Level::Info, "default_test", "信息消息"));

    // 4.2 简洁格式（无颜色）
    println!("\n   4.2 简洁格式（无颜色）:");
    let term_config2 = rat_logger::handler::term::TermConfig {
        format: Some(simple_format.clone()),
        color: None,
        ..Default::default()
    };

    let logger2 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_terminal_with_config(term_config2)
        .build();

    logger2.log(&create_test_record(Level::Error, "simple_test", "错误消息"));
    logger2.log(&create_test_record(Level::Info, "simple_test", "信息消息"));

    // 4.3 简洁格式（带颜色）
    println!("\n   4.3 简洁格式（带颜色）:");
    let term_config3 = rat_logger::handler::term::TermConfig {
        format: Some(simple_format.clone()),
        color: Some(color_config.clone()),
        ..Default::default()
    };

    let logger3 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_terminal_with_config(term_config3)
        .build();

    logger3.log(&create_test_record(Level::Error, "color_test", "错误消息"));
    logger3.log(&create_test_record(Level::Info, "color_test", "信息消息"));

    // 4.4 详细格式（带颜色）
    println!("\n   4.4 详细格式（带颜色）:");
    let term_config4 = rat_logger::handler::term::TermConfig {
        format: Some(detailed_format.clone()),
        color: Some(color_config.clone()),
        ..Default::default()
    };

    let logger4 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_terminal_with_config(term_config4)
        .build();

    logger4.log(&create_test_record(Level::Error, "detailed_test", "错误消息"));
    logger4.log(&create_test_record(Level::Info, "detailed_test", "信息消息"));

    // 4.5 仅颜色（默认格式）
    println!("\n   4.5 仅颜色（默认格式）:");
    let term_config5 = rat_logger::handler::term::TermConfig {
        format: None,
        color: Some(color_config.clone()),
        ..Default::default()
    };

    let logger5 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_terminal_with_config(term_config5)
        .build();

    logger5.log(&create_test_record(Level::Error, "color_only_test", "错误消息"));
    logger5.log(&create_test_record(Level::Info, "color_only_test", "信息消息"));

    println!("\n=== 示例完成 ===");
    println!("终端格式配置说明:");
    println!("- TermConfig.format: 控制输出格式模板");
    println!("- TermConfig.color: 控制颜色主题");
    println!("- 两者可以独立配置，也可以组合使用");
    println!("- 格式模板支持: {{timestamp}}, {{level}}, {{target}}, {{file}}, {{line}}, {{message}}");

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
            app_id: Some("term_format_example".to_string()),
        }),
        args: message.to_string(),
        module_path: Some("term_format_example".to_string()),
        file: Some("term_format_example.rs".to_string()),
        line: Some(42),
    }
}