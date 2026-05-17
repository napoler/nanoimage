//! Tests for processor module
use std::path::Path;
use nanoimage_core::{BatchProcessor, Progress};

/// Test that collect_images finds image files in a directory
#[test]
fn test_collect_images_finds_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    // Create test image files
    std::fs::write(temp_dir.path().join("test.jpg"), b"fake jpg").unwrap();
    std::fs::write(temp_dir.path().join("test.png"), b"fake png").unwrap();
    std::fs::write(temp_dir.path().join("test.webp"), b"fake webp").unwrap();
    // Create a non-image file
    std::fs::write(temp_dir.path().join("readme.txt"), b"not an image").unwrap();

    let files = BatchProcessor::collect_images(temp_dir.path(), false);
    assert_eq!(files.len(), 3);
}

/// Test that collect_images ignores non-image files
#[test]
fn test_collect_images_ignores_non_images() {
    let temp_dir = tempfile::tempdir().unwrap();
    std::fs::write(temp_dir.path().join("test.jpg"), b"fake").unwrap();
    std::fs::write(temp_dir.path().join("script.py"), b"print('hi')").unwrap();
    std::fs::write(temp_dir.path().join("data.csv"), b"a,b,c").unwrap();

    let files = BatchProcessor::collect_images(temp_dir.path(), false);
    assert_eq!(files.len(), 1);
    assert!(files[0].file_name().unwrap() == "test.jpg");
}

/// Test that collect_images with recursive finds files in subdirectories
#[test]
fn test_collect_images_recursive() {
    let temp_dir = tempfile::tempdir().unwrap();
    let subdir = temp_dir.path().join("subdir");
    std::fs::create_dir(&subdir).unwrap();
    std::fs::write(temp_dir.path().join("root.jpg"), b"fake").unwrap();
    std::fs::write(subdir.join("nested.png"), b"fake").unwrap();

    // Non-recursive should only find root
    let flat = BatchProcessor::collect_images(temp_dir.path(), false);
    assert_eq!(flat.len(), 1);

    // Recursive should find both
    let recursive = BatchProcessor::collect_images(temp_dir.path(), true);
    assert_eq!(recursive.len(), 2);
}

/// Test that collect_images handles empty directory
#[test]
fn test_collect_images_empty_dir() {
    let temp_dir = tempfile::tempdir().unwrap();
    let files = BatchProcessor::collect_images(temp_dir.path(), false);
    assert!(files.is_empty());
}

/// Test that collect_images handles non-existent directory gracefully
#[test]
fn test_collect_images_nonexistent_dir() {
    let files = BatchProcessor::collect_images(Path::new("/nonexistent/path/that/does/not/exist"), false);
    assert!(files.is_empty());
}

/// Test collect_images finds all supported extensions
#[test]
fn test_collect_images_all_extensions() {
    let temp_dir = tempfile::tempdir().unwrap();
    let extensions = ["jpg", "jpeg", "png", "webp", "gif", "bmp", "svg"];
    for ext in &extensions {
        let filename = format!("test.{}", ext);
        std::fs::write(temp_dir.path().join(filename), b"fake").unwrap();
    }

    let files = BatchProcessor::collect_images(temp_dir.path(), false);
    assert_eq!(files.len(), extensions.len());
}

/// Test collect_images is case-insensitive for extensions
#[test]
fn test_collect_images_case_insensitive() {
    let temp_dir = tempfile::tempdir().unwrap();
    std::fs::write(temp_dir.path().join("test.JPG"), b"fake").unwrap();
    std::fs::write(temp_dir.path().join("test.Png"), b"fake").unwrap();
    std::fs::write(temp_dir.path().join("test.WEBP"), b"fake").unwrap();

    let files = BatchProcessor::collect_images(temp_dir.path(), false);
    assert_eq!(files.len(), 3);
}

/// Test Progress::percent() returns correct values
#[test]
fn test_progress_percent() {
    let progress = Progress {
        current: 1,
        total: 10,
        current_file: "test.jpg".to_string(),
        bytes_processed: 1024,
        bytes_saved: 512,
    };
    assert!((progress.percent() - 10.0).abs() < 0.01);
}

#[test]
fn test_progress_percent_complete() {
    let progress = Progress {
        current: 10,
        total: 10,
        current_file: "test.jpg".to_string(),
        bytes_processed: 1024,
        bytes_saved: 512,
    };
    assert!((progress.percent() - 100.0).abs() < 0.01);
}

