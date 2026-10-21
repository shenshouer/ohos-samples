# R-1：M3 前模块整理

日期：2026-10-21。范围：只整理独立手机工程 `gpui/ohos-mobile-counter/` 的 Rust 模块边界；沿用 API 26 SDK 手写 FFI，不修改桌面计数器、原生 ABI、绘制内容或 M3 功能。

| 日期 | 顺序 | 操作与目的 | 实际结果与验证状态 |
| --- | --- | --- | --- |
| 2026-10-21 | 1 | 执行前向用户说明整理目标、做法、原因和验证方式；读取根 `AGENTS.md`、GPUI 需求/任务与 M2-2 验证记录。 | 确认用户要求先有任务告知和逐项记录；本任务编号为 R-1，M3-1 继续待办。用户明确要求 SDK 自建 FFI，本工程不采用相邻绑定项目。 |
| 2026-10-21 | 2 | 读取手机工程 `src`、构建脚本及 README，核对现有模块职责。 | `sdk_ffi` 已分为 XComponent、Node-API、NativeWindow、日志、Drawing；Drawing 单文件合并了错误码、GPU Surface 和图形对象声明。`surface.rs` 同时包含日志函数、Surface 状态、回调及触摸探针；`napi_module.rs`、`drawing_probe.rs` 在顶层。约 945 行 Rust。尚未修改代码。 |
| 2026-10-21 | 3 | 定义最小迁移边界：Drawing FFI 按错误、Surface、绘制对象细分；宿主入口/日志/Surface 归入 `host`；绘制探针归入 `render`，保持原有资源 RAII 与回调顺序。 | 方案确定，待实施与编译/真机验证；不新增依赖，不改变 ArkTS 测试界面。 |
| 2026-10-21 | 4 | 移动 Rust 宿主和渲染文件，新增 `host::logging`；将 Drawing FFI 拆为 `error`、`gpu_surface`、`paint` 子模块并从 `drawing/mod.rs` 统一导出。 | 已完成文件与 import 整理；Node-API 回调、Surface 状态机、绘制算法及链接库声明未作语义修改。待编译验证。 |
| 2026-10-21 | 5 | 将 XComponent FFI 拆为 `surface`（回调/尺寸）和 `touch`（事件/布局），保留 `xcomponent/mod.rs` 的原有公共路径；执行 `cargo fmt` 与 `cargo check --locked --offline --target aarch64-unknown-linux-ohos`。 | 格式与交叉检查均通过；结构体布局断言继续编译，未增加 Cargo 依赖。HAP 与真机仍待验证。 |
| 2026-10-21 | 6 | 执行 `scripts/build-native.sh`、DevEco `hvigorw ... assembleHap`，从签名 HAP 提取 `.so` 并用 `file`、SDK `llvm-readelf -d` 检查。 | 原生构建、HAP 打包与签名通过；仅一份 `arm64-v8a` ELF64/AArch64 库，动态依赖仍为系统 `libnative_drawing.so`、`libhilog_ndk.z.so`、`libace_napi.z.so`、`libnative_window.so`、`libace_ndk.z.so`、`libc.so`。 |
| 2026-10-21 | 7 | 用 `hdc` 安装、强制停止并启动重构后的 HAP，读取 `GPUI_M2_2` 日志；准备点击方向按钮。 | 安装/启动成功；`Rust probe bits: 7`、XComponent 注册、`1260×1979` 创建和 Drawing Flush 成功。首次 Node-API 初始化出现 `unwrap_status=2` 警告，随后注册成功。尝试点击后发现设备已锁屏，输入未到达应用；后续交互回归待设备解锁，未写成通过。 |
| 2026-10-21 | 8 | 将宿主触摸事件读取与校验移到 `host/touch.rs`，Surface 回调只保留当前窗口/代次检查；更新 README 的模块职责和历史文件路径。 | 文件和文档已改，待最后一次格式、交叉编译、HAP 与真机复验。 |
| 2026-10-21 | 9 | 最终执行 `cargo fmt --check`、目标 `cargo check --locked --offline`、`scripts/build-native.sh`、`hvigorw ... assembleHap`；从最终签名 HAP 提取库查 ELF 与依赖。 | 全部通过；最终 HAP 仅有一份 ELF64/AArch64 Rust 库，动态依赖与 M2-2 一致，没有 C++ 运行库或新增第三方渲染依赖。 |
| 2026-10-21 | 10 | 再次安装最终签名 HAP，强制停止并启动，用 `hilog -x -T GPUI_M2_2` 查新进程。 | 安装/启动成功；`probe bits: 7`、`register: ok`、`created` 的 NativeWindow `1260×1979`、`Drawing flush ok`。初始化时仍有先于注册的 `unwrap_status=2` 警告，之后成功注册。设备锁屏时 UI 点击虽然返回 `No Error`，但没有方向日志；交互回归尚未验证。 |
| 2026-10-21 | 11 | 核对本次目录、README、任务状态和文档隐私边界；询问用户解锁真机以继续交互回归。 | 源码只调整手机工程，桌面工程未改；R-1 保持待完成，M3-1 未开始。记录没有签名材料、本机 SDK 绝对路径或设备序列号。解锁后需复测横竖屏、Hide/Show、触摸日志。 |
| 2026-10-21 | 12 | 用户解锁真机后，用 `hdc list targets` 确认在线，并用 `aa start` 将已安装的最终 R-1 HAP 切回前台；点击 Landscape、Portrait，读取 `GPUI_M2_2` 日志。 | 两次窗口方向请求成功。`changed` 的 NativeWindow 几何尺寸分别为 `2720×642`、`1260×1979`；第一代窗口两次均 `old_released=true` 且 `Drawing flush ok`。先前锁屏导致的点击无效已排除。 |
| 2026-10-21 | 13 | 竖屏点击 Hide Surface、Show Surface；检查销毁和重建日志，然后在新画面点击一次。 | `destroyed: generation=1 ... drawing_released=true`；新 XComponent/窗口注册并创建 `generation=2`，`1260×1979` Flush 成功。新窗口触摸日志有 `type=0`（按下）和 `type=1`（抬起）、`points=1`，证明拆分后的触摸 FFI 与宿主路径仍可用。 |
| 2026-10-21 | 14 | 点击 Resize Surface 再恢复全宽，随后执行 Home → `aa start` 返回，并检查最终截图。 | 第二代窗口按 `1008×1979 → 1260×1979` 变化，两次均 `old_released=true`、Flush 成功；后台返回后截图显示完整深蓝背景和青色圆角矩形。截图只在临时目录，未写入仓库。本轮未观察到 Drawing 错误或黑屏。 |
| 2026-10-21 | 15 | 同步 `tasks.md`、`requirements.md` 和工程 README 的最终状态，复核只完成 R-1。 | R-1 真机回归完成；下一任务 M3-1 尚未实施。最终文档不包含本机 SDK 绝对路径、签名材料或设备序列号。 |
| 2026-10-21 | 16 | 按最新 `AGENTS.md` 要求为每条操作记录添加日期列，检查表格格式。 | 首次自动插列时遗漏一个分隔符；已修正 15 行，表头与数据行现均为四列。 |

真机：`ALN-AL00`、OpenHarmony `7.0.0.105`、API 26、aarch64。R-1 没有改变现有 Native Drawing 探针画面，也没有把 GPUI Scene 或计数器交互接入手机端。
