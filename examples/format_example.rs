//! rat_logger 格式配置示例
//!
//! 演示如何使用rat_logger的格式配置功能

use rat_logger::{TermHandler, FileHandler, Logger};
use rat_logger::handler::LogHandler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_logger 格式配置示例 ===\n");

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
    println!("4. 测试配置组合:");

    // 4.1 默认无颜色
    println!("   4.1 默认无颜色格式:");
    let logger1 = TermHandler::with_format(simple_format.clone());

    logger1.handle(&create_test_record(rat_logger::Level::Error, "test", "错误消息"));
    logger1.handle(&create_test_record(rat_logger::Level::Info, "test", "信息消息"));

    // 4.2 带颜色的详细格式
    println!("   4.2 带颜色的详细格式:");
    let logger2 = TermHandler::with_format_and_color(detailed_format.clone(), color_config.clone());

    logger2.handle(&create_test_record(rat_logger::Level::Error, "test", "错误消息"));
    logger2.handle(&create_test_record(rat_logger::Level::Info, "test", "信息消息"));

    // 4.3 文件格式配置
    println!("   4.3 文件格式配置:");
    let file_config = rat_logger::FileConfig {
        log_dir: "./format_logs".into(),
        max_file_size: 1024 * 1024,
        max_compressed_files: 2,
        compression_level: 6,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    let file_format = rat_logger::FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S".to_string(),
        level_style: rat_logger::LevelStyle {
            error: "[ERROR]".to_string(),
            warn: "[WARN] ".to_string(),
            info: "[INFO] ".to_string(),
            debug: "[DEBUG]".to_string(),
            trace: "[TRACE]".to_string(),
        },
        format_template: "{timestamp} {level} {message}".to_string(),
    };

    let logger3 = FileHandler::new(file_config).with_format(file_format);

    logger3.handle(&create_test_record(rat_logger::Level::Error, "file_test", "文件错误消息"));
    logger3.handle(&create_test_record(rat_logger::Level::Info, "file_test", "文件信息消息"));
    logger3.flush();

    println!("\n=== 示例完成 ===");
    println!("格式配置说明:");
    println!("- FormatConfig: 控制输出格式，适用于所有处理器");
    println!("- ColorConfig: 只控制终端颜色，默认无颜色");
    println!("- 每个处理器可以设置不同的FormatConfig");
    println!("- 只有TermHandler可以使用ColorConfig");

    Ok(())
}

fn create_test_record(
    level: rat_logger::Level,
    target: &str,
    message: &str,
) -> rat_logger::config::Record {
    rat_logger::config::Record {
        metadata: std::sync::Arc::new(rat_logger::config::Metadata {
            level,
            target: target.to_string(),
            auth_token: None,
            app_id: Some("format_app".to_string()),
        }),
        args: message.to_string(),
        module_path: Some("format_example".to_string()),
        file: Some("format_example.rs".to_string()),
        line: Some(42),
    }
}