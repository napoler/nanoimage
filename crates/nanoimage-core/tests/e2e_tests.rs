//! 端到端集成测试 - 验证 CLI 完整工作流程

use std::fs;
use std::path::PathBuf;

/// 测试 JPEG 端到端：创建图片 → 压缩 → 验证输出
#[test]
fn test_e2e_jpeg_compress() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input = temp_dir.path().join("test.jpg");
    let output_dir = temp_dir.path().join("output");
    let expected_output = output_dir.join("test.jpg");

    // 创建一个大的、复杂的 RGB 图片（随机噪声最难压缩）
    let rgb = image::ImageBuffer::<image::Rgb<u8>, _>::from_fn(800, 800, |x, y| {
        image::Rgb([
            ((x * 7 + y * 13) % 256) as u8,
            ((x * 11 + y * 3) % 256) as u8,
            ((x * 5 + y * 17) % 256) as u8,
        ])
    });
    rgb.save(&input).unwrap();

    // 使用 optimizer 处理（默认质量 85）
    let config = nanoimage_core::OptimizerConfig {
        overwrite: false,
        output_dir: Some(output_dir),
        ..Default::default()
    };
    let optimizer = nanoimage_core::Optimizer::new(config);
    let result = optimizer.process_file(&input);

    assert!(result.success, "JPEG 压缩应该成功");
    assert!(
        expected_output.exists(),
        "输出文件应该存在: {:?}",
        result.output_path
    );
    assert!(result.new_size > 0, "输出大小应该大于 0");
    // 验证输出路径
    assert_eq!(result.output_path, expected_output);
    // 验证文件确实被处理了（有实际内容）
    assert!(result.new_size > 1000, "JPEG 输出应该有一定大小");
}

/// 测试 PNG 端到端：创建图片 → oxipng 优化 → 验证显著压缩
#[test]
fn test_e2e_png_compress() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input = temp_dir.path().join("test.png");

    // 创建 RGBA 测试图片
    let rgba = image::ImageBuffer::<image::Rgba<u8>, _>::from_fn(100, 100, |_x, _y| {
        image::Rgba([255u8, 128, 64, 255])
    });
    rgba.save(&input).unwrap();

    let original_size = fs::metadata(&input).unwrap().len();

    let config = nanoimage_core::OptimizerConfig {
        overwrite: true,
        ..Default::default()
    };
    let optimizer = nanoimage_core::Optimizer::new(config);
    let result = optimizer.process_file(&input);

    assert!(result.success, "PNG 压缩应该成功");
    assert!(result.new_size > 0, "输出大小应该大于 0");
    // oxipng 通常能显著压缩纯色/渐变图片
    assert!(result.new_size <= original_size, "PNG 不应变大");
}

/// 测试 WebP 端到端：RGBA 图片 → WebP 转换
#[test]
fn test_e2e_webp_convert() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input = temp_dir.path().join("test.png");
    let output_dir = temp_dir.path().join("output");
    // 注意：当前实现中 determine_output_path 不改变扩展名
    // 所以输出路径是 output_dir/test.png（内容由 WebP 编码器生成）
    let expected_output = output_dir.join("test.png");

    let rgba = image::ImageBuffer::<image::Rgba<u8>, _>::from_fn(100, 100, |_, _| {
        image::Rgba([100u8, 150, 200, 255])
    });
    rgba.save(&input).unwrap();

    let config = nanoimage_core::OptimizerConfig {
        overwrite: false,
        output_dir: Some(output_dir),
        ..Default::default()
    };
    let optimizer = nanoimage_core::Optimizer::new(config);
    let result = optimizer.process_file(&input);

    assert!(result.success, "WebP 转换应该成功");
    assert!(
        expected_output.exists(),
        "WebP 输出文件应该存在: {:?}",
        result.output_path
    );
    assert!(result.new_size > 0, "WebP 输出大小应该大于 0");
}

