# rat_logger

rat_logger 是一个用 Rust 编写的高性能、线程安全的日志库，采用生产者-消费者架构和异步写入机制，提供卓越的性能表现和灵活的配置选项。

## 特性

- **极致性能**: 采用生产者-消费者架构，实测文件写入性能可达 40万+ msg/sec
- **线程安全**: 完全线程安全，支持多线程并发写入，采用原子操作避免锁竞争
- **多种输出方式**: 支持终端、文件、UDP 网络等多种输出方式
- **分层配置**: 格式配置与颜色配置分离，默认无颜色主题
- **日志宏**: 兼容标准 log 库的宏接口，提供便捷的日志记录方式
- **结构化日志**: 支持结构化的日志记录和元数据
- **压缩支持**: 内置 LZ4 压缩功能，自动压缩旧日志文件
- **UDP 网络传输**: 支持通过 UDP 协议将日志发送到远程服务器
- **认证机制**: 支持基于令牌的认证机制

## 快速开始

### 使用日志宏（推荐）

```rust
use rat_logger::{init_with_level, LevelFilter, error, warn, info, debug, trace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化全局日志器
    init_with_level(LevelFilter::Debug)?;

    // 使用日志宏记录日志
    error!("这是一个错误日志");
    warn!("这是一个警告日志");
    info!("这是一个信息日志");
    debug!("这是一个调试日志");
    trace!("这是一个跟踪日志");

    Ok(())
}
```

### 自定义处理器配置

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig, FormatConfig, LevelStyle, ColorConfig, TermHandler};

fn main() {
    // 创建格式配置
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: LevelStyle {
            error: "ERROR".to_string(),
            warn: "WARN ".to_string(),
            info: "INFO ".to_string(),
            debug: "DEBUG".to_string(),
            trace: "TRACE".to_string(),
        },
        format_template: "[{level}] {timestamp} {target}:{line} - {message}".to_string(),
    };

    // 创建终端处理器（带颜色）
    let color_config = ColorConfig {
        error: "\x1b[91m".to_string(),      // 亮红色
        warn: "\x1b[93m".to_string(),       // 亮黄色
        info: "\x1b[92m".to_string(),       // 亮绿色
        debug: "\x1b[96m".to_string(),      // 亮青色
        trace: "\x1b[95m".to_string(),      // 亮紫色
        timestamp: "\x1b[90m".to_string(),   // 深灰色
        target: "\x1b[94m".to_string(),      // 亮蓝色
        file: "\x1b[95m".to_string(),       // 亮紫色
        message: "\x1b[97m".to_string(),      // 亮白色
    };

    // 创建文件处理器
    let file_config = FileConfig {
        log_dir: "./app_logs".into(),
        max_file_size: 10 * 1024 * 1024,  // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    // 构建日志器
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()
        .add_file(file_config)
        .build();
}
```

### 单独文件处理器

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig};

fn main() {
    let file_config = FileConfig {
        log_dir: "./app_logs".into(),
        max_file_size: 10 * 1024 * 1024, // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_file(file_config)
        .build();
}
```

### UDP 网络输出

```rust
use rat_logger::{LoggerBuilder, LevelFilter, NetworkConfig};

fn main() {
    let network_config = NetworkConfig {
        server_addr: "127.0.0.1".to_string(),
        server_port: 54321,
        auth_token: "your_token".to_string(),
        app_id: "my_app".to_string(),
    };

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_udp(network_config)
        .build();
}
```

### 多输出处理器

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig};

