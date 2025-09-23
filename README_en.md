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
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .init_global_logger()?;

    // Use logging macros to record logs
    error!("This is an error log");
    warn!("This is a warning log");
    info!("This is an info log");
    debug!("This is a debug log");
    trace!("This is a trace log");

    Ok(())
}
```

### Production and Development Environment Configuration

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig};
use std::path::PathBuf;

fn main() {
    // Production environment configuration (recommended)
    let prod_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .build();

    // Development environment configuration (immediate output)
    let dev_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)  // Enable development mode to ensure immediate log output
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .build();

    // Production environment file logger
    let file_config = FileConfig {
        log_dir: PathBuf::from("./app_logs"),
        max_file_size: 10 * 1024 * 1024,  // 10MB
        max_compressed_files: 5,
        compression_level: 4,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: false,      // Asynchronous write for better performance
        format: None,
    };

    let prod_file_logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .build();
}
```

**⚠️ Important Reminders:**
- In production environments, please do not enable development mode for best performance
- Development mode forces waiting for asynchronous operations to complete, which is convenient for debugging but reduces performance
- Development mode is mainly used for testing, examples and learning scenarios

### Batch Processing Configuration Recommendations

rat_logger uses batch processing mechanisms to improve performance, but different application scenarios require different configurations:

#### Synchronous Mode (Recommended for Most Applications)

For applications with low log volume and reliable output requirements (such as CLI tools, command-line applications):

*⚠️ Performance data is for reference only, actual performance varies by hardware and environment*

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FormatConfig};

fn main() {
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle::default(),
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // Synchronous mode: automatically uses synchronous configuration to ensure immediate log output
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

**Note**: In synchronous mode, LoggerBuilder automatically uses synchronous BatchConfig (batch_size=1, batch_interval_ms=1, buffer_size=1024), no manual configuration required.

#### Asynchronous Batch Processing Mode (High Throughput Applications)

For high-concurrency, high-log-volume production environment applications:

*⚠️ Performance data is for reference only, actual performance varies by hardware and environment*

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FormatConfig, BatchConfig};

fn main() {
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle::default(),
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // Asynchronous batch processing mode: high-performance batch processing
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .with_async_mode(true)  // Enable asynchronous mode
        .with_batch_config(BatchConfig {
            batch_size: 2048,         // 2KB batch size
            batch_interval_ms: 25,    // 25ms flush interval
            buffer_size: 16384,      // 16KB buffer
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

**Note**: Asynchronous batch processing mode must enable both `with_async_mode(true)` and set appropriate BatchConfig.

#### Extreme Performance Configuration

For extreme high-throughput scenarios (such as log aggregation services):

*⚠️ Performance data is for reference only, actual performance varies by hardware and environment*

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FormatConfig, BatchConfig};

fn main() {
    let format_config = FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle::default(),
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // Extreme performance configuration
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .with_async_mode(true)  // Enable asynchronous mode
        .with_batch_config(BatchConfig {
            batch_size: 4096,          // 4KB batch size
            batch_interval_ms: 50,    // 50ms flush interval
            buffer_size: 32768,      // 32KB buffer
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

**Configuration Summary:**
- **CLI Tools/Command-line Applications**: Use default configuration (synchronous mode)
- **Web Services/Background Applications**: Use asynchronous batch configuration (2KB batch, 25ms interval)
- **High Throughput Services**: Use larger batch configuration (4KB batch, 50ms interval)
- **Testing/Development Environments**: Enable development mode (`with_dev_mode(true)`)

### File Log Batch Configuration Guide

rat_logger's file processor has an independent batch configuration mechanism. To ensure reliable file log writing, you need to choose the appropriate configuration based on the application scenario.

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
        force_sync: false,     // Asynchronous write for better performance
        format: None,
    };

    // Reliable write configuration
    LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .with_batch_config(BatchConfig {
            batch_size: 1,          // Trigger write on 1 byte
            batch_interval_ms: 1,  // Trigger write on 1ms
            buffer_size: 1,        // 1 byte buffer
        })
        .init_global_logger()
        .unwrap();
}
```

#### Balanced Configuration

For general web applications, balancing performance and reliability:

```rust
.with_batch_config(BatchConfig {
    batch_size: 512,        // 512 bytes batch size
    batch_interval_ms: 10,  // 10ms flush interval
    buffer_size: 1024,      // 1KB buffer
})
```

#### High Performance Configuration

For high throughput services, prioritizing performance:

```rust
.with_batch_config(BatchConfig {
    batch_size: 2048,       // 2KB batch size
    batch_interval_ms: 25,   // 25ms flush interval
    buffer_size: 4096,      // 4KB buffer
})
```

#### Configuration Selection Recommendations

- **Critical Business Applications**: Use reliable write configuration to ensure no log loss
- **General Web Applications**: Use balanced configuration to balance performance and reliability
- **High Concurrency Logging**: Use high performance configuration, but ensure application runtime > 2 seconds
- **Quick Start Applications**: Use reliable write configuration to avoid log loss

### File Processor Configuration

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
        force_sync: false,      // Asynchronous write for better performance
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
        .with_dev_mode(true)  // Enable in development mode to ensure immediate log sending
        .add_udp(network_config)
        .build();
}
```

### Multiple Output Processors

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
        force_sync: false,      // Asynchronous write for better performance
        format: None,
    };

    // Create multiple output logger (terminal + file)
    // LoggerBuilder uses ProcessorManager internally to coordinate multiple processors
    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())  // Add terminal output
        .add_file(file_config)  // Add file output
        .build();
}
```

## Architecture Design

rat_logger adopts an advanced asynchronous broadcast architecture:

### Core Architecture Components

- **Producer-Consumer Broadcast Mode**: Main thread serializes log records and broadcasts to all processor worker threads
- **LogProcessor Trait**: Unified processor interface, all processors (terminal, file, UDP) implement this interface
- **ProcessorManager**: Core component that coordinates and manages multiple processors
- **LogCommand Enum**: Unified command format, supporting write, rotate, compress, flush, shutdown and other operations
- **Batch Processing**: Intelligent batch processing strategy to significantly improve performance
- **Development Mode**: Optional synchronous mode to ensure immediate log output for debugging and learning

### Workflow

1. **Log Recording**: Main thread calls `log()` method
2. **Serialization**: Use bincode 2.x to serialize log records into bytes
3. **Broadcast**: Broadcast serialized data to all registered processor worker threads
4. **Asynchronous Processing**: Each worker thread asynchronously processes received data
5. **Batch Optimization**: Processors perform batch processing based on configuration to optimize performance
6. **Output**: Final output to corresponding targets (terminal, file, network, etc.)

### Technical Features

- **Fully Asynchronous**: All IO operations are asynchronous, not blocking the main thread
- **Thread Safe**: Use crossbeam-channel for lock-free inter-thread communication
- **Zero Copy**: Use zero copy technology on critical paths
- **Memory Efficient**: Intelligent buffer management to avoid memory waste
- **Cross-platform Optimization**: Synchronization strategy optimization for different platforms

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
    pub file: String,       // Filename color
    pub message: String,    // Message color
}
```

