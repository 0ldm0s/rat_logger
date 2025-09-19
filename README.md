# rat_logger

[![Crates.io](https://img.shields.io/crates/v/rat_logger.svg)](https://crates.io/crates/rat_logger)
[![Crates.io](https://img.shields.io/crates/d/rat_logger.svg)](https://crates.io/crates/rat_logger)
[![GitHub stars](https://img.shields.io/github/stars/0ldm0s/rat_logger.svg)](https://github.com/0ldm0s/rat_logger)
[![GitHub forks](https://img.shields.io/github/forks/0ldm0s/rat_logger.svg)](https://github.com/0ldm0s/rat_logger)
[![GitHub issues](https://img.shields.io/github/issues/0ldm0s/rat_logger.svg)](https://github.com/0ldm0s/rat_logger/issues)
[![License](https://img.shields.io/crates/l/rat_logger.svg)](https://crates.io/crates/rat_logger)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rust-lang.org)

[🇨🇳 中文](README.md) | [🇺🇸 English](README_en.md) | [🇯🇵 日本語](README_ja.md)

rat_logger 是一个用 Rust 编写的高性能、线程安全的日志库，采用异步广播架构和批量处理机制，提供卓越的性能表现和灵活的配置选项。

## 特性

- **极致性能**: 采用异步广播架构，在 MacBook Air M1 环境下实测终端输出性能可达 40万+ msg/sec（仅供参考）
- **线程安全**: 完全线程安全，支持多线程并发写入，采用原子操作避免锁竞争
- **多种输出方式**: 支持终端、文件、UDP 网络等多种输出方式
- **分层配置**: 格式配置与颜色配置分离，默认无颜色主题
- **日志宏**: 兼容标准 log 库的宏接口，提供便捷的日志记录方式
- **开发模式**: 可选的开发模式确保日志立即输出，便于调试和学习
- **灵活配置**: 统一的 LoggerBuilder 接口，支持链式配置
- **结构化日志**: 支持结构化的日志记录和元数据
- **压缩支持**: 内置 LZ4 压缩功能，自动压缩旧日志文件
- **UDP 网络传输**: 支持通过 UDP 协议将日志发送到远程服务器
- **认证机制**: 支持基于令牌的认证机制

## 快速开始

### 使用日志宏（推荐）

```rust
use rat_logger::{LoggerBuilder, LevelFilter, error, warn, info, debug, trace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化全局日志器
    LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()
        .init()?;

    // 使用日志宏记录日志
    error!("这是一个错误日志");
    warn!("这是一个警告日志");
    info!("这是一个信息日志");
    debug!("这是一个调试日志");
    trace!("这是一个跟踪日志");

    Ok(())
}
```

### 生产环境与开发环境配置

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig};
use std::path::PathBuf;

fn main() {
    // 生产环境配置（推荐）
    let prod_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal()
        .build();

    // 开发环境配置（立即输出）
    let dev_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)  // 启用开发模式，确保日志立即输出
        .add_terminal()
        .build();

    // 生产环境文件日志器
    let file_config = FileConfig {
        log_dir: PathBuf::from("./app_logs"),
        max_file_size: 10 * 1024 * 1024,  // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    let prod_file_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .build();
}
```

**⚠️ 重要提醒：**
- 在生产环境中，请不要启用开发模式以获得最佳性能
- 开发模式会强制等待异步操作完成，虽然便于调试但会降低性能
- 开发模式主要用于测试、示例和学习场景

### 文件处理器配置

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig};
use std::path::PathBuf;

fn main() {
    let file_config = FileConfig {
        log_dir: PathBuf::from("./app_logs"),
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
        .with_dev_mode(true)  // 开发模式下启用，确保日志立即发送
        .add_udp(network_config)
        .build();
}
```

### 多输出处理器

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig};
use std::path::PathBuf;

fn main() {
    let file_config = FileConfig {
        log_dir: PathBuf::from("./app_logs"),
        max_file_size: 10 * 1024 * 1024, // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    // 创建多输出日志器（终端 + 文件）
    // LoggerBuilder内部使用ProcessorManager协调多个处理器
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()    // 添加终端输出
        .add_file(file_config)  // 添加文件输出
        .build();
}
```

## 架构设计

rat_logger 采用了先进的异步广播架构：

### 核心架构组件

- **生产者-消费者广播模式**: 主线程序列化日志记录并广播到所有处理器工作线程
- **LogProcessor trait**: 统一的处理器接口，所有处理器（终端、文件、UDP）都实现此接口
- **ProcessorManager**: 协调管理多个处理器的核心组件
- **LogCommand 枚举**: 统一的命令格式，支持写入、轮转、压缩、刷新、关闭等操作
- **批量处理**: 智能批量处理策略，大幅提升性能
- **开发模式**: 可选的同步模式，确保日志立即输出，便于调试和学习

### 工作流程

1. **日志记录**: 主线程调用 `log()` 方法
2. **序列化**: 使用 bincode 2.x 将日志记录序列化为字节
3. **广播**: 将序列化后的数据广播到所有已注册的处理器工作线程
4. **异步处理**: 每个工作线程异步处理接收到的数据
5. **批量优化**: 处理器根据配置进行批量处理以优化性能
6. **输出**: 最终输出到相应目标（终端、文件、网络等）

### 技术特点

- **完全异步**: 所有IO操作都是异步的，不阻塞主线程
- **线程安全**: 使用 crossbeam-channel 进行无锁线程间通信
- **零拷贝**: 在关键路径上使用零拷贝技术
- **内存高效**: 智能缓冲区管理，避免内存浪费
- **跨平台优化**: 针对不同平台的同步策略优化

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

在 MacBook Air M1 本机环境下的性能表现（仅供参考）：

#### 新版本 v0.2.3 性能（异步广播架构）
- 终端输出: **2,264,813 消息/秒** - 提升5.6倍
- 文件输出: **2,417,040 消息/秒** - 提升5.9倍
- 终端+文件: **1,983,192 消息/秒** - 提升3.9倍
- 多线程环境: **3,538,831 消息/秒** - 提升14.7倍 ⭐
- 不同日志级别: **4.3M-4.7M 消息/秒** - 提升2.5-5.6倍

#### 历史版本性能（对比参考）
- 终端输出: ~400,000+ 消息/秒（优化后）
- 文件输出: ~408,025 消息/秒
- 终端+文件: ~501,567 消息/秒
- 多线程环境: ~239,808 消息/秒
- 不同日志级别: 833K-1.7M 消息/秒

#### UDP网络传输性能
- 100条消息批处理: ~725,516 消息/秒
- 1000条消息批处理: ~860,739 消息/秒
- 5000条消息批处理: ~921,326 消息/秒

*注：UDP网络传输测试基于本机loopback接口（127.0.0.1），实际网络环境下性能可能因网络条件而异*

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
rat_logger = "0.2.0"
```

## 许可证

本项目采用 LGPLv3 许可证。详见 [LICENSE](LICENSE) 文件。

## 贡献

欢迎提交 Issue 和 Pull Request 来改进 rat_logger。

## 更新日志

### v0.2.3
- **架构重构**: 完全重写为异步广播架构，移除旧的同步架构
- **开发模式**: 新增开发模式功能，便于调试和学习
- **性能优化**: 终端处理器性能提升6倍，大幅改善整体性能
- **LoggerBuilder改进**: 统一的构建器接口，支持更灵活的配置
- **示例更新**: 所有示例都添加开发模式和生产环境使用警告
- **文档完善**: 更新README和使用指南，添加多语言支持

### v0.2.2
- 修复编译错误和依赖问题
- 改进错误处理机制
- 优化内存使用

### v0.2.1
- 修复编译错误和依赖问题
- 改进错误处理机制
- 优化内存使用

### v0.2.0
- 升级到 Rust 2024 Edition
- 更新依赖项到最新版本
- 性能优化和稳定性改进
- 发布到 crates.io
- 改进文档和示例

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

- `examples/basic_usage.rs` - 基础使用示例，展示多种输出方式
- `examples/composite_handler.rs` - 多输出处理器示例，终端+文件同时输出
- `examples/file_rotation.rs` - 文件轮转和压缩功能测试
- `examples/format_example.rs` - 格式配置和颜色设置示例
- `examples/macro_example.rs` - 日志宏使用示例，支持全局初始化
- `examples/pm2_style_logging.rs` - PM2风格多文件日志管理
- `tests/level_logging_test.rs` - 日志级别过滤测试

所有示例都启用了开发模式以确保日志立即输出。在生产环境中使用时，请移除 `with_dev_mode(true)` 配置以获得最佳性能。