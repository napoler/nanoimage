//! 批量处理器
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::optimizer::Optimizer;
use crate::config::OptimizerConfig;

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
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let optimizer = optimizer.clone();
            let progress_tx = progress_tx.clone();

            let handle = tokio::spawn(async move {
                let result = optimizer.process_file(&file);
                let progress = Progress {
                    current: idx + 1,
                    total,
                    current_file: file.file_name().unwrap_or_default().to_string_lossy().to_string(),
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
            if let Ok(result) = handle.await {
                results.push(result);
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
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if extensions.contains(&ext.to_str().unwrap_or("").to_lowercase().as_str()) {
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
                            if extensions.contains(&ext.to_str().unwrap_or("").to_lowercase().as_str()) {
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