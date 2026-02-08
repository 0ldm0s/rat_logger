//! 测试空闲时 CPU 使用率
//! 运行此程序后，在另一个终端用 htop 或 ps 查看进程 CPU 使用率
//! 预期：当程序进入空闲状态后，CPU 使用率应该接近 0.0%

use rat_logger::{LoggerBuilder, LevelFilter, Level, config::Record, Logger};
use rat_logger::config::Metadata;
use std::sync::Arc;

fn main() {
    // 初始化日志器 - 设置为 Error 级别，过滤掉大部分日志
    let terminal_logger = LoggerBuilder::new()
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .with_level(LevelFilter::Error)
        .build();

    println!("日志器已初始化，LevelFilter::Error");
    println!("即将进入空闲循环，程序将一直运行...");
    println!("请在另一个终端使用以下命令查看 CPU 使用率:");
    println!("  ps -p {} -o %cpu,cmd", std::process::id());
    println!("  或使用 htop 观察此进程");
    println!();

    // 先输出一些日志，然后进入空闲状态
    for i in 0..10 {
        let record = Record {
            metadata: Arc::new(Metadata {
                level: Level::Error,
                target: "test_idle_cpu".to_string(),
                auth_token: None,
                app_id: None,
            }),
            args: format!("启动日志 #{}", i),
            module_path: Some("test_idle_cpu".to_string()),
            file: Some("test_idle_cpu.rs".to_string()),
            line: Some(i),
        };
        terminal_logger.log(&record);
    }

    println!("已完成启动日志，现在进入空闲状态...");
    println!("主线程将只调用被过滤的日志调用，应该在 __private_log_impl 的快速路径返回");

    // 创建被过滤级别的日志记录（会被快速路径过滤掉）
    let filtered_record = Record {
        metadata: Arc::new(Metadata {
            level: Level::Debug,  // Debug < Error，会被过滤
            target: "test_idle_cpu".to_string(),
            auth_token: None,
            app_id: None,
        }),
        args: "被过滤的 debug 日志".to_string(),
        module_path: Some("test_idle_cpu".to_string()),
        file: Some("test_idle_cpu.rs".to_string()),
        line: Some(100),
    };

    // 空闲循环：只调用被过滤的日志
    let mut counter = 0u64;
    loop {
        // 这个日志会被级别过滤掉，应该在 __private_log_impl 的快速路径返回
        terminal_logger.log(&filtered_record);

        counter = counter.wrapping_add(1);

        // 每 1000 万次循环输出一次状态（不会被日志记录，只是 println）
        if counter % 10_000_000 == 0 {
            // 使用 println 而不是日志，避免干扰测试
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }

        // 防止 CPU 过载，让线程稍微休眠
        std::thread::sleep(std::time::Duration::from_micros(100));
    }
}
