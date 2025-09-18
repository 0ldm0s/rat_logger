//! 基础使用示例

use rat_logger::*;

fn main() {
    // 初始化日志器
    init().unwrap();

    // 基础日志记录
    error!("这是一条错误日志");
    warn!("这是一条警告日志");
    info!("这是一条信息日志");
    debug!("这是一条调试日志");
    trace!("这是一条跟踪日志");
}