### File Configuration (FileConfig)

```rust
pub struct FileConfig {
    pub log_dir: PathBuf,              // Log directory
    pub max_file_size: u64,             // Maximum file size
    pub max_compressed_files: usize,    // Maximum compressed file count
    pub compression_level: u8,          // Compression level
    pub min_compress_threads: usize,    // Minimum compression thread count
    pub skip_server_logs: bool,        // Whether to skip server logs
    pub is_raw: bool,                  // Whether it is raw log
    pub compress_on_drop: bool,         // Whether to compress on exit
    pub force_sync: bool,               // Whether to force synchronous write to disk
    pub format: Option<FormatConfig>,  // Format configuration
}
```

### Network Configuration (NetworkConfig)

```rust
pub struct NetworkConfig {
    pub server_addr: String,    // Server address
    pub server_port: u16,       // Server port
    pub auth_token: String,     // Authentication token
    pub app_id: String,         // Application identifier
}
```

### Terminal Configuration (TermConfig)

```rust
pub struct TermConfig {
    pub enable_color: bool,          // Whether to enable color
    pub format: Option<FormatConfig>, // Format configuration
    pub color: Option<ColorConfig>,   // Color configuration
}
```

## Format and Color Usage Examples

### Custom Terminal Format

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FormatConfig, ColorConfig};
use rat_logger::handler::term::TermConfig;

