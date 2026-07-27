//! Tests for optimizer module
use nanoimage_core::{ImageFormat, OptimizerConfig};
use std::path::Path;

#[test]
fn test_image_format_detection_jpeg() {
    assert_eq!(
        ImageFormat::from_path(Path::new("test.jpg")),
        ImageFormat::Jpeg
    );
    assert_eq!(
        ImageFormat::from_path(Path::new("test.jpeg")),
        ImageFormat::Jpeg
    );
}

#[test]
fn test_image_format_detection_png() {
    assert_eq!(
        ImageFormat::from_path(Path::new("test.png")),
        ImageFormat::Png
    );
    assert_eq!(
        ImageFormat::from_path(Path::new("test.PNG")),
        ImageFormat::Png
    );
}

#[test]
fn test_image_format_detection_webp() {
    assert_eq!(
        ImageFormat::from_path(Path::new("test.webp")),
        ImageFormat::WebP
    );
}

#[test]
fn test_image_format_detection_gif() {
    assert_eq!(
        ImageFormat::from_path(Path::new("test.gif")),
        ImageFormat::Gif
    );
}

#[test]
fn test_image_format_detection_bmp() {
    assert_eq!(
        ImageFormat::from_path(Path::new("test.bmp")),
        ImageFormat::Bmp
    );
}

#[test]
fn test_image_format_detection_svg() {
    assert_eq!(
        ImageFormat::from_path(Path::new("test.svg")),
        ImageFormat::Svg
    );
}

#[test]
fn test_image_format_detection_unknown() {
    assert_eq!(
        ImageFormat::from_path(Path::new("test.txt")),
        ImageFormat::Unknown
    );
}

#[test]
fn test_default_optimizer_config() {
    let config = OptimizerConfig::default();
    assert_eq!(config.quality.lossy, 85);
    assert_eq!(config.quality.lossless, 100);
    assert_eq!(format!("{:?}", config.mode), "Lossy");
}

#[test]
fn test_optimizer_config_lossless_mode() {
    use nanoimage_core::CompressionMode;
    let config = nanoimage_core::OptimizerConfig {
        mode: CompressionMode::Lossless,
        ..Default::default()
    };
    assert_eq!(config.effective_quality(), 100);
}

#[test]
fn test_optimizer_config_lossy_mode() {
    use nanoimage_core::CompressionMode;
    let config = nanoimage_core::OptimizerConfig {
        mode: CompressionMode::Lossy,
        ..Default::default()
    };
    assert_eq!(config.effective_quality(), 85);
}

#[test]
fn test_optimizer_config_custom_quality() {
    use nanoimage_core::Quality;
    let config = nanoimage_core::OptimizerConfig {
        quality: Quality {
            lossy: 95,
            lossless: 90,
        },
        ..Default::default()
    };
    assert_eq!(config.quality.lossy, 95);
    assert_eq!(config.quality.lossless, 90);
}

#[test]
fn test_optimizer_config_max_dimensions() {
    let config = nanoimage_core::OptimizerConfig {
        max_width: Some(1920),
        max_height: Some(1080),
        ..Default::default()
    };
    assert_eq!(config.max_width, Some(1920));
    assert_eq!(config.max_height, Some(1080));
}

#[test]
fn test_optimizer_config_output_format() {
    use nanoimage_core::OutputFormat;
    let config = nanoimage_core::OptimizerConfig {
        format: OutputFormat::WebP,
        ..Default::default()
    };
    assert_eq!(config.format.as_str(), "webp");
}

#[test]
fn test_optimizer_config_overwrite() {
    let config = nanoimage_core::OptimizerConfig {
        overwrite: true,
        ..Default::default()
    };
    assert!(config.overwrite);
}

#[test]
fn test_optimizer_config_workers() {
    let config = OptimizerConfig::default();
    assert!(config.workers > 0);
    assert!(config.workers <= 16);
}

/// 测试 BMP 文件经过 Optimizer 应能被成功处理。
/// BMP 格式通过 process_bmp() 方法支持，将 BMP 转换为 PNG 进行优化。
#[test]
fn test_bmp_compress() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bmp_path = temp_dir.path().join("test.bmp");

    // 通过 image crate 写出真实的 100x100 RGB BMP 文件
    let rgb = image::ImageBuffer::<image::Rgb<u8>, _>::from_fn(100, 100, |_, _| {
        image::Rgb([255u8, 0, 0])
    });
    rgb.save(&bmp_path).expect("应能写出真实 BMP 文件");

    let config = OptimizerConfig {
        overwrite: true,
        ..Default::default()
    };
    let optimizer = nanoimage_core::Optimizer::new(config);
    let result = optimizer.process_file(&bmp_path);

    assert!(
        result.success,
        "BMP 处理应成功（process_bmp 已实现）"
    );
    assert!(result.output_path.exists(), "BMP 输出文件应存在");
    assert!(result.new_size > 0, "BMP 输出文件大小应大于 0");
}

/// 测试损坏的 BMP 文件应被报告为失败。
///
/// Note: BMP format is now fully supported via process_bmp(). This test ensures
/// invalid/non-BMP files are properly rejected by image::open().
#[test]
fn test_bmp_corrupt_fails() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("corrupt.bmp");
    std::fs::write(&path, b"not a bmp").expect("应能写出损坏字节");

    let optimizer = nanoimage_core::Optimizer::with_default();
    let result = optimizer.process_file(&path);

    assert!(
        !result.success,
        "损坏的 BMP 文件应返回失败，actual error: {:?}",
        result.error
    );
    assert!(result.error.is_some(), "失败结果应包含错误信息");
}
