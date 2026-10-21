# GPUI 鸿蒙手机计数器

这是与 [`../desktop-counter/`](../desktop-counter/) 分离的手机端工程。已完成 **M0、M1、M2、R-1、M3-1、M3-1A、M3-2、M3-3、M4-1 和 M4-2**：运行时 Node-API 模块为纯 Rust `cdylib`；真机已验证 XComponent 生命周期、GPUI 计数器文字和按钮、`Reset → + → −` 的 `0 → 1 → 0` 交互，以及按需帧和空闲/后台停帧。完整生命周期验收仍待 M4-3。

- GPUI 官方源码固定提交：`c4a82ab1ab958aa43b0c0b15c0bad4fb77a7ca7f`；关闭默认 feature。`Cargo.lock` 锁定解析出的传递依赖。
- Rust 目标：`aarch64-unknown-linux-ohos`。
- 当前真机：`ALN-AL00`、OpenHarmony `7.0.0.105`、API `26`、`aarch64` / `arm64-v8a`。这些值来自 `hdc shell`，不是由 SDK 版本推断。
- 本机 Native SDK：独立安装 `26.0.0.38`，DevEco Studio 随附 `26.0.0.105`；M1-2 HAP 用 DevEco 26.0.0 的 SDK/API 26 构建。
- 不使用其他平台/渲染项目或现成鸿蒙 Rust 绑定；缺失的 SDK 声明后续由本工程按 SDK 头文件实现。

M0 的逐项命令、结果和未验证事项见 [`../../docs/gpui/m0-validation.md`](../../docs/gpui/m0-validation.md)。M1-2 的每项操作和失败原因见 [`../../docs/gpui/m1-2-hap.md`](../../docs/gpui/m1-2-hap.md)；后续仍按任务编号逐项推进。

当前 Rust 代码按职责组织：

| 目录 | 职责 |
| --- | --- |
| [`src/sdk_ffi/`](src/sdk_ffi/) | 只放 API 26 SDK C ABI 声明和最小返回值/窗口查询封装；Drawing 分错误码、GPU Surface、几何/画笔和字形位图，XComponent 分 Surface、触摸与逐帧回调，Display 查询屏幕尺寸、密度和刷新率。 |
| [`src/host/`](src/host/) | Node-API 模块入口、hiLog、XComponent 生命周期状态、触摸与按需帧调度；回调中的窗口代次与旧资源释放由这里管理。 |
| [`src/gpui/`](src/gpui/) | 自建 GPUI `Platform`、`PlatformWindow`、单色 `PlatformAtlas`、OHOS 窗口句柄、调度器及主线程宿主事件队列；代码中以 `gpui_core` 指代上游 crate。 |
| [`src/render/`](src/render/) | `screen.rs` 管理 GPU context 与屏幕 Surface 的所有权；`scene.rs` 按 GPUI 图元批次绘制并 Flush。 |
| [`src/lib.rs`](src/lib.rs) | 模块装配与最小原生能力探针导出。 |

这次整理只改变模块边界，保留了 `gpui_mobile_probe` 的 C ABI、Node-API 模块名、回调顺序和原有测试画面。逐项操作见 [`../../docs/gpui/r-1-code-organization.md`](../../docs/gpui/r-1-code-organization.md)。

R-1 最终验证：Rust 格式与 OHOS 目标交叉检查、原生构建、HAP 打包签名和原生库架构/动态依赖检查通过；最终 HAP 在真机上记录到探针值 `7`、Surface 创建和 Drawing Flush 成功。解锁后横竖屏、Hide/Show、Resize、第二代窗口触摸按下/抬起与后台返回均通过；画面仍显示深蓝背景和青色圆角矩形。详见 R-1 记录。

M1-2A 的 Rust-only Node-API 替换过程和真机结果见 [`../../docs/gpui/m1-2a-rust-napi.md`](../../docs/gpui/m1-2a-rust-napi.md)。HAP 不再编译 C++ 源文件；`src/sdk_ffi/napi.rs` 直接声明 SDK Node-API C ABI，`src/host/napi.rs` 完成模块注册与 `probe()` 导出。`entry/src/main/cpp/types/` 只含 ArkTS 类型声明；`probes/*.c` 仅用于独立的早期 ABI/设备验证，不进入 HAP。

