//! 测试真正空闲时的 CPU 使用率
//! 这个程序只初始化日志器然后完全休眠，不产生任何日志
//! 这样可以真正测量工作线程在永久阻塞时的 CPU 使用率

use rat_logger::{LoggerBuilder, LevelFilter};

fn main() {
    println!("初始化日志器...");

    // 初始化日志器并保持引用
    let _logger = LoggerBuilder::new()
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .with_level(LevelFilter::Error)
        .build();

    println!("日志器已初始化，LevelFilter::Error");
    println!("工作线程应该永久阻塞等待日志（0% CPU）");
    println!("PID: {}", std::process::id());
    println!();
    println!("现在进入完全休眠，不产生任何日志...");
    println!("请在另一个终端使用以下命令查看 CPU 使用率:");
    println!("  ps -p {} -o %cpu,cmd", std::process::id());
    println!();

    // 完全休眠，不产生任何日志
    loop {
        std::thread::sleep(std::time::Duration::from_secs(60));
    }
}
