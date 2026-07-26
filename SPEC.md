# NanoImage - Rust + egui 图像优化器重构方案

> **版本**: v0.1.0
> **项目**: NanoImage (TerryOptImg 重构版)
> **目标**: 性能优先 + 保留CLI + 完整重写

---

## 一、需求锚定

| 需求 | 说明 |
|------|------|
| **性能追求** | Rust 原生实现，目标 5-10x 于 Python/Pillow |
| **保留CLI** | `nanoimage compress/batch/convert` 命令行工具 |
| **完整重写** | 不依赖 Python，全部 Rust 实现 |
| **跨平台** | Windows/Linux/macOS |

---

## 二、技术选型

### 2.1 核心依赖

| 功能 | Crate | 说明 |
|------|-------|------|
| 图像加载/保存 | `image` | 基础格式支持 (JPG/PNG/WebP/GIF/BMP) |
| PNG压缩 | `oxipng` | Rust 实现，接近 pngquant 效果 |
| JPG压缩 | `image` crate | `JpegEncoder`，质量可调 |
| WebP编码 | `webp` | 官方 WebP 编码器 Rust 绑定 |
| SVG处理 | std::fs copy | SVG 直通（验证格式后原样复制） |
| CLI解析 | `clap` | 结构化命令行参数 |
| GUI框架 | `egui` + `eframe` | 即时模式跨平台 UI |
| 异步运行时 | `tokio` | 多线程并发处理 |
| 配置管理 | `serde` + `serde_json` | 配置序列化 |
| 目录遍历 | `walkdir` | 递归目录遍历 |
| 配置目录 | `dirs` | 跨平台配置目录定位 |
| 文件对话框 | `rfd` | GUI 原生文件选择对话框 |

### 2.2 不采用方案

| 方案 | 原因 |
|------|------|
| `imageoptim` sys crate | 需要系统安装 imageoptim CLI |
| `slimg` 直接用 | 功能接近但架构不适配我们需要的多crate |
| `bevy_ui` | 过于重量级，适合游戏不适合工具应用 |

---

## 三、架构设计

```
nanoimage/
├── Cargo.toml # Workspace 配置
├── SPEC.md # 本文档
├── CONTRIBUTING.md # 贡献指南
├── CHANGELOG.md # 版本变更
│
├── crates/
│   ├── nanoimage-core/ # 核心库（共享）
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs        # 公共类型导出
│   │   │   ├── optimizer.rs  # 图像优化引擎 + 格式处理器
│   │   │   ├── processor.rs  # 批量处理逻辑 + is_already_optimized
│   │   │   ├── formats.rs    # 格式检测/转换 + ImageInfo
│   │   │   └── config.rs     # 配置结构
│   │   ├── tests/            # 集成测试
│   │   └── benches/          # 性能基准测试
│   │
│   ├── nanoimage-cli/ # CLI 工具
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs       # 入口
│   │       └── commands/     # 子命令
│   │           ├── compress.rs
│   │           ├── batch.rs
│   │           ├── convert.rs
│   │           ├── config_cmd.rs
│   │           ├── common.rs
│   │           ├── output.rs
│   │           └── mod.rs
│   │
│   └── nanoimage-gui/ # GUI 应用
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs          # eframe 入口
│           ├── lib.rs           # 应用状态 + UI 组合
│           ├── config_persistence.rs
│           └── ui/              # UI 组件
│               ├── mod.rs
│               ├── file_panel.rs
│               ├── settings_panel.rs
│               ├── progress.rs
│               └── log_view.rs
│
└── .github/workflows/ci.yml # CI 自动化
```

### 3.1 数据流

