---
name: rat-logger
description: rat_logger 高性能 Rust 日志库的深度知识 - 异步广播架构、处理器系统、批量处理、格式化配置及扩展指南
---

使用此 skill 快速理解项目核心架构、处理器实现、批量处理机制、格式化系统以及日常开发操作。

## 项目概述

`rat_logger` (v0.3.3) 是一个高性能 Rust 日志库，采用异步广播（生产者-消费者）架构，支持终端、文件、UDP 三种输出处理器。

**核心设计理念**：
- **异步广播**：日志序列化后通过 crossbeam-channel 广播到所有处理器工作线程，每个处理器独立消费
- **批量处理**：工作线程空闲时永久阻塞（0% CPU），有数据时短超时轮询，按 BatchConfig 触发批量写入
- **分层配置**：FormatConfig（格式）与 ColorConfig（颜色）分离，默认无颜色主题
- **兼容标准 log 库**：提供 `error!`/`warn!`/`info!`/`debug!`/`trace!` 宏，API 风格一致

**关键依赖**：
- `crossbeam-channel` — 无锁线程间通信
- `bincode 2.x` — 日志记录序列化
- `tokio` — UDP 异步 IO
- `lz4` — 日志文件压缩
- `parking_lot` — 高性能互斥锁
- `chrono` — 时间戳格式化
- `once_cell` — 全局懒初始化

## 核心架构

### 数据流

```
日志宏 (error!/info!/...)
  ↓ __private_log_impl
检查 MAX_LEVEL（原子快速路径，被过滤的日志零开销）
  ↓ bincode 序列化 Record
ProcessorManager.broadcast_write / broadcast_write_force
  ↓ crossbeam-channel（每个处理器一个 channel）
ProcessorWorker 工作线程
  ↓ 批量缓冲 + Bincode 反序列化
LogProcessor::process / process_batch
  ↓
输出目标（终端 stdout / 文件 / UDP）
```

### 分层架构

```
用户代码
  ↓
日志宏层 (lib.rs)                 → error!/warn!/info!/debug!/trace!/emergency!/startup_log!/flush_logs!
  ↓
Logger trait (core.rs)            → log() / flush() / force_flush() / emergency_log()
  ↓
LoggerCore (core.rs)              → 级别检查、bincode 序列化、dev_mode 同步等待
  ↓
ProcessorManager (producer_consumer.rs) → 广播 LogCommand 给所有 ProcessorWorker
  ↓
ProcessorWorker (producer_consumer.rs)  → 独立线程、批量缓冲、LogCommand 分发
  ↓
LogProcessor 实现 (handler/)      → TermProcessor / FileProcessor / UdpProcessor
```

### 模块职责

| 模块 | 路径 | 职责 |
|------|------|------|
| **lib** | `src/lib.rs` | 日志宏定义、公共 API 重新导出、`__private_log_impl` |
| **core** | `src/core.rs` | Logger trait、LoggerCore、LoggerBuilder、全局 LOGGER/MAX_LEVEL、LogCommand 枚举、环境变量初始化 |
| **producer_consumer** | `src/producer_consumer.rs` | LogProcessor trait、ProcessorWorker、ProcessorManager、BatchConfig、工作线程就绪计数器 |
| **config** | `src/config/mod.rs` | Level/LevelFilter、Record/Metadata、FileConfig/NetworkConfig/FormatConfig/ColorConfig/LevelTemplates/LevelStyle、NetRecord |
| **handler/term** | `src/handler/term.rs` | TermProcessor、TermConfig、终端格式化（带/不带颜色）、级别模板继承 |
| **handler/file** | `src/handler/file.rs` | FileProcessor、FileProcessorConfig、LogWriter、LogRotator、LZ4 压缩、文件轮转 |
| **handler/udp** | `src/handler/udp.rs` | UdpProcessor、UdpConfig、UdpConnectionPool、UDP 批量发送 |
| **handler/composite** | `src/handler/composite.rs` | CompositeHandler（旧的 LogHandler trait，多处理器组合） |
| **fmt_impl** | `src/fmt_impl.rs` | FmtInitializer，类似 tracing_subscriber::fmt().init() 的快速初始化 API |
| **udp_helper** | `src/udp_helper.rs` | UdpPacketHelper（编解码 Record↔NetRecord）、PacketMetadata、UdpBatchProcessor |

### 两套处理器接口

**`LogProcessor` trait**（`producer_consumer.rs`）— 当前架构使用：
```rust
pub trait LogProcessor: Send + 'static {
    fn name(&self) -> &'static str;
    fn process(&mut self, data: &[u8]) -> Result<(), String>;
    fn process_batch(&mut self, batch: &[Vec<u8>]) -> Result<(), String>; // 默认逐个处理
    fn handle_rotate(&mut self) -> Result<(), String>;  // 默认空实现
    fn handle_compress(&mut self, _path: &Path) -> Result<(), String>;  // 默认空实现
    fn flush(&mut self) -> Result<(), String>;
    fn cleanup(&mut self) -> Result<(), String>;
}
```

