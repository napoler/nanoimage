//! 生成测试图片到 test_data/ 目录
use std::path::Path;

fn main() {
    let test_data = Path::new("test_data");
    std::fs::create_dir_all(test_data).unwrap();

    // 创建一个 800x600 的彩色渐变测试图
    let rgb = image::ImageBuffer::<image::Rgb<u8>, _>::from_fn(800, 600, |x, y| {
        image::Rgb([
            (x % 256) as u8,
            (y % 256) as u8,
            (((x + y) * 2) % 256) as u8,
        ])
    });

    let png_path = test_data.join("gradient.png");
    let jpg_path = test_data.join("gradient.jpg");
    let webp_path = test_data.join("gradient.webp");

    rgb.save(&png_path).unwrap();
    rgb.save(&jpg_path).unwrap();
    rgb.save(&webp_path).unwrap();

    println!("Created test images in test_data/");
    println!(
        "  gradient.png: {} bytes",
        std::fs::metadata(&png_path).unwrap().len()
    );
    println!(
        "  gradient.jpg: {} bytes",
        std::fs::metadata(&jpg_path).unwrap().len()
    );
    println!(
        "  gradient.webp: {} bytes",
        std::fs::metadata(&webp_path).unwrap().len()
    );
}
