# GPUI 计数器鸿蒙迁移：任务与验证记录

状态：桌面基线、M0、M1-1、M1-2、M1-2A、M1-3、M2-1、M2-2 和 R-1 均完成。真机已显示 Rust/系统 Native Drawing 绘制的背景与圆角矩形，并验证前后台、横竖屏、尺寸变化、Surface 重建及触摸探针；GPUI 场景绘制仍待后续任务。下一项为 M3-1，本次未实施。技术边界与验收见 [requirements.md](requirements.md)；本次操作见 [R-1 记录](r-1-code-organization.md)。本计划使用 SDK 自建 FFI。

## 已完成：桌面基线与 SDK 静态核对

- [x] **B-1** macOS arm64 桌面计数器固定官方 `gpui 0.2.2`，`cargo fmt --check`、`cargo check --locked`、`cargo build --locked` 通过；2026-10-20 GUI 实测 `Reset`、`+`、`−` 显示 `0 → 1 → 0`。记录见 [`gpui/desktop-counter/README.md`](../../gpui/desktop-counter/README.md)。
- [x] **B-2** 本机找到 API 26 Native SDK、`aarch64-unknown-linux-ohos` Rust 目标；直接从 SDK 头文件核对 XComponent Surface/触摸/逐帧回调、Node-API 注册、Native Drawing 屏幕 Surface 与 Flush、Native VSync 声明。其中 `OH_Drawing_SurfaceCreateOnScreen` 和 `OH_Drawing_SurfaceFlush` 标记为 API 16 起。以上均为**静态检查**，未证明目标手机支持或运行正确。

## M0：固定版本与设备能力