/// 测试批量处理：多文件 → 验证所有结果
#[test]
fn test_e2e_batch_processing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_dir = temp_dir.path().join("output");

    // 创建 5 个测试图片
    let mut files = Vec::new();
    for i in 0..5 {
        let path = temp_dir.path().join(format!("test{}.jpg", i));
        let rgb = image::ImageBuffer::<image::Rgb<u8>, _>::from_fn(50, 50, |x, y| {
            image::Rgb([((i * 50 + x as usize) % 256) as u8, y as u8, 128])
        });
        rgb.save(&path).unwrap();
        files.push(path);
    }

    let config = nanoimage_core::OptimizerConfig {
        overwrite: false,
        output_dir: Some(output_dir.clone()),
        ..Default::default()
    };
    let processor = nanoimage_core::BatchProcessor::with_config(config);
    let results = processor.process_sync(&files);

    assert_eq!(results.len(), 5, "应该处理所有 5 个文件");
    for result in &results {
        assert!(result.success, "每个文件都应该处理成功");
        assert!(result.new_size > 0);
    }
}

/// 测试跳过失败文件
#[test]
fn test_e2e_skip_failed() {
    let temp_dir = tempfile::tempdir().unwrap();

    // 创建有效的 JPEG
    let valid = temp_dir.path().join("valid.jpg");
    let rgb =
        image::ImageBuffer::<image::Rgb<u8>, _>::from_fn(50, 50, |_, _| image::Rgb([255u8, 0, 0]));
    rgb.save(&valid).unwrap();

    // 创建无效的"图片"
    let invalid = temp_dir.path().join("invalid.jpg");
    fs::write(&invalid, b"not a real image").unwrap();

    let config = nanoimage_core::OptimizerConfig {
        overwrite: true,
        skip_failed: true,
        ..Default::default()
    };
    let optimizer = nanoimage_core::Optimizer::new(config);

    let valid_result = optimizer.process_file(&valid);
    let invalid_result = optimizer.process_file(&invalid);

    assert!(valid_result.success, "有效文件应该成功");
    assert!(!invalid_result.success, "无效文件应该失败");
}

/// 测试格式检测正确性
#[test]
fn test_e2e_format_detection() {
    assert_eq!(
        nanoimage_core::ImageFormat::from_path(PathBuf::from("photo.jpg").as_path()),
        nanoimage_core::ImageFormat::Jpeg
    );
    assert_eq!(
        nanoimage_core::ImageFormat::from_path(PathBuf::from("photo.jpeg").as_path()),
        nanoimage_core::ImageFormat::Jpeg
    );
    assert_eq!(
        nanoimage_core::ImageFormat::from_path(PathBuf::from("photo.png").as_path()),
        nanoimage_core::ImageFormat::Png
    );
    assert_eq!(
        nanoimage_core::ImageFormat::from_path(PathBuf::from("photo.webp").as_path()),
        nanoimage_core::ImageFormat::WebP
    );
    assert_eq!(
        nanoimage_core::ImageFormat::from_path(PathBuf::from("photo.gif").as_path()),
        nanoimage_core::ImageFormat::Gif
    );
    assert_eq!(
        nanoimage_core::ImageFormat::from_path(PathBuf::from("photo.bmp").as_path()),
        nanoimage_core::ImageFormat::Bmp
    );
    assert_eq!(
        nanoimage_core::ImageFormat::from_path(PathBuf::from("photo.svg").as_path()),
        nanoimage_core::ImageFormat::Svg
    );
    assert_eq!(
        nanoimage_core::ImageFormat::from_path(PathBuf::from("photo.xyz").as_path()),
        nanoimage_core::ImageFormat::Unknown
    );
}

/// 测试 format_size 函数
#[test]
fn test_e2e_format_size() {
    use nanoimage_core::format_size;

    assert_eq!(format_size(0), "0 B");
    assert_eq!(format_size(512), "512 B");
    assert!(format_size(1024).starts_with("1.")); // >= 1 KB
    assert!(format_size(1024 * 1024).starts_with("1.")); // >= 1 MB
    assert!(format_size(1024 * 1024 * 1024).starts_with("1.")); // >= 1 GB
}

