use rat_logger::{LoggerBuilder, LevelFilter, Level, config::Record, Logger};
use rat_logger::config::Metadata;
use std::sync::Arc;

fn main() {
    let logger = LoggerBuilder::new()
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .with_level(LevelFilter::Error)
        .build();

    println!("日志器已初始化");
    println!("PID: {}", std::process::id());
    println!("每秒产生一条 Error 日志...");
    
    let mut counter = 0;
    loop {
        let record = Record {
            metadata: Arc::new(Metadata {
                level: Level::Error,
                target: "test".to_string(),
                auth_token: None,
                app_id: None,
            }),
            args: format!("日志 #{}", counter),
            module_path: None,
            file: None,
            line: None,
        };
        logger.log(&record);
        counter += 1;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
