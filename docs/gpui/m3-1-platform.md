# M3-1：GPUI 平台宿主与事件队列

日期：2026-10-21。范围：只实现锁定 GPUI 提交所需的最小 `gpui_ohos` 平台/窗口、任务调度、显示信息与宿主事件队列，并验证 `Application::with_platform`/`run_embedded` 的启动及回调线程边界。现有 Native Drawing 探针保留；`Scene` 图元转换、字体系统实际绘制与 GPUI 计数器交互属于后续任务，不在本阶段实现。

| 日期 | 顺序 | 操作与目的 | 实际结果与验证状态 |
| --- | --- | --- | --- |
| 2026-10-21 | 1 | 开始前说明只做 M3-1 的做法、原因和验证方式；读取根 `AGENTS.md`、GPUI 需求/任务与当前手机工程源码，检查 Git 工作区。 | M2-2 与 R-1 已完成；工作区干净。用户此前明确要求手机工程按 SDK 自建 FFI，不使用相邻绑定项目。GPUI Scene、文字和计数器交互尚未实现。 |
| 2026-10-21 | 2 | 建立本阶段记录并把任务状态改为实施中，避免把 M3 后续子任务误写为已开始。 | 文档与任务状态已更新；接下来核对锁定 GPUI 提交的接口和本机 SDK。 |
| 2026-10-21 | 3 | 读取锁定 GPUI 提交 `c4a82ab` 的 `Platform`、`PlatformWindow`、`PlatformDispatcher`、`PlatformTextSystem`、`PlatformAtlas`、`Application::run_embedded` 与调度源码；检查已有 XComponent 回调、Node-API 与 ArkTS 页面。 | 确认 `run_embedded` 允许宿主拥有事件循环；`ApplicationHandle::update` 不能重入。`raw-window-handle 0.6.2` 自带 OHOS NDK 窗口句柄类型。当前 GPUI 目标编译会同时匹配 Linux 配置。 |
| 2026-10-21 | 4 | 新增 `gpui_ohos` 的平台、窗口和调度模块；直接实现 GPUI trait，使用锁定依赖图中已有的 `futures`、`raw-window-handle`、`uuid`；从 XComponent 状态读取窗口快照。 | 首次 `cargo check --offline --target aarch64-unknown-linux-ohos` 通过；当时模块尚未接入 Node-API，因此出现未使用告警。文字和 Atlas 为明确的阶段占位实现，Scene 尚未绘制。 |
| 2026-10-21 | 5 | 把 Surface 生命周期入队，导出 `gpuiStart`/`gpuiPump`，ArkTS 在页面出现时启动嵌入 GPUI 并按 50 ms 驱动 M3-1 宿主队列；页面消失时停止定时器。 | 第二次目标 `cargo check --offline --target aarch64-unknown-linux-ohos` 通过且无告警。真机启动、事件顺序与跨线程拒绝仍待验证。50 ms 轮询只是本阶段事件队列驱动，M4-2 将按帧回调替换。 |
| 2026-10-21 | 6 | 在宿主队列中处理 Surface 创建、变化、销毁；创建空 GPUI 根视图并实际调用 `App::open_window`，按 generation 关闭旧窗口；从 SDK `oh_display_manager.h` 绑定屏幕 ID、像素尺寸、逻辑密度和刷新率，窗口暴露 OHOS NDK raw handle。 | 首次遗漏 `AppContext` trait import 导致静态检查报 `E0599`，补入后 `cargo check --locked --offline --target aarch64-unknown-linux-ohos` 通过。实际 Scene 绘制仍未实现。 |
| 2026-10-21 | 7 | 执行 `cargo fmt`、`scripts/build-native.sh`、Hvigor `assembleHap`。 | 格式与 Rust aarch64 构建通过。首次 Hvigor 默认 daemon 因用户缓存锁获取失败；改用同一构建命令的 `--no-daemon` 后 ArkTS 编译、HAP 打包与调试签名通过。ArkTS 对自定义 `.so` 发出已有的非阻断类型验证警告。 |
| 2026-10-21 | 8 | 用 `hdc list targets -v`、`hdc install`、`aa start` 和 `hilog` 运行首版 M3-1 HAP。 | 默认受限环境下 `hdc` 报 `Connect server failed`；按设备操作授权在非沙箱下重试成功，真机连接、安装和启动成功。日志证实 `run_embedded` 完成，Surface 创建在 `ThreadId(1)`，宿主队列也在 `ThreadId(1)` 处理，GPUI 窗口打开，前台任务由 ArkUI pump 执行。未观察到重入 panic。 |
| 2026-10-21 | 9 | 用真机 `uitest dumpLayout` 找到按钮，模拟 Hide/Show 和 Resize，再读取日志。 | `generation=1` 销毁后队列关闭窗口，`generation=2` 创建后重新打开；Resize 的 XComponent 尺寸由 `1260×1918` 变为 `1008×1918` 并进入 GPUI 宿主队列。应用未崩溃。触摸只维持既有探针行为，尚未送入 GPUI。 |
| 2026-10-21 | 10 | 根据真机显示日志核对尺寸单位，修正 GPUI 逻辑尺寸来源。 | SDK 显示管理返回 `1260×2720 px`、密度 `3.25`，刷新率在测试期间为 `60` 或 `120 Hz`。此设备的 XComponent 尺寸值是 `1260×1918`，与 NativeWindow 像素尺寸相同，和 SDK 头文件标注 vp 不一致；GPUI 改用 NativeWindow 像素尺寸除以 SDK 密度。真机日志显示窗口逻辑尺寸约 `387.69×590.15`，对应缓冲区 `1260×1918 px`。 |
| 2026-10-21 | 11 | 执行 `cargo fmt --check`、目标 `cargo check --locked --offline`、原生构建和 `hvigorw --no-daemon ... assembleHap`；检查 HAP 内 `.so` 与 ELF 动态依赖并重新安装启动。 | 检查、构建、签名和真机运行通过。HAP 包含一份 `arm64-v8a` ELF64/AArch64 Rust 库；依赖系统 `libnative_display_manager.so`、`libnative_drawing.so`、`libhilog_ndk.z.so`、`libace_napi.z.so`、`libnative_window.so`、`libace_ndk.z.so`、`libtime_service_ndk.so` 和 `libc.so`。没有 C++ 桥接库。 |
| 2026-10-21 | 12 | 将 ArkTS/Native 日志标签统一为 `GPUI_M3_1`，再次执行格式/交叉检查、原生构建、HAP 打包签名、安装启动；在最终 HAP 上重复 Hide/Show/Resize。 | 全部通过。最终日志有 `GPUI embedded launch completed`、窗口逻辑尺寸约 `387.69×590.15`、前台任务在 `ThreadId(1)` 执行；销毁、重建到 `generation=3` 及尺寸变为 `1008×1918 px` 均进入同一线程的宿主队列，未发现 panic 或窗口打开错误。Resize 后已点击按钮恢复默认宽度。无回调重入的结论限于这些已执行的真机操作；主动从非 UI 线程调用 `gpuiPump` 的拒绝分支尚未实测。 |
| 2026-10-21 | 13 | 用进程级 ArkUI 线程标识约束 `gpuiStart` 与 `gpuiPump`；调试包启动时从专用工作线程执行一次 pump 拒绝探针。再次运行格式、目标交叉检查、原生构建、HAP 构建签名、安装启动和 `hilog -x -T GPUI_M3_1`。 | 全部通过。真机日志显示不同线程上 `GPUI pump rejected off ArkUI thread` 与诊断返回 `0`，随后 ArkUI `ThreadId(1)` 正常创建 GPUI 窗口并执行前台任务。跨线程拒绝分支已在真机验证。 |
| 2026-10-21 | 14 | 从最终 signed HAP 解出 `.so`，用 `file` 与 SDK `llvm-readelf -d` 核对架构与 `NEEDED`；执行 `git diff --check` 并核对任务状态与 README。 | HAP 仍仅有一份 `arm64-v8a` 的 stripped ELF64/AArch64 Rust 库；动态依赖与第 11 项一致，均为系统库。差异检查无空白错误；工程与任务记录一致。 |

M3-1 完成。窗口的 `draw(&Scene)` 仍为空实现，Atlas 与文字为占位；计数器内容未显示。M3-2 才将 Scene 绘制到 Surface；M3-3 再实现文字。当前 50 ms ArkTS 定时器只负责宿主队列，M4-2 将改为帧请求驱动。真机未执行长时间后台/前台压力测试，留给 M4-3。

后续内部源码模块名称从 `gpui_ohos` 改为 `gpui`；此页保留 M3-1 实施当时的操作名称，改名操作与验证见 [M3-1A 记录](m3-1a-module-rename.md)。
