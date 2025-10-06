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

### 环境变量配置（最简单）

支持通过 `RUST_LOG` 环境变量自动配置日志级别，无需任何代码初始化：

```bash
# 设置日志级别
export RUST_LOG=info  # 可选值: error, warn, info, debug, trace

# 然后直接使用日志宏
cargo run your_app.rs
```

详细使用方法请参考 `examples/env_log_example.rs` 示例。

### 使用日志宏（推荐）

```rust
use rat_logger::{LoggerBuilder, LevelFilter, error, warn, info, debug, trace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化全局日志器
    LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .init_global_logger()?;

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
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .build();

    // 开发环境配置（立即输出）
    let dev_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)  // 启用开发模式，确保日志立即输出
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
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
        force_sync: false,      // 异步写入，性能更好
        format: None,
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

### 批量处理配置建议

rat_logger使用批量处理机制来提高性能，通过BatchConfig来控制批量处理的行为：

#### 同步模式（推荐用于大多数应用）

对于日志量不大且要求可靠输出的应用（如CLI工具、命令行应用）：

*⚠️ 性能数据仅供参考，实际性能因硬件和环境而异*

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FormatConfig};

fn main() {
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle::default(),
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // 同步模式：自动使用同步配置，确保日志立即输出
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig {
            enable_color: true,
            format: Some(format_config),
            color: None,
        })
        .init_global_logger()
        .unwrap();
}
```

**注意**：同步模式下，LoggerBuilder会自动使用同步的BatchConfig（batch_size=1, batch_interval_ms=1, buffer_size=1024），无需手动配置。

#### 异步模式（高吞吐量应用）

对于高并发、大日志量的生产环境应用：

*⚠️ 性能数据仅供参考，实际性能因硬件和环境而异*

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FormatConfig, BatchConfig};

fn main() {
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle::default(),
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // 异步模式：高性能批量处理
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .with_async_mode(true)  // 启用异步模式
        .with_batch_config(BatchConfig {
            batch_size: 2048,         // 2KB批量大小
            batch_interval_ms: 25,    // 25ms刷新间隔
            buffer_size: 16384,      // 16KB缓冲区
        })
        .add_terminal_with_config(rat_logger::handler::term::TermConfig {
            enable_color: true,
            format: Some(format_config),
            color: None,
        })
        .init_global_logger()
        .unwrap();
}
```

#### 极端性能配置

对于极端高吞吐量的场景（如日志聚合服务）：

*⚠️ 性能数据仅供参考，实际性能因硬件和环境而异*

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FormatConfig, BatchConfig};

fn main() {
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle::default(),
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // 极端性能配置
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .with_async_mode(true)  // 启用异步模式
        .with_batch_config(BatchConfig {
            batch_size: 4096,          // 4KB批量大小
            batch_interval_ms: 50,    // 50ms刷新间隔
            buffer_size: 32768,      // 32KB缓冲区
        })
        .add_terminal_with_config(rat_logger::handler::term::TermConfig {
            enable_color: true,
            format: Some(format_config),
            color: None,
        })
        .init_global_logger()
        .unwrap();
}
```

**配置建议总结：**
- **CLI工具/命令行应用**: 使用默认配置（同步模式）
- **Web服务/后台应用**: 使用异步批量配置 (2KB批量，25ms间隔)
- **高吞吐量服务**: 使用较大批量配置 (4KB批量，50ms间隔)
- **测试/开发环境**: 启用开发模式 (`with_dev_mode(true)`)

### 文件日志批量配置指导

rat_logger的文件处理器具有独立的批量配置机制，为了确保文件日志的可靠写入，需要根据应用场景选择合适的配置。

#### 可靠写入配置

对于要求日志立即持久化的应用（如CLI工具、关键业务系统）：

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig, BatchConfig};
use std::path::PathBuf;

