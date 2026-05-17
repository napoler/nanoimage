//! Integration tests for nanoimage-core
use std::path::PathBuf;
use nanoimage_core::{OptimizerConfig, ProcessResult};

/// Test ProcessResult.savings_percent() calculation
#[test]
fn test_savings_percent_positive() {
    let result = ProcessResult {
        original_path: PathBuf::from("test.jpg"),
        output_path: PathBuf::from("output.jpg"),
        original_size: 1000,
        new_size: 500,
        savings: 500,
        success: true,
        error: None,
    };
    assert!((result.savings_percent() - 50.0).abs() < 0.01);
}

#[test]
fn test_savings_percent_zero() {
    let result = ProcessResult {
        original_path: PathBuf::from("test.jpg"),
        output_path: PathBuf::from("output.jpg"),
        original_size: 1000,
        new_size: 1000,
        savings: 0,
        success: true,
        error: None,
    };
    assert_eq!(result.savings_percent(), 0.0);
}

#[test]
fn test_savings_percent_negative() {
    let result = ProcessResult {
        original_path: PathBuf::from("test.jpg"),
        output_path: PathBuf::from("output.jpg"),
        original_size: 1000,
        new_size: 1200,
        savings: -200,
        success: true,
        error: None,
    };
    assert!((result.savings_percent() - (-20.0)).abs() < 0.01);
}

#[test]
fn test_savings_percent_zero_original_size() {
    let result = ProcessResult {
        original_path: PathBuf::from("test.jpg"),
        output_path: PathBuf::from("output.jpg"),
        original_size: 0,
        new_size: 0,
        savings: 0,
        success: true,
        error: None,
    };
    assert_eq!(result.savings_percent(), 0.0);
}

/// Test determine_output_path with overwrite=true
#[test]
fn test_output_path_overwrite() {
    let config = OptimizerConfig {
        overwrite: true,
        ..Default::default()
    };
    let optimizer = nanoimage_core::Optimizer::new(config);
    let input = PathBuf::from("/tmp/test.jpg");
    let output = optimizer.determine_output_path_test(&input);
    assert_eq!(output, input);
}

/// Test determine_output_path with custom output_dir
#[test]
fn test_output_path_custom_dir() {
    let config = OptimizerConfig {
        overwrite: false,
        output_dir: Some(PathBuf::from("/tmp/output")),
        ..Default::default()
    };
    let optimizer = nanoimage_core::Optimizer::new(config);
    let input = PathBuf::from("/tmp/images/test.jpg");
    let output = optimizer.determine_output_path_test(&input);
    assert_eq!(output, PathBuf::from("/tmp/output/test.jpg"));
}

/// Test determine_output_path default (optimized subdirectory)
#[test]
fn test_output_path_default() {
    let config = OptimizerConfig::default();
    let optimizer = nanoimage_core::Optimizer::new(config);
    let input = PathBuf::from("/tmp/images/test.jpg");
    let output = optimizer.determine_output_path_test(&input);
    assert_eq!(output, PathBuf::from("/tmp/images/optimized/test.jpg"));
}

/// Test determine_output_path with root directory
#[test]
fn test_output_path_root_dir() {
    let config = OptimizerConfig::default();
    let optimizer = nanoimage_core::Optimizer::new(config);
    let input = PathBuf::from("test.jpg");
    let output = optimizer.determine_output_path_test(&input);
    assert_eq!(output, PathBuf::from("optimized/test.jpg"));
}
