# 构建指南

## 前置要求

- Rust 1.75+ (`rustup`)
- 构建 GUI: X11 开发库 (Linux) 或原生 macOS/Windows 环境

```bash
# Ubuntu/Debian
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxdamage-dev libxrandr-dev libxi-dev libasound2-dev libgtk-3-dev
```

## 构建命令

```bash
# 全部构建 (debug)
cargo build

# 全部构建 (release)
cargo build --release

# 仅 CLI
cargo build --release -p nanoimage-cli

# 仅 GUI
cargo build --release -p nanoimage-gui

# 仅 core
cargo build --release -p nanoimage-core
```

## 测试

```bash
# 全部测试
cargo test

# 仅 core 测试
cargo test -p nanoimage-core

# 基准测试
cargo bench
```

## 跨平台构建

```bash
# Linux musl (静态链接)
cargo build --release --target x86_64-unknown-linux-musl

# macOS Universal
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
lipo -create -output nanoimage-gui-universal \
    target/x86_64-apple-darwin/release/nanoimage-gui \
    target/aarch64-apple-darwin/release/nanoimage-gui

# Windows
cargo build --release --target x86_64-pc-windows-msvc
```
