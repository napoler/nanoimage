//! CLI 子命令 - batch
use crate::commands::common::load_config;
use crate::commands::output::{success, error, dot, file_error};
use anyhow::Result;
use std::path::PathBuf;
use nanoimage_core::BatchProcessor;

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
    #[arg(short, long)]
    overwrite: bool,
}

pub fn execute(args: Args) -> Result<()> {
    let mut config = load_config();
    config.quality.lossy = args.quality;
    config.workers = args.workers;
    config.overwrite = args.overwrite;

    if let Some(output) = args.output {
        config.output_dir = Some(output);
    }

    let processor = BatchProcessor::with_config(config);

    success(&format!("🔍 扫描目录: {}", args.input.display()));
    let files = BatchProcessor::collect_images(&args.input, args.recursive);
    success(&format!("📁 找到 {} 个图像文件", files.len()));

    if files.is_empty() {
        error("没有找到图像文件");
        return Ok(());
    }

    success(&format!("⚡ 开始处理 ({} 线程)...", args.workers));
    let results = processor.process_sync(&files);

    let mut success_count = 0;
    let mut total_saved = 0u64;

    for result in results {
        if result.success {
            success_count += 1;
            total_saved += result.savings.max(0) as u64;
            dot();
        } else {
            println!();
            file_error(&result.original_path.display().to_string(), &result.error.unwrap_or_default());
        }
    }

    println!();
    success(&format!("\n✅ 完成! {} 个文件, 节省 {}", success_count, format_size(total_saved)));

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}