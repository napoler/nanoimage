//! CLI 子命令 - config
use crate::commands::common::{load_config, save_config, config_path};
use crate::commands::output::{success, error, info};
use anyhow::Result;
use nanoimage_core::OptimizerConfig;

#[derive(clap::Parser)]
pub struct Args {
    /// 显示当前配置
    #[arg(short, long)]
    show: bool,

    /// 重置为默认配置
    #[arg(short, long)]
    reset: bool,

    /// 质量 1-100
    #[arg(short, long)]
    quality: Option<u8>,

    /// 工作线程数
    #[arg(short, long)]
    workers: Option<usize>,
}

pub fn execute(args: Args) -> Result<()> {
    if args.show {
        let config = load_config();
        println!("\n=== NanoImage 当前配置 ===");
        println!("质量: {}", config.quality.lossy);
        println!("无损质量: {}", config.quality.lossless);
        println!("工作线程: {}", config.workers);
        println!("保留元数据: {}", config.preserve_metadata);
        println!("覆盖源文件: {}", config.overwrite);

        if let Some(path) = config_path() {
            println!("\n配置文件: {}", path.display());
        }
        println!();
        return Ok(());
    }

    if args.reset {
        let config = OptimizerConfig::default();
        save_config(&config)?;
        success("配置已重置为默认值");
        return Ok(());
    }

    // 修改配置
    let mut config = load_config();

    if let Some(q) = args.quality {
        config.quality.lossy = q;
    }

    if let Some(w) = args.workers {
        config.workers = w;
    }

    save_config(&config)?;
    success("配置已保存");

    if let Some(path) = config_path() {
        info(&format!("保存至: {}", path.display()));
    }

    Ok(())
}