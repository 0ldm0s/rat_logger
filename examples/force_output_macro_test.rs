use rat_logger::{LoggerBuilder, LevelFilter, emergency, flush_logs, info, error};
use rat_logger::producer_consumer::BatchConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 强制输出验证测试（宏版本） ===");
    println!("配置：异步模式 + 大批量(100) + 大延迟(1000ms)");
    println!("预期：普通日志被缓冲，emergency日志强制输出\n");

    // 初始化全局日志器 - 异步模式+大批量+大延迟
    LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_batch_config(BatchConfig {
            batch_size: 100,              // 需要累积100条日志才输出
            batch_interval_ms: 1000,     // 或者延迟1000ms才输出
            buffer_size: 1024,
        })
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .init_global_logger()?;

    println!("✓ 全局日志器初始化完成\n");

    // 测试1：普通日志（应该被缓冲，不会立即输出）
    println!("1. 测试普通日志（预期：被缓冲，不立即输出）：");
    for i in 1..=5 {
        info!("普通日志消息 {} - 这条消息应该被缓冲", i);
    }
    println!("   已发送5条普通日志，等待1秒看看是否会输出...\n");

    // 等待1秒，检查是否会因为时间间隔而输出
    std::thread::sleep(std::time::Duration::from_millis(1100));

    // 测试2：强制输出日志（应该立即输出）
    println!("2. 测试强制输出日志（预期：立即输出）：");
    error!("这是强制输出的紧急日志1 - 应该立即输出");
    error!("这是强制输出的紧急日志2 - 应该立即输出");

    println!("   已发送紧急日志，应该立即看到输出\n");

    // 测试3：混合场景
    println!("3. 测试混合场景（普通日志+强制输出）：");
    info!("普通日志 A - 被缓冲");
    info!("普通日志 B - 被缓冲");
    error!("紧急日志 C - 强制输出（同时会刷新缓冲区的A和B）");
    info!("普通日志 D - 被缓冲");

    println!("   发送混合日志完成，紧急日志应该立即输出并刷新之前的缓冲区\n");

    // 测试4：手动刷新功能
    println!("4. 测试手动刷新功能：");
    info!("普通日志 E - 被缓冲");
    info!("普通日志 F - 被缓冲");
    println!("   手动调用刷新命令...");
    flush_logs!();

    // 再等待一小段时间确保刷新完成
    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("\n=== 测试总结 ===");
    println!("如果测试成功，你应该看到：");
    println!("1. 前5条普通日志在1秒后批量输出（时间间隔触发）");
    println!("2. 2条紧急日志立即输出");
    println!("3. 混合场景中的紧急日志立即输出并刷新之前的缓冲区");
    println!("4. 手动刷新命令立即输出缓冲的日志");

    Ok(())
}