fn main() {
    // Create format configuration
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

    // Create color configuration
    let color_config = ColorConfig {
        error: "\x1b[91m".to_string(),      // Bright red
        warn: "\x1b[93m".to_string(),       // Bright yellow
        info: "\x1b[92m".to_string(),       // Bright green
        debug: "\x1b[96m".to_string(),      // Bright cyan
        trace: "\x1b[95m".to_string(),      // Bright purple
        timestamp: "\x1b[90m".to_string(),  // Dark gray
        target: "\x1b[94m".to_string(),     // Bright blue
        file: "\x1b[95m".to_string(),       // Bright purple
        message: "\x1b[97m".to_string(),     // Bright white
    };

    // Create terminal processor with configuration
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

### Custom File Format

```rust
use rat_logger::{LoggerBuilder, LevelFilter, FileConfig, FormatConfig};
use std::path::PathBuf;

fn main() {
    // Create JSON format configuration
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

    // Create file processor with format configuration
    let file_config = FileConfig {
        log_dir: PathBuf::from("./logs"),
        max_file_size: 10 * 1024 * 1024,  // 10MB
        max_compressed_files: 5,
        compression_level: 6,
        min_compress_threads: 2,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
        force_sync: false,      // Asynchronous write for better performance
        format: Some(json_format),  // Use custom format
    };

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .build();
}
```

## Performance Features

- **Producer-Consumer Architecture**: Separates log generation and processing to avoid blocking the main thread
- **Batch Writing**: 8KB threshold or 100ms interval intelligent batch writing
- **Asynchronous Compression**: Use thread pool for asynchronous file compression
- **Cross-platform Optimization**: Synchronization strategy optimization for different platforms
- **Zero Copy**: Use zero copy technology on critical paths
- **Memory Efficient**: Intelligent buffer management to avoid memory waste

### Performance Benchmark Results

Performance performance on MacBook Air M1 local environment (for reference only):

#### New Version v0.2.3 Performance (Asynchronous Broadcast Architecture)
- Terminal Output: **2,264,813 messages/sec** - 5.6x improvement
- File Output: **2,417,040 messages/sec** - 5.9x improvement
- Terminal+File: **1,983,192 messages/sec** - 3.9x improvement
- Multi-threaded Environment: **3,538,831 messages/sec** - 14.7x improvement ⭐
- Different Log Levels: **4.3M-4.7M messages/sec** - 2.5-5.6x improvement

#### Historical Version Performance (Comparison Reference)
- Terminal Output: ~400,000+ messages/sec (after optimization)
- File Output: ~408,025 messages/sec
- Terminal+File: ~501,567 messages/sec
- Multi-threaded Environment: ~239,808 messages/sec
- Different Log Levels: 833K-1.7M messages/sec

#### UDP Network Transmission Performance (test_client test results)
- 100 messages batch: **806,452 messages/sec**
- 1000 messages batch: **1,215,498 messages/sec**
- 5000 messages batch: **1,087,627 messages/sec**

*Note: UDP network transmission test is based on test_client tool and local loopback interface (127.0.0.1), compiled in release mode, actual network environment performance may vary due to network conditions*

## Thread Safety

rat_logger fully supports multi-threaded environments:

- Use crossbeam-channel for lock-free inter-thread communication
- Support multi-threaded concurrent writing, no data race risk
- Atomic operations for statistical information collection
- Maintain stable performance in high-concurrency scenarios

## Compression Support

Built-in log file compression functionality:

- Use LZ4 compression algorithm to balance compression ratio and performance
- Configurable compression levels (1-9)
- Asynchronous compression thread pool, not blocking the main thread
- Automatic cleanup of old compressed files

## Network Transmission

Support sending logs via UDP protocol:

- High-efficiency serialization based on bincode
- Support token-based authentication mechanism
- Compatible with zerg_creep UDP packet format
- Batch network sending optimization

## Error Handling

rat_logger provides comprehensive error handling mechanisms:

- Internal errors do not affect main program operation
- Graceful error recovery mechanism
- Detailed error logging
- Configurable error handling strategies

## Dependencies

```toml
[dependencies]
rat_logger = "0.2.0"
```

## License

This project is licensed under the LGPLv3 license. See [LICENSE](LICENSE) file for details.

## Contributing

Welcome to submit Issue and Pull Request to improve rat_logger.

## Changelog

Detailed version update records please see [CHANGELOG.md](CHANGELOG.md).

## Example Code

The project includes complete example code:

- `examples/basic_usage.rs` - Basic usage example, showing multiple output methods
- `examples/composite_handler.rs` - Multiple output processor example, terminal+file simultaneous output
- `examples/file_rotation.rs` - File rotation and compression function test
- `examples/sync_async_demo.rs` - Synchronous and asynchronous mode comparison demonstration
- `examples/term_format_example.rs` - Terminal format configuration and color setting example
- `examples/file_format_example.rs` - File format configuration example, including JSON format
- `examples/color_format_example.rs` - Color format configuration example
- `examples/macro_format_example.rs` - Macro and format configuration combined usage example
- `examples/macro_example.rs` - Logging macro usage example, supporting global initialization
- `examples/pm2_style_logging.rs` - PM2 style multi-file log management

All examples are enabled with development mode to ensure immediate log output. When using in production environments, please remove the `with_dev_mode(true)` configuration for best performance.