**M1-3 生命周期探针：** [`../../docs/gpui/m1-3-surface.md`](../../docs/gpui/m1-3-surface.md) 记录了 SDK FFI、代码改动、构建和真机操作。页面提供 `Resize Surface` 与 `Hide/Show Surface` 两个测试按钮；系统日志记录回调线程、原生窗口指针、SDK 返回值、尺寸和触摸。真机上依次观察到 `created → changed → destroyed → created`，新窗口上的触摸按下/抬起仍到达 Rust。SDK 没有提供此回调的注销接口；回调表保持进程期有效，Surface 销毁时 Rust 清空窗口和尺寸状态。

**M2-1 绘制探针历史：** [`../../docs/gpui/m2-1-drawing.md`](../../docs/gpui/m2-1-drawing.md) 记录了 Rust `sdk_ffi` → NativeWindow/Native Drawing 的实现和真机验证。当时的 `src/render/probe.rs` 为 XComponent 原生窗口建立 GPU context 和屏幕 Surface，使用 `RGBA_8888 / PREMUL` 绘制深蓝背景及青色圆角矩形并 Flush；`GPUI_M2_1` 历史日志显示窗口缓冲区 `1260×2109 → 1008×2109`、NativeWindow 格式 `RGBA_8888(12)`、两次 Flush 成功。M3-2 已用 GPUI Scene 绘制替代这个探针；Surface 资源所有权现位于 `src/render/screen.rs`。

**M2-2 生命周期验证：** [`../../docs/gpui/m2-2-lifecycle.md`](../../docs/gpui/m2-2-lifecycle.md) 记录逐次操作和真机结果。新增 Landscape/Portrait 测试按钮使用系统窗口 API 触发方向变化；`GPUI_M2_2` 日志显示第一代和第二代 Surface 均按 `1260×1979 ↔ 2720×642` 重建 Drawing 并 Flush，横竖屏截图均显示完整画面。Hide/Show 后旧 Drawing 释放，新窗口与 `generation=2` 创建成功；Home 后返回仍显示画面，本机这轮没有触发后台 Surface 销毁。无需切换到 SDK NativeWindow 缓冲区或 EGL 备用路线。此阶段仍只是 Rust Drawing 探针，GPUI Scene 尚未接入。

**M0-5 静态审计：** 鸿蒙 Rust 目标同时命中 Linux 条件编译；`gpui_platform` 会选择 Linux 桌面后端，因此本工程只依赖 GPUI 核心，并在 M3-1 自建平台实现。核心需要实现 Linux 条件下的 primary selection 方法；首版计数器不使用该功能。窗口悬停、键盘上下文和 Emoji 字体回退等行为还需在相应阶段验证。逐项位置、判断与命令见 [`../../docs/gpui/m0-5-audit.md`](../../docs/gpui/m0-5-audit.md)。

**M3-1 GPUI 平台历史：** [`../../docs/gpui/m3-1-platform.md`](../../docs/gpui/m3-1-platform.md) 记录逐次操作和真机结果。ArkTS 调用 `gpuiStart()`，Rust 用锁定 GPUI 的 `Application::with_platform(...).run_embedded(...)` 启动应用；XComponent 回调只入队，ArkUI 线程每 50 ms 调用 `gpuiPump()`，再打开、调整或关闭 GPUI 窗口。真机确认窗口创建、销毁与多次重建，前台任务在 `ThreadId(1)` 执行，专用工作线程的 pump 被拒绝并返回 `0`。SDK 显示接口实测 `1260×2720 px`、密度 `3.25`、刷新率动态为 `60` 或 `120 Hz`，NativeWindow 缓冲区 `1260×1918 px` 对应 GPUI 逻辑尺寸约 `387.69×590.15`。当时的空 GPUI 根视图不绘制 Scene；M3-2 已补上 Scene 与 Atlas。`NoopTextSystem` 和 50 ms 事件轮询分别留给 M3-3、M4-2 替换。新增的 `futures`、`raw-window-handle`、`uuid` 是 GPUI trait 签名直接需要且已在锁定依赖图中的 crate，没有引入其他平台或渲染项目。

