//! NanoImage GUI 应用

mod ui;

use eframe::egui;
use nanoimage_core::OptimizerConfig;

use ui::{file_panel::FilePanel, settings_panel::SettingsPanel, progress::ProgressPanel, log_view::LogPanel};

/// 主应用
pub struct NanoImageApp {
    config: OptimizerConfig,
    file_panel: FilePanel,
    settings_panel: SettingsPanel,
    progress_panel: ProgressPanel,
    log_panel: LogPanel,
    processing: bool,
    progress: f32,
    processed_size: u64,
}

impl NanoImageApp {
    pub fn new() -> Self {
        let config = OptimizerConfig::default();
        Self {
            settings_panel: SettingsPanel::with_config(config.clone()),
            file_panel: FilePanel::new(),
            progress_panel: ProgressPanel::new(),
            log_panel: LogPanel::new(),
            config,
            processing: false,
            progress: 0.0,
            processed_size: 0,
        }
    }

    fn process_files(&mut self) {
        if self.file_panel.is_empty() {
            return;
        }

        self.processing = true;
        self.progress = 0.0;

        let files = self.file_panel.paths();
        let total = files.len();

        self.log_panel.info(format!("开始处理 {} 个文件...", total));

        for (idx, path) in files.iter().enumerate() {
            let optimizer = nanoimage_core::Optimizer::new(self.config.clone());
            let result = optimizer.process_file(path);

            // 收集结果
            let success = result.success;
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let orig_size = result.original_size;
            let new_size = result.new_size;
            let error_msg = result.error.unwrap_or_default();
            let savings = result.savings;

            // 更新文件状态
            if success {
                self.file_panel.update_status(&path, nanoimage_core::FileStatus::Completed, Some(new_size));
                self.processed_size += orig_size;
                self.log_panel.success(format!(
                    "✓ {} ({} → {}, -{:.*}%)",
                    name,
                    format_size(orig_size),
                    format_size(new_size),
                    1,
                    if orig_size > 0 { (savings as f64 / orig_size as f64) * 100.0 } else { 0.0 }
                ));
            } else {
                self.file_panel.update_status(&path, nanoimage_core::FileStatus::Error(error_msg.clone()), None);
                self.log_panel.error(format!("✗ {}: {}", name, error_msg));
            }

            self.progress = ((idx + 1) as f32 / total as f32) * 100.0;
        }

        self.processing = false;
        self.log_panel.success(format!("处理完成! 共节省 {}", format_size(self.processed_size)));
    }
}

impl eframe::App for NanoImageApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("NanoImage");
            ui.label("图像优化器 - Rust + egui");

            ui.separator();

            self.file_panel.show(ui);
            self.file_panel.show_file_list(ui);

            ui.separator();

            self.settings_panel.show(ui);
            self.config = self.settings_panel.config().clone();

            ui.separator();

            let button_text = if self.processing { "处理中..." } else { "▶ 开始优化" };
            let can_start = !self.processing && !self.file_panel.is_empty();
            if ui.add_enabled(can_start, egui::Button::new(button_text)).clicked() {
                self.process_files();
            }

            if self.processing {
                ui.add(egui::ProgressBar::new(self.progress / 100.0).text(format!("{:.*}%", 1, self.progress)));
            }

            ui.separator();
            self.progress_panel.show(ui, self.progress, self.processing);

            ui.separator();
            self.log_panel.show(ui);
        });
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}