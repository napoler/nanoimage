//! 图像优化器核心
use std::path::{Path, PathBuf};
use anyhow::Context;
use crate::config::OptimizerConfig;
use crate::ImageFormat;

/// 处理结果
#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub original_path: PathBuf,
    pub output_path: PathBuf,
    pub original_size: u64,
    pub new_size: u64,
    pub savings: i64,
    pub success: bool,
    pub error: Option<String>,
}

impl ProcessResult {
    pub fn savings_percent(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            (self.savings as f64 / self.original_size as f64) * 100.0
        }
    }
}

/// 图像优化器
pub struct Optimizer {
    config: OptimizerConfig,
}

impl Optimizer {
    pub fn new(config: OptimizerConfig) -> Self {
        Self { config }
    }

    pub fn with_default() -> Self {
        Self::new(OptimizerConfig::default())
    }

    /// 处理单个文件
    pub fn process_file(&self, path: &Path) -> ProcessResult {
        let original_path = path.to_path_buf();
        let original_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);

        // 确定输出路径
        let output_path = self.determine_output_path(path);

        // 根据格式选择处理方式
        let format = ImageFormat::from_path(path);

        let result = match format {
            ImageFormat::Jpeg => self.process_jpeg(path, &output_path),
            ImageFormat::Png => self.process_png(path, &output_path),
            ImageFormat::WebP => self.process_webp(path, &output_path),
            ImageFormat::Gif => self.process_gif(path, &output_path),
            ImageFormat::Svg => self.process_svg(path, &output_path),
            _ => Err(anyhow::anyhow!("不支持的格式: {:?}", format)),
        };

        match result {
            Ok(_) => {
                let new_size = std::fs::metadata(&output_path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                let savings = original_size as i64 - new_size as i64;

                ProcessResult {
                    original_path,
                    output_path,
                    original_size,
                    new_size,
                    savings,
                    success: true,
                    error: None,
                }
            }
            Err(e) => ProcessResult {
                original_path,
                output_path,
                original_size,
                new_size: 0,
                savings: 0,
                success: false,
                error: Some(e.to_string()),
            },
        }
    }

    /// 确定输出路径
    fn determine_output_path(&self, input_path: &Path) -> PathBuf {
        if self.config.overwrite {
            return input_path.to_path_buf();
        }

        if let Some(ref output_dir) = self.config.output_dir {
            let filename = input_path.file_name().unwrap_or_default();
            return output_dir.join(filename);
        }

        // 默认: 同目录下的 optimized 子文件夹
        let parent = input_path.parent().unwrap_or(Path::new("."));
        parent.join("optimized").join(input_path.file_name().unwrap_or_default())
    }

    /// 处理 JPEG
    fn process_jpeg(&self, input: &Path, output: &Path) -> anyhow::Result<()> {
        let img = image::open(input)
            .with_context(|| format!("无法加载图像: {:?}", input))?;

        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let quality = self.config.effective_quality();

        let file = std::fs::File::create(output)?;
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, quality);
        img.write_with_encoder(encoder)?;

        Ok(())
    }

    /// 处理 PNG
    fn process_png(&self, input: &Path, output: &Path) -> anyhow::Result<()> {
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 读取原始 PNG 数据
        let png_data = std::fs::read(input)
            .with_context(|| format!("无法读取PNG: {:?}", input))?;

        // oxipng 优化
        let opts = oxipng::Options::default();
        let optimized = oxipng::optimize_from_memory(&png_data, &opts)
            .with_context(|| "oxipng 优化失败")?;

        std::fs::write(output, optimized)?;

        Ok(())
    }

    /// 处理 WebP
    fn process_webp(&self, input: &Path, output: &Path) -> anyhow::Result<()> {
        let img = image::open(input)
            .with_context(|| format!("无法加载图像: {:?}", input))?;

        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let quality = self.config.effective_quality();
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let encoder = webp::Encoder::from_rgba(&rgba, width, height);
        let webp_data = encoder.encode(quality as f32);

        // WebPMemory implements Deref<Target = [u8]>
        std::fs::write(output, &*webp_data)?;

        Ok(())
    }

    /// 处理 GIF
    fn process_gif(&self, input: &Path, output: &Path) -> anyhow::Result<()> {
        let img = image::open(input)
            .with_context(|| format!("无法加载图像: {:?}", input))?;

        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }

        img.save(output)?;

        Ok(())
    }

    /// 处理 SVG
    fn process_svg(&self, input: &Path, output: &Path) -> anyhow::Result<()> {
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(input, output)?;
        Ok(())
    }

    pub fn config(&self) -> &OptimizerConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: OptimizerConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_detection() {
        assert_eq!(ImageFormat::from_path(Path::new("test.jpg")), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from_path(Path::new("test.PNG")), ImageFormat::Png);
        assert_eq!(ImageFormat::from_path(Path::new("test.webp")), ImageFormat::Webp);
    }

    #[test]
    fn test_default_config() {
        let config = OptimizerConfig::default();
        assert_eq!(config.quality.lossy, 85);
    }
}