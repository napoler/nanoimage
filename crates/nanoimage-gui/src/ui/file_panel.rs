//! 文件面板 - 文件拖拽和列表显示
use eframe::egui;
use nanoimage_core::{format_size, FileStatus};
use std::path::PathBuf;

/// 文件条目显示
#[derive(Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: String,
    pub status: FileStatus,
    pub original_size: u64,
    pub new_size: Option<u64>,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let size = path
            .metadata()
            .map(|m| format_size(m.len()))
            .unwrap_or_else(|_| "Unknown".to_string());

        let original_size = path.metadata().map(|m| m.len()).unwrap_or(0);

        Self {
            path,
            name,
            size,
            status: FileStatus::Pending,
            original_size,
            new_size: None,
        }
    }

    /// 计算节省百分比
    pub fn savings_percent(&self) -> f64 {
        match self.new_size {
            Some(new) if self.original_size > 0 => {
                ((self.original_size as f64 - new as f64) / self.original_size as f64) * 100.0
            }
            _ => 0.0,
        }
    }

    /// 根据状态和节省百分比获取颜色
    fn savings_color(&self) -> egui::Color32 {
        match self.status {
            FileStatus::Completed => {
                let pct = self.savings_percent();
                if pct >= 70.0 {
                    egui::Color32::from_rgb(34, 197, 94) // 深绿
                } else if pct >= 40.0 {
                    egui::Color32::from_rgb(74, 222, 128) // 绿色
                } else if pct >= 10.0 {
                    egui::Color32::from_rgb(134, 239, 172) // 浅绿
                } else if pct >= 0.0 {
                    egui::Color32::from_rgb(253, 224, 71) // 黄色
                } else {
                    egui::Color32::from_rgb(239, 68, 68) // 红色（压缩后更大）
                }
            }
            FileStatus::Error(_) => egui::Color32::from_rgb(239, 68, 68), // 红色
            FileStatus::Processing => egui::Color32::from_rgb(251, 191, 36), // 橙色
            FileStatus::Pending | FileStatus::Skipped => egui::Color32::from_rgb(156, 163, 175), // 灰色
        }
    }
}

/// 文件面板组件
pub struct FilePanel {
    pub files: Vec<FileEntry>,
}

impl FilePanel {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
        }
    }

    /// 添加文件
    pub fn add_files(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            if !self.files.iter().any(|f| f.path == path) {
                self.files.push(FileEntry::new(path));
            }
        }
    }

    /// 清空文件列表
    pub fn clear(&mut self) {
        self.files.clear();
    }

    /// 显示拖拽区域
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if self.files.is_empty() {
                ui.label("📁 拖拽文件到此处");
            } else {
                ui.label(format!("📁 {} 个文件", self.files.len()));
            }

            if !self.files.is_empty() && ui.button("清空").clicked() {
                self.clear();
            }
        });
    }

    /// 显示文件列表（改进版：带压缩率颜色区分）
    pub fn show_file_list(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                if self.files.is_empty() {
                    ui.allocate_ui_with_layout(
                        ui.available_size(),
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            ui.label(egui::RichText::new("暂无文件").size(14.0));
                        },
                    );
                    return;
                }

                for file in &self.files {
                    ui.horizontal(|ui| {
                        // 图标
                        let icon = match file.status {
                            FileStatus::Pending => "⏳",
                            FileStatus::Processing => "🔄",
                            FileStatus::Completed => "✅",
                            FileStatus::Skipped => "⏭️",
                            FileStatus::Error(_) => "❌",
                        };
                        ui.label(icon);

                        // 文件名
                        ui.label(&file.name);

                        // 原始大小
                        ui.label(&file.size);

                        // 压缩结果
                        match file.status {
                            FileStatus::Completed => {
                                if let Some(new_size) = file.new_size {
                                    let pct = file.savings_percent();
                                    let color = file.savings_color();
                                    let sign = if pct >= 0.0 { "-" } else { "+" };
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "→ {} ({sign}{:.0}%)",
                                            format_size(new_size),
                                            pct.abs()
                                        ))
                                        .color(color)
                                        .monospace(),
                                    );
                                }
                            }
                            FileStatus::Error(ref err) => {
                                ui.label(
                                    egui::RichText::new(format!("Error: {}", err))
                                        .color(egui::Color32::from_rgb(239, 68, 68))
                                        .monospace(),
                                );
                            }
                            FileStatus::Processing => {
                                ui.label(
                                    egui::RichText::new("...")
                                        .color(egui::Color32::from_rgb(251, 191, 36)),
                                );
                            }
                            FileStatus::Pending => {
                                ui.label(
                                    egui::RichText::new("等待处理")
                                        .color(egui::Color32::from_rgb(156, 163, 175)),
                                );
                            }
                            FileStatus::Skipped => {
                                ui.label(
                                    egui::RichText::new("已跳过")
                                        .color(egui::Color32::from_rgb(156, 163, 175)),
                                );
                            }
                        }
                    });
                }

                // 总计摘要行
                ui.separator();

                let completed = self.files.iter().filter(|f| f.status == FileStatus::Completed).count();
                let failed = self.files.iter().filter(|f| matches!(f.status, FileStatus::Error(_))).count();
                let total_original: u64 = self.files.iter().map(|f| f.original_size).sum();
                let total_new: u64 = self.files.iter()
                    .filter_map(|f| f.new_size)
                    .sum();
                let total_saved = total_original.saturating_sub(total_new);

                if completed > 0 {
                    ui.horizontal(|ui| {
                        ui.strong("总计:");
                        ui.label(format!("原始 {} → 压缩后 {}", format_size(total_original), format_size(total_new)));
                        let pct = if total_original > 0 {
                            format!("{:.1}%", (total_saved as f64 / total_original as f64) * 100.0)
                        } else {
                            "0.0%".to_string()
                        };
                        ui.label(egui::RichText::new(format!("节省 {}", format_size(total_saved))).color(egui::Color32::from_rgb(34, 197, 94)));
                        ui.label(egui::RichText::new(pct).color(egui::Color32::from_rgb(34, 197, 94)));
                    });
                }

                if failed > 0 {
                    ui.label(egui::RichText::new(format!("⚠ {} 个文件处理失败", failed)).color(egui::Color32::from_rgb(239, 68, 68)));
                }
            });
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn paths(&self) -> Vec<PathBuf> {
        self.files.iter().map(|f| f.path.clone()).collect()
    }

    /// 更新文件状态
    pub fn update_status(&mut self, path: &PathBuf, status: FileStatus, new_size: Option<u64>) {
        if let Some(file) = self.files.iter_mut().find(|f| &f.path == path) {
            file.status = status;
            file.new_size = new_size;
        }
    }
}

impl Default for FilePanel {
    fn default() -> Self {
        Self::new()
    }
}
