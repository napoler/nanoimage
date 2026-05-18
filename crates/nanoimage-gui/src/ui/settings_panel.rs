//! 设置面板 - 配置 UI 组件
use eframe::egui;
use nanoimage_core::{OptimizerConfig, OutputFormat, CompressionMode};

/// 设置面板组件
pub struct SettingsPanel {
    pub config: OptimizerConfig,
    format_index: usize,
    mode_index: usize,
}

impl SettingsPanel {
    pub fn new() -> Self {
        Self {
            config: OptimizerConfig::default(),
            format_index: 0,
            mode_index: 0,
        }
    }

    pub fn with_config(config: OptimizerConfig) -> Self {
        let format_index = match config.format {
            OutputFormat::KeepOriginal => 0,
            OutputFormat::Jpeg => 1,
            OutputFormat::Png => 2,
            OutputFormat::WebP => 3,
            OutputFormat::Gif => 4,
        };
        let mode_index = match config.mode {
            CompressionMode::Lossy => 0,
            CompressionMode::Lossless => 1,
            CompressionMode::Smart => 2,
        };
        Self {
            config,
            format_index,
            mode_index,
        }
    }

    /// 显示设置面板
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // 质量设置
            ui.horizontal(|ui| {
                ui.label("质量:");
                let mut quality = self.config.quality.lossy as f32;
                ui.add(egui::Slider::new(&mut quality, 1.0..=100.0).text("%"));
                self.config.quality.lossy = quality as u8;
                ui.label(format!("{}%", self.config.quality.lossy));
            });

            // 无损质量
            ui.horizontal(|ui| {
                ui.label("无损质量:");
                let mut lossless = self.config.quality.lossless as f32;
                ui.add(egui::Slider::new(&mut lossless, 50.0..=100.0).text("%"));
                self.config.quality.lossless = lossless as u8;
                ui.label(format!("{}%", self.config.quality.lossless));
            });

            // 压缩模式
            ui.horizontal(|ui| {
                ui.label("压缩模式:");
                egui::ComboBox::from_id_salt("mode_combo")
                    .selected_text(match self.mode_index {
                        0 => "有损",
                        1 => "无损",
                        _ => "智能",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.mode_index, 0, "有损");
                        ui.selectable_value(&mut self.mode_index, 1, "无损");
                        ui.selectable_value(&mut self.mode_index, 2, "智能");
                    });
                self.config.mode = match self.mode_index {
                    0 => CompressionMode::Lossy,
                    1 => CompressionMode::Lossless,
                    _ => CompressionMode::Smart,
                };
            });

            // 输出格式
            ui.horizontal(|ui| {
                ui.label("输出格式:");
                egui::ComboBox::from_id_salt("format_combo")
                    .selected_text(match self.format_index {
                        0 => "保持原格式",
                        1 => "JPEG",
                        2 => "PNG",
                        3 => "WebP",
                        _ => "GIF",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.format_index, 0, "保持原格式");
                        ui.selectable_value(&mut self.format_index, 1, "JPEG");
                        ui.selectable_value(&mut self.format_index, 2, "PNG");
                        ui.selectable_value(&mut self.format_index, 3, "WebP");
                        ui.selectable_value(&mut self.format_index, 4, "GIF");
                    });
                self.config.format = match self.format_index {
                    0 => OutputFormat::KeepOriginal,
                    1 => OutputFormat::Jpeg,
                    2 => OutputFormat::Png,
                    3 => OutputFormat::WebP,
                    _ => OutputFormat::Gif,
                };
            });

            // 线程数
            ui.horizontal(|ui| {
                ui.label("工作线程:");
                let mut workers = self.config.workers as f32;
                ui.add(egui::Slider::new(&mut workers, 1.0..=16.0).text(""));
                self.config.workers = workers as usize;
                ui.label(format!("{}", self.config.workers));
            });

            // 复选框选项
            ui.checkbox(&mut self.config.preserve_metadata, "保留元数据");
            ui.checkbox(&mut self.config.overwrite, "覆盖源文件");
            ui.checkbox(&mut self.config.skip_failed, "跳过失败文件继续");

            // 输出目录选择
            ui.horizontal(|ui| {
                ui.label("输出目录:");
                let dir_text = self.config.output_dir
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "默认 (同目录)".to_string());
                if ui.button("选择目录").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.config.output_dir = Some(path);
                    }
                }
                ui.label(dir_text);
            });
        });
    }

    pub fn config(&self) -> &OptimizerConfig {
        &self.config
    }
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}