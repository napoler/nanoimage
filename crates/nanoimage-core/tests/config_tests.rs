//! Tests for config module
use std::path::PathBuf;
use nanoimage_core::{OptimizerConfig, CompressionMode, OutputFormat, Quality};

/// Test that default OptimizerConfig has correct values
#[test]
fn test_default_config_values() {
    let config = OptimizerConfig::default();
    assert_eq!(config.quality.lossy, 85);
    assert_eq!(config.quality.lossless, 100);
    assert_eq!(config.mode, CompressionMode::Lossy);
    assert_eq!(config.format, OutputFormat::KeepOriginal);
    assert!(config.preserve_metadata);
    assert!(!config.overwrite);
    assert!(config.workers > 0);
    assert!(config.workers <= 16);
    assert!(config.max_width.is_none());
    assert!(config.max_height.is_none());
    assert!(config.output_dir.is_none());
}

/// Test effective_quality() for each CompressionMode
#[test]
fn test_effective_quality_lossy() {
    let config = OptimizerConfig::default();
    assert_eq!(config.effective_quality(), 85);
}

#[test]
fn test_effective_quality_lossless() {
    let config = nanoimage_core::OptimizerConfig { mode: CompressionMode::Lossless, ..Default::default() };
    assert_eq!(config.effective_quality(), 100);
}

#[test]
fn test_effective_quality_smart() {
    let config = nanoimage_core::OptimizerConfig { mode: CompressionMode::Smart, ..Default::default() };
    // Smart mode uses lossy quality
    assert_eq!(config.effective_quality(), 85);
}

#[test]
fn test_effective_quality_smart_custom_lossy() {
    let mut config = nanoimage_core::OptimizerConfig { mode: CompressionMode::Smart, ..Default::default() };
    config.quality.lossy = 90;
    assert_eq!(config.effective_quality(), 90);
}

/// Test OutputFormat::as_str() returns correct strings for all variants
#[test]
fn test_output_format_as_str_keep_original() {
    assert_eq!(OutputFormat::KeepOriginal.as_str(), "keep");
}

#[test]
fn test_output_format_as_str_jpeg() {
    assert_eq!(OutputFormat::Jpeg.as_str(), "jpg");
}

#[test]
fn test_output_format_as_str_png() {
    assert_eq!(OutputFormat::Png.as_str(), "png");
}

#[test]
fn test_output_format_as_str_webp() {
    assert_eq!(OutputFormat::WebP.as_str(), "webp");
}

#[test]
fn test_output_format_as_str_gif() {
    assert_eq!(OutputFormat::Gif.as_str(), "gif");
}

/// Test CompressionMode serde roundtrip
#[test]
fn test_compression_mode_serde_roundtrip_lossy() {
    let mode = CompressionMode::Lossy;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, "\"lossy\"");
    let deserialized: CompressionMode = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, CompressionMode::Lossy);
}

#[test]
fn test_compression_mode_serde_roundtrip_lossless() {
    let mode = CompressionMode::Lossless;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, "\"lossless\"");
    let deserialized: CompressionMode = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, CompressionMode::Lossless);
}

#[test]
fn test_compression_mode_serde_roundtrip_smart() {
    let mode = CompressionMode::Smart;
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, "\"smart\"");
    let deserialized: CompressionMode = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, CompressionMode::Smart);
}

/// Test Quality serde roundtrip
#[test]
fn test_quality_serde_roundtrip() {
    let quality = Quality { lossy: 90, lossless: 95 };
    let json = serde_json::to_string(&quality).unwrap();
    let deserialized: Quality = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.lossy, 90);
    assert_eq!(deserialized.lossless, 95);
}

/// Test OptimizerConfig save_to_file and load_from_file using tempfile
#[test]
fn test_config_save_and_load_roundtrip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let config = OptimizerConfig {
        mode: CompressionMode::Lossless,
        quality: Quality { lossy: 90, lossless: 95 },
        max_width: Some(1920),
        max_height: Some(1080),
        format: OutputFormat::WebP,
        preserve_metadata: false,
        overwrite: true,
        output_dir: Some(PathBuf::from("/tmp/output")),
        workers: 8,
    };

    // Save
    config.save_to_file(&config_path).unwrap();

    // Verify file exists and has content
    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("lossless"));

    // Load
    let loaded = OptimizerConfig::load_from_file(&config_path).unwrap();
    assert_eq!(loaded.mode, CompressionMode::Lossless);
    assert_eq!(loaded.quality.lossy, 90);
    assert_eq!(loaded.quality.lossless, 95);
    assert_eq!(loaded.max_width, Some(1920));
    assert_eq!(loaded.max_height, Some(1080));
    assert_eq!(loaded.format, OutputFormat::WebP);
    assert!(!loaded.preserve_metadata);
    assert!(loaded.overwrite);
    assert_eq!(loaded.output_dir, Some(PathBuf::from("/tmp/output")));
    assert_eq!(loaded.workers, 8);
}

/// Test loading a minimal config JSON (only mode field)
#[test]
fn test_load_minimal_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("minimal.json");

    let minimal = r#"{"mode": "lossless"}"#;
    std::fs::write(&config_path, minimal).unwrap();

    let config = OptimizerConfig::load_from_file(&config_path).unwrap();
    assert_eq!(config.mode, CompressionMode::Lossless);
    // All other fields should use defaults
    assert_eq!(config.quality.lossy, 85);
    assert_eq!(config.quality.lossless, 100);
    assert!(config.max_width.is_none());
    assert!(config.preserve_metadata);
}

/// Test loading an empty JSON object uses all defaults
#[test]
fn test_load_empty_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("empty.json");

    std::fs::write(&config_path, "{}").unwrap();

    let config = OptimizerConfig::load_from_file(&config_path).unwrap();
    assert_eq!(config.mode, CompressionMode::Lossy); // default
    assert_eq!(config.quality.lossy, 85);
    assert!(config.preserve_metadata); // default is true
}

/// Test that load_from_file returns error for non-existent file
#[test]
fn test_load_nonexistent_file() {
    let result = OptimizerConfig::load_from_file(PathBuf::from("/nonexistent/path/config.json").as_path());
    assert!(result.is_err());
}

/// Test OutputFormat serde roundtrip
#[test]
fn test_output_format_serde_roundtrip() {
    let format = OutputFormat::WebP;
    let json = serde_json::to_string(&format).unwrap();
    assert_eq!(json, "\"webp\"");
    let deserialized: OutputFormat = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, OutputFormat::WebP);
}

#[test]
fn test_output_format_serde_keep_original() {
    let format = OutputFormat::KeepOriginal;
    let json = serde_json::to_string(&format).unwrap();
    assert_eq!(json, "\"keeporiginal\"");
    let deserialized: OutputFormat = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, OutputFormat::KeepOriginal);
}
