//! rat_logger 颜色和格式综合示例
//!
//! 专门演示如何同时使用颜色和格式配置，并重点展示如何覆盖颜色配置
//! 解决其他项目中颜色配置不生效的问题
//!
//! ⚠️  重要提醒：
//! - 本示例启用开发模式以确保日志立即输出，方便演示和学习
//! - 在生产环境中，请禁用开发模式以获得最佳性能
//! - 生产环境推荐：LoggerBuilder::new().add_terminal_with_config(config).build()

use rat_logger::{LoggerBuilder, LevelFilter, Level, Logger};
use rat_logger::config::{Record, Metadata};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== rat_logger 颜色和格式综合示例 ===\n");

    // 1. 创建不同的颜色主题
    println!("1. 创建不同的颜色主题:");

    // 1.1 经典主题（覆盖默认颜色）
    let classic_theme = rat_logger::ColorConfig {
        error: "\x1b[91m".to_string(),      // 亮红色
        warn: "\x1b[93m".to_string(),       // 亮黄色
        info: "\x1b[92m".to_string(),       // 亮绿色
        debug: "\x1b[96m".to_string(),      // 亮青色
        trace: "\x1b[95m".to_string(),      // 亮紫色
        timestamp: "\x1b[90m".to_string(),   // 深灰色
        target: "\x1b[94m".to_string(),      // 亮蓝色
        file: "\x1b[95m".to_string(),       // 亮紫色
        message: "\x1b[97m".to_string(),      // 亮白色
    };

    // 1.2 暗黑主题
    let dark_theme = rat_logger::ColorConfig {
        error: "\x1b[38;5;196m".to_string(),  // 红色
        warn: "\x1b[38;5;214m".to_string(),   // 橙色
        info: "\x1b[38;5;40m".to_string(),    // 绿色
        debug: "\x1b[38;5;39m".to_string(),   // 蓝色
        trace: "\x1b[38;5;243m".to_string(),  // 暗灰色
        timestamp: "\x1b[38;5;240m".to_string(), // 更暗的灰色
        target: "\x1b[38;5;45m".to_string(),   // 青色
        file: "\x1b[38;5;201m".to_string(),   // 粉色
        message: "\x1b[38;5;252m".to_string(), // 浅灰色
    };

    // 1.3 高对比度主题
    let high_contrast_theme = rat_logger::ColorConfig {
        error: "\x1b[1;31m".to_string(),     // 粗体红色
        warn: "\x1b[1;33m".to_string(),      // 粗体黄色
        info: "\x1b[1;32m".to_string(),      // 粗体绿色
        debug: "\x1b[1;36m".to_string(),      // 粗体青色
        trace: "\x1b[1;37m".to_string(),      // 粗体白色
        timestamp: "\x1b[1;30m".to_string(),  // 粗体暗灰色
        target: "\x1b[1;34m".to_string(),     // 粗体蓝色
        file: "\x1b[1;35m".to_string(),      // 粗体紫色
        message: "\x1b[0m".to_string(),       // 重置
    };

    // 1.4 柔和主题
    let soft_theme = rat_logger::ColorConfig {
        error: "\x1b[38;5;167m".to_string(),  // 柔和红色
        warn: "\x1b[38;5;179m".to_string(),   // 柔和橙色
        info: "\x1b[38;5;72m".to_string(),    // 柔和绿色
        debug: "\x1b[38;5;110m".to_string(),  // 柔和青色
        trace: "\x1b[38;5;145m".to_string(),  // 柔和紫色
        timestamp: "\x1b[38;5;244m".to_string(), // 柔和灰色
        target: "\x1b[38;5;104m".to_string(),  // 柔和蓝紫色
        file: "\x1b[38;5;133m".to_string(),   // 柔和品红
        message: "\x1b[38;5;251m".to_string(), // 极浅灰色
    };

    println!("   ✓ 已创建4种颜色主题\n");

    // 2. 创建不同的格式配置
    println!("2. 创建不同的格式配置:");

    // 2.1 JSON风格格式
    let json_format = rat_logger::FormatConfig {
        timestamp_format: "%Y-%m-%dT%H:%M:%S%.3fZ".to_string(),
        level_style: rat_logger::LevelStyle {
            error: "error".to_string(),
            warn: "warn".to_string(),
            info: "info".to_string(),
            debug: "debug".to_string(),
            trace: "trace".to_string(),
        },
        format_template: "{{\"timestamp\":\"{timestamp}\",\"level\":\"{level}\",\"target\":\"{target}\",\"message\":\"{message}\"}}".to_string(),
    };

    // 2.2 简洁风格格式
    let minimal_format = rat_logger::FormatConfig {
        timestamp_format: "%H:%M:%S".to_string(),
        level_style: rat_logger::LevelStyle {
            error: "ERR".to_string(),
            warn: "WRN".to_string(),
            info: "INF".to_string(),
            debug: "DBG".to_string(),
            trace: "TRC".to_string(),
        },
        format_template: "{timestamp} [{level}] {message}".to_string(),
    };

    // 2.3 详细风格格式
    let detailed_format = rat_logger::FormatConfig {
        timestamp_format: "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        level_style: rat_logger::LevelStyle {
            error: "ERROR".to_string(),
            warn: "WARN ".to_string(),
            info: "INFO ".to_string(),
            debug: "DEBUG".to_string(),
            trace: "TRACE".to_string(),
        },
        format_template: "[{timestamp}] {level} | {target} | {file}:{line} | {message}".to_string(),
    };

    // 2.4 自定义分隔符格式
    let custom_sep_format = rat_logger::FormatConfig {
        timestamp_format: "%Y/%m/%d %H:%M:%S".to_string(),
        level_style: rat_logger::LevelStyle {
            error: "🔴 ERROR".to_string(),
            warn: "🟡 WARN".to_string(),
            info: "🟢 INFO".to_string(),
            debug: "🔵 DEBUG".to_string(),
            trace: "⚪ TRACE".to_string(),
        },
        format_template: "┌─ {timestamp}\n├─ {level}\n├─ {target}\n├─ {file}:{line}\n└─ {message}".to_string(),
    };

    println!("   ✓ 已创建4种格式配置\n");

    // 3. 测试不同的颜色和格式组合
    println!("3. 测试不同的颜色和格式组合:");

    // 3.1 经典主题 + 详细格式（完全覆盖默认配置）
    println!("   3.1 经典主题 + 详细格式（完全覆盖默认配置）:");
    let term_config1 = rat_logger::handler::term::TermConfig {
        format: Some(detailed_format.clone()),
        color: Some(classic_theme.clone()),
        ..Default::default()
    };

    let logger1 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_terminal_with_config(term_config1)
        .build();

    logger1.log(&create_test_record(Level::Error, "classic_detailed", "这是一个经典主题的详细格式错误消息"));
    logger1.log(&create_test_record(Level::Info, "classic_detailed", "这是一个经典主题的详细格式信息消息"));
    logger1.log(&create_test_record(Level::Debug, "classic_detailed", "这是一个经典主题的详细格式调试消息"));

    // 3.2 暗黑主题 + JSON格式
    println!("\n   3.2 暗黑主题 + JSON格式:");
    let term_config2 = rat_logger::handler::term::TermConfig {
        format: Some(json_format.clone()),
        color: Some(dark_theme.clone()),
        ..Default::default()
    };

    let logger2 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_terminal_with_config(term_config2)
        .build();

    logger2.log(&create_test_record(Level::Error, "dark_json", "这是一个暗黑主题的JSON格式错误消息"));
    logger2.log(&create_test_record(Level::Info, "dark_json", "这是一个暗黑主题的JSON格式信息消息"));

    // 3.3 高对比度主题 + 简洁格式
    println!("\n   3.3 高对比度主题 + 简洁格式:");
    let term_config3 = rat_logger::handler::term::TermConfig {
        format: Some(minimal_format.clone()),
        color: Some(high_contrast_theme.clone()),
        ..Default::default()
    };

    let logger3 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_terminal_with_config(term_config3)
        .build();

    logger3.log(&create_test_record(Level::Error, "high_contrast_minimal", "这是一个高对比度主题的简洁格式错误消息"));
    logger3.log(&create_test_record(Level::Warn, "high_contrast_minimal", "这是一个高对比度主题的简洁格式警告消息"));
    logger3.log(&create_test_record(Level::Info, "high_contrast_minimal", "这是一个高对比度主题的简洁格式信息消息"));

    // 3.4 柔和主题 + 自定义分隔符格式
    println!("\n   3.4 柔和主题 + 自定义分隔符格式:");
    let term_config4 = rat_logger::handler::term::TermConfig {
        format: Some(custom_sep_format.clone()),
        color: Some(soft_theme.clone()),
        ..Default::default()
    };

    let logger4 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_terminal_with_config(term_config4)
        .build();

    logger4.log(&create_test_record(Level::Error, "soft_custom", "这是一个柔和主题的自定义分隔符格式错误消息"));
    logger4.log(&create_test_record(Level::Info, "soft_custom", "这是一个柔和主题的自定义分隔符格式信息消息"));

    // 4. 演示颜色配置覆盖技巧
    println!("\n4. 演示颜色配置覆盖技巧:");

    // 4.1 仅覆盖特定颜色的示例
    println!("   4.1 仅覆盖特定颜色（其他使用默认）:");
    let partial_color_override = rat_logger::ColorConfig {
        error: "\x1b[1;31;41m".to_string(),  // 红色背景
        warn: "\x1b[1;33;43m".to_string(),   // 黄色背景
        info: "\x1b[1;32;42m".to_string(),   // 绿色背景
        // debug、trace等使用默认值
        debug: "\x1b[36m".to_string(),       // 青色（与默认相同）
        trace: "\x1b[37m".to_string(),       // 白色（与默认相同）
        timestamp: "\x1b[90m".to_string(),   // 深灰色（与默认相同）
        target: "\x1b[34m".to_string(),      // 蓝色（与默认相同）
        file: "\x1b[35m".to_string(),       // 紫色（与默认相同）
        message: "\x1b[0m".to_string(),      // 重置（与默认相同）
    };

    let term_config5 = rat_logger::handler::term::TermConfig {
        format: Some(detailed_format.clone()),
        color: Some(partial_color_override.clone()),
        ..Default::default()
    };

    let logger5 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_terminal_with_config(term_config5)
        .build();

    logger5.log(&create_test_record(Level::Error, "partial_override", "这是错误级别的背景色效果"));
    logger5.log(&create_test_record(Level::Warn, "partial_override", "这是警告级别的背景色效果"));
    logger5.log(&create_test_record(Level::Info, "partial_override", "这是信息级别的背景色效果"));

    // 4.2 动态颜色配置（基于内容）
    println!("\n   4.2 动态颜色配置示例:");
    for (i, level) in [Level::Error, Level::Warn, Level::Info, Level::Debug, Level::Trace].iter().enumerate() {
        let dynamic_theme = create_dynamic_theme(i);
        let term_config = rat_logger::handler::term::TermConfig {
            format: Some(minimal_format.clone()),
            color: Some(dynamic_theme),
            ..Default::default()
        };

        let logger = LoggerBuilder::new()
            .with_level(LevelFilter::Debug)
            .with_dev_mode(true)
            .add_terminal_with_config(term_config)
            .build();

        logger.log(&create_test_record(*level, "dynamic", &format!("动态颜色示例 {} - {}", i + 1, level)));
    }

    // 5. 常见问题和解决方案
    println!("\n5. 常见问题和解决方案:");

    // 5.1 检查颜色是否启用的示例
    println!("   5.1 检查颜色启用状态:");
    let color_check_config = rat_logger::handler::term::TermConfig {
        enable_color: true,  // 确保启用颜色
        format: Some(detailed_format.clone()),
        color: Some(classic_theme.clone()),
        ..Default::default()
    };

    let logger6 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_terminal_with_config(color_check_config)
        .build();

    logger6.log(&create_test_record(Level::Info, "color_check", "这是一个确保颜色启用的示例"));

    // 5.2 无颜色输出的对比
    println!("\n   5.2 无颜色输出的对比:");
    let no_color_config = rat_logger::handler::term::TermConfig {
        enable_color: false,  // 禁用颜色
        format: Some(detailed_format.clone()),
        color: None,          // 不提供颜色配置
        ..Default::default()
    };

    let logger7 = LoggerBuilder::new()
        .with_level(LevelFilter::Debug)
        .with_dev_mode(true)
        .add_terminal_with_config(no_color_config)
        .build();

    logger7.log(&create_test_record(Level::Info, "no_color", "这是一个无颜色输出的示例"));

    println!("\n=== 示例完成 ===");
    println!("\n重要提示:");
    println!("1. 要覆盖颜色配置，必须在TermConfig中提供color参数");
    println!("2. enable_color必须设置为true才能看到颜色效果");
    println!("3. 颜色代码使用ANSI转义序列：\\x1b[颜色代码m");
    println!("4. 重置颜色代码：\\x1b[0m");
    println!("5. 可以部分覆盖颜色配置，只修改需要的字段");
    println!("6. 格式和颜色配置可以独立使用，也可以组合使用");

    println!("\n颜色代码参考:");
    println!("- 基础颜色: 30-37（黑色到白色）");
    println!("- 亮色: 90-97（亮黑到亮白）");
    println!("- 256色: 38;5;n（n=0-255）");
    println!("- RGB色: 38;2;r;g;b（r,g,b=0-255）");
    println!("- 粗体: 1");
    println!("- 下划线: 4");
    println!("- 背景色: 40-47（黑色到白色背景）");

    Ok(())
}

