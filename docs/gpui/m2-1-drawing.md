# M2-1：Native Drawing 屏幕 Surface 真机探针

日期：2026-10-21。范围：仅在 M1-3 已验证的 XComponent 原生窗口上，用 API 26 SDK 的 Rust FFI 创建 GPU context / 屏幕 Surface，绘制纯色与圆角矩形并 Flush；记录尺寸、像素格式、错误码、真机画面和资源释放。本阶段不接入 GPUI Scene，也不开展 M2-2 的前后台/旋转专项验证。

| 顺序 | 操作与目的 | 实际结果与验证状态 |
| --- | --- | --- |
| 1 | 执行前告知本次做法、原因和验证方式；读取仓库 `AGENTS.md`、GPUI 需求/任务、M1-3 源码、已有 Drawing FFI 和 ArkTS 宿主。 | 确认 M1-3 已完成，M2-1 待做；已有 Drawing 函数声明但尚无真正绘制路径；本文件此前不存在。真机绘制未验证。 |
| 2 | 核对 API 26 SDK 的 `drawing_surface.h`、`drawing_gpu_context.h`、`drawing_types.h`、canvas/brush/round-rect/error 头文件，并查阅官方资料。 | `SurfaceCreateOnScreen`/`Flush` 为 API 16 起；`ImageInfo` 宽高为像素，选用 SDK 枚举 `COLOR_FORMAT_RGBA_8888=4`、`ALPHA_FORMAT_PREMUL=2`；`Flush` 返回 Drawing 错误码。空指针或 void 返回的接口须用 `OH_Drawing_ErrorCodeGet` 并在调用前重置。仅完成静态核对。 |
| 3 | 核对 SDK `native_window/external_window.h` 的 `GET_BUFFER_GEOMETRY`/`GET_FORMAT`，决定屏幕 Surface 的像素尺寸来源。 | SDK 提供 `OH_NativeWindow_NativeWindowHandleOpt`；几何查询的输出参数顺序为 `height, width`。计划成功时优先采用 NativeWindow 缓冲区尺寸，失败或零尺寸时记录错误并回退到 XComponent 宽高。尚未真机验证。 |
| 4 | 增加 `sdk_ffi/native_window.rs` 的最小查询绑定和 Drawing 抗锯齿声明；新增 `drawing_probe.rs` 的 GPU context、屏幕 Surface、背景/圆角矩形、Flush 与 RAII 清理；在 `surface.rs` 的创建/变化/销毁回调中接入。 | 代码已写入；绘制选 `RGBA_8888` / `PREMUL`，每个返回指针和 Drawing 错误状态均检查，变更时重建 Drawing Surface，销毁时先释放 Surface 再释放 GPU context。仅静态验证。 |
| 5 | 执行 `cargo fmt`、目标 `cargo check --locked --offline` 和 `scripts/build-native.sh`，用 SDK `llvm-readelf` 检查产物。 | 格式、交叉检查、原生构建通过；产物为 ELF64/AArch64，新增系统 `libnative_window.so` 动态依赖，无新增 Cargo 依赖。真机绘制仍未验证。 |
| 6 | 用 DevEco `hvigorw ... assembleHap` 构建并签名；`hdc list targets`、安装、强制停止/启动应用，再用 `hilog -x -T GPUI_M2_1` 查结果。 | 构建/签名、真机安装/启动均成功。创建时 XComponent 尺寸和 NativeWindow 缓冲区均为 `1260×2109`；窗口格式值为 `12`，SDK `buffer_common.h` 对应 `NATIVEBUFFER_PIXEL_FMT_RGBA_8888`；Drawing 选 `RGBA_8888(4)`、`PREMUL(2)`，日志 `Drawing flush ok`。 |
| 7 | 用 `uitest screenCap` 截取本应用并在本机视觉检查，区分 Flush 成功与真正显示。 | 画面显示深色背景与青色圆角矩形；截图仅用于本次验证，保留在临时目录，未写入仓库。真机像素显示已验证。 |
| 8 | 点击 `Resize Surface`，读 `GPUI_M2_1` 日志，再截图检查重绘。 | `changed` 时 NativeWindow 与 XComponent 均变为 `1008×2109`，格式仍为 `12`，第二次 Flush 成功；截图确认 XComponent 收窄后背景和圆角矩形按新宽度显示。 |
| 9 | 检查重建/销毁日志可观测性，并补强空指针失败的错误报告。 | 代码改为记录旧 Drawing 资源是否释放；若 SDK 返回空指针但错误状态为成功，用内部 `-1` 标明“空指针无错误码”。此补强尚未重新构建与真机验证。 |
| 10 | 再次执行 `cargo fmt --check`、目标 `cargo check --locked --offline`、原生构建和 `hvigorw ... assembleHap`，安装最终 HAP 并启动。 | 静态检查、构建、签名、安装和启动均通过；最终 HAP 的首次 `created` 为 `1260×2109`、NativeWindow 格式 `12`、`Drawing flush ok`，`old_released=false`。 |
| 11 | 点击 `Hide Surface`，再点 `Show Surface`，查询最终 HAP 的资源与绘制日志。 | `destroyed` 日志 `drawing_released=true`；新 XComponent 注册后 `generation=2`，再次创建 GPU context/屏幕 Surface 并 Flush 成功。第二代窗口地址可与第一代相同，本工程以代号区分生命周期。 |
| 12 | 在最终 HAP 上点击 `Resize Surface`，核对资源替换、像素尺寸和 Flush。 | `changed` 后尺寸与缓冲区几何均为 `1008×2109`，格式仍为 `12`；日志 `Drawing flush ok ... old_released=true`。此过程没有观察到 Drawing 错误码或应用崩溃。 |
| 13 | 最终执行 `cargo fmt --check`、目标 `cargo check --locked --offline`，从签名 HAP 提取原生库并查 ELF 架构/动态依赖。 | 检查通过；HAP 仅有一份 `arm64-v8a` 原生库，ELF64/AArch64，依赖系统 `libnative_drawing.so`、`libnative_window.so`、`libhilog_ndk.z.so`、`libace_napi.z.so`、`libace_ndk.z.so` 和 `libc.so`。 |
| 14 | 更新 `tasks.md`、`requirements.md` 和工程 README，使 M2-1 状态、真机画面、依赖及未做范围一致。 | M2-1 标为完成，M2-2 仍待办；README 写明显示图形来自 Rust/系统 Drawing，不是 GPUI Scene。 |
| 15 | 检查本阶段文档是否含本机绝对路径、签名材料、口令或设备序列号，并复核只实施 M2-1。 | 本阶段只新增手写 SDK FFI 与 Drawing 探针，没有增加 Cargo 依赖；个人配置不进入本阶段文档。 |

真机：`ALN-AL00`、OpenHarmony `7.0.0.105` / API 26、aarch64。截图视觉核验：XComponent 从原本黑色变成深蓝背景，中间有青色圆角矩形；收窄后仍按新宽度绘制。背景与形状由 Rust 直接调用系统 Native Drawing 生成，尚未由 GPUI Scene 生成。`NATIVEBUFFER_PIXEL_FMT_RGBA_8888=12` 是 NativeWindow 缓冲区格式；Drawing 的 `COLOR_FORMAT_RGBA_8888=4` 属另一套枚举，数值不同但格式名称一致。当前仅验证了页面按钮触发的宽度变化和显式隐藏/显示；前后台、旋转与其他尺寸变化序列留给 M2-2。
