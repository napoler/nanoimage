//! NanoImage GUI 应用
mod ui;
mod config_persistence;

use eframe::egui;
use nanoimage_core::{format_size, BatchProcessor, FileStatus, OptimizerConfig, ProcessResult, Progress};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use ui::{file_panel::FilePanel, settings_panel::SettingsPanel, progress::ProgressPanel, log_view::LogPanel};

/// 消息类型：后台线程向UI线程发送的消息
enum WorkerMsg {
    Progress(Progress),
    Completed(Vec<ProcessResult>),
}

/// 主应用
pub struct NanoImageApp {
    config: OptimizerConfig,
    file_panel: FilePanel,
    settings_panel: SettingsPanel,
    progress_panel: ProgressPanel,
    log_panel: LogPanel,
    // 处理状态
    processing: bool,
    progress: f32,
    current_file: String,
    // worker 线程的 channel
    worker_rx: Option<Receiver<WorkerMsg>>,
    /// 处理完成是否已通知
    show_completion_dialog: bool,
    /// 完成时节省的总字节数
    total_saved_bytes: u64,
    /// 配置是否已修改（用于防抖自动保存）
    config_dirty: bool,
}

impl NanoImageApp {
    pub fn new() -> Self {
        let config = config_persistence::load_config();
        Self {
            settings_panel: SettingsPanel::with_config(config.clone()),
            file_panel: FilePanel::new(),
            progress_panel: ProgressPanel::new(),
            log_panel: LogPanel::new(),
            config,
            processing: false,
            progress: 0.0,
            current_file: String::new(),
            worker_rx: None,
            show_completion_dialog: false,
            total_saved_bytes: 0,
            config_dirty: false,
        }
    }

    /// 添加文件
    fn add_files(&mut self, paths: Vec<PathBuf>) {
        let image_paths: Vec<PathBuf> = paths.into_iter().filter(|p| {
            let ext = p.extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();
            matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" | "svg")
        }).collect();

        if !image_paths.is_empty() {
            self.file_panel.add_files(image_paths);
            let count = self.file_panel.len();
            self.log_panel.info(format!("添加了 {} 个文件", count));
        }
    }

    /// 保存配置到文件
    fn save_config(&self) {
        config_persistence::save_config(&self.config);
    }

    /// 处理文件 (使用 channel 与后台线程通信)
    fn process_files(&mut self) {
        if self.file_panel.is_empty() {
            return;
        }

        self.processing = true;
        self.progress = 0.0;
        self.current_file = String::new();

        let files = self.file_panel.paths();
        let total = files.len();
        let config = self.config.clone();

        self.log_panel.info(format!("开始处理 {} 个文件...", total));

        // 创建 channel 用于接收进度和结果
        let (tx, rx) = mpsc::channel::<WorkerMsg>();
        self.worker_rx = Some(rx);

        // 启动后台线程
        let _handle = thread::spawn(move || {
            let processor = BatchProcessor::with_config(config);
            let (_total_saved, results) = processor.process_sync_with_results(&files, &|progress| {
                tx.send(WorkerMsg::Progress(progress)).ok();
            });

            tx.send(WorkerMsg::Completed(results)).ok();
        });
    }

