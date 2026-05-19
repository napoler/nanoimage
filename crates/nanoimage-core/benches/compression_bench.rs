//! NanoImage 压缩性能基准测试
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nanoimage_core::{Optimizer, OptimizerConfig};
use std::path::Path;

/// 创建一个 500x500 的渐变彩色测试图片（更大的图片，更真实的基准）
fn create_large_test_image(path: &Path) {
    let rgb = image::ImageBuffer::<image::Rgb<u8>, _>::from_fn(500, 500, |x, y| {
        image::Rgb([x as u8, y as u8, ((x + y) / 2) as u8])
    });
    rgb.save(path).unwrap();
}

/// JPEG 压缩基准测试
fn bench_jpeg_compression(c: &mut Criterion) {
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("bench_input.jpg");
    create_large_test_image(&input_path);

    let config = OptimizerConfig {
        overwrite: true,
        ..Default::default()
    };
    let optimizer = Optimizer::new(config);

    c.bench_function("JPEG compression (500x500)", |b| {
        b.iter(|| {
            let result = optimizer.process_file(black_box(&input_path));
            assert!(result.success, "JPEG 压缩应该成功");
        })
    });
}

/// PNG 压缩基准测试
fn bench_png_compression(c: &mut Criterion) {
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("bench_input.png");
    create_large_test_image(&input_path);

    let config = OptimizerConfig {
        overwrite: true,
        ..Default::default()
    };
    let optimizer = Optimizer::new(config);

    c.bench_function("PNG compression (500x500)", |b| {
        b.iter(|| {
            let result = optimizer.process_file(black_box(&input_path));
            assert!(result.success, "PNG 压缩应该成功");
        })
    });
}

/// WebP 转换基准测试
fn bench_webp_conversion(c: &mut Criterion) {
    let temp_dir = tempfile::tempdir().unwrap();
    let input_path = temp_dir.path().join("bench_input.webp");
    create_large_test_image(&input_path);

    let config = OptimizerConfig {
        overwrite: true,
        ..Default::default()
    };
    let optimizer = Optimizer::new(config);

    c.bench_function("WebP conversion (500x500)", |b| {
        b.iter(|| {
            let result = optimizer.process_file(black_box(&input_path));
            assert!(result.success, "WebP 转换应该成功");
        })
    });
}

criterion_group!(
    benches,
    bench_jpeg_compression,
    bench_png_compression,
    bench_webp_conversion,
);
criterion_main!(benches);
