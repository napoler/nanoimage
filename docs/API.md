# API 参考

## nanoimage-core

### Optimizer

主优化引擎。处理单个文件。

```rust
pub struct Optimizer {
    config: OptimizerConfig,
}

impl Optimizer {
    /// 使用指定配置创建优化器
    pub fn new(config: OptimizerConfig) -> Self;

    /// 使用默认配置创建优化器
    pub fn with_default() -> Self;

    /// 处理单个文件，返回结果
    pub fn process_file(&self, path: &Path) -> ProcessResult;

    /// 获取当前配置
    pub fn config(&self) -> &OptimizerConfig;

    /// 更新配置
    pub fn set_config(&mut self, config: OptimizerConfig);
}
```

### BatchProcessor

批量处理多个文件，支持同步和异步模式。

```rust
pub struct BatchProcessor {
    optimizer: Arc<Optimizer>,
}

impl BatchProcessor {
    /// 使用已有 Optimizer 创建
    pub fn new(optimizer: Optimizer) -> Self;

    /// 使用配置直接创建
    pub fn with_config(config: OptimizerConfig) -> Self;

    /// 同步处理文件列表
    pub fn process_sync(&self, files: &[PathBuf]) -> Vec<ProcessResult>;

    /// 同步处理 + 进度回调，返回总节省字节数
    pub fn process_sync_with_progress<F>(&self, files: &[PathBuf], on_progress: F) -> u64;

    /// 同步处理 + 进度回调，返回 (总节省字节, 详细结果列表)
    pub fn process_sync_with_results<F>(&self, files: &[PathBuf], on_progress: F) -> (u64, Vec<ProcessResult>);

    /// 异步处理 + 通过 channel 发送进度更新
    pub async fn process_async(&self, files: &[PathBuf], progress_tx: tokio::sync::mpsc::Sender<Progress>) -> Vec<ProcessResult>;

    /// 递归扫描目录收集图像文件
    pub fn collect_images(dir: &Path, recursive: bool) -> Vec<PathBuf>;
}
```

### ProcessResult

单文件处理结果。

```rust
pub struct ProcessResult {
    pub original_path: PathBuf,
    pub output_path: PathBuf,
    pub original_size: u64,
    pub new_size: u64,
    pub savings: i64,       // 负数表示文件变大
    pub success: bool,
    pub error: Option<String>,
}

impl ProcessResult {
    /// 计算节省百分比（可为负数）
    pub fn savings_percent(&self) -> f64;
}
```

### OptimizerConfig

优化配置。

```rust
pub struct OptimizerConfig {
    pub mode: CompressionMode,
    pub quality: Quality,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub format: OutputFormat,
    pub preserve_metadata: bool,
    pub overwrite: bool,
    pub output_dir: Option<PathBuf>,
    pub skip_failed: bool,
    pub workers: usize,
}
```

### Quality

质量配置。

```rust
pub struct Quality {
    pub lossy: u8,      // 有损质量 1-100
    pub lossless: u8,   // 无损优化等级 0-8
}
```

### CompressionMode

压缩模式。

```rust
pub enum CompressionMode {
    Lossy,     // 有损压缩
    Lossless,  // 无损压缩
    Smart,     // 智能模式（自动选择）
}
```

### OutputFormat

输出格式。

```rust
pub enum OutputFormat {
    KeepOriginal,
    Jpeg,
    Png,
    WebP,
    Gif,
}
```

### ImageFormat

检测到的图像格式。

```rust
pub enum ImageFormat {
    Jpeg, Png, WebP, Gif, Bmp, Svg, Unknown,
}

impl ImageFormat {
    /// 从文件路径扩展名推断格式
    pub fn from_path(path: &Path) -> Self;
}
```

### Progress

处理进度信息。

```rust
pub struct Progress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub bytes_processed: u64,
    pub bytes_saved: u64,
}

impl Progress {
    /// 计算完成百分比
    pub fn percent(&self) -> f32;
}
```

### 工具函数

```rust
/// 格式化字节大小为人类可读字符串 (e.g., "1.5 MB")
pub fn format_size(bytes: u64) -> String;

/// 从扩展名检测图像格式
pub fn detect_format(path: &Path) -> ImageFormat;
```
