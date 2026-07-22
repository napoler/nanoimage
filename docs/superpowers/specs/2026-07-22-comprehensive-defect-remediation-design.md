---
comet_change: comprehensive-defect-remediation
role: technical-design
canonical_spec: openspec
archived-with: 2026-07-22-comprehensive-defect-remediation
status: final
---

# 综合缺陷修复 — Design Doc

## 1. Context

nanoimage v0.1.0（2026-07-18）发布时干净：83 测试通过、clippy 严格、全审计（commit 3ea25c9）。2026-07-22 的二审发现 5 项一审接受或遗漏的硬缺陷。本 Design Doc 为每项缺陷给出实现级技术规范，配套全路径覆盖 + 错误注入测试。

5 项缺陷在 `openspec/changes/comprehensive-defect-remediation/proposal.md` 中已列出（Why / What Changes / Impact）。高层架构（BMP→PNG、GUI 通道 warn、`_optimized` 后缀锚定）在 open 阶段 `design.md` 已锁定。本文把高层细化到实现级决策。

## 2. Goals / Non-Goals

**Goals：**
- 消除产线代码（非 `#[cfg(test)]`）中对 `ProcessResult.error` 的 `unwrap` / `unwrap_or_default`（6 处站点）
- 让 BMP 升格为一等输入格式（分发分支 + 测试）
- 让 `is_already_optimized` 边界锚定（`file_stem` 后缀匹配）
- 让 GUI 通道发送错误经 `tracing::warn!` 可观测
- 收紧一处静默回退（`metadata().unwrap_or(0)`）加显式错误路径
- 为 `config_persistence` 的 HOME 兜底加 `tracing::warn!`
- 提供全路径覆盖 + 错误注入测试（用户已确认深度）

**Non-Goals：**
- 不引入新功能、不加 API、不加新依赖
- 不重构 optimizer 分发架构
- 不加 GUI 功能（配置保存防抖等后续工作）
- 不删除 `#[allow(dead_code)]`，除非能证明可达

## 3. Decisions（实现级）

### D1：`Optimizer::process_bmp` 实现

**签名：**`fn process_bmp(&self, input: &Path, output: &Path) -> anyhow::Result<()>`

**算法：**
```rust
let img = image::open(input)?;            // image crate 解码 BMP
// 调用方已保证 output 父目录存在
let file = std::fs::File::create(output)?;
let encoder = image::codecs::png::PngEncoder::new_with_quality(
    file,
    image::codecs::png::CompressionType::Default,
    image::codecs::png::FilterType::Adaptive,
);
img.write_with_encoder(encoder)?;
Ok(())
```

**为何用 PngEncoder 而非 `img.save()` 自动识别扩展名：** 显式 encoder 防止依赖 output 扩展名推断。即便用户设了 `output_dir` 但忘了改名，行为确定性。

**文件扩展名策略：** 输入文件 `.bmp` 在输出路径保留。输出内容是 PNG。测试断言：
- `output_path.ends_with(".bmp")`（文件名保留）
- 文件 magic bytes 以 PNG 签名 `\x89PNG\r\n\x1a\n` 开头

**为何不改名为 `.png`：** 会破坏依赖「扫描输出目录中 .bmp 文件」的下游脚本。测试中标注已知不对称。

### D2：`is_already_optimized` 边界锚定

**签名：**`fn is_already_optimized(path: &Path) -> bool`

**算法：**
```rust
fn is_already_optimized(path: &Path) -> bool {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.ends_with("_optimized"))
        .unwrap_or(false)
}
```

**为何 `ends_with` 而非 regex：** 4 行 regex 不值得引入依赖。尾缀 `_optimized` 即契约。

**测试矩阵**（5 case）：

| 输入 | 期望 | 原因 |
|------|------|------|
| `photo_optimized.jpg` | true | 后缀匹配 |
| `not_optimized.png` | false | `_optimized` 在中段，非后缀 |
| `optimized.png` | false | 缺少 `_` 前缀 |
| `_optimized.png` | true | stem 是 `_optimized`，自身即后缀 |
| `no_extension` | false | 无扩展名；`file_stem` 返回 `Some("no_extension")`，不以 `_optimized` 结尾 |

### D3：错误提取模式（6 处）

**模式：**`if let Some(e) = &result.error { ... } else { /* 默认值 */ }`

站点清单：
1. `crates/nanoimage-core/src/processor.rs:86` — `let detail = if let Some(e) = &result.error { e.as_str() } else { "未知错误" };`
2. `crates/nanoimage-cli/src/commands/batch.rs:218-219` — 两次 `error` 读取都走 `if let Some`（当前 `.clone().unwrap_or_default()`）
3. `crates/nanoimage-cli/src/commands/convert.rs:77` — `if let Some(e) = &result.error { error(&format!("✗ 转换失败: {}", e)) } else { error("✗ 转换失败: 未知错误") }`
4. `crates/nanoimage-cli/src/commands/compress.rs:65` — 与 convert 同型
5. `crates/nanoimage-gui/src/lib.rs:219` — `if let Some(msg) = result.error.clone() { FileStatus::Error(msg) } else { FileStatus::Error("Unknown error".to_string()) }`

**为何不用 `.as_deref().unwrap_or("")`：** 保留类型可见性；类型告诉读者「此处预期是 Some」。

