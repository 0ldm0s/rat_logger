//! 过滤日志示例

use rat_logger::*;
use std::path::PathBuf;
use std::sync::Arc;

fn main() {
    let file_config = FileConfig {
        log_dir: PathBuf::from("./logs/filtered"),
        max_file_size: 1024 * 1024,
        ..Default::default()
    };

    // 创建带过滤的组合处理器
    let mut filtered = crate::handler::composite::FilteredCompositeHandler::new()
        .with_level_filter(LevelFilter::Warn) // 只记录警告及以上级别
        .with_target_filter(vec!["filtered_logging"]); // 只记录特定模块

    filtered.add_handler(Arc::new(crate::handler::term::TermHandler::new()));
    filtered.add_handler(Arc::new(crate::handler::file::FileHandler::new(file_config)));

    let logger = LoggerCore::new(LevelFilter::Trace);
    logger.add_handler(Arc::new(filtered));

    core::set_logger(Arc::new(logger)).unwrap();

    // 这些日志会被记录（警告及以上，且模块名匹配）
    error!("这条错误日志会被记录");
    warn!("这条警告日志会被记录");

    // 这些日志不会被记录（级别不够）
    info!("这条信息日志会被过滤掉");
    debug!("这条调试日志会被过滤掉");
    trace!("这条跟踪日志会被过滤掉");
}