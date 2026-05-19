use image::{ImageEncoder, ImageFormat};
use std::fs::File;

fn main() {
    // 创建测试图片 - 1920x1080 RGB 图像
    let width = 1920;
    let height = 1080;
    
    // 创建 JPEG 测试图片
    {
        let mut file = File::create("benches/test_data/test.jpg").unwrap();
        let encoder = image::JPEGEncoder::new(&mut file);
        let mut data = vec![0u8; (width * height * 3) as usize];
        // 填充渐变色数据
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize * 3;
                data[idx] = (x as u8) % 255;       // R
                data[idx + 1] = (y as u8) % 255;   // G
                data[idx + 2] = ((x + y) as u8) % 255; // B
            }
        }
        encoder.encode(&data, width, height, image::ColorType::Rgb8).unwrap();
    }
    
    // 创建 PNG 测试图片
    {
        let mut file = File::create("benches/test_data/test.png").unwrap();
        let encoder = image::PNGEncoder::new(&mut file);
        let mut data = vec![0u8; (width * height * 3) as usize];
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize * 3;
                data[idx] = (x as u8) % 255;
                data[idx + 1] = (y as u8) % 255;
                data[idx + 2] = ((x + y) as u8) % 255;
            }
        }
        encoder.encode(&data, width, height, image::ColorType::Rgb8).unwrap();
    }
    
    // 创建 WebP 测试图片
    {
        let mut file = File::create("benches/test_data/test.webp").unwrap();
        let encoder = image::WebPEncoder::new(&mut file);
        let mut data = vec![0u8; (width * height * 3) as usize];
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize * 3;
                data[idx] = (x as u8) % 255;
                data[idx + 1] = (y as u8) % 255;
                data[idx + 2] = ((x + y) as u8) % 255;
            }
        }
        encoder.encode(&data, width, height, image::ColorType::Rgb8).unwrap();
    }
    
    println!("Test images created successfully!");
}
