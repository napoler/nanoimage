---
change: comprehensive-defect-remediation
design-doc: docs/superpowers/specs/2026-07-22-comprehensive-defect-remediation-design.md
base-ref: da63d9d6059b84f28d8064b72fdeee455ce36285
---

# 综合缺陷修复 — 实施计划

## 路线图

按 tasks.md 5 个组顺序执行。每组内子任务先写失败测试（TDD Red），委派给 `code-assistant` 实现（Green），然后跑 `cargo test --workspace` 与 `cargo clippy --workspace --all-targets -- -D warnings` 双重验证。每完成一个子任务 → git commit（按 `review_mode` 完成验收后再勾选）。

## 强制约束

- 所有 `.rs` 文件改动委派给 `code-assistant` subagent（CLAUDE.md `code-edit-routing.md` P0）
- `.md` / 配置改由主进程直接 Edit/Write
- TDD：失败测试先于实现
- 验证：`cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` exit 0 必达
- commit：每个子任务一个独立 commit（除非同测试组可合并）

## 任务分派

### 1. 先写失败测试（TDD Red）→ 委派 code-assistant

- [x] **1.1** `crates/nanoimage-core/tests/optimizer_tests.rs` 加失败测试：BMP 通过 `process_file` 在合法 fixture 上成功
  - 失败原因：BMP 命中 `_ => Err("不支持的格式")` 分支
  - 验证：`cargo test -p nanoimage-core test_bmp_compress` 失败（非 success）
  - commit: `test(core): add failing test for BMP compression`

- [x] **1.2** `crates/nanoimage-core/tests/optimizer_tests.rs` 加失败测试：BMP 损坏文件 → `success=false`
  - 失败原因：BMP 路径不存在
  - 验证：`cargo test -p nanoimage-core test_bmp_corrupt_fails` 失败
  - commit: 同 1.1 可合并

- [x] **1.3** `crates/nanoimage-core/tests/processor_tests.rs` 加失败单元测试：`is_already_optimized("not_optimized.png") == false`
  - 失败原因：当前 `.contains("_optimized")` 误判为 true
  - 验证：`cargo test -p nanoimage-core test_is_already_optimized_substring_false` 失败
  - commit: `test(core): add failing test for is_already_optimized boundary`

- [x] **1.4** 同文件加 `photo_optimized.jpg` 后缀匹配测试
  - 验证：`cargo test -p nanoimage-core test_is_already_optimized_suffix_true` 通过（当前行为也对，但作为回归证据）
  - commit: 可与 1.3 合并

- [x] **1.5** `crates/nanoimage-core/tests/processor_tests.rs` 加 `BatchProcessor::collect_images` 回归测试确认 BMP 仍收集
  - 验证：`cargo test -p nanoimage-core test_collect_images_all_extensions` 已存在；扩 1 个专门 BMP 用例
  - commit: 同 1.3

### 2. 核心实现（TDD Green）→ 委派 code-assistant

- [x] **2.1** `crates/nanoimage-core/src/optimizer.rs` 实现 `Optimizer::process_bmp`
  - API：D1 节算法
  - 验证：1.1 测试转为 GREEN
  - 验证：`cargo test -p nanoimage-core test_bmp_compress` 通过
  - commit: `feat(core): implement BMP→PNG conversion in optimizer`

- [x] **2.2** `crates/nanoimage-core/src/optimizer.rs` 分发器新增 `ImageFormat::Bmp => self.process_bmp(...)`
  - 验证：1.2 测试转为 GREEN
  - commit: `feat(core): wire BMP arm into optimizer dispatcher`

- [x] **2.3** `crates/nanoimage-core/src/processor.rs` `is_already_optimized` 改为边界锚定
  - 验证：1.3 测试转为 GREEN；1.4、1.5 仍通过
  - commit: `fix(core): anchor is_already_optimized to suffix match`

