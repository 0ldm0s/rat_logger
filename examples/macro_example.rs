//! rat_logger 日志宏使用示例
//!
//! 演示如何使用rat_logger的日志宏，类似于标准log库的使用方式
//!
//! ⚠️  重要提醒：
//! - 本示例启用开发模式以确保日志立即输出，方便演示和学习
//! - 在生产环境中，请禁用开发模式以获得最佳性能
//! - 生产环境推荐：LoggerBuilder::new().add_terminal().init()

use rat_logger::{LoggerBuilder, LevelFilter, FileConfig, error, warn, info, debug, trace, Logger};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_logger 日志宏使用示例 ===\n");

    // 1. 初始化全局日志器
    println!("1. 初始化全局日志器:");
    LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true) // 示例启用开发模式，确保日志立即输出
        .add_terminal()
        .init()?;
    println!("   ✓ 全局日志器已初始化\n");

    // 2. 使用日志宏记录不同级别的日志
    println!("2. 使用日志宏记录日志:");
    error!("这是一个错误日志");
    warn!("这是一个警告日志");
    info!("这是一个信息日志");
    debug!("这是一个调试日志");
    trace!("这是一个跟踪日志");
    println!();

    // 3. 在函数中使用日志宏
    println!("3. 在函数中使用日志宏:");
    process_data("test_data");
    calculate_result(10, 5);
    println!();

    // 4. 使用格式化参数
    println!("4. 使用格式化参数:");
    let user = "张三";
    let count = 42;
    let items = vec!["苹果", "香蕉", "橙子"];

    info!("用户 {} 登录系统", user);
    warn!("库存不足，剩余 {} 件", count);
    error!("处理失败: {:?}", items);
    println!();

    // 5. 自定义日志器 + 日志宏
    println!("5. 自定义日志器 + 日志宏:");
    let file_config = FileConfig {
        log_dir: PathBuf::from("./macro_logs"),
        max_file_size: 1024 * 1024, // 1MB
        max_compressed_files: 3,
        compression_level: 6,
        min_compress_threads: 1,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    let custom_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Trace)
        .with_dev_mode(true) // 示例启用开发模式，确保日志立即输出
        .add_terminal()
        .add_file(file_config)
        .build();

    // 注意：日志宏使用的是全局日志器，这里为了演示创建自定义日志器
    let record = rat_logger::config::Record {
        metadata: std::sync::Arc::new(rat_logger::config::Metadata {
            level: rat_logger::Level::Info,
            target: "custom_logger".to_string(),
            auth_token: None,
            app_id: Some("macro_app".to_string()),
        }),
        args: "自定义日志器记录的消息".to_string(),
        module_path: Some("macro_example".to_string()),
        file: Some("macro_example.rs".to_string()),
        line: Some(71),
    };
    custom_logger.log(&record);
    println!();

    println!("=== 示例完成 ===");
    println!("日志文件已保存到 ./macro_logs 目录");

    Ok(())
}

// 示例函数
fn process_data(data: &str) {
    info!("开始处理数据: {}", data);

    if data.is_empty() {
        error!("数据为空，无法处理");
        return;
    }

    debug!("数据长度: {}", data.len());

    // 模拟处理过程
    for (i, ch) in data.chars().enumerate() {
        trace!("处理字符 {}: {}", i, ch);
    }

    info!("数据处理完成");
}

fn calculate_result(a: i32, b: i32) -> i32 {
    info!("计算 {} + {}", a, b);

    if b == 0 {
        error!("除数不能为零");
        return 0;
    }

    let result = a + b;
    debug!("计算结果: {}", result);

    if result > 100 {
        warn!("计算结果较大: {}", result);
    }

    result
}