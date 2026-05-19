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
| JPG压缩 | `mozjpeg` (sys) | MozJPEG 绑定，高质量有损压缩 |
| WebP编码 | `webp` | 官方 WebP 编码器 Rust 绑定 |
| SVG优化 | `resvg` + `svgo-api` | 矢量图处理 |
| CLI解析 | `clap` | 结构化命令行参数 |
| GUI框架 | `egui` + `eframe` | 即时模式跨平台 UI |
| 异步运行时 | `tokio` | 多线程并发处理 |
| 配置管理 | `serde` + `serde_json` | 配置序列化 |

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
├── Cargo.toml          # Workspace 配置
├── SPEC.md             # 本文档
│
├── crates/
│   ├── nanoimage-core/       # 核心库（共享）
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── optimizer.rs   # 图像优化引擎
│   │   │   ├── processor.rs   # 批量处理逻辑
│   │   │   ├── formats.rs     # 格式检测/转换
│   │   │   └── config.rs      # 配置结构
│   │   └── benches/          # 性能基准测试
│   │
│   ├── nanoimage-cli/         # CLI 工具
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs        # 入口
│   │       ├── commands/      # 子命令
│   │       │   ├── compress.rs
│   │       │   ├── batch.rs
│   │       │   └── convert.rs
│   │       └── output.rs      # 彩色输出
│   │
│   └── nanoimage-gui/         # GUI 应用
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs        # eframe 入口
│           ├── app.rs         # 主应用状态
│           ├── ui/            # UI 组件
│           │   ├── file_panel.rs
│           │   ├── settings_panel.rs
│           │   ├── progress.rs
│           │   └── log_view.rs
│           └── i18n.rs        # 国际化
│
├── assets/                    # 资源文件
│   └── icons/
│
└── tests/                     # 集成测试
```

### 3.1 数据流

```
┌─────────────────────────────────────────────────────────────┐
│                        nanoimage-gui                        │
│  ┌─────────┐    ┌─────────────┐    ┌──────────────────┐   │
│  │ 文件选择 │ -> │ AppState    │ -> │ Worker (tokio)   │   │
│  │ 拖拽接收 │    │ (文件列表)  │    │ (后台处理)       │   │
│  └─────────┘    └─────────────┘    └────────┬─────────┘   │
│                                              │              │
│  ┌─────────────────────────────────────────────▼─────────┐ │
│  │                    nanoimage-core                      │ │
│  │  ┌─────────┐  ┌──────────┐  ┌─────────┐  ┌────────┐ │ │
│  │  │ image   │  │ oxipng   │  │ mozjpeg │  │ webp   │ │ │
│  │  │ (加载)  │  │ (PNG)    │  │ (JPG)   │  │ (WebP) │ │ │
│  │  └─────────┘  └──────────┘  └─────────┘  └────────┘ │ │
│  └──────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                        nanoimage-cli                        │
│  $ nanoimage batch -i ./photos -o ./optimized -q 85       │
│                           │                                │
│                           ▼                                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    nanoimage-core                   │   │
│  │              (与 GUI 完全共享)                       │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 四、核心模块设计

### 4.1 nanoimage-core

```rust
// 核心优化器
pub struct Optimizer {
    config: OptimizerConfig,
    // 内部工具检测
}

impl Optimizer {
    pub fn process_file(&self, path: &Path) -> Result<ProcessResult>;
    pub fn process_batch(&self, paths: &[Path]) -> Vec<ProcessResult>;
}

// 配置结构
#[derive(Serialize, Deserialize)]
pub struct OptimizerConfig {
    pub quality: u8,          // 1-100
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
    pub format: OutputFormat,
    pub preserve_metadata: bool,
    pub workers: usize,
}

// 处理结果
pub struct ProcessResult {
    pub original_path: PathBuf,
    pub output_path: PathBuf,
    pub original_size: u64,
    pub new_size: u64,
    pub savings: u64,
    pub success: bool,
    pub error: Option<String>,
}
```

### 4.2 图像处理策略

| 格式 | 策略 |
|------|------|
| JPEG | mozjpeg 有损/无损 → `image` crate 保存 |
| PNG | oxipng 优化 (Zopfli+Delta) |
| WebP | `webp` crate 编码 |
| GIF | `image` crate 优化 |
| SVG | resvg 渲染 + svgo 优化 |
| BMP/TIFF | `image` crate 直通 |

### 4.3 并发模型

```rust
// 使用 tokio 进行并行处理
pub async fn process_batch_async(
    files: Vec<PathBuf>,
    config: &OptimizerConfig,
    progress_callback: impl Fn(Progress),
) -> Vec<ProcessResult> {
    let semaphore = Semaphore::new(config.workers);
    // 并发处理 + 进度回调
}
```

---

## 五、GUI 设计 (egui)

### 5.1 主界面布局

```
┌─────────────────────────────────────────────────────────────┐
│  NanoImage                              [_][□][X]           │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────┐    │
│  │                                                      │    │
│  │              📁 拖拽文件到此处                       │    │
│  │                                                      │    │
│  │         [ 添加文件 ]    [ 添加文件夹 ]              │    │
│  │                                                      │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
│  文件: 12 个 (共 45.2 MB)              [ 清空列表 ]        │
│                                                             │
│  ┌─ 设置 ────────────────────────────────────────────────┐  │
│  │  质量: [=======|=====] 85%   格式: [保持原格式 ▼]   │  │
│  │  最大尺寸: [ 2048 ] px      线程: [ 8  ▼]           │  │
│  │  ☑ 保留元数据    ☑ 覆盖源文件    ☐ 无损模式        │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                             │
│  [ ▶ 开始优化 ]                    进度: 45% ████████░░░   │
│                                                             │
│  ┌─ 日志 ────────────────────────────────────────────────┐  │
│  │ 12:30:01 完成 image1.jpg (2.1MB → 890KB, -58%)      │  │
│  │ 12:30:02 完成 image2.png (5.4MB → 1.2MB, -78%)     │  │
│  │ 12:30:03 跳过 image3.gif (已优化)                   │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 5.2 状态管理

egui 是即时模式，状态驱动 UI：

```rust
struct AppState {
    files: Vec<FileEntry>,           // 文件列表
    config: OptimizerConfig,         // 当前配置
    processing: ProcessingState,     // 处理状态
    log_entries: Vec<LogEntry>,      // 日志
    theme: Theme,                    // 主题
}

struct FileEntry {
    path: PathBuf,
    status: FileStatus,  // Pending/Processing/Done/Skipped/Error
    original_size: u64,
    new_size: Option<u64>,
}

enum ProcessingState {
    Idle,
    Running { progress: f32, current_file: String },
    Cancelled,
    Completed { total_saved: u64 },
}
```

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

## 九、实现计划 (20次迭代)

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

*文档状态: 迭代 0 ✅ + 迭代 1 ✅ + 迭代 2 ✅ + 迭代 3 ✅ + 迭代 4 ✅ + 迭代 5 ✅ + 迭代 6 ✅。编译零警告，72 tests pass，3 个 criterion 基准测试通过。*
*完成度: 迭代 7/7 (100%)*