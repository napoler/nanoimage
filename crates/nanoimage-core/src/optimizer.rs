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
        // Validate input file exists before processing
        if !path.exists() {
            return ProcessResult {
                original_path: path.to_path_buf(),
                output_path: self.determine_output_path(path),
                original_size: 0,
                new_size: 0,
                savings: 0,
                success: false,
                error: Some(format!("输入文件不存在: {}", path.display())),
            };
        }

        let original_path = path.to_path_buf();
        let original_size = std::fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);

        // 确定输出路径
        let output_path = self.determine_output_path(path);

        // Ensure output parent directory exists before dispatching to format handlers.
        // This covers overwrite mode (output == input, parent always exists) and
        // custom output_dir modes where the directory may not exist yet.
        if let Some(parent) = output_path.parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return ProcessResult {
                        original_path,
                        output_path,
                        original_size,
                        new_size: 0,
                        savings: 0,
                        success: false,
                        error: Some(format!("无法创建输出目录: {}", e)),
                    };
                }
            }
        }

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
        let base = parent.join("optimized").join(input_path.file_name().unwrap_or_default());
        // Strip leading "./" for cleaner paths
        if let Ok(stripped) = base.strip_prefix("./") {
            PathBuf::from(stripped)
        } else {
            base
        }
    }

    /// Test helper: expose determine_output_path for testing
    pub fn determine_output_path_test(&self, input_path: &Path) -> PathBuf {
        self.determine_output_path(input_path)
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

    /// 处理 SVG — 验证内容有效性后复制
    fn process_svg(&self, input: &Path, output: &Path) -> anyhow::Result<()> {
        // 读取 SVG 内容并验证它是有效的 XML/SVG
        let svg_content = std::fs::read_to_string(input)
            .with_context(|| format!("无法读取SVG文件: {:?}", input))?;

        // 基本验证：SVG 文件应包含 <svg 标签
        let trimmed = svg_content.trim();
        let has_svg_tag = trimmed.starts_with("<svg")
            || trimmed.find(|c: char| c != '\n' && c != '\r' && c != ' ')
                .map(|pos| trimmed[pos..].starts_with("<svg"))
                .unwrap_or(false);

        if !has_svg_tag {
            return Err(anyhow::anyhow!("文件不是有效的 SVG: 缺少 <svg> 根元素"));
        }

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
    use crate::{CompressionMode, Quality, OutputFormat};
    use std::fs;

    /// 辅助函数：创建一个 100x100 的红色 RGB 测试图片并保存到指定路径
    fn create_test_image(path: &Path) {
        // 使用 RGB 格式以兼容 JPEG（JPEG 不支持 alpha 通道）
        let rgb = image::ImageBuffer::<image::Rgb<u8>, _>::from_fn(100, 100, |_, _| {
            image::Rgb([255u8, 0, 0])
        });
        rgb.save(path).unwrap();
    }

    #[test]
    fn test_image_format_detection() {
        assert_eq!(
            ImageFormat::from_path(Path::new("test.jpg")),
            ImageFormat::Jpeg
        );
        assert_eq!(
            ImageFormat::from_path(Path::new("test.PNG")),
            ImageFormat::Png
        );
        assert_eq!(
            ImageFormat::from_path(Path::new("test.webp")),
            ImageFormat::WebP
        );
    }

    #[test]
    fn test_default_config() {
        let config = OptimizerConfig::default();
        assert_eq!(config.quality.lossy, 85);
    }

    /// 测试 JPEG 压缩：创建 JPEG 测试图片，验证处理后的输出文件存在且大小合理
    #[test]
    fn test_jpeg_compression() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input_path = temp_dir.path().join("test.jpg");
        create_test_image(&input_path);

        let config = OptimizerConfig {
            overwrite: true,
            ..Default::default()
        };
        let optimizer = Optimizer::new(config);
        let result = optimizer.process_file(&input_path);

        assert!(result.success, "JPEG 处理应该成功");
        assert!(result.output_path.exists(), "输出文件应该存在");
        assert!(result.new_size > 0, "输出文件大小应该大于 0");
    }

    /// 测试 PNG 压缩：创建 PNG 测试图片，用 oxipng 处理，验证输出文件存在
    #[test]
    fn test_png_compression() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input_path = temp_dir.path().join("test.png");
        create_test_image(&input_path);

        let config = OptimizerConfig {
            overwrite: true,
            ..Default::default()
        };
        let optimizer = Optimizer::new(config);
        let result = optimizer.process_file(&input_path);

        assert!(result.success, "PNG 处理应该成功");
        assert!(result.output_path.exists(), "输出文件应该存在");
        assert!(result.new_size > 0, "输出文件大小应该大于 0");
    }

    /// 测试 WebP 转换：创建 RGBA 测试图片，处理为 WebP，验证输出文件存在且大小合理
    #[test]
    fn test_webp_conversion() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input_path = temp_dir.path().join("test.webp");
        create_test_image(&input_path);

        let original_size = fs::metadata(&input_path).unwrap().len();

        let config = OptimizerConfig {
            overwrite: true,
            ..Default::default()
        };
        let optimizer = Optimizer::new(config);
        let result = optimizer.process_file(&input_path);

        assert!(result.success, "WebP 处理应该成功");
        assert!(result.output_path.exists(), "输出文件应该存在");
        assert!(result.new_size > 0, "输出文件大小应该大于 0");
        assert!(result.new_size <= original_size, "输出文件不应该比原始文件大");
    }

    /// 测试不支持的格式：创建 .xyz 文件，验证 process_file 返回 success=false
    #[test]
    fn test_process_unsupported_format() {
        let temp_dir = tempfile::tempdir().unwrap();
        let input_path = temp_dir.path().join("test.xyz");
        fs::write(&input_path, b"not an image").unwrap();

        let optimizer = Optimizer::with_default();
        let result = optimizer.process_file(&input_path);

        assert!(!result.success, "不支持的格式应该返回失败");
        assert!(result.error.is_some(), "应该包含错误信息");
        assert!(result.error.as_ref().unwrap().contains("不支持的格式"),
            "错误信息应包含 '不支持的格式'，实际: {:?}", result.error);
    }

    /// 测试配置序列化：创建 OptimizerConfig，序列化为 JSON 字符串，反序列化验证字段一致
    #[test]
    fn test_config_serialization() {
        let config = OptimizerConfig {
            mode: CompressionMode::Lossless,
            quality: Quality { lossy: 90, lossless: 95 },
            max_width: Some(1920),
            max_height: Some(1080),
            format: OutputFormat::WebP,
            preserve_metadata: false,
            overwrite: true,
            output_dir: Some(PathBuf::from("/tmp/output")),
            skip_failed: true,
            workers: 8,
        };

        // 序列化为 JSON 字符串
        let json = serde_json::to_string(&config).expect("序列化应该成功");

        // 反序列化回配置
        let deserialized: OptimizerConfig = serde_json::from_str(&json).expect("反序列化应该成功");

        // 验证所有字段一致
        assert_eq!(deserialized.mode, config.mode);
        assert_eq!(deserialized.quality.lossy, config.quality.lossy);
        assert_eq!(deserialized.quality.lossless, config.quality.lossless);
        assert_eq!(deserialized.max_width, config.max_width);
        assert_eq!(deserialized.max_height, config.max_height);
        assert_eq!(deserialized.format, config.format);
        assert_eq!(deserialized.preserve_metadata, config.preserve_metadata);
        assert_eq!(deserialized.overwrite, config.overwrite);
        assert_eq!(deserialized.output_dir, config.output_dir);
        assert_eq!(deserialized.workers, config.workers);
    }
}