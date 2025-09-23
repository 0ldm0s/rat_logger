//! 简单测试异步模式

use rat_logger::{LoggerBuilder, LevelFilter, info, error, producer_consumer::BatchConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 测试异步模式 ===");

    LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_async_mode(true)   // 设置异步模式
        .with_batch_config(BatchConfig {
            batch_size: 2,
            batch_interval_ms: 100,
            buffer_size: 1024,
        })
        .add_terminal()
        .init_global_logger()?;

    println!("初始化完成");

    info!("这是第一条INFO日志");
    info!("这是第二条INFO日志");

    println!("发送了2条INFO日志，等待批量处理...");
    std::thread::sleep(std::time::Duration::from_millis(150));

    error!("这是一条ERROR日志");

    println!("发送了1条ERROR日志，等待超时处理...");
    std::thread::sleep(std::time::Duration::from_millis(200));

    println!("=== 测试完成 ===");
    Ok(())
}