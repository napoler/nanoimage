//! NanoImage GUI 应用
mod ui;

use eframe::egui;
use nanoimage_core::{BatchProcessor, OptimizerConfig, Progress};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use ui::{file_panel::FilePanel, settings_panel::SettingsPanel, progress::ProgressPanel, log_view::LogPanel};

/// 消息类型：后台线程向UI线程发送的消息
enum WorkerMsg {
    Progress(Progress),
    Done(u64), // total_saved
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
            current_file: String::new(),
            worker_rx: None,
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
            let total_saved = processor.process_sync_with_progress(&files, &|progress| {
                tx.send(WorkerMsg::Progress(progress)).ok();
                // 发送每个文件的结果
            });

            tx.send(WorkerMsg::Done(total_saved)).ok();
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
                    WorkerMsg::Done(total_saved) => {
                        self.processing = false;
                        self.progress = 100.0;
                        self.progress_panel.reset();
                        self.log_panel.success(format!("处理完成! 共节省 {}", format_size(total_saved)));
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
            self.config = self.settings_panel.config().clone();

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