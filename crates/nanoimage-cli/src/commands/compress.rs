//! CLI 子命令 - compress
use crate::commands::common::load_config;
use crate::commands::output::{error, success};
use anyhow::Result;
use nanoimage_core::{format_size, Optimizer};
use std::path::PathBuf;

/// compress 子命令的参数
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

/// 执行压缩命令
///
/// 验证输入文件存在性，加载配置，调用 Optimizer 处理单个文件
pub fn execute(args: Args) -> Result<()> {
    if !args.input.exists() {
        anyhow::bail!("输入文件不存在: {}", args.input.display());
    }

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
        error(&format!(
            "✗ 处理失败: {} - {}",
            args.input.display(),
            result.error.unwrap_or_default()
        ));
    }

    Ok(())
}
