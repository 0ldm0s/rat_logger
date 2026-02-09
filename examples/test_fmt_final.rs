fn main() {
    println!("=== 测试 fmt().init() ===");
    let result = rat_logger::fmt().init();
    println!("初始化结果: {:?}", result);
    
    // 调用日志
    rat_logger::info!("测试 info 日志");
    rat_logger::error!("测试 error 日志");
    
    // 刷新日志缓冲区，等待异步处理完成
    rat_logger::flush_logs!();
    
    // 等待一下确保日志输出
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    println!("=== 完成 ===");
}
