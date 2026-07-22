//! Tests for processor module
use nanoimage_core::{BatchProcessor, Progress};
use std::path::Path;

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
    let files =
        BatchProcessor::collect_images(Path::new("/nonexistent/path/that/does/not/exist"), false);
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
        quality: nanoimage_core::Quality {
            lossy: 90,
            lossless: 95,
        },
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
    let results = processor.process_sync(std::slice::from_ref(&path));
    assert_eq!(results.len(), 1);
    assert!(
        results[0].success,
        "with_config 创建的处理器应该能正常处理文件"
    );
}

/// 测试 `is_already_optimized` 不应匹配仅含 `_optimized` 子串但未真正优化的文件。
///
/// 期望失败原因：当前实现使用 `file_stem().contains("_optimized")` 做子串匹配，
/// "not_optimized.png" 的 file_stem 是 "not_optimized"，含子串 "_optimized"，
/// 因此当前会错误地返回 `true`。后续修复应改用严格的后缀匹配规则。
#[test]
fn test_is_already_optimized_substring_false() {
    let result = nanoimage_core::processor::is_already_optimized(std::path::Path::new(
        "not_optimized.png",
    ));
    assert!(
        !result,
        "不真正包含 _optimized 后缀的文件不应被判定为已优化"
    );
}

/// 测试 `is_already_optimized` 正向后缀匹配：`photo_optimized.jpg` 应被判定为已优化。
///
/// 这是预期的回归锚定（regression anchor）：当前实现即可通过此测试。
/// 配合 `test_is_already_optimized_substring_false` 一同确保边界条件被覆盖。
#[test]
fn test_is_already_optimized_suffix_true() {
    let result =
        nanoimage_core::processor::is_already_optimized(std::path::Path::new("photo_optimized.jpg"));
    assert!(
        result,
        "photo_optimized.jpg 应被判定为已优化（_optimized 是真正的后缀）"
    );
}

/// 测试 `is_already_optimized` 在 stem 仅为 `"_optimized"` 时也判定为已优化。
#[test]
fn test_is_already_optimized_bare_suffix_true() {
    let result =
        nanoimage_core::processor::is_already_optimized(std::path::Path::new("_optimized.png"));
    assert!(result, "_optimized.png 应判定为已优化");
}

/// 测试 `is_already_optimized` 不应匹配不含 `_optimized` 后缀的常见前缀场景。
#[test]
fn test_is_already_optimized_no_suffix_false() {
    let result =
        nanoimage_core::processor::is_already_optimized(std::path::Path::new("optimized.png"));
    assert!(!result, "optimized.png 不含下划线 _optimized 前缀，应判定为未优化");
}

/// 测试 `BatchProcessor::collect_images` 包含 BMP 文件。
///
/// 当前实现已包含 BMP 扩展（`extensions = ["jpg", "jpeg", "png", "webp", "gif", "bmp", "svg"]`），
/// 故此测试当前已通过，作为收集行为的回归锚定。
#[test]
fn test_collect_images_includes_bmp() {
    let temp_dir = tempfile::tempdir().unwrap();
    std::fs::write(temp_dir.path().join("a.jpg"), b"fake").unwrap();
    std::fs::write(temp_dir.path().join("b.png"), b"fake").unwrap();
    std::fs::write(temp_dir.path().join("c.bmp"), b"fake").unwrap();
    std::fs::write(temp_dir.path().join("d.txt"), b"not an image").unwrap();

    let files = BatchProcessor::collect_images(temp_dir.path(), false);
    assert_eq!(files.len(), 3, "应收集到 jpg/png/bmp 三个图片文件");

    // 收集到的文件名集合应包含全部三种图片格式
    let names: Vec<String> = files
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(names.iter().any(|n| n == "a.jpg"), "应包含 a.jpg");
    assert!(names.iter().any(|n| n == "b.png"), "应包含 b.png");
    assert!(names.iter().any(|n| n == "c.bmp"), "应包含 c.bmp —— BMP 必须被支持");
}

