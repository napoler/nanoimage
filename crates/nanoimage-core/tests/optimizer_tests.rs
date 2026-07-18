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