/// 测试配置序列化/反序列化一致性
#[test]
fn test_e2e_config_roundtrip() {
    let config = nanoimage_core::OptimizerConfig {
        mode: nanoimage_core::CompressionMode::Lossless,
        quality: nanoimage_core::Quality {
            lossy: 90,
            lossless: 95,
        },
        max_width: Some(1920),
        max_height: Some(1080),
        format: nanoimage_core::OutputFormat::WebP,
        preserve_metadata: false,
        overwrite: true,
        output_dir: Some(PathBuf::from("/tmp/test")),
        skip_failed: true,
        workers: 8,
    };

    // 序列化
    let json = serde_json::to_string_pretty(&config).expect("序列化失败");

    // 反序列化
    let restored: nanoimage_core::OptimizerConfig =
        serde_json::from_str(&json).expect("反序列化失败");

    // 验证所有字段
    assert_eq!(restored.mode, config.mode);
    assert_eq!(restored.quality.lossy, config.quality.lossy);
    assert_eq!(restored.quality.lossless, config.quality.lossless);
    assert_eq!(restored.max_width, config.max_width);
    assert_eq!(restored.max_height, config.max_height);
    assert_eq!(restored.format, config.format);
    assert_eq!(restored.preserve_metadata, config.preserve_metadata);
    assert_eq!(restored.overwrite, config.overwrite);
    assert_eq!(restored.output_dir, config.output_dir);
    assert_eq!(restored.skip_failed, config.skip_failed);
    assert_eq!(restored.workers, config.workers);
}

/// 测试 ProcessResult.savings_percent 计算
#[test]
fn test_e2e_savings_percent() {
    let result = nanoimage_core::ProcessResult {
        original_path: PathBuf::from("test.jpg"),
        output_path: PathBuf::from("output.jpg"),
        original_size: 1000,
        new_size: 500,
        savings: 500,
        success: true,
        error: None,
    };
    assert!((result.savings_percent() - 50.0).abs() < 0.01);

    // 零大小边界
    let zero_result = nanoimage_core::ProcessResult {
        original_path: PathBuf::from("test"),
        output_path: PathBuf::from("out"),
        original_size: 0,
        new_size: 0,
        savings: 0,
        success: true,
        error: None,
    };
    assert_eq!(zero_result.savings_percent(), 0.0);
}

/// 测试 Progress.percent 计算
#[test]
fn test_e2e_progress_percent() {
    let progress = nanoimage_core::Progress {
        current: 3,
        total: 10,
        current_file: "test.jpg".to_string(),
        bytes_processed: 1024,
        bytes_saved: 512,
    };
    assert!((progress.percent() - 30.0).abs() < 0.01);

    // 零总数
    let zero_progress = nanoimage_core::Progress {
        current: 0,
        total: 0,
        current_file: "".to_string(),
        bytes_processed: 0,
        bytes_saved: 0,
    };
    assert_eq!(zero_progress.percent(), 0.0);
}

/// 测试输出目录创建
#[test]
fn test_e2e_output_dir_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let input = temp_dir.path().join("input.jpg");
    let output_dir = temp_dir.path().join("nested").join("deep").join("output");
    let output = output_dir.join("input.jpg");

    let rgb = image::ImageBuffer::<image::Rgb<u8>, _>::from_fn(50, 50, |_, _| {
        image::Rgb([255u8, 100, 50])
    });
    rgb.save(&input).unwrap();

    let config = nanoimage_core::OptimizerConfig {
        overwrite: false,
        output_dir: Some(output_dir),
        ..Default::default()
    };
    let optimizer = nanoimage_core::Optimizer::new(config);
    let result = optimizer.process_file(&input);

    assert!(result.success, "嵌套目录创建后应该成功");
    assert!(output.exists(), "输出文件应该在嵌套目录中");
}
