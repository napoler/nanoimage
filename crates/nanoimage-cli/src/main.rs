//! NanoImage CLI - 命令行图像优化工具
use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;

use commands::{compress, batch, convert, config_cmd};

/// NanoImage - 高性能图像优化工具
#[derive(Parser)]
#[command(name = "nanoimage")]
#[command(version = "0.1.0")]
#[command(about = "高性能图像优化工具，支持 JPEG/PNG/WebP 等格式")]
struct Cli {
    /// 启用详细日志
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 压缩单个文件
    Compress(compress::Args),
    /// 批量处理目录
    Batch(batch::Args),
    /// 格式转换
    Convert(convert::Args),
    /// 配置管理
    Settings(config_cmd::Args),
}

fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("nanoimage=info".parse()?))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Compress(args) => compress::execute(args),
        Commands::Batch(args) => batch::execute(args),
        Commands::Convert(args) => convert::execute(args),
        Commands::Settings(args) => config_cmd::execute(args),
    }
}