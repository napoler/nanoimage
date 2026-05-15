//! CLI 输出模块 - 彩色终端输出
use std::io::Write;

/// 输出颜色
#[derive(Clone, Copy)]
pub enum Color {
    Green,
    Yellow,
    Red,
    Blue,
    Bold,
    Reset,
}

impl Color {
    pub fn as_str(&self) -> &str {
        match self {
            Color::Green => "\x1b[32m",
            Color::Yellow => "\x1b[33m",
            Color::Red => "\x1b[31m",
            Color::Blue => "\x1b[34m",
            Color::Bold => "\x1b[1m",
            Color::Reset => "\x1b[0m",
        }
    }
}

/// 打印带颜色的消息
pub fn print_color(color: Color, msg: &str) {
    print!("{}{}{}", color.as_str(), msg, Color::Reset.as_str());
    std::io::stdout().flush().ok();
}

/// 打印带颜色的消息并换行
pub fn println_color(color: Color, msg: &str) {
    println!("{}{}{}", color.as_str(), msg, Color::Reset.as_str());
}

/// 成功消息 (绿色)
pub fn success(msg: &str) {
    println_color(Color::Green, msg);
}

/// 警告消息 (黄色)
pub fn warn(msg: &str) {
    println_color(Color::Yellow, msg);
}

/// 错误消息 (红色)
pub fn error(msg: &str) {
    eprintln!("{}{}{}", Color::Red.as_str(), msg, Color::Reset.as_str());
}

/// 信息消息 (蓝色)
pub fn info(msg: &str) {
    println_color(Color::Blue, msg);
}

/// 进度点 (绿色圆点)
pub fn dot() {
    print_color(Color::Green, ".");
    std::io::stdout().flush().ok();
}

/// 进度完成符号 (绿色勾)
pub fn checkmark() {
    print_color(Color::Green, "✓");
}

/// 文件处理成功
pub fn file_success(name: &str, orig_size: &str, new_size: &str, savings: f64) {
    print!("{} ", Color::Green.as_str());
    print!("✓ ");
    print!("{}", Color::Reset.as_str());
    println!("{} ({} → {}, -{:.1}%)", name, orig_size, new_size, savings);
}

/// 文件处理失败
pub fn file_error(name: &str, err: &str) {
    eprint!("{} ", Color::Red.as_str());
    eprint!("✗ ");
    eprint!("{}", Color::Reset.as_str());
    eprintln!(" {}: {}", name, err);
}