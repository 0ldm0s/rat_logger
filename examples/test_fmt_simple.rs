fn main() {
    println!("=== 测试 fmt().init() ===");
    let result = rat_logger::fmt().init();
    println!("初始化结果: {:?}", result);
    
    rat_logger::info!("测试 info 日志");
    rat_logger::error!("测试 error 日志");
    println!("=== 完成 ===");
}
