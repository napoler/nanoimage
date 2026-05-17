//! CLI 子命令 - convert
use crate::commands::common::load_config;
use crate::commands::output::{success, error};
use anyhow::Result;
use std::path::PathBuf;
use nanoimage_core::{Optimizer, OutputFormat};

#[derive(clap::Parser)]
pub struct Args {
    /// 输入文件
    #[arg(short, long)]
    input: PathBuf,

    /// 输出文件
    #[arg(short, long)]
    output: PathBuf,

    /// 目标格式
    #[arg(short, long, value_enum)]
    format: ConvertFormat,

    /// 质量 1-100
    #[arg(short, long, default_value = "85")]
    quality: u8,
}

#[derive(clap::ValueEnum, Clone)]
pub enum ConvertFormat {
    Jpg,
    Png,
    WebP,
    Gif,
}

impl From<ConvertFormat> for OutputFormat {
    fn from(f: ConvertFormat) -> Self {
        match f {
            ConvertFormat::Jpg => OutputFormat::Jpeg,
            ConvertFormat::Png => OutputFormat::Png,
            ConvertFormat::WebP => OutputFormat::WebP,
            ConvertFormat::Gif => OutputFormat::Gif,
        }
    }
}

pub fn execute(args: Args) -> Result<()> {
    let mut config = load_config();
    config.quality.lossy = args.quality;
    config.format = args.format.into();

    // 设置输出路径
    config.output_dir = args.output.parent().map(|p| p.to_path_buf());
    config.overwrite = true;

    let optimizer = Optimizer::new(config);
    let result = optimizer.process_file(&args.input);

    if result.success {
        // 如果输出路径不匹配，重命名文件以匹配用户指定的输出路径
        if result.output_path != args.output {
            std::fs::rename(&result.output_path, &args.output)?;
        }
        success(&format!("✓ 转换完成: {} → {}", args.input.display(), args.output.display()));
    } else {
        error(&format!("✗ 转换失败: {}", result.error.unwrap_or_default()));
    }

    Ok(())
}