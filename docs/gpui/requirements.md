# GPUI 桌面计数器迁移到鸿蒙：需求与技术路线

状态：桌面计数器已在 macOS arm64 上编译并交互验证；独立手机工程已完成 M0、M1-1、M1-2、M1-2A、M1-3、M2-1、M2-2 和 R-1。系统 Native Drawing 的屏幕 Surface、纯色背景、圆角矩形及 Flush 已在真机显示；模块整理后前后台、横竖屏、尺寸变化、Surface 重建和触摸探针再次通过。GPUI 场景绘制和计数器交互尚未实现。分阶段任务见 [tasks.md](tasks.md)，最新操作与验证状态见 [R-1 记录](r-1-code-organization.md)。

## 目标与依赖边界

将 [`gpui/desktop-counter/`](../../gpui/desktop-counter/) 的 `0`、`+`、`−`、`Reset` 交互迁移到 HarmonyOS/OpenHarmony 手机。屏幕内容必须由 GPUI 的 `Render` 视图产生，触摸必须进入 GPUI 事件系统，再由 `cx.notify()` 触发重绘。ArkTS 只负责页面和生命周期，不能用 ArkTS 控件模拟计数器作为验收结果。

鸿蒙端直接依赖**固定提交的 GPUI 官方源码、本项目自写代码和目标 SDK/系统库**。不依赖或参考 `ohos-sys`，也不引入 winit、wgpu、`gpui_wgpu`、移动端 GPUI 封装或其他外部平台/渲染项目。SDK 缺少 Rust 声明时，在本示例的 `sdk_ffi` 模块手写最小 `extern "C"`、`#[repr(C)]` 类型和安全封装，以实际 SDK 头文件为 ABI 依据。根据用户确认，M1-2A 将 Node-API 模块注册和 Rust 入口桥接也改为直接 Rust FFI；C 源文件只用于不进入 HAP 的 ABI/设备预检。

“不引入第三方项目”在本计划中指**不新增平台适配、窗口或图形库**。GPUI 自身已有的传递依赖仍会进入 Cargo 构建；固定 GPUI 提交后须输出完整依赖清单并审计目标兼容性。若要求连 GPUI 的传递依赖都为零，必须另立替换这些依赖的工程，不能把该目标写成此示例已经满足。

## 版本与工程边界

- 当前桌面示例固定发布版 `gpui = 0.2.2`，使用 `Application::new()`，保持独立且不为手机适配修改。手机工程 `gpui/ohos-mobile-counter/` 固定官方源码提交 `c4a82ab1ab958aa43b0c0b15c0bad4fb77a7ca7f`，该源码提供 `Application::with_platform`/`run_embedded`；复用计数器视图时在手机工程内按此提交适配，不能把两个版本的 API 混用。
- 手机工程后续增加最小 DevEco/HAP 宿主、`gpui_ohos` Rust 平台 crate、`sdk_ffi`、薄 Node-API 注册桥接和计数器视图。所有 SDK 路径、签名材料和本机配置均不提交。
- 本机已有 API 26 Native SDK 和 `aarch64-unknown-linux-ohos` Rust 目标。已连接手机为 `ALN-AL00`、OpenHarmony `7.0.0.105`、API 26、aarch64；此前拒绝 SDK 测试签名，现已接受并运行本工程 HarmonyOS 调试签名 HAP。M2-1 已确认 API 16 起的 Native Drawing 屏幕 Surface 接口可在此机创建、绘制和 Flush；M2-2 的前后台、横竖屏重绘及显式 Surface 重建亦通过。GPUI 到 Drawing 的映射仍未实现。
- 鸿蒙 Rust 目标可能同时匹配 `target_os = "linux"` 与 `target_env = "ohos"`。编译阶段须关闭 GPUI 默认桌面 feature，逐项清理错误启用 X11/Wayland 等桌面路径的条件编译；必要时仅对固定 GPUI 源码维护可复现的最小补丁。

## 第一版接入路径

