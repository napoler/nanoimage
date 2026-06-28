//! CLI 子命令 - batch
use crate::commands::common::load_config;
use crate::commands::output::{success, error, dot, file_error, print_table};
use anyhow::Result;
use nanoimage_core::{format_size, BatchProcessor, OutputFormat};
use std::path::PathBuf;

/// 输出格式（CLI 参数）
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormatArg {
    Jpg,
    Png,
    #[value(alias = "webp")]
    WebP,
    Gif,
    #[value(alias = "keeporiginal")]
    KeepOriginal,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(f: OutputFormatArg) -> Self {
        match f {
            OutputFormatArg::Jpg => OutputFormat::Jpeg,
            OutputFormatArg::Png => OutputFormat::Png,
            OutputFormatArg::WebP => OutputFormat::WebP,
            OutputFormatArg::Gif => OutputFormat::Gif,
            OutputFormatArg::KeepOriginal => OutputFormat::KeepOriginal,
        }
    }
}

#[derive(clap::Parser)]
pub struct Args {
    /// 输入目录
    #[arg(short, long)]
    input: PathBuf,

    /// 输出目录
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// 质量 1-100
    #[arg(short, long, default_value = "85")]
    quality: u8,

    /// 工作线程数
    #[arg(short, long, default_value = "4")]
    workers: usize,

    /// 递归处理子目录
    #[arg(short, long)]
    recursive: bool,

    /// 覆盖源文件
    #[arg(short = 'W', long)]
    overwrite: bool,

    /// 目标输出格式 (keeporiginal/jpg/png/webp/gif)
    #[arg(short, long, value_enum)]
    format: Option<OutputFormatArg>,

    /// 最大宽度 (像素)
    #[arg(long)]
    max_width: Option<u32>,

    /// 最大高度 (像素)
    #[arg(long)]
    max_height: Option<u32>,

    /// 预览模式：仅显示处理计划，不实际处理
    #[arg(long)]
    dry_run: bool,

    /// 跳过处理失败的文件，继续处理其他文件
    #[arg(short = 's', long = "skip-failed")]
    skip_failed: bool,

    /// 仅处理未优化的文件（跳过文件名包含 _optimized 的文件）
    #[arg(long = "only-unoptimized")]
    only_unoptimized: bool,
}

/// 获取目标格式字符串（用于 dry-run 预览）
fn target_format_str(args: &Args, config: &nanoimage_core::OptimizerConfig) -> String {
    let fmt = args
        .format
        .as_ref()
        .map(|f| f.clone().into())
        .unwrap_or(config.format);
    match fmt {
        OutputFormat::KeepOriginal => "keep".to_string(),
        OutputFormat::Jpeg => "jpg".to_string(),
        OutputFormat::Png => "png".to_string(),
        OutputFormat::WebP => "webp".to_string(),
        OutputFormat::Gif => "gif".to_string(),
    }
}

/// 截断字符串以适应表格列宽
fn truncate_str(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut width = 0;
    
    for c in s.chars() {
        let char_width = if c as u32 > 0x2FF && c as u32 != 0x3030 { 2 } else { 1 };
        if width + char_width > max_width - 3 {
            result.push_str("...");
            break;
        }
        result.push(c);
        width += char_width;
    }
    
    // 补齐空格
    while width < max_width - 3 {
        result.push(' ');
        width += 1;
    }
    
    result
}

