//! 批量处理器
use crate::config::OptimizerConfig;
use crate::optimizer::Optimizer;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;

/// 检查文件是否已被优化（通过比较文件修改时间与文件名）
/// 如果文件名包含 "_optimized" 后缀，则认为已优化。
fn is_already_optimized(path: &Path) -> bool {
    // 检查文件名是否包含优化标记
    path.file_stem()
        .map(|s| s.to_string_lossy().contains("_optimized"))
        .unwrap_or(false)
}

/// 进度信息
#[derive(Debug, Clone)]
pub struct Progress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub bytes_processed: u64,
    pub bytes_saved: u64,
}

impl Progress {
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }
}

/// 批量处理器
pub struct BatchProcessor {
    optimizer: Arc<Optimizer>,
}

impl BatchProcessor {
    pub fn new(optimizer: Optimizer) -> Self {
        Self {
            optimizer: Arc::new(optimizer),
        }
    }

    pub fn with_config(config: OptimizerConfig) -> Self {
        Self::new(Optimizer::new(config))
    }

    /// 串行处理 (同步)
    pub fn process_sync(&self, files: &[PathBuf]) -> Vec<crate::optimizer::ProcessResult> {
        files
            .iter()
            .map(|path| self.optimizer.process_file(path))
            .collect()
    }

    /// 串行处理 (同步) 带跳过失败文件和仅处理未优化文件的选项
    pub fn process_sync_with_options(
        &self,
        files: &[PathBuf],
        skip_failed: bool,
        only_unoptimized: bool,
    ) -> (Vec<crate::optimizer::ProcessResult>, usize) {
        let mut results = Vec::with_capacity(files.len());
        let mut failed_count = 0;

        for file in files {
            // 检查是否已优化
            if only_unoptimized && is_already_optimized(file) {
                eprintln!("跳过已优化文件: {}", file.display());
                continue;
            }

            let result = self.optimizer.process_file(file);

            if result.success {
                results.push(result);
            } else if skip_failed {
                eprintln!(
                    "跳过失败文件 {}: {}",
                    file.display(),
                    result.error.as_ref().unwrap_or(&String::from("未知错误"))
                );
                failed_count += 1;
            } else {
                results.push(result);
            }
        }

        (results, failed_count)
    }

    /// 串行处理 (同步) 带进度回调
    pub fn process_sync_with_progress<F>(&self, files: &[PathBuf], on_progress: F) -> u64
    where
        F: Fn(Progress),
    {
        let total = files.len();
        let mut total_saved: u64 = 0;

        for (idx, path) in files.iter().enumerate() {
            let result = self.optimizer.process_file(path);
            let progress = Progress {
                current: idx + 1,
                total,
                current_file: path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                bytes_processed: result.original_size,
                bytes_saved: result.savings.max(0) as u64,
            };
            on_progress(progress);

            if result.success {
                total_saved += result.savings.max(0) as u64;
            }
        }

        total_saved
    }

    /// 串行处理 (同步) 返回进度和结果
    pub fn process_sync_with_results<F>(
        &self,
        files: &[PathBuf],
        on_progress: F,
    ) -> (u64, Vec<crate::optimizer::ProcessResult>)
    where
        F: Fn(Progress),
    {
        let total = files.len();
        let mut total_saved: u64 = 0;
        let mut results = Vec::with_capacity(files.len());

        for (idx, path) in files.iter().enumerate() {
            let result = self.optimizer.process_file(path);
            let progress = Progress {
                current: idx + 1,
                total,
                current_file: path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                bytes_processed: result.original_size,
                bytes_saved: result.savings.max(0) as u64,
            };
            on_progress(progress);

            if result.success {
                total_saved += result.savings.max(0) as u64;
            }
            results.push(result);
        }

        (total_saved, results)
    }

    /// 并行处理 (tokio)
    pub async fn process_async(
        &self,
        files: &[PathBuf],
        progress_tx: mpsc::Sender<Progress>,
    ) -> Vec<crate::optimizer::ProcessResult> {
        let files = files.to_vec();
        let optimizer = self.optimizer.clone();
        let total = files.len();

        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.optimizer.config().workers));
        let mut handles = Vec::with_capacity(files.len());

        for (idx, file) in files.into_iter().enumerate() {
            // Acquire semaphore permit; if semaphore is closed (shutdown), skip this file
            let Ok(permit) = semaphore.clone().acquire_owned().await else {
                continue;
            };
            let optimizer = optimizer.clone();
            let progress_tx = progress_tx.clone();

            let handle = tokio::spawn(async move {
                let result = optimizer.process_file(&file);
                let progress = Progress {
                    current: idx + 1,
                    total,
                    current_file: file
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    bytes_processed: result.original_size,
                    bytes_saved: result.savings.max(0) as u64,
                };
                let _ = progress_tx.send(progress).await;
                drop(permit);
                result
            });
            handles.push(handle);
        }

        let mut results = Vec::with_capacity(handles.len());
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(crate::optimizer::ProcessResult {
                        original_path: PathBuf::from("unknown"),
                        output_path: PathBuf::from("unknown"),
                        original_size: 0,
                        new_size: 0,
                        savings: 0,
                        success: false,
                        error: Some(format!("Task panicked: {}", e)),
                    });
                }
            }
        }
        results
    }

    /// 收集目录下所有图片文件
    pub fn collect_images(dir: &Path, recursive: bool) -> Vec<PathBuf> {
        let mut images = Vec::new();
        let extensions = ["jpg", "jpeg", "png", "webp", "gif", "bmp", "svg"];

        if recursive {
            for entry in walkdir::WalkDir::new(dir)
                .follow_links(true)
                .max_depth(32)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if extensions.contains(&ext.to_str().unwrap_or("").to_lowercase().as_str())
                        {
                            images.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        } else {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if extensions
                                .contains(&ext.to_str().unwrap_or("").to_lowercase().as_str())
                            {
                                images.push(path);
                            }
                        }
                    }
                }
            }
        }

        images
    }
}
