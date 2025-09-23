use rat_logger::{LoggerBuilder, LevelFilter, info, error, startup_log, emergency, flush_logs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 测试新的日志功能 ===");

    // 初始化全局日志器
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .init()?;

    println!("全局日志器初始化完成");

    // 测试启动日志 - 立即输出
    println!("\n1. 测试启动日志（立即输出）：");
    startup_log!("系统启动，版本：1.0.0");
    startup_log!("配置加载完成，监听端口：8080");
    startup_log!("数据库连接池大小：10");

    // 等待一下确保启动日志输出
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 测试普通日志 - 可能被缓冲
    println!("\n2. 测试普通日志（可能被缓冲）：");
    info!("用户登录请求：user123");
    info!("处理业务逻辑...");
    info!("计算结果：42");

    // 测试强制刷新
    println!("\n3. 测试强制刷新：");
    flush_logs!();
    println!("强制刷新完成");

    // 测试错误日志 - 自动使用紧急模式
    println!("\n4. 测试错误日志（自动紧急模式）：");
    info!("开始处理文件...");
    error!("文件读取失败：/path/to/file.txt");
    error!("权限不足，无法访问资源");
    info!("处理完成（但实际上错误前面的上下文会被一起输出）");

    // 测试Info级别日志（不使用紧急模式）
    println!("\n4.1. 测试Info级别日志（不使用紧急模式）：");
    info!("这是一个Info级别的日志，应该使用正常流程");
    info!("这是另一个Info级别的日志");

    // 测试紧急日志宏
    println!("\n5. 测试紧急日志宏：");
    emergency!("系统即将关闭，请保存所有工作！");
    emergency!("内存使用率过高：95%");

    // 等待所有日志输出
    std::thread::sleep(std::time::Duration::from_millis(500));

    println!("\n=== 测试完成 ===");
    Ok(())
}