**`LogHandler` trait**（`handler/mod.rs`）— 旧接口，仅 CompositeHandler 使用：
```rust
pub trait LogHandler: Send + Sync + Any {
    fn handle(&self, record: &Record);
    fn flush(&self);
    fn handler_type(&self) -> HandlerType;
    fn as_any(&self) -> &dyn Any;
}
```

新增处理器应实现 `LogProcessor` trait。

## LogCommand 枚举

所有处理器工作线程通过 `LogCommand` 接收指令：

```rust
pub enum LogCommand {
    Write(Vec<u8>),                    // 普通写入，走批量缓冲
    WriteForce(Vec<u8>),               // 强制写入，绕过批量直接处理
    Rotate,                            // 文件轮转（仅文件处理器响应）
    Compress(PathBuf),                 // 文件压缩（仅文件处理器响应）
    Flush,                             // 刷新缓冲
    Shutdown(&'static str),            // 关闭工作线程
    HealthCheck(Sender<bool>),         // 健康检查（初始化时验证）
}
```

## 批量处理机制

### 工作线程状态机

```
空闲状态 → batch_buffer 为空
  ↓ recv() 永久阻塞（0% CPU）
收到 LogCommand::Write(data)
  ↓ push 到 batch_buffer
  ↓ 检查：len >= batch_size || elapsed >= batch_interval_ms?
  ↓ 是 → process_batch + 清空缓冲
  ↓ 否 → 继续等待

有数据状态 → batch_buffer 非空
  ↓ recv_timeout(batch_interval_ms) 短超时轮询
收到 Write → push + 检查阈值
超时 → 检查是否需要刷新
收到 WriteForce → 先 flush 缓冲 + process 单条 + flush
收到 Flush → flush 缓冲
收到 Shutdown → flush 缓冲 + cleanup + exit(0)
```

### BatchConfig

```rust
pub struct BatchConfig {
    pub batch_size: usize,          // 批量大小阈值（条数），默认 2048
    pub batch_interval_ms: u64,     // 刷新间隔，默认 25ms
    pub buffer_size: usize,         // 缓冲区容量，默认 16KB
}
```

### 同步 vs 异步模式

- **同步模式**（默认）：`batch_size=1, batch_interval_ms=1, buffer_size=1024`，每条日志立即处理
- **异步模式**：需显式配置 `with_batch_config()`，适合高吞吐量场景

## 全局初始化

### 三种初始化方式

```rust
// 1. 最简单（推荐新用户）
rat_logger::fmt().init();

// 2. 环境变量自动初始化（无需代码，设置 RUST_LOG=info）
// 日志宏内部自动调用 try_init_from_env()

// 3. 完整配置
LoggerBuilder::new()
    .with_level(LevelFilter::Debug)
    .add_terminal_with_config(TermConfig { enable_color: true, format: Some(format_config), color: Some(color_config) })
    .add_file(file_config)
    .with_batch_config(batch_config)
    .init_global_logger()?;
```

### 全局状态

- `LOGGER: Lazy<Mutex<Option<Arc<dyn Logger>>>>` — 全局单例
- `MAX_LEVEL: AtomicUsize` — 日志宏快速路径级别检查（Relaxed ordering）
- `LOGGER_LOCK: RwLock<()>` — 初始化互斥（防并发初始化）
- `WORKER_READY_COUNT / EXPECTED_WORKER_COUNT` — 全局原子计数器，工作线程就绪协调

### 初始化流程

1. LoggerBuilder.build() 创建 LoggerCore
2. init_global_logger() 将 LoggerCore 存入 LOGGER
3. 调用 set_max_level() 设置 MAX_LEVEL
4. 等待所有工作线程就绪（最多 5 秒超时）
5. 工作线程启动时调用 increment_ready_count() 通知就绪

### 开发模式 vs 生产模式

- **开发模式**（`with_dev_mode(true)`）：允许重新初始化；每次 log() 后同步 flush + sleep 10ms
- **生产模式**：重复初始化仅警告，不覆盖已有 LoggerCore

## 日志宏

| 宏 | 行为 |
|----|------|
| `error!` / `warn!` / `info!` / `debug!` / `trace!` | 检查 MAX_LEVEL 快速路径 → try_init_from_env → 序列化 → 发送 |
| `emergency!` | 无视级别检查，直接 emergency_log（用于启动阶段） |
| `startup_log!` | Info 级别的 emergency_log（用于程序启动配置输出） |
| `flush_logs!` | force_flush 所有处理器 + sleep 50ms |

