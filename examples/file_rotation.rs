//! 文件轮转示例

use rat_logger::*;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn main() {
    let file_config = FileConfig {
        log_dir: PathBuf::from("./logs/rotation"),
        max_file_size: 1024, // 1KB - 很小以测试轮转
        max_compressed_files: 5,
        compression_level: 4,
        ..Default::default()
    };

    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .init()
        .unwrap();

    println!("开始测试文件轮转...");

    // 生成大量日志以触发轮转
    for i in 0..100 {
        info!("这是第 {} 条日志，用于测试文件轮转功能", i);
        warn!("警告日志 {}", i);
        error!("错误日志 {}", i);

        // 小延迟确保时间戳不同
        thread::sleep(Duration::from_millis(10));
    }

    println!("日志生成完成，请检查 ./logs/rotation 目录");
}