//! Tests for formats module
use nanoimage_core::formats::{format_name, format_size, ImageInfo};
use nanoimage_core::ImageFormat;

/// Test format_size for bytes
#[test]
fn test_format_size_bytes() {
    assert_eq!(format_size(0), "0 B");
    assert_eq!(format_size(1), "1 B");
    assert_eq!(format_size(512), "512 B");
    assert_eq!(format_size(1023), "1023 B");
}

/// Test format_size for kilobytes
#[test]
fn test_format_size_kb() {
    assert_eq!(format_size(1024), "1.00 KB");
    assert_eq!(format_size(2048), "2.00 KB");
    assert_eq!(format_size(1536), "1.50 KB");
}

/// Test format_size for megabytes
#[test]
fn test_format_size_mb() {
    assert_eq!(format_size(1024 * 1024), "1.00 MB");
    assert_eq!(format_size(5 * 1024 * 1024), "5.00 MB");
    assert_eq!(format_size(1536 * 1024), "1.50 MB");
}

/// Test format_size for gigabytes
#[test]
fn test_format_size_gb() {
    assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    assert_eq!(format_size(2 * 1024 * 1024 * 1024), "2.00 GB");
}

/// Test format_size boundary values
#[test]
fn test_format_size_boundaries() {
    // Just below KB threshold
    assert_eq!(format_size(1023), "1023 B");
    // Exactly KB
    assert_eq!(format_size(1024), "1.00 KB");
    // Just below MB threshold — 1048575/1024 = 1023.999 → rounds to 1024.00
    assert_eq!(format_size(1024 * 1024 - 1), "1024.00 KB");
    // Exactly MB
    assert_eq!(format_size(1024 * 1024), "1.00 MB");
}

/// Test format_name returns correct names for all variants
#[test]
fn test_format_name_all_variants() {
    assert_eq!(format_name(ImageFormat::Jpeg), "JPEG");
    assert_eq!(format_name(ImageFormat::Png), "PNG");
    assert_eq!(format_name(ImageFormat::WebP), "WebP");
    assert_eq!(format_name(ImageFormat::Gif), "GIF");
    assert_eq!(format_name(ImageFormat::Bmp), "BMP");
    assert_eq!(format_name(ImageFormat::Svg), "SVG");
    assert_eq!(format_name(ImageFormat::Unknown), "Unknown");
}

/// Test ImageInfo::from_path returns None for non-existent file
#[test]
fn test_image_info_nonexistent_file() {
    let info = ImageInfo::from_path(std::path::Path::new("/nonexistent/file.jpg"));
    assert!(info.is_none());
}

/// Test ImageInfo::from_path returns None for non-image file
#[test]
fn test_image_info_non_image_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.txt");
    std::fs::write(&path, "not an image").unwrap();
    let info = ImageInfo::from_path(&path);
    assert!(info.is_none());
}