**关键**：Error 级别日志在 LoggerCore.log() 中自动使用 `broadcast_write_force` 路径。

## 配置系统

### FormatConfig（格式配置）

```rust
pub struct FormatConfig {
    pub timestamp_format: String,             // chrono 格式，默认 "%Y-%m-%d %H:%M:%S%.3f"
    pub level_style: LevelStyle,              // 各级别显示文本，默认 "ERROR"/"WARN"/"INFO"/"DEBUG"/"TRACE"
    pub format_template: String,              // 通用模板
    pub level_templates: Option<LevelTemplates>, // 各级别专用模板
}
```

**模板变量**：`{timestamp}` `{level}` `{target}` `{file}` `{line}` `{message}`

**级别模板继承**：设为 `"+"` 则继承通用模板，设为具体字符串则使用专用模板。

### ColorConfig（颜色配置）

ANSI 转义码，控制 `{timestamp}` `{level}` `{target}` `{file}` `{message}` 各部分的颜色。默认 `ColorConfig::default()` 提供基础配色。

### FileConfig（文件配置）

```rust
pub struct FileConfig {
    pub log_dir: PathBuf,              // 日志目录，默认 "./logs"
    pub max_file_size: u64,            // 最大文件大小，默认 10MB
    pub max_compressed_files: usize,   // 最大压缩文件数，默认 10
    pub compression_level: u8,         // LZ4 压缩级别 0-9，默认 4
    pub min_compress_threads: usize,   // 最小压缩线程数，默认 2
    pub skip_server_logs: bool,        // 跳过无 app_id 的日志（服务端日志过滤）
    pub is_raw: bool,                  // 原始模式：直接输出消息文本，不添加格式
    pub compress_on_drop: bool,        // 退出时压缩，默认 false
    pub force_sync: bool,              // 强制同步写入磁盘（性能代价大），默认 false
    pub format: Option<FormatConfig>,  // 自定义文件格式（默认使用内置格式）
}
```

**约束**：`is_raw` 与 `format` 互斥，配置验证会 panic。

### TermConfig（终端配置）

```rust
pub struct TermConfig {
    pub enable_color: bool,           // 启用颜色，默认 true
    pub format: Option<FormatConfig>, // 格式配置
    pub color: Option<ColorConfig>,   // 颜色配置
}
```

**约束**：`enable_color=false` 时提供 `color` 会验证失败。

### NetworkConfig（UDP 配置）

```rust
pub struct NetworkConfig {
    pub server_addr: String,   // 服务器地址，默认 "127.0.0.1"
    pub server_port: u16,      // 端口，默认 5140
    pub auth_token: String,    // 认证令牌
    pub app_id: String,        // 应用标识
}
```

## 处理器实现详解

### TermProcessor

- 实现 `LogProcessor`，`name()` 返回 `"term_processor"`
- 内部持有 `Arc<Mutex<BufWriter<Stdout>>>`
- 创建时根据 `(format_config, use_color)` 四种组合选择格式化闭包
- `process_batch` 批量反序列化 + 格式化 + 一次性写入

### FileProcessor

- 实现 `LogProcessor`，`name()` 返回 `"file_processor"`
- 内部持有 `Arc<Mutex<LogWriter>>` + `Arc<LogRotator>` + 格式化闭包
- `LogWriter`：BufWriter 包装，`write_direct()` 在 `force_sync` 模式下每次写入后 sync_all
- `LogRotator`：按时间戳命名文件 `app_YYYYMMDD_HHMMSS.log`，超 max_files 清理最旧文件
- 轮转：当前文件超 max_file_size → flush + 关闭 → 新建文件 → 异步 LZ4 压缩旧文件（使用全局 `COMPRESSION_POOL` 线程池）
- `skip_server_logs`：过滤 `app_id.is_none()` 的日志

### UdpProcessor

- 实现 `LogProcessor`，`name()` 返回 `"udp_processor"`
- 内部持有 `UdpConnectionPool`（DashMap 缓存连接）+ tokio Runtime
- 数据流：bincode Record → `UdpPacketHelper::encode_record` → bincode NetRecord → UDP 发送
- 批量发送：逐条反序列化 + 编码 + 合并数据 + 一次发送
- 支持重试（默认 3 次，间隔 100ms）

### CompositeHandler

- 实现**旧的** `LogHandler` trait（不是 `LogProcessor`）
- 组合多个 `Arc<dyn LogHandler>`，支持并行处理（rayon）
- 新架构中已被 ProcessorManager + LogProcessor 取代

## 序列化体系

### Record（内部日志记录）

```rust
pub struct Record {
    pub metadata: Arc<Metadata>,  // level, target, auth_token, app_id
    pub args: String,             // 格式化后的消息文本
    pub module_path: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}
```

