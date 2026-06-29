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
            ImageFormat::Svg => {
                // 尝试从 SVG 文件中提取尺寸信息
                let (width, height) = Self::extract_svg_dimensions(path);
                Some(Self {
                    path: path.to_path_buf(),
                    format,
                    width,
                    height,
                    size_bytes,
                    has_transparency: true, // SVG 始终支持透明度
                })
            }
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

    /// 从 SVG 文件中提取宽度和高度
    fn extract_svg_dimensions(path: &Path) -> (u32, u32) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return (0, 0),
        };

        // 简单提取: 查找 width="NNN" 或 width='NNN' 或 width=NNN
        let width = Self::extract_svg_attr(&content, "width");
        let height = Self::extract_svg_attr(&content, "height");

        match (width, height) {
            (Some(w), Some(h)) => (w, h),
            _ => (0, 0),
        }
    }

    /// 从 SVG 内容中提取指定属性的数值（去除单位）
    fn extract_svg_attr(svg: &str, attr: &str) -> Option<u32> {
        // 查找属性名（不区分大小写）
        let lower_svg = svg.to_lowercase();
        let attr_lower = format!("{}=", attr);
        let pos = lower_svg.find(&attr_lower)?;

        // 从属性名之后开始查找值
        let after_attr = &svg[pos + attr_lower.len()..];

        // 跳过引号
        let rest = after_attr.trim_start();
        let value_start = if rest.starts_with('"') || rest.starts_with('\'') {
            &rest[1..]
        } else {
            rest
        };

        // 提取连续的数字字符（可能带小数点）
        let num_str: String = value_start.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
        num_str.parse::<u32>().ok()
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