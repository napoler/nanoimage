# NanoImage 使用手册

## CLI 使用指南

### 基本用法

```bash
nanoimage <command> [OPTIONS]
```

### 命令列表

#### 1. compress — 单文件压缩

压缩单个图像文件，支持 JPEG/PNG/WebP。

```bash
# 基本用法
nanoimage compress -i photo.jpg

# 指定输出目录
nanoimage compress -i photo.jpg -o ./optimized/

# 指定输出文件
nanoimage compress -i photo.jpg -o ./photo_compressed.jpg

# 覆盖源文件
nanoimage compress -i photo.jpg -w

# 自定义质量
nanoimage compress -i photo.jpg -q 95
```

**参数说明：**
| 参数 | 简写 | 默认值 | 说明 |
|------|------|--------|------|
| --input | -i | *(必填)* | 输入文件路径 |
| --output | -o | `./optimized/` | 输出文件或目录 |
| --quality | -q | 85 | 压缩质量 1-100 |
| --overwrite | -w | false | 覆盖源文件 |

#### 2. batch — 批量处理

批量处理目录中的所有图像文件。

```bash
# 基本批量处理
nanoimage batch -i ./photos -o ./optimized

# 递归处理子目录
nanoimage batch -i ./photos -o ./optimized -r

# 指定线程数
nanoimage batch -i ./photos -o ./optimized -w 8

# 转换为 WebP
nanoimage batch -i ./photos -o ./optimized -f webp

# 限制最大尺寸
nanoimage batch -i ./photos -o ./optimized --max-width 1920 --max-height 1080

# 预览模式（不实际处理）
nanoimage batch -i ./photos --dry-run

# 跳过失败文件
nanoimage batch -i ./photos -o ./optimized --skip-failed

# 仅处理未优化的文件
nanoimage batch -i ./photos -o ./optimized --only-unoptimized
```

**参数说明：**
| 参数 | 简写 | 默认值 | 说明 |
|------|------|--------|------|
| --input | -i | *(必填)* | 输入目录路径 |
| --output | -o | `./optimized/` | 输出目录 |
| --quality | -q | 85 | 压缩质量 1-100 |
| --workers | -w | 4 | 工作线程数 |
| --recursive | -r | false | 递归处理子目录 |
| --overwrite | -W | false | 覆盖源文件 |
| --format | -f | keeporiginal | 目标格式 |
| --max-width | | none | 最大宽度 |
| --max-height | | none | 最大高度 |
| --dry-run | | false | 预览模式 |
| --skip-failed | -s | false | 跳过失败文件 |
| --only-unoptimized | | false | 仅处理未优化文件 |

**支持的格式：** `jpg`, `png`, `webp`, `gif`, `keeporiginal`

#### 3. convert — 格式转换

将图像转换为另一种格式。

```bash
# PNG → WebP
nanoimage convert -i image.png -o image.webp --format webp

# JPEG → PNG
nanoimage convert -i photo.jpg -o photo.png --format png

# 转换时调整质量
nanoimage convert -i image.png -o image.webp --format webp -q 90
```

#### 4. settings — 配置管理

管理全局配置。

```bash
# 查看当前配置
nanoimage settings --show

# 设置质量
nanoimage settings --quality 90

# 设置线程数
nanoimage settings --workers 8

# 重置为默认配置
nanoimage settings --reset
```

**配置文件位置：**
- Linux: `~/.config/nanoimage/config.json`
- macOS: `~/Library/Preferences/nanoimage/config.json`
- Windows: `%APPDATA%\nanoimage\config.json`

## GUI 使用指南

### 启动

```bash
nanoimage-gui
# 或直接运行
target/release/nanoimage-gui
```

### 操作流程

1. **添加文件**
   - 拖拽文件到主窗口
   - 点击「添加文件」按钮选择文件
   - 点击「添加文件夹」批量选择目录

2. **配置参数**
   - 拖动质量滑块调整压缩质量
   - 选择压缩模式（有损/无损/智能）
   - 选择输出格式
   - 调整工作线程数
   - 勾选「保留元数据」「覆盖源文件」「跳过失败文件」
   - 选择输出目录

3. **开始处理**
   - 点击「▶ 开始优化」按钮
   - 观察进度条和日志面板

4. **查看结果**
   - 文件列表显示压缩前后大小和压缩率
   - 压缩率用颜色区分（深绿 >70%, 绿色 >40%, 黄色 >10%, 红色 <0%）
   - 处理完成后弹出汇总对话框

5. **配置管理**
   - 配置自动保存，下次启动自动加载
   - 可导入/导出配置文件

### 界面布局

```
┌─────────────────────────────────────────────────┐
│  NanoImage                              [_][□][X]│
├─────────────────────────────────────────────────┤
│  📁 12 个文件              [清空]               │
│  ─────────────────────────────────────────────  │
│  ✅ photo1.jpg  2.1MB  → 890KB (-58%)  [绿色]  │
│  ✅ photo2.png  5.4MB  → 1.2MB (-78%)  [深绿]  │
│  ⏳ photo3.gif  等待处理              [灰色]    │
│  ❌ photo4.bmp  Error: xxxxx          [红色]    │
│  ─────────────────────────────────────────────  │
│  总计: 原始 45MB → 压缩后 12MB  节省 33MB (73%)│
│                                                 │
│  ┌─ 设置 ─────────────────────────────────────┐│
│  │ 质量: [======|====] 85%                     ││
│  │ 压缩模式: [有损 ▼]                          ││
│  │ 输出格式: [保持原格式 ▼]                    ││
│  │ 工作线程: [4]                               ││
│  │ ☑ 保留元数据  ☐ 覆盖源文件  ☑ 跳过失败      ││
│  │ 输出目录: [选择目录] ./optimized            ││
│  │ [导入配置] [导出配置]                       ││
│  └─────────────────────────────────────────────┘│
│                                                 │
│  [▶ 开始优化] [添加文件] [添加文件夹]           │
│  ████████████████░░░░ 75%                       │
│                                                 │
│  📝 日志:                                       │
│  ℹ️ 添加了 12 个文件                            │
│  ✅ 完成 photo1.jpg                             │
│  ──────────────────────────────────────────────  │
└─────────────────────────────────────────────────┘
```

## 常见问题

### Q: 为什么有些图片压缩后变大了？
A: 小尺寸图片（< 1KB）或已经高度压缩的图片，重新编码后可能会略微变大。这是正常现象。

### Q: 如何恢复默认配置？
A: 运行 `nanoimage settings --reset` 或删除配置文件。

### Q: GUI 支持哪些平台？
A: Linux (X11/Wayland), Windows, macOS。需要图形环境。

### Q: 批量处理支持多少文件？
A: 取决于可用内存。建议使用 `--workers` 控制并发线程数以避免内存溢出。
