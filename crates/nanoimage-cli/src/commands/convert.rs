//! CLI 子命令 - convert
use crate::commands::common::load_config;
use crate::commands::output::{error, success};
use anyhow::Result;
use nanoimage_core::{Optimizer, OutputFormat};
use std::path::PathBuf;

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
    #[value(alias = "webp")]
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
    // Validate input file exists
    if !args.input.exists() {
        return Err(anyhow::anyhow!("输入文件不存在: {}", args.input.display()));
    }
    // Validate input is a file (not directory)
    if !args.input.is_file() {
        return Err(anyhow::anyhow!("输入必须是文件，而不是目录: {}", args.input.display()));
    }

    let mut config = load_config();
    // Normalize quality values to valid ranges
    config.quality = config.quality.normalize();
    // Set both lossy and lossless for consistent behavior regardless of compression mode
    config.quality.lossy = args.quality.clamp(1, 100);
    config.quality.lossless = args.quality.clamp(1, 100);
    // Clamp workers to valid range (1-16) to prevent resource exhaustion
    config.workers = config.workers.clamp(1, 16);

    // Determine expected extension based on target format
    let expected_ext = match &args.format {
        ConvertFormat::Jpg => "jpg",
        ConvertFormat::Png => "png",
        ConvertFormat::WebP => "webp",
        ConvertFormat::Gif => "gif",
    };

    // Validate output file extension matches target format
    if let Some(output_ext) = args.output.extension() {
        let output_ext_str = output_ext.to_str().unwrap_or("");
        if output_ext_str.to_lowercase() != expected_ext.to_lowercase() {
            tracing::warn!(
                "输出文件扩展名 '{}' 与目标格式 '{}' 不匹配",
                output_ext_str, expected_ext
            );
            // Continue anyway since user explicitly specified the path
        }
    }

    config.format = args.format.clone().into();

    // 设置输出路径
    config.output_dir = args.output.parent().map(|p| p.to_path_buf());
    config.overwrite = true;

    let optimizer = Optimizer::new(config);
    let result = optimizer.process_file(&args.input);

    if result.success {
        // 如果输出路径不匹配，移动文件到用户指定的输出路径
        // 优先使用 rename（原子操作），失败时回退到 copy+remove
        if result.output_path != args.output {
            if let Err(e) = std::fs::rename(&result.output_path, &args.output) {
                // 跨文件系统情况：rename 失败，回退到 copy+remove
                tracing::warn!("rename 失败 (可能跨文件系统)，使用 copy+remove: {}", e);
                std::fs::copy(&result.output_path, &args.output)?;
                std::fs::remove_file(&result.output_path)
                    .map_err(|e| anyhow::anyhow!("清理临时文件失败: {}", e))?;
            }
        }
        success(&format!(
            "✓ 转换完成: {} → {}",
            args.input.display(),
            args.output.display()
        ));
    } else {
        match &result.error {
            Some(e) => error(&format!("✗ 转换失败: {}", e)),
            None => error("✗ 转换失败: 未知错误"),
        }
    }

    Ok(())
}
