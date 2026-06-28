# NanoImage

> 高性能图像优化工具 — Rust + egui，支持 JPEG/PNG/WebP 批量压缩与格式转换

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey)

## 快速开始

```bash
# 批量压缩目录下所有图片
nanoimage batch -i ./photos -o ./optimized -q 85

# 压缩单个文件
nanoimage compress -i photo.jpg -o output/ -q 90

# 格式转换（PNG → WebP）
nanoimage convert -i image.png -o image.webp --format webp

# 查看/修改配置
nanoimage settings --show
nanoimage settings --quality 90 --workers 8
```

## 功能特性

| 功能 | CLI | GUI | 说明 |
|------|-----|-----|------|
| JPEG 压缩 | ✅ | ✅ | mozjpeg 质量可调 |
| PNG 优化 | ✅ | ✅ | oxipng Zopfli 压缩 |
| WebP 转换 | ✅ | ✅ | RGBA 支持 |
| GIF 处理 | ✅ | ✅ | 基础压缩 |
| 批量处理 | ✅ | ✅ | 递归目录扫描 |
| 格式转换 | ✅ | ✅ | 跨格式互转 |
| 配置持久化 | ✅ | ✅ | 自动保存/导入导出 |
| 进度显示 | ✅ | ✅ | 实时进度条 |
| 跳过失败 | ✅ | ✅ | 容错处理 |

## 安装

### Debian/Ubuntu

```bash
sudo dpkg -i nanoimage_0.1.0_amd64.deb
nanoimage --help
```

### 从源码构建

```bash
git clone <repo-url>
cd nanoimage
cargo build --release
sudo cp target/release/nanoimage /usr/local/bin/
```

### Windows

下载 [release 中的 .exe](https://github.com/napoler/nanoimage/releases) 或使用源码构建：

```bash
cargo build --release
# 输出: target/release/nanoimage.exe
```

### macOS

```bash
cargo build --release
# GUI: target/release/nanoimage-gui
# CLI: target/release/nanoimage
```

## 详细文档

- [使用手册](docs/USAGE.md) — CLI 和 GUI 完整使用指南
- [API 参考](docs/API.md) — 核心库 API 文档
- [构建指南](docs/BUILD.md) — 跨平台构建和打包说明
- [变更日志](CHANGELOG.md) — 版本更新记录

## 架构

```
nanoimage/
├── crates/
│   ├── nanoimage-core/    # 核心库（共享优化引擎）
│   ├── nanoimage-cli/     # 命令行工具
│   └── nanoimage-gui/     # GUI 应用 (egui)
├── tests/                  # 集成测试
└── benches/               # 性能基准测试
```

## 性能

| 场景 | 预期压缩率 |
|------|-----------|
| JPEG 照片 | 30-70% |
| PNG 图形 | 50-95% |
| WebP 转换 | 25-50% |

## 许可证

[MIT License](LICENSE)