- [x] **M0-1** 用 `hdc` 确认手机 `ALN-AL00`、OpenHarmony `7.0.0.105` / API 26、aarch64 / arm64-v8a；核对本机两套 API 26 Native SDK 与 Rust OHOS target。
- [x] **M0-2** 在独立手机工程固定 GPUI 官方提交及 `Cargo.lock`，关闭默认 feature；`cargo check --locked --target aarch64-unknown-linux-ohos` 通过。M0 锁文件为 312 个包，尚未评估全部传递依赖的运行行为。
- [x] **M0-3** 确认 OHOS Rust target 同时匹配 `target_os = "linux"`、`target_env = "ohos"`；手机工程不引入 `gpui_platform` 桌面后端。桌面计数器保持原样，`cargo check --locked --offline` 仍通过。
- [x] **M0-4** 确认 DevEco 签名配置和真机 Native Drawing / Node-API 动态库实际加载；M1-2 的 HarmonyOS 调试签名 HAP 已安装并启动，`GPUI_M1_2` 日志显示 Rust 探针值 `7`。详见 [M0-4 记录](m0-validation.md#m0-4签名与真机动态库加载预检)和 [M1-2 记录](m1-2-hap.md)。
- [x] **M0-5** 逐项审计 GPUI 中 Linux 条件编译的运行语义；确认 `gpui_platform` 会误选 Linux 桌面后端，手机工程继续只依赖 `gpui` 核心。核心的 Linux 分支可由手机平台实现处理，首版暂不需要补丁；悬停、键盘和字体行为留待相应运行阶段验证。已记录桌面版与固定提交的启动 API 差异；视图级差异在实际移植时继续记录。详见 [M0-5 审计](m0-5-audit.md)。

## M1：SDK FFI 与最小 HAP

- [x] **M1-1** 在 `gpui/ohos-mobile-counter/` 增加首批 `sdk_ffi`：手写 XComponent/Native Drawing 后续任务所需的 C ABI 类型、常量、函数和系统库链接。两套 API 26 SDK 的目标 Clang 布局断言、aarch64 Rust 链接探针、目标 `cargo check` 通过；实际调用和真机加载未验证，见 [M1-1 记录](m1-1-ffi.md)。
- [x] **M1-2** 用系统 Node-API 小型 C/C++ 注册层连接 Rust `staticlib`，ArkTS 页面创建 `XComponent`；配置本工程签名、打包并安装 HAP，在应用进程内确认 Native Drawing / Node-API 动态库与原生模块可加载、日志可见；据此关闭 **M0-4**。真机 `loadContent succeeded`、`Rust probe bits: 7`，见 [M1-2 记录](m1-2-hap.md)。
- [x] **M1-2A** 根据用户确认，将 M1-2 的运行时 C++ Node-API 桥接替换为 SDK C ABI 的手写 Rust FFI；Rust `cdylib` 直接作为原生模块打包。保留不进入 HAP 的 C ABI 验证探针；重新构建、签名、检查 HAP 原生库及真机 `probe bits: 7`。真机复验成功，见 [M1-2A 记录](m1-2a-rust-napi.md)。
- [x] **M1-3** 注册 Surface 创建/变化/销毁、触摸回调，核对返回值、回调线程、窗口指针与尺寸；实现防重复初始化及资源释放。真机经 Hide/Show 与 Resize 按钮验证 `created → changed → destroyed → created`，重建后触摸仍有效；本阶段无 Drawing 资源，静态回调表保留进程期，销毁时清空窗口状态。详见 [M1-3 记录](m1-3-surface.md)。

## M2：系统 Drawing 真机探针

- [x] **M2-1** 用 SDK FFI 在同一原生窗口上创建 GPU context/屏幕 Drawing Surface，绘制纯色和圆角矩形并 Flush；记录像素格式、尺寸变化和错误码。真机可见深色背景与青色圆角矩形；`1260×2109 → 1008×2109` 重绘成功；NativeWindow 为 RGBA_8888、Flush 成功，资源替换/销毁日志已核对。详见 [M2-1 记录](m2-1-drawing.md)。
- [x] **M2-2** 测试前后台、旋转/尺寸变化、Surface 销毁与重建。真机横竖屏 `2720×642 ↔ 1260×1979` 重绘成功；Hide/Show 释放第一代 Drawing 并用第二代窗口 Flush；两次后台返回仍显示原有画面。这些序列未出现 Drawing 失败，因此无须启用 NativeWindow 缓冲区或 EGL 备用路线。详见 [M2-2 记录](m2-2-lifecycle.md)。

## R：M3 前代码整理

- [x] **R-1** 按 SDK 功能细分 Drawing 与 XComponent FFI；将 Node-API 宿主、XComponent 生命周期、触摸、日志和绘制探针分别收敛到清晰模块，保持现有 C ABI、回调行为、依赖和画面不变。交叉编译、HAP 构建与真机横竖屏、Hide/Show、Resize、触摸和后台返回回归通过，见 [R-1 记录](r-1-code-organization.md)。

## M3：GPUI 平台与绘制

- [ ] **M3-1** 实现 `gpui_ohos` 的 `Platform`、`PlatformWindow`、调度器、显示/密度、窗口句柄与宿主事件队列；接入 `Application::with_platform`/`run_embedded`，验证无回调重入与跨线程误用。
- [ ] **M3-2** 实现 `Scene::batches()` → Native Drawing：先画背景/矩形/圆角/边框/裁剪，再做自有 `PlatformAtlas` 和单色字形；记录此计数器实际产生的图元清单及缺口。
- [ ] **M3-3** 实现最小 `PlatformTextSystem`：系统字体、度量、字形布局/光栅化与 Atlas，显示 `GPUI Counter`、数字、`−` 和 `Reset`；与桌面基线比较布局。

## M4：交互、调度与验收

- [ ] **M4-1** 将触摸按下/移动/抬起/取消和密度换算送入 GPUI，实测三个按钮分别改变状态；禁止 ArkTS 直接修改计数值。
- [ ] **M4-2** 用 SDK XComponent 帧回调驱动待绘制帧；检查空闲/后台是否停止请求。确有缺口时再增加 SDK Native VSync FFI。
- [ ] **M4-3** 验证重复进入、前后台、Surface 重建、退出的回调注销和资源释放；执行格式/静态/构建检查、HAP 架构及动态依赖检查，并记录真机日志、操作和结果。
- [ ] **M4-4** 完善 `gpui/ohos-mobile-counter/README.md`，包含 SDK/API/设备、GPUI 提交、工程结构、构建签名安装步骤、预期结果、实测结果与未实现能力；代码及记录留本地供 review。

## 2026-10-20：子任务编号操作记录

| 顺序 | 操作与目的 | 实际结果 |
| --- | --- | --- |
| 1 | 读取本任务清单，确认全部子任务和已有完成状态。 | 找到基线 2 项、M0 5 项、M1 3 项、M2 2 项、M3 3 项、M4 4 项。 |
| 2 | 读取 `m0-validation.md`，核对已完成任务的验证依据。 | M0 设备、SDK、依赖和交叉编译记录存在；未验证项仍须保持未勾选。 |
| 3 | 读取仓库 `AGENTS.md`，核对任务告知与操作记录要求。 | 已按规范在执行前告知做法和原因，并在此记录操作。 |
| 4 | 为每条子任务增加阶段内固定编号。 | 已按原顺序标为 `B-1` 至 `B-2`、`M0-1` 至 `M4-4`；勾选状态未改变。 |
| 5 | 用脚本核对编号总数、唯一性、顺序及勾选数。 | 检查通过：共 19 项、19 个唯一编号、顺序正确；已完成 5 项、待办 14 项。此次仅改文档，无适用的代码构建检查。 |
