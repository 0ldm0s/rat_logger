fn main() {
    println!("=== 测试 fmt().init() ===");
    let result = rat_logger::fmt().init();
    println!("初始化结果: {:?}", result);
    
    // 检查最大级别
    println!("最大级别: {:?}", rat_logger::core::max_level());
    
    // 检查全局日志器是否存在
    {
        let logger_guard = rat_logger::core::LOGGER.lock().unwrap();
        println!("全局日志器存在: {}", logger_guard.is_some());
    }
    
    // 测试级别检查
    println!("Info.should_log_at(Info): {}", rat_logger::Level::Info.should_log_at(rat_logger::LevelFilter::Info));
    println!("Error.should_log_at(Info): {}", rat_logger::Level::Error.should_log_at(rat_logger::LevelFilter::Info));
    
    // 调用日志
    println!("调用 info!()...");
    rat_logger::info!("测试 info 日志");
    
    println!("调用 error!()...");
    rat_logger::error!("测试 error 日志");
    
    println!("=== 完成 ===");
}