```
┌─────────────────────────────────────────────────────────────┐
│ nanoimage-gui │
│ ┌─────────┐ ┌─────────────┐ ┌──────────────────┐ │
│ │ 文件选择 │ -> │ AppState │ -> │ Worker (tokon) │ │
│ │ 拖拽接收 │ │ (文件列表) │ │ (后台处理) │ │
│ └─────────┘ └─────────────┘ └────────┬─────────┘ │
│ │ │ │
│ ┌─────────────────────────────────────────────▼─────────┐
│ │ nanoimage-core │
│ │ ┌─────────┐ ┌──────────┐ ┌─────────┐ ┌────────┐ │ │
│ │ │ image │ │ oxipng │ │ image │ │ webp │ │ │
│ │ │ (加载) │ │ (PNG) │ │ (JPG) │ │ (WebP) │ │ │
│ │ └─────────┘ └──────────┘ └─────────┘ └────────┘ │ │
│ └──────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ nanoimage-cli │
│ $ nanoimage batch -i ./photos -o ./optimized -q 85 │ │
│ │ │
│ ▼ │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ nanoimage-core │ │
│ │ (与 GUI 完全共享) │ │
│ └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## 四、核心模块设计

### 4.1 nanoimage-core 类型定义

```rust
// 核心优化器
pub struct Optimizer {
    config: OptimizerConfig,
}

impl Optimizer {
    pub fn new(config: OptimizerConfig) -> Self;
    pub fn with_default() -> Self;
    pub fn process_file(&self, path: &Path) -> ProcessResult;
    pub fn config(&self) -> &OptimizerConfig;
    pub fn set_config(&mut self, config: OptimizerConfig);
    fn process_jpeg(&self, input: &Path, output: &Path) -> anyhow::Result<()>;
    fn process_png(&self, input: &Path, output: &Path) -> anyhow::Result<()>;
    fn process_webp(&self, input: &Path, output: &Path) -> anyhow::Result<()>;
    fn process_gif(&self, input: &Path, output: &Path) -> anyhow::Result<()>;
    fn process_svg(&self, input: &Path, output: &Path) -> anyhow::Result<()>;
    fn process_bmp(&self, input: &Path, output: &Path) -> anyhow::Result<()>;
}

// 配置结构
#[derive(Serialize, Deserialize)]
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
    pub workers: usize,  // default: available_parallelism().min(16)
}

// 质量配置
#[derive(Serialize, Deserialize)]
pub struct Quality {
    pub lossy: u8,    // 有损质量 1-100 (默认 85)
    pub lossless: u8, // 无损等级 0-8 (默认 100)
}

// 压缩模式
pub enum CompressionMode {
    Lossy,    // 有损压缩
    Lossless, // 无损压缩
    Smart,    // 根据格式自动选择
}

// 输出格式
pub enum OutputFormat {
    KeepOriginal,  // 保持原格式
    Jpeg, Png, WebP, Gif,
}

// 处理结果
pub struct ProcessResult {
    pub original_path: PathBuf,
    pub output_path: PathBuf,
    pub original_size: u64,
    pub new_size: u64,
    pub savings: i64,  // 负数表示文件变大
    pub success: bool,
    pub error: Option<String>,
}

// 检查文件是否已被优化 (segment-based, 否定前缀敏感)
pub fn is_already_optimized(path: &Path) -> bool;

// 批量处理器（独立于 Optimizer）
pub struct BatchProcessor {
    optimizer: Arc<Optimizer>,
}

impl BatchProcessor {
    pub fn new(optimizer: Optimizer) -> Self;
    pub fn with_config(config: OptimizerConfig) -> Self;
    pub fn process_sync(&self, files: &[PathBuf]) -> Vec<ProcessResult>;
    pub fn process_sync_with_options(&self, files: &[PathBuf], skip_failed: bool, only_unoptimized: bool) -> (Vec<ProcessResult>, usize);
    pub fn process_sync_with_progress<F: Fn(Progress)>(&self, files: &[PathBuf], on_progress: F) -> u64;
    pub fn process_sync_with_results<F: Fn(Progress)>(&self, files: &[PathBuf], on_progress: F) -> (u64, Vec<ProcessResult>);
    pub async fn process_async(&self, files: &[PathBuf], progress_tx: mpsc::Sender<Progress>) -> Vec<ProcessResult>;
    pub fn collect_images(dir: &Path, recursive: bool) -> Vec<PathBuf>;
}

