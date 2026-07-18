//! CLI 输出模块 - 彩色终端输出
use std::io::Write;

/// 输出颜色
#[derive(Clone, Copy)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn checkmark() {
    print_color(Color::Green, "✓");
}

/// 文件处理成功
#[allow(dead_code)]
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

/// 打印表格（使用 Unicode 边框，CJK 感知对齐）
pub fn print_table(headers: &[&str], rows: Vec<Vec<String>>) {
    if rows.is_empty() {
        return;
    }

    /// 计算字符串的显示宽度（CJK 字符算 2 列宽）
    fn display_width(s: &str) -> usize {
        s.chars().map(|c| {
            // CJK Unified Ideographs start at U+4E00; treat code points above that
            // (and excluding the half-width katakana-middle dot U+3030) as double-width.
            if c as u32 > 0x4E00 && c as u32 != 0x3030 {
                2
            } else {
                1
            }
        }).sum()
    }

    let num_cols = headers.len();
    // 计算每列最大显示宽度
    let mut col_widths: Vec<usize> = headers.iter().map(|h| display_width(h)).collect();

    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                col_widths[i] = col_widths[i].max(display_width(cell));
            }
        }
    }

    // 确保最小列宽（至少和表头一样宽）
    for (i, h) in headers.iter().enumerate() {
        if col_widths[i] < display_width(h) {
            col_widths[i] = display_width(h);
        }
    }

    // 添加 padding
    let padding: usize = 2;

    // 计算总表格宽度（用于窄终端检测）
    let total_width: usize = col_widths.iter()
        .map(|&w: &usize| w + padding * 2)
        .sum::<usize>()
        + col_widths.len().saturating_sub(1);

    // 如果总宽度超过 80 列，考虑截断过长的列
    let max_terminal_width = 80;
    let needs_truncation = total_width > max_terminal_width;

    let effective_col_widths: Vec<usize> = if needs_truncation {
        // 保留前 N-1 列不变，最后一列自动调整
        let mut widths = col_widths.clone();
        let last_idx = widths.len() - 1;
        let current_val = widths[last_idx];
        let extra = total_width - max_terminal_width + 4; // +4 为边框留空间
        widths[last_idx] = current_val.saturating_sub(extra);
        widths
    } else {
        col_widths
    };

    // 构建分隔行（┼ 占 1 列宽，所以 dashes 少 1）
    let mut sep = String::from("├");
    for &w in &effective_col_widths {
        sep.push_str(&"─".repeat(w + padding * 2 - 1));
        sep.push('┼');
    }

    // 打印表头
    print!("┌");
    for &w in &effective_col_widths {
        print!("{}", "─".repeat(w + padding * 2 - 1));
        print!("┬");
    }
    println!("┐");

    print!("│");
    for (i, h) in headers.iter().enumerate() {
        let w = effective_col_widths[i];
        let h_width = display_width(h);
        print!(" {}{} ", h, " ".repeat(w - h_width));
    }
    println!("│");

    // 分隔线
    println!("{}", sep);

    // 打印行
    for row in &rows {
        print!("│");
        for (i, cell) in row.iter().enumerate() {
            let w = if i < effective_col_widths.len() {
                effective_col_widths[i]
            } else {
                display_width(cell)
            };
            let c_width = display_width(cell);
            print!(" {}{} ", cell, " ".repeat(w.saturating_sub(c_width)));
        }
        println!("│");
    }

    // 底部
    print!("└");
    for &w in &effective_col_widths {
        print!("{}", "─".repeat(w + padding * 2 - 1));
        print!("┴");
    }
    println!("┘");
}