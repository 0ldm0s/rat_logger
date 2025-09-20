//! rat_logger 按级别输出日志示例
//!
//! 展示如何使用LoggerBuilder初始化日志系统并输出不同级别的日志

use rat_logger::{LoggerBuilder, LevelFilter, Level, Logger};
use rat_logger::config::{Record, Metadata};
use std::sync::Arc;

#[test]
fn test_level_logging() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_logger 级别过滤直观演示 ===");
    println!("🎯 该测试清晰展示不同日志级别过滤器的效果\n");

    println!("📋 日志级别优先级（从高到低）:");
    println!("   ERROR > WARN > INFO > DEBUG > TRACE > OFF");
    println!("💡 只有优先级 >= 过滤器级别的消息才会显示\n");

    // 测试所有过滤级别
    let filter_configs = [
        (LevelFilter::Error, "🔴 ERROR级别过滤器", "只显示ERROR消息"),
        (LevelFilter::Warn, "🟠 WARN级别过滤器", "显示WARN、ERROR消息"),
        (LevelFilter::Info, "🟢 INFO级别过滤器", "显示INFO、WARN、ERROR消息"),
        (LevelFilter::Debug, "🔵 DEBUG级别过滤器", "显示DEBUG、INFO、WARN、ERROR消息"),
        (LevelFilter::Trace, "🟣 TRACE级别过滤器", "显示所有级别消息"),
        (LevelFilter::Off, "⚫ OFF级别过滤器", "不显示任何消息"),
    ];

    // 要发送的测试消息
    let test_messages = [
        (Level::Error, "🚨 ERROR - 系统错误"),
        (Level::Warn, "⚠️  WARN - 警告信息"),
        (Level::Info, "ℹ️  INFO - 一般信息"),
        (Level::Debug, "🔍 DEBUG - 调试信息"),
        (Level::Trace, "📝 TRACE - 详细跟踪"),
    ];

    for (filter_level, title, description) in filter_configs {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║ {}║", title);
        println!("║ {}║", description);
        println!("╚══════════════════════════════════════════════════════════════╝");

        // 使用LoggerBuilder创建过滤器
        let logger = LoggerBuilder::new()
            .with_level(filter_level)
            .with_dev_mode(true) // 确保立即输出
            .add_terminal()
            .build();

        println!("📤 发送测试消息到 {:?} 过滤器:", filter_level);

        for (msg_level, message) in test_messages {
            let will_show = match (filter_level, msg_level) {
                (LevelFilter::Off, _) => false,
                (LevelFilter::Error, Level::Error) => true,
                (LevelFilter::Error, _) => false,
                (LevelFilter::Warn, level) if level as u32 >= LevelFilter::Warn as u32 => true,
                (LevelFilter::Warn, _) => false,
                (LevelFilter::Info, level) if level as u32 >= LevelFilter::Info as u32 => true,
                (LevelFilter::Info, _) => false,
                (LevelFilter::Debug, level) if level as u32 >= LevelFilter::Debug as u32 => true,
                (LevelFilter::Debug, _) => false,
                (LevelFilter::Trace, _) => true,
                _ => false,
            };

            if will_show {
                println!("  ✅ 将显示: {}", message);
            } else {
                println!("  ❌ 将过滤: {}", message);
            }

            log_with_logger(&logger, msg_level, message);
        }

        println!("  ──────────────────────────────────────────────────────────");
        // 等待当前过滤器处理完成
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    println!("\n🎓 LoggerBuilder使用总结:");
    println!("🔧 LoggerBuilder::new()");
    println!("   .with_level(LevelFilter::Error)  // 最严格，只显示错误");
    println!("   .with_level(LevelFilter::Info)   // 生产环境推荐");
    println!("   .with_level(LevelFilter::Debug)  // 开发环境推荐");
    println!("   .with_level(LevelFilter::Trace)  // 最详细，调试用");
    println!("   .with_dev_mode(true)             // 开发模式，立即输出");
    println!("   .add_terminal()                  // 添加终端输出");
    println!("   .build()                         // 构建日志器");

    println!("\n🏗️  完整示例:");
    println!("```rust");
    println!("let logger = LoggerBuilder::new()");
    println!("    .with_level(LevelFilter::Info)");
    println!("    .with_dev_mode(true)");
    println!("    .add_terminal()");
    println!("    .build();");
    println!("```");

    println!("\n✅ 级别过滤演示完成");

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