// 进度信息
pub struct Progress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub bytes_processed: u64,
    pub bytes_saved: u64,
}

// 图像格式检测
pub enum ImageFormat { Jpeg, Png, WebP, Gif, Bmp, Svg, Unknown }

// 文件处理状态
pub enum FileStatus { Pending, Processing, Completed, Skipped, Error(String) }
```

### 4.2 图像处理策略

| 格式 | 策略 |
|------|------|
| JPEG | `image` crate `JpegEncoder` + quality |
| PNG | oxipng optimize_from_memory |
| WebP | `webp` crate Encoder::from_rgba |
| GIF | `image` crate 直接保存 |
| SVG | 验证 `<svg>` 标签后原样复制 |
| BMP | 加载后重编码为 PNG (PngEncoder + Adaptive filter) |

### 4.3 并发模型

```rust
// BatchProcessor::process_async — tokio 并行处理
// 使用 Arc<Semaphore> 控制并发数 (config.workers)
// 每个文件 spawn 独立 task，进度通过 mpsc::Sender<Progress> 传递
// 返回值: Vec<ProcessResult> (含 panic 安全回退)
```

---

## 五、GUI 设计 (egui)

### 5.1 主界面布局

```
┌─────────────────────────────────────────────────────────────┐
│ NanoImage [_][□][X] │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐ │
│ │ │ │
│ │ 📁 拖拽文件到此处 │ │
│ │ │ │
│ │ [ 添加文件 ] [ 添加文件夹 ] │ │
│ │ │ │
│ └─────────────────────────────────────────────────────┘ │
│ │
│ 文件: 12 个 (共 45.2 MB) [ 清空列表 ] │
│ │
│ ┌─ 设置 ────────────────────────────────────────────────┐ │
│ │ 质量: [=======|=====] 85% 格式: [保持原格式 ▼] │ │
│ │ 最大尺寸: [ 2048 ] px 线程: [ 8 ▼] │ │
│ │ ☑ 保留元数据 ☑ 覆盖源文件 ☐ 无损模式 │ │
│ └──────────────────────────────────────────────────────┘ │
│ │
│ [ ▶ 开始优化 ] 进度: 45% ████████░░░ │ │
│ │
│ ┌─ 日志 ────────────────────────────────────────────────┐ │
│ │ 12:30:01 完成 image1.jpg (2.1MB → 890KB, -58%) │ │
│ │ 12:30:02 完成 image2.png (5.4MB → 1.2MB, -78%) │ │
│ │ 12:30:03 跳过 image3.gif (已优化) │ │
│ └──────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 状态管理

```rust
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

    // worker 线程 channel
    worker_rx: Option<Receiver<WorkerMsg>>,
    worker_handle: Option<JoinHandle<()>>,
    cancel_flag: Arc<AtomicBool>,

    // 完成状态
    show_completion_dialog: bool,
    total_saved_bytes: u64,

    // 配置防抖保存
    config_dirty: bool,
}
```

快捷键: Ctrl+Enter (开始), Ctrl+O (添加文件), Esc (取消)

---

## 六、CLI 设计 (clap)

### 6.1 命令结构

```bash
# 压缩单个文件
nanoimage compress input.jpg -o output/ -q 85

# 批量处理
nanoimage batch -i ./photos -o ./optimized --quality 85 --workers 8

# 格式转换
nanoimage convert input.png -o output.webp --format webp

# 查看帮助
nanoimage --help
nanoimage compress --help
```

### 6.2 子命令定义

| 命令 | 说明 | 关键参数 |
|------|------|----------|
| `compress` | 单文件压缩 | input, output, quality, format |
| `batch` | 批量处理 | input-dir, output-dir, quality, workers, recursive |
| `convert` | 格式转换 | input, output, format, quality |

---

## 七、性能基准

### 7.1 预期收益