**M3-1A 源码模块改名：** [`../../docs/gpui/m3-1a-module-rename.md`](../../docs/gpui/m3-1a-module-rename.md) 记录将内部 `src/gpui_ohos/` 改为 `src/gpui/`，上游 GPUI crate 在代码中别名为 `gpui_core`。Cargo package、HAP 原生库名和 Node-API 导出保持不变。当时格式、目标交叉检查、原生构建、签名 HAP 和包内 ELF 检查通过；改名后的代码现已随 M3-2 HAP 在真机运行。

**M3-2 Scene 绘制历史：** [`../../docs/gpui/m3-2-scene.md`](../../docs/gpui/m3-2-scene.md) 记录当时 SDK FFI、构建与真机结果。当时 `NoopTextSystem` 不产生真实字形，另追加固定 `A` 掩码验证 Atlas。该诊断字形已在 M3-3 移除。现有 Scene 绘制仍只支持纯色背景、统一圆角/边框、矩形裁剪及单色 Sprite；其他图元会明确报错。

**M3-3 系统文字：** [`../../docs/gpui/m3-3-text.md`](../../docs/gpui/m3-3-text.md) 记录 API 26 Native Drawing 字体 FFI 与真机结果。`src/gpui/text.rs` 实现计数器所需的系统默认字体度量、逐字符布局、字形边界和 Alpha8 光栅化；字形经 GPUI 原始 Scene 和自有 Atlas 绘制。真机显示 `GPUI Counter`、`0`、`−`、`+`、`Reset`，日志为 4 个 Quad、19 个单色 Sprite，Flush 成功。布局使用与桌面基线相同的 GPUI `gap_4`、字号、内边距与颜色；手机 Surface 尺寸不同，未作同分辨率像素差比较。当前未支持复杂文字 shaping、字体回退和自定义字体；M3-3 当时尚无触摸交互。

**M4-1 触摸交互：** [`../../docs/gpui/m4-1-touch.md`](../../docs/gpui/m4-1-touch.md) 记录 SDK 触摸读取、Surface 代次排队、密度换算和 GPUI `Touch` 分发。`+`、`−`、`Reset` 的数值变化仅在 GPUI Rust 视图的点击回调中发生。aarch64 原生构建、签名 HAP 构建与安装通过；真机受控点击 `Reset → + → −` 显示 `0 → 1 → 0`，截图确认数字 `1`，从 `+` 拖出后未误点击。系统 `Cancel` 事件映射已实现，尚未由真机触发验证。2026-10-21 的 M4-1 延迟追查中，受控点击从 SDK 抬起到计数回调约 45 ms、到 Drawing Flush 约 53 ms；当时仍使用 50 ms ArkTS 轮询。后续 M4-2 已改为按需帧调度。

**M4-2 按需帧：** [`../../docs/gpui/m4-2-frame.md`](../../docs/gpui/m4-2-frame.md) 记录 SDK XComponent 帧回调 FFI、GPUI `frame_waker`/`schedule_frame`、前后台暂停恢复和真机结果。待绘制时注册帧回调，回调到达后注销并在 ArkUI 线程推进 GPUI；ArkTS 的 250 ms 定时器只做 Surface 销毁及后台任务的安全检查，不再驱动绘制。本机曾在重复绘制时记录系统 `NativeWindowFlushBuffer failed ret=41207000`，即 SDK 定义的缓冲区状态无效；当前逐帧重建屏幕 Drawing Surface、复用 GPU context 的最终包未再观察到该错误。真机受控 `+` 点击从 SDK 抬起到 GPUI 计数回调约 9 ms、到 Scene Flush 约 18 ms，截图显示 `1`；空闲约 14 秒无连续帧回调，Home 后也停帧。此机无需 Native VSync FFI。一次高频触摸期间的受控点击未触发 `on_click`，后续静默复测正常；原因仍待观察。Flush 到实际屏幕呈现的耗时尚未测量，因此用户报告的偶发秒级视觉延迟不能宣称已在所有情况下消除。

**M0-4 预检与结论：** SDK aarch64 库导出计划使用的 Drawing、XComponent 和 Node-API 符号。真机先前拒绝执行放在 `/data/local/tmp` 的独立探针，并拒绝 SDK 测试签名 HAP；HarmonyOS 调试签名 HAP 安装后，应用日志 `Rust probe bits: 7` 已确认最小加载链。可复现 shell 探针源码在 [`probes/m0_library_load.c`](probes/m0_library_load.c)。

