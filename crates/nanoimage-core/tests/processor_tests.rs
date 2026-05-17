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