/// 创建动态颜色主题
fn create_dynamic_theme(seed: usize) -> rat_logger::ColorConfig {
    let colors = [
        "\x1b[31m", "\x1b[32m", "\x1b[33m", "\x1b[34m", "\x1b[35m", "\x1b[36m",
        "\x1b[91m", "\x1b[92m", "\x1b[93m", "\x1b[94m", "\x1b[95m", "\x1b[96m",
        "\x1b[38;5;196m", "\x1b[38;5;202m", "\x1b[38;5;208m", "\x1b[38;5;214m",
        "\x1b[38;5;220m", "\x1b[38;5;226m", "\x1b[38;5;40m", "\x1b[38;5;46m",
        "\x1b[38;5;82m", "\x1b[38;5;118m", "\x1b[38;5;154m", "\x1b[38;5;190m",
    ];

    let get_color = |index: usize| colors[index % colors.len()].to_string();

    rat_logger::ColorConfig {
        error: get_color(seed),
        warn: get_color(seed + 1),
        info: get_color(seed + 2),
        debug: get_color(seed + 3),
        trace: get_color(seed + 4),
        timestamp: "\x1b[90m".to_string(),
        target: "\x1b[94m".to_string(),
        file: "\x1b[95m".to_string(),
        message: "\x1b[0m".to_string(),
    }
}

/// 创建测试日志记录
fn create_test_record(
    level: Level,
    target: &str,
    message: &str,
) -> Record {
    Record {
        metadata: Arc::new(Metadata {
            level,
            target: target.to_string(),
            auth_token: None,
            app_id: Some("color_format_example".to_string()),
        }),
        args: message.to_string(),
        module_path: Some("color_format_example".to_string()),
        file: Some("color_format_example.rs".to_string()),
        line: Some(42),
    }
}