//! rat_logger 性能对比测试
//!
//! 测试不同配置下的性能表现：
//! 1. 纯终端输出
//! 2. 纯文件输出
//! 3. 终端+文件输出

use rat_logger::{LoggerBuilder, LevelFilter, Level, FileConfig, config::Record, Logger};
use rat_logger::config::Metadata;
use std::sync::Arc;
use std::time::Instant;
use std::path::PathBuf;
use std::fs;

const ITERATIONS: usize = 10000;
const THREAD_COUNT: usize = 4;

fn create_test_record(level: Level, message: &str) -> Record {
    Record {
        metadata: Arc::new(Metadata {
            level,
            target: "performance_test".to_string(),
            auth_token: None,
            app_id: Some("test_app".to_string()),
        }),
        args: message.to_string(),
        module_path: Some("performance_test".to_string()),
        file: Some("performance_test.rs".to_string()),
        line: Some(42),
    }
}

fn benchmark_terminal_only() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 纯终端输出性能测试 ===");

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal()
        .build();

    let start = Instant::now();

    for i in 0..ITERATIONS {
        let record = create_test_record(
            Level::Info,
            &format!("终端日志消息 #{}", i)
        );
        logger.log(&record);
    }

    let duration = start.elapsed();
    let throughput = ITERATIONS as f64 / duration.as_secs_f64();

    println!("迭代次数: {}", ITERATIONS);
    println!("总耗时: {:?}", duration);
    println!("吞吐量: {:.0} 条/秒", throughput);
    println!("平均延迟: {:.3} 毫秒/条", duration.as_millis() as f64 / ITERATIONS as f64);

    Ok(())
}

fn benchmark_file_only() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 纯文件输出性能测试 ===");

    // 清理测试目录
    let test_dir = PathBuf::from("./file_test_logs");
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }
    fs::create_dir_all(&test_dir)?;

    let file_config = FileConfig {
        log_dir: test_dir.clone(),
        max_file_size: 1024 * 1024 * 100, // 100MB
        max_compressed_files: 0, // 不压缩以测试纯写入性能
        compression_level: 0,
        min_compress_threads: 0,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .build();

    let start = Instant::now();

    for i in 0..ITERATIONS {
        let record = create_test_record(
            Level::Info,
            &format!("文件日志消息 #{}", i)
        );
        logger.log(&record);
    }

    let duration = start.elapsed();
    let throughput = ITERATIONS as f64 / duration.as_secs_f64();

    println!("迭代次数: {}", ITERATIONS);
    println!("总耗时: {:?}", duration);
    println!("吞吐量: {:.0} 条/秒", throughput);
    println!("平均延迟: {:.3} 毫秒/条", duration.as_millis() as f64 / ITERATIONS as f64);

    // 清理测试文件
    fs::remove_dir_all(&test_dir)?;

    Ok(())
}

fn benchmark_terminal_and_file() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 终端+文件输出性能测试 ===");

    // 清理测试目录
    let test_dir = PathBuf::from("./combined_test_logs");
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }
    fs::create_dir_all(&test_dir)?;

    let file_config = FileConfig {
        log_dir: test_dir.clone(),
        max_file_size: 1024 * 1024 * 100, // 100MB
        max_compressed_files: 0, // 不压缩以测试纯写入性能
        compression_level: 0,
        min_compress_threads: 0,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_terminal()
        .add_file(file_config)
        .build();

    let start = Instant::now();

    for i in 0..ITERATIONS {
        let record = create_test_record(
            Level::Info,
            &format!("终端+文件日志消息 #{}", i)
        );
        logger.log(&record);
    }

    let duration = start.elapsed();
    let throughput = ITERATIONS as f64 / duration.as_secs_f64();

    println!("迭代次数: {}", ITERATIONS);
    println!("总耗时: {:?}", duration);
    println!("吞吐量: {:.0} 条/秒", throughput);
    println!("平均延迟: {:.3} 毫秒/条", duration.as_millis() as f64 / ITERATIONS as f64);

    // 清理测试文件
    fs::remove_dir_all(&test_dir)?;

    Ok(())
}

