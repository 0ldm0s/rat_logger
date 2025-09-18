//! 多处理器示例

use rat_logger::*;
use std::path::PathBuf;

fn main() {
    let file_config = FileConfig {
        log_dir: PathBuf::from("./logs"),
        max_file_size: 1024 * 1024, // 1MB
        ..Default::default()
    };

    let network_config = NetworkConfig {
        server_addr: "127.0.0.1".to_string(),
        server_port: 5140,
        auth_token: "test_token".to_string(),
        app_id: "multi_handler_app".to_string(),
    };

    LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()
        .add_file(file_config)
        .add_udp(network_config)
        .init()
        .unwrap();

    info!("日志将同时输出到终端、文件和网络");
    warn!("这是多处理器测试");
    error!("测试错误日志");
}