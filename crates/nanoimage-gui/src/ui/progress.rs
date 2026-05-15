//! 进度面板 - 显示处理进度
use eframe::egui;

/// 进度面板组件
pub struct ProgressPanel {
    current_file: String,
}

impl ProgressPanel {
    pub fn new() -> Self {
        Self {
            current_file: String::new(),
        }
    }

    /// 显示进度条
    pub fn show(&mut self, ui: &mut egui::Ui, progress: f32, processing: bool) {
        if processing {
            ui.horizontal(|ui| {
                ui.label("处理进度:");
                ui.add(egui::ProgressBar::new(progress / 100.0).text(format!("{:.*}%", 1, progress)));
            });

            if !self.current_file.is_empty() {
                ui.label(format!("当前: {}", self.current_file));
            }
        }
    }

    /// 设置当前处理文件
    pub fn set_current_file(&mut self, file: String) {
        self.current_file = file;
    }

    /// 清空状态
    pub fn reset(&mut self) {
        self.current_file.clear();
    }
}

impl Default for ProgressPanel {
    fn default() -> Self {
        Self::new()
    }
}