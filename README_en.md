# rat_logger

[![Crates.io](https://img.shields.io/crates/v/rat_logger.svg)](https://crates.io/crates/rat_logger)
[![Crates.io](https://img.shields.io/crates/d/rat_logger.svg)](https://crates.io/crates/rat_logger)
[![GitHub stars](https://img.shields.io/github/stars/0ldm0s/rat_logger.svg)](https://github.com/0ldm0s/rat_logger)
[![GitHub forks](https://img.shields.io/github/forks/0ldm0s/rat_logger.svg)](https://github.com/0ldm0s/rat_logger)
[![GitHub issues](https://img.shields.io/github/issues/0ldm0s/rat_logger.svg)](https://github.com/0ldm0s/rat_logger/issues)
[![License](https://img.shields.io/crates/l/rat_logger.svg)](https://crates.io/crates/rat_logger)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://rust-lang.org)

[🇨🇳 中文](README.md) | [🇺🇸 English](README_en.md) | [🇯🇵 日本語](README_ja.md)

rat_logger is a high-performance, thread-safe logging library written in Rust, featuring an asynchronous broadcast architecture and batch processing mechanisms that deliver excellent performance and flexible configuration options.

## Features

- **Extreme Performance**: Asynchronous broadcast architecture with terminal output performance up to 400K+ msg/sec on MacBook Air M1 (for reference only)
- **Thread Safety**: Fully thread-safe, supports multi-threaded concurrent writing using atomic operations to avoid lock contention
- **Multiple Output Methods**: Supports terminal, file, UDP network and other output methods
- **Layered Configuration**: Format configuration separated from color configuration, default no color theme
- **Logging Macros**: Compatible with standard log library macro interfaces, providing convenient logging methods
- **Development Mode**: Optional development mode ensures immediate log output for debugging and learning
- **Flexible Configuration**: Unified LoggerBuilder interface supporting chain configuration
- **Structured Logging**: Supports structured logging and metadata
- **Compression Support**: Built-in LZ4 compression functionality, automatically compresses old log files
- **UDP Network Transmission**: Supports sending logs to remote servers via UDP protocol
- **Authentication Mechanism**: Supports token-based authentication

## Quick Start

### Using Logging Macros (Recommended)

```rust
use rat_logger::{LoggerBuilder, LevelFilter, error, warn, info, debug, trace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize global logger
    LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal()
        .init()?;

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
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig, FormatConfig, LevelStyle, ColorConfig};

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
        format: None,
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
        format: None,
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

## Batch Processing Configuration Guide

rat_logger uses batch processing mechanisms to improve performance, but different application scenarios require different configurations:

### Synchronous Mode (Recommended for Most Applications)

For applications with low log volume that require reliable output (such as CLI tools, command-line applications):

*⚠️ Performance data is for reference only, actual performance varies by hardware and environment*

```rust
use rat_logger::{LoggerBuilder, LevelFilter, TermConfig, FormatConfig};

fn main() {
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle::default(),
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // Synchronous mode: ensure immediate log output
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(TermConfig {
            enable_color: true,
            enable_async: false,  // Synchronous mode
            batch_size: 1,        // Meaningless in synchronous mode
            flush_interval_ms: 1, // Meaningless in synchronous mode
            format: Some(format_config),
            color: None,
        })
        .init_global_logger()
        .unwrap();
}
```

### Asynchronous Mode (High Throughput Applications)

For high-concurrency, high-log-volume production environment applications:

*⚠️ Performance data is for reference only, actual performance varies by hardware and environment*

```rust
use rat_logger::{LoggerBuilder, LevelFilter, TermConfig, FormatConfig};

fn main() {
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle::default(),
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // Asynchronous mode: high-performance batch processing
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(TermConfig {
            enable_color: true,
            enable_async: true,       // Asynchronous mode
            batch_size: 2048,         // 2KB batch size
            flush_interval_ms: 25,    // 25ms flush interval
            format: Some(format_config),
            color: None,
        })
        .init_global_logger()
        .unwrap();
}
```

### Extreme Performance Configuration

For extreme high-throughput scenarios (such as log aggregation services):

*⚠️ Performance data is for reference only, actual performance varies by hardware and environment*

```rust
use rat_logger::{LoggerBuilder, LevelFilter, TermConfig, FormatConfig};

fn main() {
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle::default(),
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // Extreme performance configuration
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(TermConfig {
            enable_color: true,
            enable_async: true,        // Asynchronous mode
            batch_size: 4096,          // 4KB batch size
            flush_interval_ms: 50,    // 50ms flush interval
            format: Some(format_config),
            color: None,
        })
        .init_global_logger()
        .unwrap();
}
```

**Configuration Summary:**
- **CLI Tools/Command-line Applications**: Use synchronous mode (`enable_async: false`)
- **Web Services/Background Applications**: Use default asynchronous configuration (2KB batch, 25ms interval)
- **High Throughput Services**: Use larger batch configuration (4KB batch, 50ms interval)
- **Test/Development Environment**: Enable development mode (`with_dev_mode(true)`)

### File Log Batch Configuration Guide

rat_logger's file processor has an independent batch configuration mechanism. To ensure reliable file log writing, you need to choose appropriate configuration based on application scenarios.

#### Reliable Write Configuration

For applications that require immediate log persistence (such as CLI tools, critical business systems):

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
        format: None,
    };

    // Reliable write configuration
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .with_batch_config(BatchConfig {
            batch_size: 1,          // Trigger write with 1 byte
            batch_interval_ms: 1,  // Trigger write with 1ms
            buffer_size: 1,        // 1 byte buffer
        })
        .init_global_logger()
        .unwrap();
}
```

#### Balanced Configuration

For general web applications, balance performance and reliability:

```rust
.with_batch_config(BatchConfig {
    batch_size: 512,        // 512 byte batch size
    batch_interval_ms: 10,  // 10ms flush interval
    buffer_size: 1024,      // 1KB buffer
})
```

#### High Performance Configuration

For high throughput services, prioritize performance:

```rust
.with_batch_config(BatchConfig {
    batch_size: 2048,       // 2KB batch size
    batch_interval_ms: 25,   // 25ms flush interval
    buffer_size: 4096,      // 4KB buffer
})
```

#### Configuration Selection Advice

- **Critical Business Applications**: Use reliable write configuration to ensure no log loss
- **General Web Applications**: Use balanced configuration to balance performance and reliability
- **High Concurrency Logging**: Use high performance configuration, but ensure application runtime > 2s
- **Quick Start Applications**: Use reliable write configuration to avoid log loss

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
    pub compression_level: u8,          // Compression level (1-9)
    pub min_compress_threads: usize,    // Minimum compression threads
    pub skip_server_logs: bool,        // Whether to skip server logs
    pub is_raw: bool,                  // Whether it's raw log
    pub compress_on_drop: bool,         // Whether to compress on exit
    pub format: Option<FormatConfig>,  // Format configuration
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

Performance on MacBook Air M1 local environment (for reference only):

#### New Version v0.2.3 Performance (Asynchronous Broadcast Architecture)
- Terminal output: **2,264,813 messages/sec** - 5.6x improvement
- File output: **2,417,040 messages/sec** - 5.9x improvement
- Terminal+File: **1,983,192 messages/sec** - 3.9x improvement
- Multi-threaded environment: **3,538,831 messages/sec** - 14.7x improvement ⭐
- Different log levels: **4.3M-4.7M messages/sec** - 2.5-5.6x improvement

#### Historical Version Performance (Comparison Reference)
- Terminal output: ~400,000+ messages/sec (optimized)
- File output: ~408,025 messages/sec
- Terminal+File: ~501,567 messages/sec
- Multi-threaded environment: ~239,808 messages/sec
- Different log levels: 833K-1.7M messages/sec

#### UDP Network Transmission Performance (test_client results)
- 100 messages batch: **806,452 messages/sec**
- 1000 messages batch: **1,215,498 messages/sec**
- 5000 messages batch: **1,087,627 messages/sec**

*Note: UDP network transmission tests are based on test_client tool and local loopback interface (127.0.0.1), using release mode compilation, actual network performance may vary depending on network conditions*

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

### v0.2.3
- **Architecture Refactor**: Complete rewrite to asynchronous broadcast architecture, removing old synchronous architecture
- **Development Mode**: Added development mode functionality for debugging and learning
- **Performance Optimization**: Terminal processor performance improved by 6x, significantly enhancing overall performance
- **LoggerBuilder Improvements**: Unified builder interface supporting more flexible configuration
- **Examples Update**: All examples now include development mode and production environment usage warnings
- **Documentation Enhancement**: Updated README and usage guides with multi-language support

### v0.2.2
- Fixed compilation errors and dependency issues
- Improved error handling mechanism
- Optimized memory usage

### v0.2.1
- Fixed compilation errors and dependency issues
- Improved error handling mechanism
- Optimized memory usage

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

- `examples/basic_usage.rs` - Basic usage example, demonstrates multiple output methods
- `examples/composite_handler.rs` - Multi-output handler example, terminal + file simultaneous output
- `examples/file_rotation.rs` - File rotation and compression functionality test
- `examples/term_format_example.rs` - Terminal format configuration and color settings example
- `examples/file_format_example.rs` - File format configuration example, including JSON format
- `examples/macro_format_example.rs` - Macro and format configuration combined usage example
- `examples/macro_example.rs` - Logging macro usage example, supports global initialization
- `examples/pm2_style_logging.rs` - PM2-style multi-file log management