fn main() {
    let file_config = FileConfig {
        log_dir: PathBuf::from("./logs"),
        max_file_size: 10 * 1024 * 1024, // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: false,     // 异步写入，性能更好
        format: None,
    };

    // 可靠写入配置
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .with_batch_config(BatchConfig {
            batch_size: 1,          // 1字节就触发写入
            batch_interval_ms: 1,  // 1ms就触发写入
            buffer_size: 1,        // 1字节缓冲区
        })
        .init_global_logger()
        .unwrap();
}
```

#### 平衡配置

对于一般Web应用，平衡性能和可靠性：

```rust
.with_batch_config(BatchConfig {
    batch_size: 512,        // 512字节批量大小
    batch_interval_ms: 10,  // 10ms刷新间隔
    buffer_size: 1024,      // 1KB缓冲区
})
```

#### 高性能配置

对于高吞吐量服务，优先性能：

```rust
.with_batch_config(BatchConfig {
    batch_size: 2048,       // 2KB批量大小
    batch_interval_ms: 25,   // 25ms刷新间隔
    buffer_size: 4096,      // 4KB缓冲区
})
```

#### 配置选择建议

- **关键业务应用**: 使用可靠写入配置，确保日志不丢失
- **一般Web应用**: 使用平衡配置，在性能和可靠性间取得平衡
- **高并发日志**: 使用高性能配置，但确保应用运行时间>2秒
- **快速启动应用**: 使用可靠写入配置，避免日志丢失

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
        force_sync: false,      // 异步写入，性能更好
        format: None,
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
        force_sync: false,      // 异步写入，性能更好
        format: None,
    };

    // 创建多输出日志器（终端 + 文件）
    // LoggerBuilder内部使用ProcessorManager协调多个处理器
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())  // 添加终端输出
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
    pub compression_level: u8,          // 压缩级别
    pub min_compress_threads: usize,    // 最小压缩线程数
    pub skip_server_logs: bool,        // 是否跳过服务器日志
    pub is_raw: bool,                  // 是否为原始日志
    pub compress_on_drop: bool,         // 退出时是否压缩
    pub force_sync: bool,               // 是否强制同步写入磁盘
    pub format: Option<FormatConfig>,  // 格式配置
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

### 终端配置 (TermConfig)

```rust
pub struct TermConfig {
    pub enable_color: bool,          // 是否启用颜色
    pub format: Option<FormatConfig>, // 格式配置
    pub color: Option<ColorConfig>,   // 颜色配置
}
```

## 格式和颜色使用示例

### 自定义终端格式

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FormatConfig, ColorConfig};
use rat_logger::handler::term::TermConfig;

fn main() {
    // 创建格式配置
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle {
            error: "ERROR".to_string(),
            warn: "WARN ".to_string(),
            info: "INFO ".to_string(),
            debug: "DEBUG".to_string(),
            trace: "TRACE".to_string(),
        },
        format_template: "[{level}] {timestamp} {target}:{line} - {message}".to_string(),
    };

    // 创建颜色配置
    let color_config = ColorConfig {
        error: "\x1b[91m".to_string(),      // 亮红色
        warn: "\x1b[93m".to_string(),       // 亮黄色
        info: "\x1b[92m".to_string(),       // 亮绿色
        debug: "\x1b[96m".to_string(),      // 亮青色
        trace: "\x1b[95m".to_string(),      // 亮紫色
        timestamp: "\x1b[90m".to_string(),  // 深灰色
        target: "\x1b[94m".to_string(),     // 亮蓝色
        file: "\x1b[95m".to_string(),      // 亮紫色
        message: "\x1b[97m".to_string(),     // 亮白色
    };

    // 创建带配置的终端处理器
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal_with_config(TermConfig {
            enable_color: true,
            format: Some(format_config),
            color: Some(color_config),
        })
        .build();
}
```

### 自定义文件格式

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig, FormatConfig};
use std::path::PathBuf;

fn main() {
    // 创建JSON格式配置
    let json_format = FormatConfig {
        timestamp_format: "%Y-%m-%dT%H:%M:%S%.3fZ".to_string(),
        level_style: rat_logger::LevelStyle {
            error: "error".to_string(),
            warn: "warn".to_string(),
            info: "info".to_string(),
            debug: "debug".to_string(),
            trace: "trace".to_string(),
        },
        format_template: r#"{{"timestamp":"{timestamp}","level":"{level}","target":"{target}","message":"{message}"}}"#.to_string(),
    };

    // 创建带格式配置的文件处理器
    let file_config = FileConfig {
        log_dir: PathBuf::from("./logs"),
        max_file_size: 10 * 1024 * 1024,  // 10MB
        max_compressed_files: 5,
        compression_level: 6,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: false,      // 异步写入，性能更好
        format: Some(json_format),  // 使用自定义格式
    };

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .build();
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

#### UDP网络传输性能 (test_client测试结果)
- 100条消息批处理: **806,452 消息/秒**
- 1000条消息批处理: **1,215,498 消息/秒**
- 5000条消息批处理: **1,087,627 消息/秒**

*注：UDP网络传输测试基于test_client工具和本机loopback接口（127.0.0.1），使用release模式编译，实际网络环境下性能可能因网络条件而异*

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
rat_logger = "0.2.9"
```

## 许可证

本项目采用 LGPLv3 许可证。详见 [LICENSE](LICENSE) 文件。

## 贡献

欢迎提交 Issue 和 Pull Request 来改进 rat_logger。

## 更新日志

详细的版本更新记录请查看 [CHANGELOG.md](CHANGELOG.md)。

## 示例代码

项目包含完整的示例代码：

- `examples/env_log_example.rs` - 环境变量RUST_LOG配置示例，最简单的使用方式
- `examples/basic_usage.rs` - 基础使用示例，展示多种输出方式
- `examples/composite_handler.rs` - 多输出处理器示例，终端+文件同时输出
- `examples/file_rotation.rs` - 文件轮转和压缩功能测试
- `examples/sync_async_demo.rs` - 同步与异步模式对比演示
- `examples/term_format_example.rs` - 终端格式配置和颜色设置示例
- `examples/file_format_example.rs` - 文件格式配置示例，包括JSON格式
- `examples/color_format_example.rs` - 颜色格式配置示例
- `examples/macro_format_example.rs` - 宏与格式配置结合使用示例
- `examples/macro_example.rs` - 日志宏使用示例，支持全局初始化
- `examples/pm2_style_logging.rs` - PM2风格多文件日志管理

所有示例都启用了开发模式以确保日志立即输出。在生产环境中使用时，请移除 `with_dev_mode(true)` 配置以获得最佳性能。