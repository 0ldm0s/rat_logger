fn main() {
    println!("=== 测试 fmt().init() ===");
    let result = rat_logger::fmt().init();
    println!("初始化结果: {:?}", result);
    
    // 检查最大级别
    println!("最大级别: {:?}", rat_logger::core::max_level());
    
    // 手动调用日志
    println!("调用 info!()...");
    rat_logger::info!("测试 info 日志");
    
    println!("调用 error!()...");
    rat_logger::error!("测试 error 日志");
    
    println!("=== 完成 ===");
}
