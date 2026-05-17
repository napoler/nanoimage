//! 文件面板 - 文件拖拽和列表显示
use eframe::egui;
use std::path::PathBuf;
use nanoimage_core::FileStatus;

/// 格式化文件大小
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
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let size = path.metadata()
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

    pub fn savings_percent(&self) -> f64 {
        match self.new_size {
            Some(new) if self.original_size > 0 => {
                ((self.original_size as f64 - new as f64) / self.original_size as f64) * 100.0
            }
            _ => 0.0,
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
        // 简单提示区域
        ui.horizontal(|ui| {
            if self.files.is_empty() {
                ui.label("📁 拖拽文件到此处");
            } else {
                ui.label(format!("📁 {} 个文件", self.files.len()));
            }

            if !self.files.is_empty()
                && ui.button("清空").clicked() {
                    self.clear();
                }
        });
    }

    /// 显示文件列表
    pub fn show_file_list(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for file in &self.files {
                    ui.horizontal(|ui| {
                        let icon = match file.status {
                            FileStatus::Pending => "⏳",
                            FileStatus::Processing => "🔄",
                            FileStatus::Completed => "✅",
                            FileStatus::Skipped => "⏭️",
                            FileStatus::Error(_) => "❌",
                        };
                        ui.label(icon);
                        ui.label(&file.name);
                        ui.label(&file.size);
                        if let Some(new_size) = file.new_size {
                            ui.label(format!("→ {} (-{:.*}%)", format_size(new_size), 1, file.savings_percent()));
                        }
                    });
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