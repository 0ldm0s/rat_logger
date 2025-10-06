//! rat_logger 环境变量RUST_LOG配置示例
//!
//! 演示如何使用RUST_LOG环境变量自动配置日志级别
//! 验证环境变量初始化的各种场景
//!
//! 测试规则：
//! 1. 无RUST_LOG + 无代码初始化 → 无输出
//! 2. 有RUST_LOG + 无代码初始化 → 默认配置输出
//! 3. 有RUST_LOG + 有代码初始化 → 忽略RUST_LOG
//! 4. 不同RUST_LOG值 (error, warn, info, debug, trace)

use rat_logger::{LoggerBuilder, LevelFilter, Level, FileConfig, Logger, parse_log_level_from_env};
use rat_logger::config::{Record, Metadata};
use std::sync::Arc;
use std::path::PathBuf;

// 导入rat_logger的宏
use rat_logger::{error, warn, info, debug, trace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_logger 环境变量RUST_LOG配置示例 ===\n");

    // 检查当前RUST_LOG环境变量
    match std::env::var("RUST_LOG") {
        Ok(val) => println!("检测到RUST_LOG环境变量: {}", val),
        Err(_) => println!("未设置RUST_LOG环境变量"),
    }

    // 检查解析后的日志级别
    match parse_log_level_from_env() {
        Some(level) => println!("解析后的日志级别: {:?}\n", level),
        None => println!("无法解析日志级别或未设置RUST_LOG\n"),
    }

    // 场景1: 仅使用环境变量，不进行代码初始化
    println!("=== 场景1: 仅使用环境变量（无代码初始化）===");
    println!("直接使用日志宏，依赖环境变量自动初始化:\n");

    error!("错误消息 - 应该显示");
    warn!("警告消息 - 应该显示（如果级别允许）");
    info!("信息消息 - 应该显示（如果级别允许）");
    debug!("调试消息 - 应该显示（如果级别允许）");
    trace!("跟踪消息 - 应该显示（如果级别允许）");

    // 等待一下确保输出完成
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 场景2: 测试解析函数
    println!("\n=== 场景2: 测试环境变量解析函数 ===");

    // 测试不同的环境变量值
    let test_values = ["error", "warn", "info", "debug", "trace", "invalid"];
    for &value in &test_values {
        unsafe { std::env::set_var("RUST_LOG", value); }
        match parse_log_level_from_env() {
            Some(level) => println!("RUST_LOG={} → {:?}", value, level),
            None => println!("RUST_LOG={} → 解析失败", value),
        }
    }

    println!("\n=== 测试完成 ===");
    println!("环境变量配置说明:");
    println!("- 设置RUST_LOG=error/warn/info/debug/trace来控制日志级别");
    println!("- 如果没有代码初始化，会使用默认配置自动初始化");
    println!("- 默认配置为同步模式输出到终端，带颜色和简洁格式");

    Ok(())
}