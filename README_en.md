# rat_logger

[![Crates.io](https://img.shields.io/crates/v/rat_logger.svg)](https://crates.io/crates/rat_logger)
[![Crates.io](https://img.shields.io/crates/d/rat_logger.svg)](https://crates.io/crates/rat_logger)
[![GitHub stars](https://img.shields.io/github/stars/0ldm0s/rat_logger.svg)](https://github.com/0ldm0s/rat_logger)
[![GitHub forks](https://img.shields.io/github/forks/0ldm0s/rat_logger.svg)](https://github.com/0ldm0s/rat_logger)
[![GitHub issues](https://img.shields.io/github/issues/0ldm0s/rat_logger.svg)](https://github.com/0ldm0s/rat_logger/issues)
[![License](https://img.shields.io/crates/l/rat_logger.svg)](https://crates.io/crates/rat_logger)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rust-lang.org)

[🇨🇳 中文](README.md) | [🇺🇸 English](README_en.md) | [🇯🇵 日本語](README_ja.md)

rat_logger is a high-performance, thread-safe logging library written in Rust, featuring a producer-consumer architecture and asynchronous writing mechanisms that deliver excellent performance and flexible configuration options.

## Features

- **Extreme Performance**: Producer-consumer architecture with实测 file writing performance up to 400K+ msg/sec on MacBook Air M1 (for reference only)
- **Thread Safety**: Fully thread-safe, supports multi-threaded concurrent writing using atomic operations to avoid lock contention
- **Multiple Output Methods**: Supports terminal, file, UDP network and other output methods
- **Layered Configuration**: Format configuration separated from color configuration, default no color theme
- **Logging Macros**: Compatible with standard log library macro interfaces, providing convenient logging methods
- **Structured Logging**: Supports structured logging and metadata
- **Compression Support**: Built-in LZ4 compression functionality, automatically compresses old log files
- **UDP Network Transmission**: Supports sending logs to remote servers via UDP protocol
- **Authentication Mechanism**: Supports token-based authentication

## Quick Start

### Using Logging Macros (Recommended)

```rust
use rat_logger::{init_with_level, LevelFilter, error, warn, info, debug, trace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize global logger
    init_with_level(LevelFilter::Debug)?;

    // Use logging macros to record logs
    error!("This is an error log");
    warn!("This is a warning log");
    info!("This is an info log");
    debug!("This is a debug log");
    trace!("This is a trace log");

    Ok(())
}
```

### Custom Handler Configuration

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig, FormatConfig, LevelStyle, ColorConfig, TermHandler};

fn main() {
    // Create format configuration
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

    // Create terminal handler (with colors)
    let color_config = ColorConfig {
        error: "\x1b[91m".to_string(),      // Bright red
        warn: "\x1b[93m".to_string(),       // Bright yellow
        info: "\x1b[92m".to_string(),       // Bright green
        debug: "\x1b[96m".to_string(),      // Bright cyan
        trace: "\x1b[95m".to_string(),      // Bright purple
        timestamp: "\x1b[90m".to_string(),   // Dark gray
        target: "\x1b[94m".to_string(),      // Bright blue
        file: "\x1b[95m".to_string(),       // Bright purple
        message: "\x1b[97m".to_string(),      // Bright white
    };

    // Create file handler
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

    // Build logger
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()
        .add_file(file_config)
        .build();
}
```

### Standalone File Handler

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

### UDP Network Output

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

### Multiple Output Handlers

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

    // Create multi-output logger (terminal + file)
    // LoggerBuilder automatically uses CompositeHandler internally
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()    // Add terminal output
        .add_file(file_config)  // Add file output
        .build();
}
```

## Architecture Design

rat_logger adopts an advanced producer-consumer architecture:

- **Producer-Consumer Pattern**: Main thread sends logging commands, worker thread asynchronously handles file operations
- **Command Pattern**: Uses FileCommand enum to separate business logic, supporting write, flush, rotate, compress operations
- **Batch Writing**: 8KB threshold or 100ms interval batch writing strategy, significantly improves performance
- **Cross-platform Optimization**: Windows platform uses sync_data, other platforms use sync_all
- **Lock-free Design**: Uses crossbeam-channel for inter-thread communication, avoiding lock contention

## Log Levels

rat_logger supports standard log levels (from low to high):

- `Trace` - Most detailed log information
- `Debug` - Debug information
- `Info` - General information
- `Warn` - Warning information
- `Error` - Error information

Each level has corresponding logging macros: `trace!`, `debug!`, `info!`, `warn!`, `error!`

## Configuration System

### Format Configuration (FormatConfig)

```rust
pub struct FormatConfig {
    pub timestamp_format: String,    // Timestamp format
    pub level_style: LevelStyle,     // Log level style
    pub format_template: String,     // Format template
}

pub struct LevelStyle {
    pub error: String,  // Error level display
    pub warn: String,   // Warning level display
    pub info: String,   // Info level display
    pub debug: String,  // Debug level display
    pub trace: String,  // Trace level display
}
```

### Color Configuration (ColorConfig)

```rust
pub struct ColorConfig {
    pub error: String,      // Error level color (ANSI)
    pub warn: String,       // Warning level color
    pub info: String,       // Info level color
    pub debug: String,      // Debug level color
    pub trace: String,      // Trace level color
    pub timestamp: String,  // Timestamp color
    pub target: String,     // Target color
    pub file: String,       // File name color
    pub message: String,    // Message color
}
```

### File Configuration (FileConfig)

```rust
pub struct FileConfig {
    pub log_dir: PathBuf,              // Log directory
    pub max_file_size: u64,             // Maximum file size
    pub max_compressed_files: usize,    // Maximum compressed files
    pub compression_level: u32,         // Compression level
    pub min_compress_threads: usize,    // Minimum compression threads
    pub skip_server_logs: bool,        // Whether to skip server logs
    pub is_raw: bool,                  // Whether it's raw log
    pub compress_on_drop: bool,         // Whether to compress on exit
}
```

### Network Configuration (NetworkConfig)

```rust
pub struct NetworkConfig {
    pub server_addr: String,    // Server address
    pub server_port: u16,       // Server port
    pub auth_token: String,     // Authentication token
    pub app_id: String,         // Application ID
}
```

## Performance Features

- **Producer-Consumer Architecture**: Separates log generation and processing, avoiding main thread blocking
- **Batch Writing**: Intelligent batch writing with 8KB threshold or 100ms interval
- **Asynchronous Compression**: Uses thread pool for asynchronous file compression
- **Cross-platform Optimization**: Synchronization strategy optimization for different platforms
- **Zero Copy**: Uses zero-copy technology on critical paths
- **Memory Efficient**: Smart buffer management, avoids memory waste

### Performance Benchmark Results

Performance on MacBook Air M1 (for reference only):

- Terminal output: ~63,828 messages/sec
- File output: ~408,025 messages/sec
- Terminal+File: ~501,567 messages/sec
- Multi-threaded environment: ~239,808 messages/sec
- Different log levels: 833K-1.7M messages/sec

## Thread Safety

rat_logger fully supports multi-threaded environments:

- Uses crossbeam-channel for lock-free inter-thread communication
- Supports multi-threaded concurrent writing without data race risks
- Atomic operations for statistics collection
- Maintains stable performance in high-concurrency scenarios

## Compression Support

Built-in log file compression functionality:

- Uses LZ4 compression algorithm, balancing compression ratio and performance
- Configurable compression levels (1-9)
- Asynchronous compression thread pool, doesn't block main thread
- Automatic cleanup of old compressed files

## Network Transmission

Supports sending logs via UDP protocol:

- Efficient serialization based on bincode
- Supports token-based authentication mechanism
- Compatible with zerg_creep UDP packet format
- Batch network sending optimization

## Error Handling

rat_logger provides comprehensive error handling mechanisms:

- Internal errors don't affect main program execution
- Graceful error recovery mechanisms
- Detailed error logging
- Configurable error handling strategies

## Dependencies

```toml
[dependencies]
rat_logger = "0.2.0"
```

## License

This project is licensed under LGPLv3. See [LICENSE](LICENSE) file for details.

## Contributing

Welcome to submit Issues and Pull Requests to improve rat_logger.

## Changelog

### v0.2.0
- Upgraded to Rust 2024 Edition
- Updated dependencies to latest versions
- Performance optimizations and stability improvements
- Published to crates.io
- Improved documentation and examples

### v0.1.0
- Initial version release
- Producer-consumer architecture implementation
- Basic logging functionality support
- File and network output support
- LZ4 compression functionality
- Thread safety guarantee
- Logging macro support
- Layered configuration system
- Cross-platform optimization

## Example Code

The project includes complete example code:

- `examples/basic_usage.rs` - Basic usage example
- `examples/composite_handler.rs` - Multi-output handler example
- `examples/file_rotation.rs` - File rotation functionality test
- `examples/pm2_style_logging.rs` - PM2-style multi-file log management
- `tests/performance_comparison.rs` - Performance comparison tests