    /// 轮询处理 worker 消息
    fn poll_worker(&mut self) {
        if let Some(ref rx) = self.worker_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    WorkerMsg::Progress(progress) => {
                        self.progress = progress.percent() as f32;
                        self.current_file = progress.current_file.clone();
                        self.progress_panel.set_current_file(progress.current_file);

                        // 更新文件列表中对应文件的状态
                        let idx = progress.current.saturating_sub(1);
                        if idx < self.file_panel.files.len() {
                            // 获取 path 的克隆
                            let path = self.file_panel.files[idx].path.clone();
                            self.file_panel.update_status(&path, nanoimage_core::FileStatus::Processing, None);
                        }
                    }
                    WorkerMsg::Completed(results) => {
                        let mut total_saved: u64 = 0;
                        for result in results {
                            if result.success {
                                total_saved += result.savings.max(0) as u64;
                                self.file_panel.update_status(
                                    &result.original_path,
                                    FileStatus::Completed,
                                    Some(result.new_size),
                                );
                            } else {
                                // 如果 skip_failed 为 true，标记为 Skipped 而非 Error
                                let status = if self.config.skip_failed {
                                    FileStatus::Skipped
                                } else {
                                    FileStatus::Error(
                                        result.error.unwrap_or_else(|| "Unknown error".to_string()),
                                    )
                                };
                                self.file_panel.update_status(
                                    &result.original_path,
                                    status,
                                    None,
                                );
                            }
                        }
                        self.show_completion_dialog = true;
                        self.total_saved_bytes = total_saved;
                        self.processing = false;
                        self.progress = 100.0;
                        self.progress_panel.reset();
                        self.log_panel
                            .success(format!("处理完成! 共节省 {}", format_size(total_saved)));
                        self.worker_rx = None;
                        break;
                    }
                }
            }
        }

        // 如果处理完成且没有更多消息，清空 worker
        if !self.processing && self.worker_rx.is_some() {
            self.worker_rx = None;
        }
    }
}

impl eframe::App for NanoImageApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 轮询 worker 消息
        self.poll_worker();

        // 处理拖放
        let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped_files.is_empty() {
            let paths: Vec<PathBuf> = dropped_files.into_iter()
                .filter_map(|f| f.path)
                .collect();
            self.add_files(paths);
        }

        // 处理完成弹窗
        if self.show_completion_dialog {
            let total_saved = self.total_saved_bytes;
            let success_count = self.file_panel.files.iter()
                .filter(|f| f.status == FileStatus::Completed)
                .count();
            let total = self.file_panel.files.len();

            egui::Window::new("处理完成")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("✅ 处理完成!").heading());
                    ui.separator();
                    ui.label(format!("共 {} 个文件，成功 {} 个", total, success_count));
                    ui.label(format!("节省: {}", format_size(total_saved)));
                    ui.separator();
                    if ui.button("确定").clicked() {
                        self.show_completion_dialog = false;
                    }
                });

            self.show_completion_dialog = false;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("NanoImage");
            ui.label("图像优化器 - Rust + egui");

            ui.separator();

            // 文件面板
            self.file_panel.show(ui);
            self.file_panel.show_file_list(ui);

            ui.separator();

            // 设置面板
            self.settings_panel.show(ui);
            let new_config = self.settings_panel.config().clone();
            if new_config != self.config {
                self.config = new_config;
                self.config_dirty = true;
            }
            if self.config_dirty {
                self.save_config();
                self.config_dirty = false;
            }

            ui.separator();

            // 按钮行
            ui.horizontal(|ui| {
                let button_text = if self.processing { "处理中..." } else { "▶ 开始优化" };
                let can_start = !self.processing && !self.file_panel.is_empty();

                if ui.add_enabled(can_start, egui::Button::new(button_text)).clicked() {
                    self.process_files();
                }

                if ui.button("添加文件").clicked() {
                    if let Some(paths) = rfd::FileDialog::new().pick_files() {
                        self.add_files(paths);
                    }
                }

                if ui.button("添加文件夹").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        let files = BatchProcessor::collect_images(&path, true);
                        self.add_files(files);
                    }
                }
            });

            // 进度条
            if self.processing {
                ui.add(egui::ProgressBar::new(self.progress / 100.0)
                    .text(format!("{:.*}% - {}", 1, self.progress, self.current_file)));
            }

            ui.separator();

            // 进度面板
            self.progress_panel.show(ui, self.progress, self.processing);

            ui.separator();

            // 日志面板
            self.log_panel.show(ui);
        });

        // 处理进行中时请求继续渲染
        if self.processing {
            ctx.request_repaint();
        }
    }
}