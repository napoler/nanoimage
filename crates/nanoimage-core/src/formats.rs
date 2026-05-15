//! 格式检测与转换
use std::path::{Path, PathBuf};
use image::GenericImageView;
use crate::ImageFormat;

/// 图像信息
#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub path: PathBuf,
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub size_bytes: u64,
    pub has_transparency: bool,
}

impl ImageInfo {
    /// 从文件获取图像信息
    pub fn from_path(path: &Path) -> Option<Self> {
        let size_bytes = std::fs::metadata(path).ok()?.len();
        let format = ImageFormat::from_path(path);

        match format {
            ImageFormat::Svg => Some(Self {
                path: path.to_path_buf(),
                format,
                width: 0,
                height: 0,
                size_bytes,
                has_transparency: false,
            }),
            _ => {
                let img = image::open(path).ok()?;
                let (width, height) = img.dimensions();
                // 检查是否有透明通道
                let has_transparency = matches!(img, image::DynamicImage::ImageRgba8(_));

                Some(Self {
                    path: path.to_path_buf(),
                    format,
                    width,
                    height,
                    size_bytes,
                    has_transparency,
                })
            }
        }
    }

    /// 人类可读的大小字符串
    pub fn size_string(&self) -> String {
        format_size(self.size_bytes)
    }

    /// 人类可读的尺寸字符串
    pub fn dimensions_string(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}

/// 格式化文件大小
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// 格式名称映射
pub fn format_name(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Jpeg => "JPEG",
        ImageFormat::Png => "PNG",
        ImageFormat::WebP => "WebP",
        ImageFormat::Gif => "GIF",
        ImageFormat::Bmp => "BMP",
        ImageFormat::Svg => "SVG",
        ImageFormat::Unknown => "Unknown",
    }
}