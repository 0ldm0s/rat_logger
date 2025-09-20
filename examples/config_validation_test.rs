//! 测试配置冲突检测
use rat_logger::{LoggerBuilder, LevelFilter};
use rat_logger::handler::term::TermConfig;
use rat_logger::handler::file::FileProcessorConfig;
use rat_logger::config::{FileConfig, FormatConfig, ColorConfig};

fn main() {
    println!("=== 配置冲突检测测试 ===\n");

    // 测试1: TermConfig 颜色冲突
    println!("1. 测试 TermConfig 颜色冲突:");
    let invalid_term_config = TermConfig {
        enable_color: false,  // 禁用颜色
        color: Some(ColorConfig::default()),  // 但提供了颜色配置
        ..Default::default()
    };

    match std::panic::catch_unwind(|| {
        LoggerBuilder::new()
            .with_level(LevelFilter::Debug)
            .add_terminal_with_config(invalid_term_config)
            .build()
    }) {
        Ok(_) => println!("   ❌ 错误: 应该检测到配置冲突但没有"),
        Err(_) => println!("   ✅ 正确: 成功检测到颜色配置冲突"),
    }

    // 测试2: TermConfig 无效批量大小
    println!("\n2. 测试 TermConfig 无效批量大小:");
    let invalid_batch_config = TermConfig {
        batch_size: 0,  // 无效的批量大小
        ..Default::default()
    };

    match std::panic::catch_unwind(|| {
        LoggerBuilder::new()
            .with_level(LevelFilter::Debug)
            .add_terminal_with_config(invalid_batch_config)
            .build()
    }) {
        Ok(_) => println!("   ❌ 错误: 应该检测到批量大小错误但没有"),
        Err(_) => println!("   ✅ 正确: 成功检测到批量大小错误"),
    }

    // 测试3: FileConfig 原始模式冲突
    println!("\n3. 测试 FileConfig 原始模式冲突:");
    let invalid_file_config = FileConfig {
        is_raw: true,  // 原始模式
        format: Some(FormatConfig::default()),  // 但提供了格式配置
        ..Default::default()
    };

    let file_processor_config = FileProcessorConfig {
        file_config: invalid_file_config,
        ..Default::default()
    };

    match std::panic::catch_unwind(|| {
        rat_logger::handler::file::FileProcessor::with_config(file_processor_config)
    }) {
        Ok(_) => println!("   ❌ 错误: 应该检测到原始模式冲突但没有"),
        Err(_) => println!("   ✅ 正确: 成功检测到原始模式冲突"),
    }

    // 测试4: FileConfig 无效文件大小
    println!("\n4. 测试 FileConfig 无效文件大小:");
    let invalid_size_config = FileConfig {
        max_file_size: 0,  // 无效的文件大小
        ..Default::default()
    };

    let file_processor_config = FileProcessorConfig {
        file_config: invalid_size_config,
        ..Default::default()
    };

    match std::panic::catch_unwind(|| {
        rat_logger::handler::file::FileProcessor::with_config(file_processor_config)
    }) {
        Ok(_) => println!("   ❌ 错误: 应该检测到文件大小错误但没有"),
        Err(_) => println!("   ✅ 正确: 成功检测到文件大小错误"),
    }

    // 测试5: 正确配置应该工作
    println!("\n5. 测试正确配置:");
    let valid_term_config = TermConfig {
        enable_color: true,
        color: Some(ColorConfig::default()),
        format: Some(FormatConfig::default()),
        ..Default::default()
    };

    match std::panic::catch_unwind(|| {
        LoggerBuilder::new()
            .with_level(LevelFilter::Debug)
            .add_terminal_with_config(valid_term_config)
            .build()
    }) {
        Ok(_) => println!("   ✅ 正确: 有效配置正常工作"),
        Err(_) => println!("   ❌ 错误: 有效配置被错误拒绝"),
    }

    println!("\n=== 配置冲突检测测试完成 ===");
}