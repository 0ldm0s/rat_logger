# rat_logger 示例

本目录包含了 rat_logger 日志库的各种使用示例。

## 示例列表

### 基础示例

- `basic_usage.rs` - 基础日志记录功能
- `multi_handler.rs` - 多处理器同时输出
- `composite_handler.rs` - 组合处理器使用
- `custom_format.rs` - 自定义日志格式
- `file_rotation.rs` - 文件轮转功能
- `filtered_logging.rs` - 日志过滤功能

## 运行示例

```bash
# 运行基础示例
cargo run --example basic_usage

# 运行多处理器示例
cargo run --example multi_handler

# 运行组合处理器示例
cargo run --example composite_handler

# 运行自定义格式示例
cargo run --example custom_format

# 运行文件轮转示例
cargo run --example file_rotation

# 运行过滤日志示例
cargo run --example filtered_logging
```

## 功能特性

### 多处理器支持
- 同时输出到终端、文件、网络
- 支持自定义处理器组合
- 并行或顺序处理模式

### 高级功能
- 文件轮转和压缩
- 自定义日志格式
- 级别和目标过滤
- UDP网络传输

### 性能优化
- 全局共享线程池
- 异步文件操作
- 连接池管理
- 批处理支持