fn benchmark_multithreaded() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 多线程文件输出性能测试 ===");

    // 清理测试目录
    let test_dir = PathBuf::from("./multithread_test_logs");
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }
    fs::create_dir_all(&test_dir)?;

    let file_config = FileConfig {
        log_dir: test_dir.clone(),
        max_file_size: 1024 * 1024 * 100, // 100MB
        max_compressed_files: 0,
        compression_level: 0,
        min_compress_threads: 0,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    let logger = Arc::new(LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .build());

    let start = Instant::now();
    let mut handles = vec![];

    for thread_id in 0..THREAD_COUNT {
        let logger_clone = logger.clone();
        let handle = std::thread::spawn(move || {
            let per_thread_iterations = ITERATIONS / THREAD_COUNT;
            for i in 0..per_thread_iterations {
                let record = create_test_record(
                    Level::Info,
                    &format!("线程{}日志消息 #{}", thread_id, i)
                );
                logger_clone.log(&record);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    let total_operations = ITERATIONS;
    let throughput = total_operations as f64 / duration.as_secs_f64();

    println!("线程数量: {}", THREAD_COUNT);
    println!("总操作次数: {}", total_operations);
    println!("总耗时: {:?}", duration);
    println!("吞吐量: {:.0} 条/秒", throughput);
    println!("平均延迟: {:.3} 毫秒/条", duration.as_millis() as f64 / total_operations as f64);

    // 清理测试文件
    fs::remove_dir_all(&test_dir)?;

    Ok(())
}

fn benchmark_different_log_levels() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 不同日志级别性能测试 ===");

    let test_dir = PathBuf::from("./levels_test_logs");
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir)?;
    }
    fs::create_dir_all(&test_dir)?;

    let file_config = FileConfig {
        log_dir: test_dir.clone(),
        max_file_size: 1024 * 1024 * 100,
        max_compressed_files: 0,
        compression_level: 0,
        min_compress_threads: 0,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    let levels = vec![
        (Level::Error, "Error"),
        (Level::Warn, "Warn"),
        (Level::Info, "Info"),
        (Level::Debug, "Debug"),
        (Level::Trace, "Trace"),
    ];

    for (level, level_name) in levels {
        let logger = LoggerBuilder::new()
            .with_level(LevelFilter::Trace)
            .add_file(file_config.clone())
            .build();

        let start = Instant::now();

        for i in 0..ITERATIONS / 20 { // 进一步减少迭代次数
            let record = create_test_record(
                level,
                &format!("{}级别日志消息 #{}", level_name, i)
            );
            logger.log(&record);
        }

        let duration = start.elapsed();
        let throughput = (ITERATIONS / 10) as f64 / duration.as_secs_f64();

        println!("{}级别 - 吞吐量: {:.0} 条/秒, 平均延迟: {:.3} 毫秒/条",
                 level_name, throughput, duration.as_millis() as f64 / (ITERATIONS / 10) as f64);
    }

    // 清理测试文件
    fs::remove_dir_all(&test_dir)?;

    Ok(())
}

#[test]
fn test_performance_comparison() {
    println!("开始rat_logger性能对比测试");
    println!("测试配置:");
    println!("- 迭代次数: {}", ITERATIONS);
    println!("- 线程数量: {}", THREAD_COUNT);
    println!("================================");

    // 运行各项性能测试
    benchmark_terminal_only().unwrap();
    benchmark_file_only().unwrap();
    benchmark_terminal_and_file().unwrap();
    benchmark_multithreaded().unwrap();
    benchmark_different_log_levels().unwrap();

    println!("\n================================");
    println!("性能测试完成！");
}

#[test]
fn test_basic_functionality() {
    // 基本功能测试
    let test_dir = PathBuf::from("./basic_test_logs");
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir).unwrap();
    }
    fs::create_dir_all(&test_dir).unwrap();

    let file_config = FileConfig {
        log_dir: test_dir.clone(),
        max_file_size: 1024 * 1024,
        max_compressed_files: 1,
        compression_level: 0,
        min_compress_threads: 0,
        skip_server_logs: false,
        is_raw: false,
        compress_on_drop: false,
    };

    let logger = LoggerBuilder::new()
        .with_level(LevelFilter::Info)
        .add_file(file_config)
        .build();

    let record = create_test_record(Level::Info, "功能测试消息");
    logger.log(&record);

    // 清理
    let _ = fs::remove_dir_all(&test_dir); // 忽略清理错误
}

fn main() {
    test_performance_comparison();
    test_basic_functionality();
}