**编译期保证：** 不写 `#[allow(...)]` —— clippy 的 `clippy::unwrap_used` 在工作区 `-D warnings` 下会失败 CI，未来若有 `unwrap` 漏入会被卡住。修复后用 `cargo clippy --workspace --all-targets -- -D warnings` 验证。

### D4：GUI 通道发送可观测

**当前（有问题）：**
```rust
tx.send(WorkerMsg::Progress(progress)).ok();
```

**修复：**
```rust
if let Err(e) = tx.send(WorkerMsg::Progress(progress)) {
    tracing::warn!(target: "nanoimage_gui", event = "gui_channel_send_discarded", error = %e);
}
```

**为何用 `tracing::warn!` 而非 `eprintln!`：** tracing 是项目标准；`eprintln` 污染输出捕获（GUI 日志面板在用）。

**测试：** 在 `crates/nanoimage-gui/src/lib.rs` 加 `#[cfg(test)] mod channel_tests`，构造已关闭的 `mpsc::channel()`，取走 receiver 后丢弃，调用镜像发送逻辑的小辅助函数。用自定义 `Subscriber` 捕获日志（不引入新 dev-dep —— `tracing-subscriber` 已通过 `tracing_subscriber::fmt` 在树中）。

### D5：`config_persistence.rs` HOME 兜底

**当前：**
```rust
.unwrap_or_else(|_| String::new())  // → PathBuf::from("")/.config/nanoimage
```

**修复：** 路径解析逻辑不变，加 warn：
```rust
let home = std::env::var("HOME").ok();
let home = match home.as_deref() {
    Some(h) if !h.is_empty() => h.to_string(),
    _ => {
        tracing::warn!(target: "nanoimage_gui", event = "config_no_home_dir");
        String::new()
    }
};
```

**测试：** 通过 `std::env::set_var` 设 `HOME=""`（仅测试，结束后恢复）。调用 `config_dir()`（需 `pub(crate)` 让测试可见）。断言 warn event 被捕获。

## 4. 测试策略（全路径 + 错误注入）

### 4.1 缺陷修复测试（5）

按 `tasks.md` §1。

### 4.2 核心路径加固（4）

brainstorming 检查点已定，加上：
- `test_optimizer_success_path_metadata` — 真实 JPEG 跑一次，断言 `new_size > 0`、`output_path != input_path`（默认模式）、`output_path == input_path`（overwrite 模式）
- `test_processor_skip_failed_drops_failures` — 合法+非法 JPEG 混合，`skip_failed=true`，断言 results 仅含成功，`failed_count` 正确
- `test_batch_processor_collect_includes_bmp` — `test_collect_images_all_extensions` 已有 BMP 存在；增 sibling 测试单独断言 BMP 出现 + 可被处理
- `test_gui_worker_msg_drop_on_cancel` — 线程内手动 drop receiver，触发 `process_files`，断言无 panic + warn 已发

### 4.3 错误注入（4）

- `test_gui_channel_send_on_closed_recv` — 见 D4
- `test_config_persistence_home_unset` — 见 D5
- `test_optimizer_unwritable_output_path` — `output_dir` 设为不可写路径（如 `/dev/null/foo/bar`）；期望 `success=false` 带父目录创建错误
- `test_processor_partial_failure_aggregation` — 喂混合失败文件，断言 `failed_count` 等于预期

**新增测试合计：** 13。既有 83。最终 ~96。

## 5. Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| BMP 测试依赖 `image` crate 的 BMP 解码器 | 测试中用 `image::ImageBuffer::<Rgb<u8>, _>` 保存为 `.bmp` 造已知良好 fixture |
| GUI 测试当前不上 CI（egui 无 headless 测试基建） | 通道发送测试针对 `WorkerMsg` + `mpsc`，无需 GUI 上下文；独立运行验证 |
| `image::codecs::png::PngEncoder` API 表面在跨版本时可能变化 | 通过 Cargo.toml workspace dep `image = "0.25"` 锁定；0.25 API 稳定 |
| 改名 `_optimized` 检测对持有 `not_optimized.png` 的用户是行为变更 | 这就是 bug；修复前行为是错的 |
| `tracing::warn!` 增加测试输出噪声 | 测试用 `Once` subscriber 把事件捕获到 Vec；不污染 stderr |
| HOME 缺失测试会变更全局 env | 在 fixture 的 `Drop` 中恢复 HOME；测试中注释 |
| 输出文件名 `.bmp` 但内容 PNG 是已知不对称 | 测试 + 用户可见日志消息中标注 |
| `image::open` 对部分格式返回 lazy `ImageReader` | 对 BMP 不需要 `.into_decoder()`；`JpegEncoder::new_with_quality` / `PngEncoder` 在 `write_with_encoder` 时会强制 decode |

## 6. Migration Plan

- 无 DB、无 schema
- BMP 文件从失败变成功
- 其余行为向后兼容
- 回滚：revert 单 commit；无就地数据风险

## 7. Open Questions

无。所有决策受既有类型与 crate 限定。

若审阅者对 D1（BMP→PNG 是否改名）有意见，回退方案是 BMP→BMP 复制（仍满足「填补分发器空缺」目标，只是无压缩收益）。决策可逆。