pub fn execute(args: Args) -> Result<()> {
    let mut config = load_config();
    config.quality.lossy = args.quality;
    config.workers = args.workers;
    config.overwrite = args.overwrite;

    if let Some(output) = &args.output {
        config.output_dir = Some(output.clone());
    }

    // 应用 --format, --max-width, --max-height
    if let Some(fmt) = &args.format {
        config.format = fmt.clone().into();
    }
    if let Some(w) = args.max_width {
        config.max_width = Some(w);
    }
    if let Some(h) = args.max_height {
        config.max_height = Some(h);
    }

    let processor = BatchProcessor::with_config(config.clone());

    success(&format!("🔍 扫描目录: {}", args.input.display()));
    let files = BatchProcessor::collect_images(&args.input, args.recursive);
    success(&format!("📁 找到 {} 个图像文件", files.len()));

    if files.is_empty() {
        error("没有找到图像文件");
        return Ok(());
    }

    // dry-run 模式：打印预览表格并退出
    if args.dry_run {
        let target_fmt = target_format_str(&args, &config);
        println!("\n📋 预览模式 - 共 {} 个文件待处理", files.len());

        let mut rows: Vec<Vec<String>> = Vec::with_capacity(files.len());
        for file in &files {
            let name = file
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let orig_size = std::fs::metadata(file)
                .map(|m| format_size(m.len()))
                .unwrap_or_else(|_| "0 B".to_string());
            rows.push(vec![name, orig_size, target_fmt.clone(), "待处理".to_string()]);
        }

        print_table(
            &["文件名", "原始大小", "目标格式", "状态"],
            rows,
        );
        return Ok(());
    }

    success(&format!("⚡ 开始处理 ({} 线程)...", args.workers));
    let (results, _failed_count) = processor.process_sync_with_options(
        &files,
        args.skip_failed,
        args.only_unoptimized,
    );

    let mut success_count = 0;
    let mut total_original: u64 = 0;
    let mut total_new: u64 = 0;
    let mut failures: Vec<(String, String)> = Vec::new();

    for result in &results {
        let name = result
            .original_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if result.success {
            success_count += 1;
            total_original += result.original_size;
            total_new += result.new_size;
            dot();
        } else {
            println!();
            file_error(&name, &result.error.clone().unwrap_or_default());
            failures.push((name, result.error.clone().unwrap_or_default()));
        }
    }

    println!();

    // 打印结果表格
    if success_count > 0 {
        println!("\n📊 压缩结果");
        println!("┌{}┐", "─".repeat(78));
        println!(
            "│ {:<40} {:>12} {:>12} {:>9} │",
            "文件名", "原始大小", "压缩后", "压缩率"
        );
        println!("├{}┤", "─".repeat(78));

        let mut total_original_table: u64 = 0;
        let mut total_new_table: u64 = 0;

        for result in &results {
            if !result.success {
                continue;
            }
            let name = result
                .original_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let orig = format_size(result.original_size);
            let new = format_size(result.new_size);
            let rate = if result.original_size > 0 {
                format!(
                    "{:>8.1}%",
                    (1.0 - result.new_size as f64 / result.original_size as f64) * 100.0
                )
            } else {
                "0.0%".to_string()
            };
            println!(
                "│ {:<40} {:>12} {:>12} {:>9} │",
                truncate_str(&name, 40),
                orig,
                new,
                rate
            );
            total_original_table += result.original_size;
            total_new_table += result.new_size;
        }

        // 添加总计行
        let total_saved = total_original_table.saturating_sub(total_new_table);
        let total_rate = if total_original_table > 0 {
            format!(
                "{:>8.1}%",
                (total_saved as f64 / total_original_table as f64) * 100.0
            )
        } else {
            "0.0%".to_string()
        };
        println!(
            "│ {:<40} {:>12} {:>12} {:>9} │",
            "总计",
            format_size(total_original_table),
            format_size(total_new_table),
            total_rate
        );
        println!("└{}┘", "─".repeat(78));
    }

    // 打印失败文件列表
    if !failures.is_empty() {
        println!("\n⚠ 失败 ({}):", failures.len());
        for (name, err) in &failures {
            println!("  ✗ {}: {}", name, err);
        }
    }

    // 最终总结
    let total_saved = total_original.saturating_sub(total_new);
    if failures.is_empty() {
        success(&format!(
            "\n✅ 完成! {} 个文件, 节省 {}",
            success_count,
            format_size(total_saved)
        ));
    } else {
        success(&format!(
            "\n✅ 完成! {} 个文件成功, 节省 {}, {} 个文件失败",
            success_count,
            format_size(total_saved),
            failures.len()
        ));
    }

    Ok(())
}