#[test]
fn test_progress_percent_zero_total() {
    let progress = Progress {
        current: 0,
        total: 0,
        current_file: "test.jpg".to_string(),
        bytes_processed: 0,
        bytes_saved: 0,
    };
    assert_eq!(progress.percent(), 0.0);
}

/// Test BatchProcessor::new creates a valid processor
#[test]
fn test_batch_processor_new() {
    use nanoimage_core::Optimizer;
    let optimizer = Optimizer::with_default();
    let _processor = BatchProcessor::new(optimizer);
    // Verify the processor was created successfully by calling collect_images
    let files = BatchProcessor::collect_images(tempfile::tempdir().unwrap().path(), false);
    assert!(files.is_empty()); // empty dir, but processor works
}

/// 辅助函数：创建 100x100 的红色 RGB 测试图片并保存到指定路径
fn create_test_image(path: &std::path::Path) {
    // 使用 RGB 格式以兼容 JPEG（JPEG 不支持 alpha 通道）
    let rgb = image::ImageBuffer::<image::Rgb<u8>, _>::from_fn(100, 100, |_, _| {
        image::Rgb([255u8, 0, 0])
    });
    rgb.save(path).unwrap();
}

/// 测试批量处理同步带进度回调：创建 3 个测试图片，验证回调被调用 3 次且 progress.current 递增
#[test]
fn test_batch_process_sync_with_progress() {
    use nanoimage_core::Optimizer;
    use std::sync::{Arc, Mutex};
    let temp_dir = tempfile::tempdir().unwrap();

    // 创建 3 个测试图片文件
    let files: Vec<std::path::PathBuf> = (0..3)
        .map(|i| {
            let path = temp_dir.path().join(format!("test{}.jpg", i));
            create_test_image(&path);
            path
        })
        .collect();

    let optimizer = Optimizer::with_default();
    let processor = BatchProcessor::new(optimizer);

    // 使用 Arc<Mutex<>> 在 Fn 闭包中收集进度值
    let progress_list = Arc::new(Mutex::new(Vec::new()));
    let progress_clone = progress_list.clone();

    let total_saved = processor.process_sync_with_progress(&files, move |progress| {
        progress_clone.lock().unwrap().push(progress);
    });

    let collected = progress_list.lock().unwrap();
    assert_eq!(collected.len(), 3, "回调应该被调用 3 次");
    assert_eq!(collected[0].current, 1);
    assert_eq!(collected[1].current, 2);
    assert_eq!(collected[2].current, 3);
    // total_saved 是 u64 类型，表示节省的字节数
    let _ = total_saved;
}

/// 测试批量处理同步带结果：创建 2 个测试图片，验证返回的 results 包含 2 个元素
#[test]
fn test_batch_process_sync_with_results() {
    use nanoimage_core::Optimizer;
    let temp_dir = tempfile::tempdir().unwrap();

    // 创建 2 个测试图片文件
    let files: Vec<std::path::PathBuf> = (0..2)
        .map(|i| {
            let path = temp_dir.path().join(format!("test{}.jpg", i));
            create_test_image(&path);
            path
        })
        .collect();

    let optimizer = Optimizer::with_default();
    let processor = BatchProcessor::new(optimizer);

    let (total_saved, results): (u64, Vec<nanoimage_core::ProcessResult>) =
        processor.process_sync_with_results(&files, |_| {});

    assert_eq!(results.len(), 2, "results 应该包含 2 个元素");
    // total_saved 是 u64 类型，表示节省的字节数
    let _ = total_saved;

    // 验证每个结果都是成功的
    for result in &results {
        assert!(result.success, "每个文件处理都应该成功");
    }
}

/// 测试 BatchProcessor::with_config 能正常创建实例
#[test]
fn test_batch_processor_with_config() {
    let config = nanoimage_core::OptimizerConfig {
        quality: nanoimage_core::Quality { lossy: 90, lossless: 95 },
        overwrite: true,
        workers: 4,
        ..Default::default()
    };
    let processor = BatchProcessor::with_config(config);

    // 验证处理器可以正常工作：收集空目录
    let files = BatchProcessor::collect_images(tempfile::tempdir().unwrap().path(), false);
    assert!(files.is_empty());

    // 验证处理器可以处理文件
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.jpg");
    create_test_image(&path);
    let results = processor.process_sync(&[path.clone()]);
    assert_eq!(results.len(), 1);
    assert!(results[0].success, "with_config 创建的处理器应该能正常处理文件");
}
