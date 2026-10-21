# M3-1A：平台源码模块改名

日期：2026-10-21。范围：仅将手机工程内部 `src/gpui_ohos/` 改为 `src/gpui/`，同步 Rust 模块引用及文档。上游 GPUI 依赖、Cargo package 名称、Node-API 导出和运行逻辑保持原样；不启动 M3-2。

| 日期 | 顺序 | 操作与目的 | 实际结果与验证状态 |
| --- | --- | --- | --- |
| 2026-10-21 | 1 | 开始前告知改名范围、同名冲突处理与验证方式；读取 `AGENTS.md`、GPUI 需求/任务、手机工程 Rust 源码引用和 Git 状态。 | 找到 `src/gpui_ohos/` 的四个文件、`lib.rs`/宿主引用、README/需求/任务文字。当前 M3-1 变更仍在本地未提交；M3-2 未开始。与上游 `gpui` crate 同名需显式区分。 |
| 2026-10-21 | 2 | 将 `src/gpui_ohos/` 移为 `src/gpui/`；根模块把上游依赖别名为 `gpui_core`，内部代码和宿主引用改用明确路径。同步 README、需求和任务描述。 | 源码目录与引用已改名；Cargo package 名、构建脚本中的 `libgpui_ohos_mobile_counter.so` 以及 Node-API 导出未修改。静态检查和构建尚未运行，当前状态为未验证。 |
| 2026-10-21 | 3 | 搜索遗留路径并执行 `cargo fmt --check`、`cargo check --locked --offline --target aarch64-unknown-linux-ohos`、`git diff --check`。 | 遗留 `gpui_ohos` 仅出现在历史记录、任务说明和未改名的 Cargo 产物名。目标交叉检查及差异检查通过；首次格式检查因导入顺序和换行失败，运行 `cargo fmt` 后复查通过。 |
| 2026-10-21 | 4 | 执行 `scripts/build-native.sh` 和 `hvigorw --no-daemon ... assembleHap`，确认源码目录改名未影响原生模块与 HAP。 | Rust aarch64 原生库构建和签名 HAP 打包均通过；仍输出 `libgpui_ohos_mobile_counter.so` 并暂存为 `libgpui_mobile_counter.so`，对外 Node-API 模块名未变。 |
| 2026-10-21 | 5 | 从 signed HAP 解出 `.so`，执行 `file`、SDK `llvm-readelf -d` 和 HAP 库清单检查。 | HAP 仅包含一份 `arm64-v8a` ELF64/AArch64 Rust 库，动态依赖仍为 M3-1 记录的八个系统库。该纯命名整理后的 HAP **未重新安装/真机运行**；M3-1 的真机运行结论来自改名前版本。 |

M3-1A 完成。后续 M3-2 从 `src/gpui/` 继续实现 Scene 绘制。
