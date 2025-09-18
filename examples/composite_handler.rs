//! 组合处理器示例

use rat_logger::*;
use std::path::PathBuf;
use std::sync::Arc;

fn main() {
    let file_config = FileConfig {
        log_dir: PathBuf::from("./logs/composite"),
        max_file_size: 512 * 1024, // 512KB
        ..Default::default()
    };

    // 创建并行组合处理器
    let mut composite = crate::handler::composite::CompositeHandler::new()
        .with_parallel();

    // 添加多个处理器
    composite.add_handler(Arc::new(crate::handler::term::TermHandler::new()));
    composite.add_handler(Arc::new(crate::handler::file::FileHandler::new(file_config)));

    let logger = LoggerCore::new(LevelFilter::Info);
    logger.add_handler(Arc::new(composite));

    // 设置全局日志器
    core::set_logger(Arc::new(logger)).unwrap();

    info!("使用组合处理器，日志将并行处理");
    warn!("组合处理器测试");
    error!("错误日志将通过多个处理器并行处理");
}