- [x] **2.4** 替换 5 处 CLI/GUI 站点 `result.error.unwrap_or_default()` 为显式 `if let Some(e) = &result.error`
  - 站点：
    - `crates/nanoimage-cli/src/commands/batch.rs:218,219`（两次）
    - `crates/nanoimage-cli/src/commands/convert.rs:77`
    - `crates/nanoimage-cli/src/commands/compress.rs:65`
    - `crates/nanoimage-gui/src/lib.rs:219`
  - 验证：`cargo clippy --workspace --all-targets -- -D warnings` 通过；现有测试不受影响
  - commit: `refactor: replace unwrap_or_default on ProcessResult.error with explicit match`

- [x] **2.5** `crates/nanoimage-core/src/processor.rs:86` 替换 `result.error.as_ref().unwrap_or(...)` 为显式 match
  - commit: 与 2.4 同组可合并

### 3. GUI 信道可观测性 → 委派 code-assistant

- [x] **3.1** `crates/nanoimage-gui/src/lib.rs` 替换 `tx.send(...).ok()` 为 `if let Err(e) = ... { tracing::warn!(...) }`
  - 站点：line 147、151
  - 验证：`cargo build -p nanoimage-gui` 成功；现有测试通过
  - commit: `feat(gui): warn-log dropped GUI channel sends`

- [x] **3.2** `crates/nanoimage-gui/src/lib.rs` `#[cfg(test)] mod channel_tests` 新增「closed receiver → warn」测试
  - 验证：`cargo test -p nanoimage-gui test_gui_channel_send_on_closed_recv` 通过
  - commit: `test(gui): cover dropped channel send warn`

### 4. 核心路径加固 + 错误注入测试 → 委派 code-assistant

- [x] **4.1** `test_optimizer_success_path_metadata`：默认模式 `output_path != input`，overwrite 模式 `== input`
  - commit: `test(core): assert optimizer output path semantics`

- [x] **4.2** `test_processor_skip_failed_drops_failures`：混合 valid+invalid + `skip_failed=true`
  - commit: 与 4.1 同组

- [x] **4.3** `test_processor_partial_failure_aggregation`：检查 `failed_count` 准确
  - commit: `test(core): assert processor failure aggregation`

- [x] **4.4** `test_optimizer_unwritable_output_path`：`output_dir=/dev/null/foo/bar` → 期望 `success=false` 带父目录错误
  - commit: 与 4.1 同组

- [x] **4.5** `test_config_persistence_home_unset`：临时 `HOME=""`，断言 warn event + 路径退化为 `/.config/nanoimage`
  - commit: `test(gui): cover HOME-unset config persistence`

### 5. 仓库级验证（主进程）

- [x] **5.1** `cargo build --workspace --all-targets` exit 0
- [x] **5.2** `cargo clippy --workspace --all-targets -- -D warnings` exit 0
- [x] **5.3** `cargo test --workspace` 全部通过（目标 ≥ 90 测试）
- [x] **5.4** `git diff --stat` 仅含预设文件

### 6. 归档（主进程）

- [x] **6.1** `comet guard comprehensive-defect-remediation build --apply` PASS
- [x] **6.2** 通过 `comet state next` 交接给 `/comet-verify`

## 风险与兜底

| 风险 | 兜底 |
|------|------|
| code-assistant agent 误改设计 | 主进程在合并 commit 前 `git diff` 实际修改 |
| GUI channel test 失败因 subscriber 重载 | 退回直接 `tracing` `Once` callback，零新增依赖 |
| HOME unset 测试污染其它测试 | `Drop` impl 恢复 HOME；加 `serial_test` 不必要 |
| cargo clippy 引入新 lints 版本 | TDD 测试必须先写，已涵盖；失败 lint 也是失败测试 |

## 验收证据要求

每个 commit 至少配一句中文 commit body 说明「为什么改」、对应 Design Doc 章节（D1-D5）。
