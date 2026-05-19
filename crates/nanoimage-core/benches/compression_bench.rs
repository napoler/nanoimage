//! 基准测试 - JPEG/PNG/WebP 压缩性能评估
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nanoimage_core::{Optimizer, OptimizerConfig};
use std::path::Path;

fn benchmark_jpeg_compression(c: &mut Criterion) {
    let config = OptimizerConfig::default();
    let optimizer = Optimizer::new(config);
    
    c.bench_function("JPEG compression", |b| {
        b.iter(|| {
            let input = black_box(Path::new("benches/test_data/test.jpg"));
            optimizer.process_file(input);
        })
    });
}

fn benchmark_png_compression(c: &mut Criterion) {
    let config = OptimizerConfig::default();
    let optimizer = Optimizer::new(config);
    
    c.bench_function("PNG compression", |b| {
        b.iter(|| {
            let input = black_box(Path::new("benches/test_data/test.png"));
            optimizer.process_file(input);
        })
    });
}

fn benchmark_webp_compression(c: &mut Criterion) {
    let config = OptimizerConfig::default();
    let optimizer = Optimizer::new(config);
    
    c.bench_function("WebP compression", |b| {
        b.iter(|| {
            let input = black_box(Path::new("benches/test_data/test.webp"));
            optimizer.process_file(input);
        })
    });
}

criterion_group!(
    benches,
    benchmark_jpeg_compression,
    benchmark_png_compression,
    benchmark_webp_compression,
);
criterion_main!(benches);
