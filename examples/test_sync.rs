//! 简单测试同步模式

use rat_logger::{LoggerBuilder, LevelFilter, info, error};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 测试同步模式 ===");

    LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_async_mode(false)  // 设置同步模式
        .add_terminal()
        .init_global_logger()?;

    println!("初始化完成");

    info!("这是一条INFO日志");
    error!("这是一条ERROR日志");

    println!("日志发送完成");
    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("=== 测试完成 ===");
    Ok(())
}