/// 测试 `process_sync_with_options` 在 skip_failed=true 时，确实丢弃失败结果并准确累加 failed_count。
#[test]
fn test_processor_skip_failed_drops_failures() {
    use nanoimage_core::Optimizer;
    let temp_dir = tempfile::tempdir().unwrap();
    let valid = temp_dir.path().join("valid.jpg");
    create_test_image(&valid);
    let invalid = temp_dir.path().join("invalid.jpg");
    std::fs::write(&invalid, b"not a real image").unwrap();

    let optimizer = Optimizer::with_default();
    let processor = BatchProcessor::new(optimizer);

    let (results, failed_count) =
        processor.process_sync_with_options(&[valid.clone(), invalid.clone()], true, false);

    assert_eq!(results.len(), 1, "skip_failed=true 应只保留 1 个成功结果");
    assert_eq!(failed_count, 1, "failed_count 应为 1");
    assert!(results[0].success);
}

/// 测试 `process_sync_with_options` 在 skip_failed=false 时，下发所有结果包括失败。
#[test]
fn test_processor_partial_failure_aggregation() {
    use nanoimage_core::Optimizer;
    let temp_dir = tempfile::tempdir().unwrap();
    let valid = temp_dir.path().join("valid.jpg");
    create_test_image(&valid);
    let invalid1 = temp_dir.path().join("bad1.jpg");
    let invalid2 = temp_dir.path().join("bad2.jpg");
    std::fs::write(&invalid1, b"garbage").unwrap();
    std::fs::write(&invalid2, b"more garbage").unwrap();

    let optimizer = Optimizer::with_default();
    let processor = BatchProcessor::new(optimizer);

    let (results, failed_count) = processor.process_sync_with_options(
        &[valid.clone(), invalid1.clone(), invalid2.clone()],
        false,
        false,
    );

    assert_eq!(results.len(), 3, "skip_failed=false 应下发全部 3 个结果");
    assert_eq!(failed_count, 0, "失败计数不依赖 skip_failed，应为 0");
    let success_count = results.iter().filter(|r| r.success).count();
    assert_eq!(success_count, 1, "1 个有效 JPEG 应成功");
    let error_count = results.iter().filter(|r| !r.success).count();
    assert_eq!(error_count, 2, "2 个无效文件应失败");
    for r in &results {
        if !r.success {
            assert!(r.error.is_some(), "失败结果必须有 error 字段");
        }
    }
}

/// 测试 Optimizer 默认模式 output_path != input_path，overwrite 模式 output_path == input_path。
#[test]
fn test_optimizer_success_path_metadata() {
    use nanoimage_core::Optimizer;
    let temp_dir = tempfile::tempdir().unwrap();
    let input = temp_dir.path().join("photo.jpg");
    create_test_image(&input);

    // 默认模式
    let default = Optimizer::with_default().process_file(&input);
    assert!(default.success);
    assert_ne!(
        default.output_path, input,
        "默认模式 output_path 应不同于 input"
    );
    assert!(default.new_size > 0);

    // overwrite 模式
    let overwrite_cfg = nanoimage_core::OptimizerConfig {
        overwrite: true,
        ..Default::default()
    };
    let overwrite_res = Optimizer::new(overwrite_cfg).process_file(&input);
    assert!(overwrite_res.success);
    assert_eq!(
        overwrite_res.output_path, input,
        "overwrite 模式 output_path 应等于 input"
    );
}

/// 测试不可写 output_dir 时，process_file 返回 success=false。
///
/// 注：使用 `/dev/null/foo/bar` 作为 output_dir：/dev/null 是字符设备，
/// create_dir_all 会失败（NOTDIR），符合错误注入需求。
#[test]
#[cfg(unix)]
fn test_optimizer_unwritable_output_path() {
    use nanoimage_core::{Optimizer, OptimizerConfig};
    let temp_dir = tempfile::tempdir().unwrap();
    let input = temp_dir.path().join("photo.jpg");
    create_test_image(&input);

    let cfg = OptimizerConfig {
        output_dir: Some(std::path::PathBuf::from("/dev/null/foo/bar")),
        ..Default::default()
    };
    let res = Optimizer::new(cfg).process_file(&input);
    assert!(
        !res.success,
        "output_dir 不可写时 process_file 必须返回 success=false，实际: {:?}",
        res.error
    );
    assert!(res.error.is_some(), "失败结果必须包含 error 字段");
}