| 场景 | Python/Pillow | Rust/NanoImage | 加速比 |
|------|---------------|----------------|--------|
| JPEG 压缩 | ~200ms | ~30ms | **6-7x** |
| PNG 压缩 | ~500ms | ~80ms | **6x** |
| 批量 100 图 | ~45s | ~8s | **5-6x** |
| 内存峰值 | ~800MB | ~200MB | **4x** |

### 7.2 基准测试

```rust
// benches/optimizer.rs
use criterion::{black_box, criterion_group, Criterion};

fn bench_jpeg_compress(c: &mut Criterion) {
    let path = PathBuf::from("test_data/sample.jpg");
    let optimizer = Optimizer::new(Default::default());

    c.bench_function("jpeg_compress_1mb", |b| {
        b.iter(|| optimizer.process_file(black_box(&path)))
    });
}
```

---

## 八、构建与发布

### 8.1 构建目标

| 平台 | 方式 | 输出 |
|------|------|------|
| Linux | `cargo build --release` | `nanoimage` 二进制 |
| Windows | `cross build --target x86_64-pc-windows-msvc` | `nanoimage.exe` |
| macOS | `cross build --target x86_64-apple-darwin` | `nanoimage` |

### 8.2 依赖打包

```toml
[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "target-feature=-crt-static"]
```

---

## 九、实现计划

| 迭代 | 任务 | 验证方式 |
|------|------|----------|
| 0 | ~~Bug 修复 + 警告清理~~ | ~~`cargo build` 零警告, 72 tests pass~~ ✅ |
| 1 | ~~补充核心测试 (72 tests)~~ | ~~`cargo test` 72 tests pass~~ ✅ |
| 2 | ~~CLI batch 完善 (--format/--max-width/--max-height/--dry-run/结果表格)~~ | ~~`cargo build` 零警告, 72 tests pass~~ ✅ |
| 3 | ~~GUI 集成完善~~ | ~~处理完成弹窗通知 + 输出目录选择 + 文件列表摘要行 + 跳过失败选项~~ ✅ |
| 4 | ~~配置持久化~~ | ~~启动时加载上次配置~~ ✅ |
| 5 | ~~错误处理与用户体验~~ | ~~跳过失败文件选项 + 跳过已优化文件~~ ✅ |
| 6 | ~~性能基准测试~~ | ~~criterion 报告 (JPEG/PNG/WebP 各一个)~~ ✅ |

---

## 十、风险与应对

| 风险 | 严重度 | 应对 |
|------|--------|------|
| mozjpeg sys crate 绑定复杂 | 中 | 备选 `jpeg-encoder` crate |
| egui 文件拖拽 API 变化 | 低 | 锁定 egui 版本 |
| 跨平台二进制体积大 | 中 | 使用 `upx` 压缩 |
| 配置兼容旧版本 | 低 | V1 版本不考虑 |

---

## 十一、5个主动挑刺

| # | 问题 | 严重度 | 解决方案 | 优先级 |
|---|------|--------|----------|--------|
| 1 | CLI 和 GUI 共享 core 但依赖不同 | 中 | core 保持最小依赖，CLI/GUI 各自扩展 | P1 |
| 2 | mozjpeg 是 sys crate，需要系统库 | 高 | 使用 `mozjpeg` crate 的 bundled 特性，或切换 `jpeg-encoder` | P1 |
| 3 | 图像处理进度回调在 async 中复杂 | 中 | 使用 channel 传递进度，简单可靠 | P2 |
| 4 | egui 即时模式状态管理复杂度 | 中 | AppState 集中管理，UI 只负责渲染 | P2 |
| 5 | 批量处理时内存占用 | 中 | 流式处理 + 内存池，避免全部加载 | P3 |

---

*文档状态: 迭代 0-6 ✅ + 迭代 7 (Autopilot) ✅。编译零警告，83 tests pass，3 个 criterion 基准测试通过，DEB 包就绪。*
*完成度: 100% — 开发、测试、文档、打包、验证全部完成。*