fn main() {
    let file_config = FileConfig {
        log_dir: "./app_logs".into(),
        max_file_size: 10 * 1024 * 1024, // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    // 创建多输出日志器（终端 + 文件）
    // LoggerBuilder内部会自动使用CompositeHandler
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()    // 添加终端输出
        .add_file(file_config)  // 添加文件输出
        .build();
}
```

## 架构设计

rat_logger 采用了先进的生产者-消费者架构：

- **生产者-消费者模式**: 主线程发送日志指令，工作线程异步处理文件操作
- **命令模式**: 使用 FileCommand 枚举分离业务逻辑，支持写入、刷新、轮转、压缩等操作
- **批量写入**: 8KB 阈值或 100ms 间隔的批量写入策略，大幅提升性能
- **跨平台优化**: Windows 平台使用 sync_data，其他平台使用 sync_all
- **无锁设计**: 使用 crossbeam-channel 进行线程间通信，避免锁竞争

## 日志级别

rat_logger 支持标准的日志级别（从低到高）：

- `Trace` - 最详细的日志信息
- `Debug` - 调试信息
- `Info` - 一般信息
- `Warn` - 警告信息
- `Error` - 错误信息

每个级别都有相应的日志宏：`trace!`、`debug!`、`info!`、`warn!`、`error!`

## 配置系统

### 格式配置 (FormatConfig)

```rust
pub struct FormatConfig {
    pub timestamp_format: String,    // 时间戳格式
    pub level_style: LevelStyle,     // 日志级别样式
    pub format_template: String,     // 格式模板
}

pub struct LevelStyle {
    pub error: String,  // 错误级别显示
    pub warn: String,   // 警告级别显示
    pub info: String,   // 信息级别显示
    pub debug: String,  // 调试级别显示
    pub trace: String,  // 跟踪级别显示
}
```

### 颜色配置 (ColorConfig)

```rust
pub struct ColorConfig {
    pub error: String,      // 错误级别颜色 (ANSI)
    pub warn: String,       // 警告级别颜色
    pub info: String,       // 信息级别颜色
    pub debug: String,      // 调试级别颜色
    pub trace: String,      // 跟踪级别颜色
    pub timestamp: String,  // 时间戳颜色
    pub target: String,     // 目标颜色
    pub file: String,       // 文件名颜色
    pub message: String,    // 消息颜色
}
```

### 文件配置 (FileConfig)

```rust
pub struct FileConfig {
    pub log_dir: PathBuf,              // 日志目录
    pub max_file_size: u64,             // 最大文件大小
    pub max_compressed_files: usize,    // 最大压缩文件数
    pub compression_level: u32,         // 压缩级别
    pub min_compress_threads: usize,    // 最小压缩线程数
    pub skip_server_logs: bool,        // 是否跳过服务器日志
    pub is_raw: bool,                  // 是否为原始日志
    pub compress_on_drop: bool,         // 退出时是否压缩
}
```

### 网络配置 (NetworkConfig)

```rust
pub struct NetworkConfig {
    pub server_addr: String,    // 服务器地址
    pub server_port: u16,       // 服务器端口
    pub auth_token: String,     // 认证令牌
    pub app_id: String,         // 应用标识
}
```

## 性能特性

- **生产者-消费者架构**: 分离日志生成和处理，避免阻塞主线程
- **批量写入**: 8KB 阈值或 100ms 间隔的智能批量写入
- **异步压缩**: 使用线程池进行异步文件压缩
- **跨平台优化**: 针对不同平台的同步策略优化
- **零拷贝**: 在关键路径上使用零拷贝技术
- **内存高效**: 智能缓冲区管理，避免内存浪费

### 性能基准测试结果

在标准测试环境下的性能表现：

- 终端输出: ~63,828 消息/秒
- 文件输出: ~408,025 消息/秒
- 终端+文件: ~501,567 消息/秒
- 多线程环境: ~239,808 消息/秒
- 不同日志级别: 833K-1.7M 消息/秒

## 线程安全

rat_logger 完全支持多线程环境：

- 使用 crossbeam-channel 进行无锁线程间通信
- 支持多线程并发写入，无数据竞争风险
- 原子操作用于统计信息收集
- 在高并发场景下保持稳定性能

## 压缩支持

内置日志文件压缩功能：

- 使用 LZ4 压缩算法，平衡压缩率和性能
- 可配置压缩级别 (1-9)
- 异步压缩线程池，不阻塞主线程
- 自动清理旧压缩文件

## 网络传输

支持通过 UDP 协议发送日志：

- 基于 bincode 的高效序列化
- 支持基于令牌的认证机制
- 兼容 zerg_creep 的 UDP 包格式
- 批量网络发送优化

## 错误处理

rat_logger 提供了完善的错误处理机制：

- 内部错误不会影响主程序运行
- 优雅的错误恢复机制
- 详细的错误日志记录
- 可配置的错误处理策略

## 依赖项

```toml
[dependencies]
rat_logger = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
chrono = "0.4"
lz4 = "1.24"
crossbeam-channel = "0.5"
threadpool = "1.8"
parking_lot = "0.12"
```

## 许可证

本项目采用 LGPLv3 许可证。详见 [LICENSE](LICENSE) 文件。

## 贡献

欢迎提交 Issue 和 Pull Request 来改进 rat_logger。

## 更新日志

### v0.1.0
- 初始版本发布
- 生产者-消费者架构实现
- 支持基本日志记录功能
- 文件和网络输出支持
- LZ4 压缩功能
- 线程安全保证
- 日志宏支持
- 分层配置系统
- 跨平台优化

## 示例代码

项目包含完整的示例代码：

- `examples/basic_usage.rs` - 基础使用示例
- `examples/composite_handler.rs` - 多输出处理器示例
- `examples/file_rotation.rs` - 文件轮转功能测试
- `examples/pm2_style_logging.rs` - PM2风格多文件日志管理
- `tests/performance_comparison.rs` - 性能对比测试