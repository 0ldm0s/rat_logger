use rat_logger::{LoggerBuilder, LevelFilter, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始测试全局日志器...");

    // 只初始化全局日志器
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .init()?;

    println!("全局日志器初始化完成");

    println!("即将调用宏...");
    info!("第一条日志消息");
    println!("第一条日志应该已经输出了");

    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("即将调用第二条宏...");
    info!("第二条日志消息");
    println!("第二条日志应该已经输出了");

    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}