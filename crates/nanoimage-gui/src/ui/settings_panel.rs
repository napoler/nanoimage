//! 设置面板 - 配置 UI 组件
use eframe::egui;
use nanoimage_core::{OptimizerConfig, OutputFormat, CompressionMode};
use std::path::Path;

/// 设置面板组件
pub struct SettingsPanel {
    pub config: OptimizerConfig,
    format_index: usize,
    mode_index: usize,
}

impl SettingsPanel {
    /// 创建默认设置面板
    pub fn new() -> Self {
        Self {
            config: OptimizerConfig::default(),
            format_index: 0,
            mode_index: 0,
        }
    }

    /// 使用给定配置创建设置面板
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

            // 配置导入/导出
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("导入配置").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("配置文件", &["json"])
                        .pick_file()
                    {
                        if let Ok(config) = OptimizerConfig::load_from_file(&path) {
                            self.config = config;
                            // 更新索引以匹配加载的配置
                            self.format_index = match self.config.format {
                                OutputFormat::KeepOriginal => 0,
                                OutputFormat::Jpeg => 1,
                                OutputFormat::Png => 2,
                                OutputFormat::WebP => 3,
                                OutputFormat::Gif => 4,
                            };
                            self.mode_index = match self.config.mode {
                                CompressionMode::Lossy => 0,
                                CompressionMode::Lossless => 1,
                                CompressionMode::Smart => 2,
                            };
                            tracing::info!("导入配置成功: {}", path.display());
                            // 保存导入的配置
                            let _ = self.save_config_to_path(&path);
                        } else {
                            tracing::error!("导入配置失败: {}", path.display());
                        }
                    }
                }

                if ui.button("导出配置").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON 文件", &["json"])
                        .set_file_name("nanoimage-config.json")
                        .save_file()
                    {
                        let _ = self.config.save_to_file(&path);
                        tracing::info!("导出配置成功: {}", path.display());
                    }
                }
            });
        });
    }

    /// 保存配置到指定路径
    fn save_config_to_path(&self, path: &Path) -> Result<(), String> {
        self.config.save_to_file(path)
            .map_err(|e| format!("保存配置失败: {}", e))
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