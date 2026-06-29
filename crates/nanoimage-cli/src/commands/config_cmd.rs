//! CLI 子命令 - config
use crate::commands::common::{load_config, save_config, config_path};
use crate::commands::output::{success, info, error};
use anyhow::Result;
use nanoimage_core::OptimizerConfig;

/// config 子命令的参数
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

/// 执行 config 子命令
///
/// 支持 show（显示配置）、reset（重置默认配置）和修改配置项
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
        if let Err(e) = save_config(&config) {
            error(&format!("保存配置失败: {}", e));
            anyhow::bail!("无法保存配置到默认目录: {}", e);
        }
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

    if let Err(e) = save_config(&config) {
        error(&format!("保存配置失败: {}", e));
        // 提供诊断信息
        if let Some(path) = config_path() {
            error(&format!("尝试写入的位置: {}", path.display()));
            if !path.exists() {
                if let Some(parent) = path.parent() {
                    error(&format!("配置目录不存在: {}", parent.display()));
                    error("请手动创建目录或使用 --reset 重新初始化配置");
                }
            }
        }
        anyhow::bail!("无法保存配置: {}", e);
    }
    success("配置已保存");

    if let Some(path) = config_path() {
        info(&format!("保存至: {}", path.display()));
    }

    Ok(())
}