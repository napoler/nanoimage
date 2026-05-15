//! CLI 子命令 - compress
use crate::commands::common::load_config;
use crate::commands::output::{success, error};
use anyhow::Result;
use std::path::PathBuf;
use nanoimage_core::Optimizer;

#[derive(clap::Parser)]
pub struct Args {
    /// 输入文件
    #[arg(short, long)]
    input: PathBuf,

    /// 输出文件/目录
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// 质量 1-100
    #[arg(short, long, default_value = "85")]
    quality: u8,

    /// 覆盖源文件
    #[arg(short = 'w', long = "overwrite")]
    overwrite: bool,
}

pub fn execute(args: Args) -> Result<()> {
    let mut config = load_config();
    config.quality.lossy = args.quality;
    config.overwrite = args.overwrite;

    if let Some(output) = args.output {
        if output.is_dir() {
            config.output_dir = Some(output);
        } else {
            config.output_dir = output.parent().map(|p| p.to_path_buf());
            config.overwrite = true;
        }
    }

    let optimizer = Optimizer::new(config);
    let result = optimizer.process_file(&args.input);

    if result.success {
        success(&format!(
            "✓ 压缩完成: {} ({} → {}, -{:.*}%)",
            result.original_path.display(),
            format_size(result.original_size),
            format_size(result.new_size),
            1,
            result.savings_percent()
        ));
    } else {
        error(&format!("✗ 处理失败: {} - {}", args.input.display(), result.error.unwrap_or_default()));
    }

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