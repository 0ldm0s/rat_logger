//! 快速初始化示例 - 类似 tracing_subscriber::fmt().init()
//!
//! 这个示例展示了如何使用 rat_logger::fmt() 进行快速初始化
//!
//! 运行方式：
//!   cargo run --example fmt_quick_init
//!   RUST_LOG=debug cargo run --example fmt_quick_init

use rat_logger::{error, warn, info, debug, trace};

fn main() {
    // 最简单的用法 - 完全等同于 tracing_subscriber::fmt().init()
    // 使用默认配置（Info 级别，默认格式）
    rat_logger::fmt().init();

    println!("=== 使用默认配置 (Info 级别) ===");
    info!("这是 info 日志（会显示）");
    debug!("这是 debug 日志（不会显示，因为默认级别是 Info）");
    error!("这是 error 日志（会显示）");

    // 注意：rat_logger 使用异步架构，日志在后台线程处理
    // 程序退出前建议刷新日志缓冲区
    rat_logger::flush_logs!();
    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("\n=== 提示 ===");
    println!("1. 你可以用环境变量控制级别：RUST_LOG=debug cargo run --example fmt_quick_init");
    println!("2. 长期运行的服务器程序不需要手动 flush，日志会持续输出");
}
