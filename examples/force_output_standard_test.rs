use rat_logger::{LoggerBuilder, LevelFilter, emergency, flush_logs};
use rat_logger::producer_consumer::BatchConfig;
use rat_logger::config::{Record, Metadata};

fn format_time_ms(instant: &std::time::Instant) -> String {
    format!("{:.0}ms", instant.elapsed().as_millis())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 强制输出验证测试（标准代码） ===");
    println!("配置：异步模式 + 大批量(100) + 大延迟(1000ms)");
    println!("预期：普通日志被缓冲，emergency日志强制输出\n");

    // 初始化全局日志器 - 异步模式+大批量+大延迟
    LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_async_mode(true)             // 启用异步模式
        .with_batch_config(BatchConfig {
            batch_size: 100,              // 需要累积100条日志才输出
            batch_interval_ms: 1000,     // 或者延迟1000ms才输出
            buffer_size: 1024,
        })
        .add_terminal_with_config(rat_logger::handler::term::TermConfig::default())
        .init_global_logger()?;

    println!("✓ 全局日志器初始化完成\n");

    // 获取全局日志器引用
    let logger = {
        let guard = rat_logger::core::LOGGER.lock().unwrap();
        guard.as_ref().unwrap().clone()
    };

    // 测试1：普通日志（应该被缓冲，不会立即输出）
    println!("1. 测试普通日志（预期：被缓冲，不立即输出）：");
    let start_time = std::time::Instant::now();
    println!("   [{}] ===== 开始发送5条普通日志 =====", format_time_ms(&start_time));

    for i in 1..=5 {
        println!("   [{}] --- 发送普通日志 {} 前 ---", format_time_ms(&start_time), i);
        let record = Record {
            metadata: std::sync::Arc::new(Metadata {
                level: rat_logger::Level::Info,
                target: "test".to_string(),
                auth_token: None,
                app_id: None,
            }),
            args: format!("普通日志消息 {} - 这条消息应该被缓冲", i),
            module_path: Some("force_output_standard_test".to_string()),
            file: Some("force_output_standard_test.rs".to_string()),
            line: Some(24),
        };
        logger.log(&record);
        println!("   [{}] --- 发送普通日志 {} 后 ---", format_time_ms(&start_time), i);
        std::thread::sleep(std::time::Duration::from_millis(100)); // 短暂延迟，避免混在一起
    }

    println!("   [{}] ===== 5条普通日志发送完成，现在等待1秒 =====\n", format_time_ms(&start_time));

    // 等待1秒，检查是否会因为时间间隔而输出
    std::thread::sleep(std::time::Duration::from_millis(1100));
    println!("   [{}] ===== 1秒等待结束 =====\n", format_time_ms(&start_time));

    // 测试2：强制输出日志（应该立即输出）
    println!("2. 测试强制输出日志（预期：立即输出）：");
    println!("   [{}] 开始发送紧急日志1", format_time_ms(&start_time));
    let emergency_record1 = Record {
        metadata: std::sync::Arc::new(Metadata {
            level: rat_logger::Level::Error,
            target: "test".to_string(),
            auth_token: None,
            app_id: None,
        }),
        args: "这是强制输出的紧急日志1 - 应该立即输出".to_string(),
        module_path: Some("force_output_standard_test".to_string()),
        file: Some("force_output_standard_test.rs".to_string()),
        line: Some(39),
    };
    logger.emergency_log(&emergency_record1);
    println!("   [{}] 紧急日志1发送完成", format_time_ms(&start_time));

    println!("   [{}] 开始发送紧急日志2", format_time_ms(&start_time));
    let emergency_record2 = Record {
        metadata: std::sync::Arc::new(Metadata {
            level: rat_logger::Level::Error,
            target: "test".to_string(),
            auth_token: None,
            app_id: None,
        }),
        args: "这是强制输出的紧急日志2 - 应该立即输出".to_string(),
        module_path: Some("force_output_standard_test".to_string()),
        file: Some("force_output_standard_test.rs".to_string()),
        line: Some(50),
    };
    logger.emergency_log(&emergency_record2);
    println!("   [{}] 紧急日志2发送完成，应该立即看到输出\n", format_time_ms(&start_time));

    // 测试3：混合场景
    println!("3. 测试混合场景（普通日志+强制输出）：");
    println!("   [{}] 发送普通日志A（缓冲）", format_time_ms(&start_time));
    let normal_record_a = Record {
        metadata: std::sync::Arc::new(Metadata {
            level: rat_logger::Level::Info,
            target: "test".to_string(),
            auth_token: None,
            app_id: None,
        }),
        args: "普通日志 A - 被缓冲".to_string(),
        module_path: Some("force_output_standard_test".to_string()),
        file: Some("force_output_standard_test.rs".to_string()),
        line: Some(65),
    };
    logger.log(&normal_record_a);

    println!("   [{}] 发送普通日志B（缓冲）", format_time_ms(&start_time));
    let normal_record_b = Record {
        metadata: std::sync::Arc::new(Metadata {
            level: rat_logger::Level::Info,
            target: "test".to_string(),
            auth_token: None,
            app_id: None,
        }),
        args: "普通日志 B - 被缓冲".to_string(),
        module_path: Some("force_output_standard_test".to_string()),
        file: Some("force_output_standard_test.rs".to_string()),
        line: Some(76),
    };
    logger.log(&normal_record_b);

    println!("   [{}] 发送紧急日志C（强制输出，应该刷新A和B）", format_time_ms(&start_time));
    let emergency_record_c = Record {
        metadata: std::sync::Arc::new(Metadata {
            level: rat_logger::Level::Error,
            target: "test".to_string(),
            auth_token: None,
            app_id: None,
        }),
        args: "紧急日志 C - 强制输出（同时会刷新缓冲区的A和B）".to_string(),
        module_path: Some("force_output_standard_test".to_string()),
        file: Some("force_output_standard_test.rs".to_string()),
        line: Some(87),
    };
    logger.emergency_log(&emergency_record_c);

    println!("   [{}] 发送普通日志D（缓冲）", format_time_ms(&start_time));
    let normal_record_d = Record {
        metadata: std::sync::Arc::new(Metadata {
            level: rat_logger::Level::Info,
            target: "test".to_string(),
            auth_token: None,
            app_id: None,
        }),
        args: "普通日志 D - 被缓冲".to_string(),
        module_path: Some("force_output_standard_test".to_string()),
        file: Some("force_output_standard_test.rs".to_string()),
        line: Some(98),
    };
    logger.log(&normal_record_d);

    println!("   [{}] 混合日志发送完成，紧急日志应该立即输出并刷新之前的缓冲区\n", format_time_ms(&start_time));

    // 测试4：手动刷新功能
    println!("4. 测试手动刷新功能：");
    println!("   [{}] 发送普通日志E（缓冲）", format_time_ms(&start_time));
    let normal_record_e = Record {
        metadata: std::sync::Arc::new(Metadata {
            level: rat_logger::Level::Info,
            target: "test".to_string(),
            auth_token: None,
            app_id: None,
        }),
        args: "普通日志 E - 被缓冲".to_string(),
        module_path: Some("force_output_standard_test".to_string()),
        file: Some("force_output_standard_test.rs".to_string()),
        line: Some(113),
    };
    logger.log(&normal_record_e);

    println!("   [{}] 发送普通日志F（缓冲）", format_time_ms(&start_time));
    let normal_record_f = Record {
        metadata: std::sync::Arc::new(Metadata {
            level: rat_logger::Level::Info,
            target: "test".to_string(),
            auth_token: None,
            app_id: None,
        }),
        args: "普通日志 F - 被缓冲".to_string(),
        module_path: Some("force_output_standard_test".to_string()),
        file: Some("force_output_standard_test.rs".to_string()),
        line: Some(124),
    };
    logger.log(&normal_record_f);

    println!("   [{}] 手动调用刷新命令...", format_time_ms(&start_time));
    logger.force_flush();
    println!("   [{}] 刷新命令调用完成", format_time_ms(&start_time));

    // 再等待一小段时间确保刷新完成
    std::thread::sleep(std::time::Duration::from_millis(100));
    println!("   [{}] 等待刷新完成", format_time_ms(&start_time));

    println!("\n=== 测试总结 ===");
    println!("如果测试成功，你应该看到：");
    println!("1. 前5条普通日志在1秒后批量输出（时间间隔触发）");
    println!("2. 2条紧急日志立即输出");
    println!("3. 混合场景中的紧急日志立即输出并刷新之前的缓冲区");
    println!("4. 手动刷新命令立即输出缓冲的日志");

    Ok(())
}