# rat_logger

rat_logger 是一个用 Rust 编写的高性能、线程安全的日志库，提供了灵活的日志记录功能，支持多种输出方式和高级特性。

## 特性

- **高性能**: 采用异步写入和多线程处理，最小化对应用程序性能的影响
- **线程安全**: 完全线程安全，支持多线程并发写入
- **多种输出方式**: 支持终端、文件、UDP 网络等多种输出方式
- **灵活配置**: 支持运行时配置和细粒度的日志级别控制
- **结构化日志**: 支持结构化的日志记录和元数据
- **压缩支持**: 内置日志文件压缩功能，节省存储空间
- **UDP 网络传输**: 支持通过 UDP 协议将日志发送到远程服务器
- **认证机制**: 支持基于令牌的认证机制

## 快速开始

### 基本使用

```rust
use rat_logger::{LoggerBuilder, LevelFilter, Level, config::Record};
use rat_logger::config::Metadata;
use std::sync::Arc;

fn main() {
    // 创建日志记录器
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal()  // 输出到终端
        .build();

    // 记录日志
    let record = Record {
        metadata: Arc::new(Metadata {
            level: Level::Info,
            target: "my_app".to_string(),
            auth_token: None,
            app_id: Some("main".to_string()),
        }),
        args: "应用程序启动".to_string(),
        module_path: Some("main".to_string()),
        file: Some("main.rs".to_string()),
        line: Some(10),
    };

    logger.log(&record);
}
```

### 文件输出

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig};

fn main() {
    let file_config = FileConfig {
        log_dir: "./logs".into(),
        max_file_size: 1024 * 1024 * 10, // 10MB
        max_compressed_files: 5,
        compression_level: 6,
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

## 架构设计

rat_logger 采用了模块化的设计架构：

- **Core**: 核心日志处理逻辑
- **Appender**: 输出器接口，支持多种输出方式
- **Config**: 配置管理
- **Network**: 网络传输支持
- **Compression**: 压缩功能
- **Formatter**: 日志格式化

## 日志级别

rat_logger 支持标准的日志级别：

- `Trace` - 最详细的日志信息
- `Debug` - 调试信息
- `Info` - 一般信息
- `Warn` - 警告信息
- `Error` - 错误信息

## 配置选项

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

- **异步写入**: 采用异步写入机制，避免阻塞主线程
- **批量处理**: 支持批量日志处理，提高吞吐量
- **内存管理**: 智能内存管理，避免内存泄漏
- **零拷贝**: 在关键路径上使用零拷贝技术

## 线程安全

rat_logger 完全支持多线程环境：

- 使用原子操作和锁确保线程安全
- 支持多线程并发写入
- 无数据竞争风险
- 在高并发场景下保持稳定

## 压缩支持

内置日志文件压缩功能：

- 支持 gzip 压缩格式
- 可配置压缩级别
- 自动压缩旧日志文件
- 支持多线程压缩

## 网络传输

支持通过 UDP 协议发送日志：

- 可靠的 UDP 传输
- 支持认证机制
- 自动重连功能
- 批量网络发送

## 错误处理

rat_logger 提供了完善的错误处理机制：

- 内部错误不会影响主程序
- 优雅的错误恢复
- 详细的错误日志
- 可配置的错误处理策略

## 依赖项

```toml
[dependencies]
rat_logger = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
chrono = "0.4"
flate2 = "1.0"
crossbeam = "0.8"
```

## 许可证

本项目采用 LGPLv3 许可证。详见 [LICENSE](LICENSE) 文件。

## 贡献

欢迎提交 Issue 和 Pull Request 来改进 rat_logger。

## 更新日志

### v0.1.0
- 初始版本发布
- 支持基本日志记录功能
- 文件和网络输出支持
- 压缩功能
- 线程安全保证