1. **宿主与 Surface。** ArkTS `XComponent(type: 'surface')` 提供可显示区域；手写 SDK FFI 注册 Surface 创建、变化、销毁和触摸回调，取得原生窗口指针、尺寸与触摸坐标。Node-API 桥接只传递初始化及生命周期信息。每次 Surface 创建分配代号，延迟任务携带代号，销毁后丢弃旧任务，禁止访问失效指针。
2. **系统绘制探针。** 优先用 SDK 的 `OH_Drawing_GpuContextCreate`、`OH_Drawing_SurfaceCreateOnScreen`、`OH_Drawing_SurfaceGetCanvas`、`OH_Drawing_SurfaceFlush` 在 XComponent Surface 上画背景与矩形。该路径由系统 Native Drawing 管理 GPU 后端，无需应用自己接入 Vulkan。先在真机确认窗口指针、像素格式、尺寸更新、错误码和资源释放；若失败，记录可复现证据，再评估同一 SDK 提供的原生窗口缓冲区或 EGL 接口。
3. **GPUI 宿主。** 本项目实现 `Platform`、`PlatformWindow`、调度器、显示信息和窗口句柄接口，用 `Application::with_platform` 与 `run_embedded` 接入 ArkUI 宿主。保存应用句柄；所有 GPUI 更新在确定的单一线程串行执行。原生回调只入队，不在回调中重入 GPUI；销毁时先停止帧请求，再释放 GPUI 窗口、Drawing Surface、GPU context 与回调。
4. **Scene → Native Drawing。** 实现 `PlatformWindow::draw(&Scene)` 与 `PlatformAtlas`。按 GPUI `Scene::batches()` 的顺序先支持此计数器实际使用的纯色背景、矩形/圆角矩形、边框、裁剪、单色字形；映射像素密度、颜色/透明度及 Atlas tile。遇到该页面实际产生而尚未支持的图元时输出明确错误，不把部分画面当作迁移完成。阴影、复杂路径、彩色图像和渐变按需求逐项增加。
5. **文字。** GPUI 的 `PlatformTextSystem` 需要字体选择、度量、字形布局和光栅化，不能仅在 `draw` 中直接绘制字符串。先用系统 Native Drawing 字体接口实现此计数器所需的拉丁字母、数字和 `−`，把光栅化结果写入自有 Atlas；比较桌面与真机的文字位置和点击区域。复杂文字、字体回退和输入法进入后续里程碑。若系统 API 无法提供某个必需信息，先做最小复现，再补足本项目 FFI/算法或评估 GPUI 核心的最小适配点。
6. **输入与帧调度。** 将 XComponent 触摸的按下、移动、抬起和取消转换为 GPUI 的窗口输入事件，按设备密度换算坐标，让 GPUI 自己命中 `+`、`−`、`Reset`。计数变化标记窗口待绘制；先验证 XComponent 的逐帧回调能否安全调度刷新，必要时直接绑定 SDK `native_vsync` 做按需帧请求。VSync 只提供时序，不创建 Surface。

## 验收与阶段限制

第一版须构建、签名并安装 aarch64 HAP；检查原生 `.so` 架构与动态依赖。真机上看到 GPUI 生成的文字和图形，触摸 `+`、`−`、`Reset` 后数字按预期变化；重复进入、退出、前后台切换及 Surface 重建无崩溃、黑屏或向失效 Surface 提交。日志可追踪 ArkTS → 桥接 → GPUI/SDK 的事件与错误。README 记录 GPUI 提交、SDK/API、目标设备、构建签名安装方法、结果和限制；未真机验证的项明确标注。

首版只承诺单窗口、基础绘制和触摸。键盘/输入法、剪贴板、无障碍、多窗口和完整 GPUI 图元覆盖在计数器跑通后按实际需求推进。每个阶段都以运行证据为准，编译成功不等于设备可用。

## 依据

- [GPUI 官方平台接口、窗口绘制及文字接口](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/platform.rs)
- [GPUI 官方 Scene 与图元批次](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/scene.rs)
- [GPUI 官方 Application 嵌入入口](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/app.rs)
- 本机 API 26 SDK 头文件：`ace/xcomponent/native_interface_xcomponent.h`、`native_drawing/drawing_surface.h`、`native_drawing/drawing_gpu_context.h`、`native_vsync/native_vsync.h`、`node_api.h`。实际实现时还须逐个核对目标设备对应版本的头文件与链接库。
