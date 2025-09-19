//! rat_logger 格式配置示例
//!
//! 演示如何使用rat_logger的格式配置功能
//!
//! ⚠️  重要提醒：
//! - 本示例启用开发模式以确保日志立即输出，方便演示和学习
//! - 在生产环境中，请禁用开发模式以获得最佳性能
//! - 生产环境推荐：LoggerBuilder::new().add_terminal().build()

use rat_logger::{LoggerBuilder, LevelFilter, Level, FileConfig, NetworkConfig, FormatConfig, ColorConfig, Logger};
use rat_logger::config::{Record, Metadata};
use std::sync::Arc;
use std::path::PathBuf;

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

    // 4.1 基本终端格式测试
    println!("   4.1 基本终端格式:");
    let logger1 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true) // 示例启用开发模式，确保日志立即输出
        .add_terminal()
        .build();

    logger1.log(&create_test_record(Level::Error, "test", "错误消息"));
    logger1.log(&create_test_record(Level::Info, "test", "信息消息"));

    // 4.2 文件格式测试
    println!("   4.2 文件格式:");
    let file_config = FileConfig {
        log_dir: PathBuf::from("./format_logs"),
        max_file_size: 1024 * 1024,
        max_compressed_files: 2,
        compression_level: 6,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    let logger2 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true) // 示例启用开发模式，确保日志立即输出
        .add_file(file_config)
        .build();

    logger2.log(&create_test_record(Level::Error, "file_test", "文件错误消息"));
    logger2.log(&create_test_record(Level::Info, "file_test", "文件信息消息"));

    // 4.3 网络日志测试
    println!("   4.3 网络日志:");
    let network_config = rat_logger::NetworkConfig {
        server_addr: "127.0.0.1".to_string(),
        server_port: 54321,
        auth_token: "format_token".to_string(),
        app_id: "format_app".to_string(),
    };

    let logger3 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true) // 示例启用开发模式，确保日志立即输出
        .add_udp(network_config)
        .build();

    logger3.log(&create_test_record(Level::Error, "network_test", "网络错误消息"));
    logger3.log(&create_test_record(Level::Info, "network_test", "网络信息消息"));

    println!("\n=== 示例完成 ===");
    println!("格式配置说明:");
    println!("- FormatConfig: 控制输出格式，适用于所有处理器");
    println!("- ColorConfig: 只控制终端颜色，默认无颜色");
    println!("- 每个处理器可以设置不同的FormatConfig");
    println!("- 只有TermHandler可以使用ColorConfig");

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
            app_id: Some("format_app".to_string()),
        }),
        args: message.to_string(),
        module_path: Some("format_example".to_string()),
        file: Some("format_example.rs".to_string()),
        line: Some(42),
    }
}