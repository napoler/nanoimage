//! 日志面板 - 显示处理日志
use eframe::egui;

/// 日志条目
#[derive(Clone)]
pub struct LogEntry {
    pub message: String,
    pub level: LogLevel,
}

/// 日志级别
#[derive(Clone, Copy)]
pub enum LogLevel {
    Info,
    Success,
    Error,
}

/// 日志面板组件
pub struct LogPanel {
    pub entries: Vec<LogEntry>,
    max_entries: usize,
}

impl LogPanel {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 100,
        }
    }

    /// 添加日志
    pub fn log(&mut self, message: String, level: LogLevel) {
        self.entries.push(LogEntry { message, level });
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    /// 添加信息日志
    pub fn info(&mut self, msg: String) {
        self.log(msg, LogLevel::Info);
    }

    /// 添加成功日志
    pub fn success(&mut self, msg: String) {
        self.log(msg, LogLevel::Success);
    }

    #[allow(dead_code)]
    /// 添加错误日志
    pub fn error(&mut self, msg: String) {
        self.log(msg, LogLevel::Error);
    }

    /// 清空日志
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 显示日志面板
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label("📝 日志:");

        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                for entry in &self.entries {
                    let icon = match entry.level {
                        LogLevel::Info => "ℹ️",
                        LogLevel::Success => "✅",
                        LogLevel::Error => "❌",
                    };
                    ui.horizontal(|ui| {
                        ui.label(icon);
                        ui.label(&entry.message);
                    });
                }
            });

        if !self.entries.is_empty() {
            if ui.button("清空日志").clicked() {
                self.clear();
            }
        }
    }
}

impl Default for LogPanel {
    fn default() -> Self {
        Self::new()
    }
}