use rat_logger::{LoggerBuilder, LevelFilter, info, Logger};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始测试...");

    // 创建独立的日志器实例（不使用全局）
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal()
        .build();

    println!("创建独立日志器后");

    let record = rat_logger::config::Record {
        metadata: std::sync::Arc::new(rat_logger::config::Metadata {
            level: rat_logger::Level::Info,
            target: "test".to_string(),
            auth_token: None,
            app_id: Some("test_app".to_string()),
        }),
        args: "来自独立日志器的消息".to_string(),
        module_path: Some("test".to_string()),
        file: Some("test.rs".to_string()),
        line: Some(42),
    };

    println!("即将调用日志器...");
    logger.log(&record);
    println!("日志调用完成，应该已经输出了");

    // 测试全局日志器
    println!("\n测试全局日志器...");
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal()
        .init()?;

    println!("全局日志器初始化完成");

    println!("即将调用宏...");
    info!("来自全局日志器宏的消息");
    println!("宏调用完成");

    Ok(())
}