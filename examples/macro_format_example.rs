//! rat_logger 宏格式配置示例
//!
//! 演示如何使用rat_logger宏与格式配置结合使用
//! 展示兼容标准日志库宏的使用方式
//!
//! ⚠️  重要提醒：
//! - 本示例启用开发模式以确保日志立即输出，方便演示和学习
//! - 在生产环境中，请禁用开发模式以获得最佳性能
//! - 生产环境推荐：LoggerBuilder::new().add_terminal_with_config(config).init()

use rat_logger::{LoggerBuilder, LevelFilter, Level, FileConfig, Logger};
use rat_logger::config::{Record, Metadata};
use std::sync::Arc;
use std::path::PathBuf;

// 导入rat_logger的宏
use rat_logger::{error, warn, info, debug, trace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_logger 宏格式配置示例 ===\n");

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

    // 4. 测试宏与不同格式配置的组合
    println!("4. 测试宏与不同格式配置的组合:");

    // 4.1 宏 + 简洁格式 + 颜色
    println!("   4.1 宏 + 简洁格式 + 颜色:");
    let term_config1 = rat_logger::handler::term::TermConfig {
        format: Some(simple_format.clone()),
        color: Some(color_config.clone()),
        ..Default::default()
    };

    // 初始化全局日志器
    LoggerBuilder::new()
        .with_level(LevelFilter::Trace)
        .with_dev_mode(true)
        .add_terminal_with_config(term_config1)
        .init()?;

    println!("   使用宏输出简洁格式日志:");
    error!("这是一个错误消息 - 来自宏");
    warn!("这是一个警告消息 - 来自宏");
    info!("这是一个信息消息 - 来自宏");
    debug!("这是一个调试消息 - 来自宏");
    trace!("这是一个跟踪消息 - 来自宏");

    // 4.2 宏 + 文件格式
    println!("\n   4.2 宏 + 文件格式:");
    let mut file_config = FileConfig {
        log_dir: PathBuf::from("./macro_format_logs"),
        max_file_size: 1024 * 1024,
        max_compressed_files: 2,
        compression_level: 6,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        format: Some(detailed_format.clone()),
    };

    // 重新初始化为文件输出（开发模式允许）
    LoggerBuilder::new()
        .with_level(LevelFilter::Trace)
        .with_dev_mode(true)
        .add_file(file_config)
        .init()?;

    println!("   使用宏输出到文件（详细格式）:");
    error!("这是一个错误消息 - 写入文件");
    warn!("这是一个警告消息 - 写入文件");
    info!("这是一个信息消息 - 写入文件");
    debug!("这是一个调试消息 - 写入文件");
    trace!("这是一个跟踪消息 - 写入文件");

    // 4.3 宏 + 混合输出（终端 + 文件）
    println!("\n   4.3 宏 + 混合输出（终端 + 文件）:");
    let term_config3 = rat_logger::handler::term::TermConfig {
        format: Some(simple_format.clone()),
        color: Some(color_config.clone()),
        ..Default::default()
    };

    let mut file_config2 = FileConfig {
        log_dir: PathBuf::from("./macro_mixed_logs"),
        max_file_size: 1024 * 1024,
        max_compressed_files: 2,
        compression_level: 6,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        format: Some(detailed_format.clone()),
    };

    LoggerBuilder::new()
        .with_level(LevelFilter::Trace)
        .with_dev_mode(true)
        .add_terminal_with_config(term_config3)
        .add_file(file_config2)
        .init()?;

    println!("   使用宏同时输出到终端和文件:");
    error!("这是一个错误消息 - 同时输出");
    warn!("这是一个警告消息 - 同时输出");
    info!("这是一个信息消息 - 同时输出");

    // 4.4 条件日志使用
    println!("\n   4.4 条件日志使用:");
    let should_log = true;
    let should_not_log = false;

    if should_log {
        error!("条件错误消息 - 应该显示");
    }
    if should_not_log {
        error!("条件错误消息 - 不应该显示");
    }

    // 4.5 不同模块的日志
    println!("\n   4.5 不同模块的日志:");
    // 模拟不同模块的日志输出
    println!("   模拟网络模块日志:");
    error!("网络连接失败");
    info!("网络连接成功");

    println!("   模拟数据库模块日志:");
    error!("数据库查询失败");
    info!("数据库查询成功");

    // 等待文件写入完成
    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("\n5. 查看生成的日志文件内容:");

    // 显示文件内容
    if let Ok(entries) = std::fs::read_dir("./macro_format_logs") {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("log") {
                println!("   详细格式文件: {}", entry.path().display());
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for line in content.lines().take(3) {
                        println!("     {}", line);
                    }
                }
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir("./macro_mixed_logs") {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("log") {
                println!("   混合输出文件: {}", entry.path().display());
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    for line in content.lines().take(3) {
                        println!("     {}", line);
                    }
                }
            }
        }
    }

    println!("\n=== 示例完成 ===");
    println!("宏格式配置说明:");
    println!("- rat_logger宏完全兼容标准日志库宏的使用方式");
    println!("- 支持所有标准级别：error!, warn!, info!, debug!, trace!");
    println!("- 支持条件日志：使用if语句控制");
    println!("- 支持模块化日志：在不同模块中使用");
    println!("- 可以与任何格式配置结合使用");
    println!("- 自动获取模块路径、文件名、行号等信息");

    Ok(())
}

// 自定义函数中使用宏
fn process_data(data: &str) -> Result<(), String> {
    info!("开始处理数据: {}", data);

    if data.is_empty() {
        warn!("输入数据为空");
        return Err("数据为空".to_string());
    }

    debug!("数据长度: {}", data.len());

    // 模拟处理过程
    if data.len() > 100 {
        error!("数据过长: {} 字节", data.len());
        return Err("数据过长".to_string());
    }

    info!("数据处理完成");
    Ok(())
}

// 自定义模块中的宏使用
mod network_module {
    use rat_logger::{info, error, warn};

    pub fn send_request(endpoint: &str) -> Result<(), String> {
        info!("网络模块: 发送请求到: {}", endpoint);

        if endpoint.starts_with("https") {
            info!("网络模块: HTTPS请求安全");
            Ok(())
        } else {
            error!("网络模块: 不安全的HTTP请求: {}", endpoint);
            Err("不安全协议".to_string())
        }
    }
}