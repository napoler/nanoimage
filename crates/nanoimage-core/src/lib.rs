//! NanoImage Core - 图像优化引擎
//!
//! 提供图像压缩、格式转换、批量处理的核心功能

pub mod config;
pub mod optimizer;
pub mod formats;
pub mod processor;

pub use config::{OptimizerConfig, OutputFormat, Quality, CompressionMode};
pub use optimizer::{Optimizer, ProcessResult};
pub use processor::{BatchProcessor, Progress};

/// 图像格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
    WebP,
    Gif,
    Bmp,
    Svg,
    Unknown,
}

impl ImageFormat {
    pub fn from_path(path: &std::path::Path) -> Self {
        match path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()).as_deref() {
            Some("jpg") | Some("jpeg") => ImageFormat::Jpeg,
            Some("png") => ImageFormat::Png,
            Some("webp") => ImageFormat::WebP,
            Some("gif") => ImageFormat::Gif,
            Some("bmp") => ImageFormat::Bmp,
            Some("svg") => ImageFormat::Svg,
            _ => ImageFormat::Unknown,
        }
    }
}

/// 文件处理状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileStatus {
    Pending,
    Processing,
    Completed,
    Skipped,
    Error(String),
}