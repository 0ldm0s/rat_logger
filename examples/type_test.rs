use rat_logger::{LoggerBuilder, LevelFilter, Logger};
use std::any::Any;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 比较LoggerCore实例类型 ===");

    // 创建独立的LoggerCore实例
    let direct_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .build();

    println!("直接创建的logger type_id: {:?}", direct_logger.type_id());
    println!("直接创建的logger type_name: {:?}", std::any::type_name::<rat_logger::core::LoggerCore>());

    // 创建测试用的record
    let record = rat_logger::config::Record {
        metadata: std::sync::Arc::new(rat_logger::config::Metadata {
            level: rat_logger::Level::Info,
            target: "type_test".to_string(),
            auth_token: None,
            app_id: None,
        }),
        args: "测试日志消息".to_string(),
        module_path: Some("type_test".to_string()),
        file: Some("type_test.rs".to_string()),
        line: Some(42),
    };

    // 包装成Arc<dyn Logger>
    let wrapped_logger: Arc<dyn Logger> = Arc::new(direct_logger.clone());
    println!("包装后的logger type_id: {:?}", wrapped_logger.type_id());
    println!("包装后的logger type_name: {:?}", std::any::type_name::<Arc<dyn Logger>>());

    // 测试包装后的logger是否正常工作
    println!("即将调用wrapped_logger.log...");
    wrapped_logger.log(&record);
    println!("wrapped_logger.log调用完成");

    // 初始化全局日志器
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .init_global_logger()?;

    println!("全局日志器初始化完成");

    // 测试直接调用LoggerCore的log方法
    let record2 = rat_logger::config::Record {
        metadata: std::sync::Arc::new(rat_logger::config::Metadata {
            level: rat_logger::Level::Info,
            target: "type_test".to_string(),
            auth_token: None,
            app_id: None,
        }),
        args: "直接调用LoggerCore::log方法".to_string(),
        module_path: Some("type_test".to_string()),
        file: Some("type_test.rs".to_string()),
        line: Some(42),
    };

    println!("即将调用direct_logger.log...");
    direct_logger.log(&record2);
    println!("direct_logger.log调用完成");

    Ok(())
}