- `Level` 自定义 bincode 编解码为字符串（"ERROR"/"WARN" 等）
- Record 和 Metadata 均手动实现 Encode/Decode

### NetRecord（网络传输）

```rust
pub struct NetRecord {
    pub level: Level, pub target: String, pub message: String,
    pub module_path: Option<String>, pub file: Option<String>, pub line: Option<u32>,
    pub timestamp: u64, pub auth_token: Option<String>, pub app_id: Option<String>,
}
```

- `Record` → `NetRecord`：`From` trait 自动转换，添加 UNIX 时间戳
- `NetRecord` → `Record`：`UdpPacketHelper::net_record_to_record()`

## 常用命令

```bash
# 构建（仅 debug 模式，禁止未经许可使用 release）
cargo build
cargo check

# 测试
cargo test                         # 所有测试
cargo test -- --nocapture          # 显示输出
cargo test test_processor_worker   # 匹配名称
cargo test --test level_logging_test  # 集成测试

# 代码质量
cargo fmt                         # 格式化
cargo clippy -- -D warnings       # lint（警告视为错误）

# 运行示例
cargo run --example basic_usage
cargo run --example fmt_quick_init
cargo run --example env_log_example
cargo run --example file_rotation
cargo run --example composite_handler
```

## 扩展新处理器

分步指南：添加新的日志输出处理器（例如 TCP）

### 步骤 1：在 `src/handler/` 创建处理器文件

```rust
// src/handler/tcp.rs
use crate::producer_consumer::LogProcessor;
use crate::config::Record;

pub struct TcpProcessor {
    // ...
}

impl LogProcessor for TcpProcessor {
    fn name(&self) -> &'static str {
        "tcp_processor"  // 必须唯一，用于工作线程就绪协调
    }

    fn process(&mut self, data: &[u8]) -> Result<(), String> {
        // 反序列化 Record
        let record = bincode::decode_from_slice::<Record, _>(data, bincode::config::standard())
            .map_err(|e| format!("反序列化失败: {}", e))?.0;
        // 处理逻辑...
        Ok(())
    }

    fn process_batch(&mut self, batch: &[Vec<u8>]) -> Result<(), String> {
        // 批量处理优化...
        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> { Ok(()) }
    fn cleanup(&mut self) -> Result<(), String> { Ok(()) }
}
```

### 步骤 2：在 `src/handler/mod.rs` 注册

```rust
pub mod tcp;
pub use tcp::TcpProcessor;
```

### 步骤 3：在 `src/core.rs` 添加构建器方法和处理器类型常量

```rust
// processor_types 模块
pub const TCP: &str = "tcp_processor";

// LoggerBuilder
pub fn add_tcp(mut self, config: TcpConfig) -> Self {
    use crate::handler::tcp::TcpProcessor;
    let processor = TcpProcessor::new(config);
    let batch_config = self.batch_config.clone().unwrap_or_else(|| BatchConfig {
        batch_size: 1, batch_interval_ms: 1, buffer_size: 1024,
    });
    self.processor_manager.add_processor(processor, batch_config).unwrap();
    self.expected_processor_types.insert(processor_types::TCP.to_string());
    self
}
```

### 步骤 4：在 `src/lib.rs` 重新导出

```rust
pub use handler::tcp::TcpProcessor;
```

## 关键设计决策

### 为什么 Error 级别自动走 WriteForce？

Error 日志代表严重问题，必须立即输出。在 LoggerCore.log() 中，Error 级别自动调用 `broadcast_write_force`，绕过批量缓冲直接处理。

### 为什么工作线程使用 exit(0) 而非 break？

ProcessorWorker 的 Shutdown 处理中调用 `std::process::exit(0)` 而非 break 退出循环。这是因为日志器是全局的，程序退出时需要确保所有日志被刷出。

### 为什么使用 crossbeam-channel 而非 mpsc？

crossbeam-channel 的 unbounded channel 性能优于 std::sync::mpsc，且支持 `recv_timeout` 用于批量超时控制。

### 为什么有两个 LogCommand::Write 变体？

- `Write`：普通路径，进入批量缓冲，按 batch_size/batch_interval 触发
- `WriteForce`：紧急路径，先 flush 缓冲再直接处理单条，用于 Error 级别和 emergency_log

## 关键限制（CRITICAL）

1. **全局单例**：LOGGER 只能初始化一次（生产模式），开发模式允许重新初始化
2. **工作线程名称唯一**：每个处理器的 `name()` 返回值必须唯一，用于就绪协调
3. **bincode 版本**：使用 bincode 2.x，Record/Metadata/Level 自定义编解码
4. **禁止 release 编译**：除非获得明确许可
5. **必须使用 rat_logger**：项目规范禁止其他日志库（log/tracing/env_logger 等）
