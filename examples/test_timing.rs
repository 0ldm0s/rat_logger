// 简单的测试程序来验证时序问题
use rat_logger::{LoggerBuilder, LevelFilter};

fn main() {
    println!("开始初始化...");
    let logger = LoggerBuilder::new()
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .with_level(LevelFilter::Error)
        .build();

    println!("build() 完成，现在等待工作线程就绪...");
    match logger.wait_for_workers_ready(5000) {
        Ok(_) => println!("工作线程就绪成功"),
        Err(e) => println!("工作线程就绪失败: {}", e),
    }

    println!("程序继续执行...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    println!("程序结束");
}
