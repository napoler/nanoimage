# API 参考

## nanoimage-core

### Optimizer

```rust
pub struct Optimizer {
    config: OptimizerConfig,
}

impl Optimizer {
    pub fn new(config: OptimizerConfig) -> Self;
    pub fn with_default() -> Self;
    pub fn process_file(&self, path: &Path) -> ProcessResult;
    pub fn config(&self) -> &OptimizerConfig;
    pub fn set_config(&mut self, config: OptimizerConfig);
}
```

### OptimizerConfig

```rust
pub struct OptimizerConfig {
    pub mode: CompressionMode,       // Lossy | Lossless | Smart
    pub quality: Quality,             // lossy: u8, lossless: u8
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub format: OutputFormat,         // KeepOriginal | Jpeg | Png | WebP | Gif
    pub preserve_metadata: bool,
    pub overwrite: bool,
    pub output_dir: Option<PathBuf>,
    pub skip_failed: bool,
    pub workers: usize,
}
```

### BatchProcessor

```rust
pub struct BatchProcessor {
    optimizer: Arc<Optimizer>,
}

impl BatchProcessor {
    pub fn new(optimizer: Optimizer) -> Self;
    pub fn with_config(config: OptimizerConfig) -> Self;
    pub fn process_sync(&self, files: &[PathBuf]) -> Vec<ProcessResult>;
    pub fn process_sync_with_progress<F>(&self, files: &[PathBuf], on_progress: F) -> u64;
    pub fn process_sync_with_results<F>(&self, files: &[PathBuf], on_progress: F) -> (u64, Vec<ProcessResult>);
    pub async fn process_async(&self, files: &[PathBuf], progress_tx: Sender<Progress>) -> Vec<ProcessResult>;
    pub fn collect_images(dir: &Path, recursive: bool) -> Vec<PathBuf>;
}
```

### ProcessResult

```rust
pub struct ProcessResult {
    pub original_path: PathBuf,
    pub output_path: PathBuf,
    pub original_size: u64,
    pub new_size: u64,
    pub savings: i64,
    pub success: bool,
    pub error: Option<String>,
}

impl ProcessResult {
    pub fn savings_percent(&self) -> f64;
}
```

### 工具函数

```rust
pub fn format_size(bytes: u64) -> String;
pub fn format_name(format: ImageFormat) -> &'static str;
```