**M1-1 FFI：** [`src/sdk_ffi/`](src/sdk_ffi/) 按 API 26 SDK 头文件手写 XComponent 与 Native Drawing 的最小 C ABI，直接链接系统 `libace_ndk.z.so` 和 `libnative_drawing.so`，没有新增 Cargo 依赖。`probes/m1_ffi_layout.c` 对两套 SDK 的 aarch64 布局检查通过；`probes/m1_ffi_link.rs` 的 aarch64 ELF 动态依赖包含两个系统库。完整命令、API 级别与未验证项见 [`../../docs/gpui/m1-1-ffi.md`](../../docs/gpui/m1-1-ffi.md)。这些是 M1-1 当时的静态与链接检查；后续 M1-2 至 M2-2 已在应用进程中调用相应 SDK 接口。

**M1-2 最小宿主历史：** `src/lib.rs` 导出 Rust 探针；当前由纯 Rust Node-API 模块提供给 ArkTS。真机日志和用户提供的早期截图均显示成功值 `7`。早期截图上方白色区域是 ArkTS 标题和探针文字；下方大面积黑色区域是当时尚无绘制内容的 XComponent Surface。工程没有权限申请；M1-3 接入了 Surface 回调，M2-1 已将黑色区域替换为 Drawing 探针画面。

构建时在本机设置 `DEVECO_SDK_HOME` 为 DevEco SDK 根目录、`NODE_HOME` 为 DevEco Node 目录、`JAVA_HOME` 为 DevEco JBR 目录，并将 DevEco 的 `hvigorw` 加入 `PATH`。在本目录运行：

```sh
cargo fmt --check
scripts/build-native.sh
hvigorw --no-daemon --mode module -p module=entry@default -p product=default -p buildMode=debug assembleHap
```

产物目录为 `entry/build/default/outputs/default/`。`scripts/build-native.sh` 使用 SDK 的目标 Clang 构建 Rust `cdylib` 并将其暂存到忽略的 `entry/libs/arm64-v8a/`。当前 HAP 仅包含一份 ELF64/AArch64 Rust 原生模块；其动态依赖为系统 `libace_napi.z.so`、`libace_ndk.z.so`、`libnative_window.so`、`libnative_drawing.so`、`libnative_display_manager.so`、`libtime_service_ndk.so`、`libhilog_ndk.z.so` 和 `libc.so`，没有因 C++ 桥接而带入的 `libc++_shared.so`。未关联签名方案时仅生成 `entry-default-unsigned.hap`，DevEco 部署它会报 `9568320: no signature file`。

首次配置时，复制 [`build-profile.example.json5`](build-profile.example.json5) 为本机 `build-profile.json5`。按 [Huawei DevEco 自动签名说明](https://developer.huawei.com/consumer/cn/doc/doccenter-deveco-studio/ide-signing-auto)，在本工程的 `File > Project Structure > Project > Signing Configs` 勾选 `Support HarmonyOS` 与 `Automatically generate signature`，为当前包名和设备取得调试材料。**再检查** `app.products` 中 `name: "default"` 的对象含 `"signingConfig": "default"`，且 `app.signingConfigs` 有同名方案；[Huawei 构建配置文档](https://developer.huawei.com/consumer/cn/doc/doccenter-deveco-studio/ide-hmos-hvigor-build-profile-app)说明缺少这一引用时默认不签名。含本机口令与路径的 `build-profile.json5` 已被 `.gitignore` 排除，密钥、证书、profile 和口令均不可提交。

2026-10-21 修正该引用后，`entry-default-signed.hap` 通过 SDK 签名校验并由 `hdc install` 在真机安装成功。首次启动时手机锁屏返回 `10106102`；解锁后运行 `hdc shell aa start -a EntryAbility -b com.example.gpuiohosmobilecounter` 成功。当前版本可用 `hdc shell hilog -x -T GPUI_M4_2` 观察 GPUI Scene、触摸分发、按钮数值和绘制日志。M4-2 真机结果见上文及阶段记录。
