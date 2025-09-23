use rat_logger::{LoggerBuilder, LevelFilter, info, Logger};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始测试...");

    // 初始化全局日志器
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal()
        .init()?;

    println!("全局日志器初始化完成");

    // 创建独立的日志器实例
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal()
        .build();

    println!("创建独立日志器完成");

    println!("\n=== 测试1：独立日志器直接调用 ===");
    let record = rat_logger::config::Record {
        metadata: std::sync::Arc::new(rat_logger::config::Metadata {
            level: rat_logger::Level::Info,
            target: "direct_test".to_string(),
            auth_token: None,
            app_id: Some("direct_test_app".to_string()),
        }),
        args: "独立日志器直接调用".to_string(),
        module_path: Some("direct_test".to_string()),
        file: Some("direct_test.rs".to_string()),
        line: Some(42),
    };

    println!("调用独立日志器之前，logger type_id: {:?}", logger.type_id());
    logger.log(&record);
    println!("调用独立日志器之后");

    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("\n=== 测试2：全局日志器宏调用 ===");
    println!("调用宏之前");
    info!("全局日志器宏调用");
    println!("调用宏之后");

    std::thread::sleep(std::time::Duration::from_millis(100));

    Ok(())
}