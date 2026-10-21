# M2-2：Drawing Surface 生命周期真机验证

日期：2026-10-21。范围：在 M2-1 的 Rust/SDK 绘制探针上验证前后台、横竖屏/尺寸变化和 Surface 销毁重建；检查旧窗口不再 Flush、资源释放、错误码与实际画面。若出现失败，先记录最小复现和 SDK 返回值，再判断是否需要研究 NativeWindow 缓冲区或 EGL；本阶段不引入新渲染库，不进入 M3。

| 顺序 | 操作与目的 | 实际结果与验证状态 |
| --- | --- | --- |
| 1 | 先向用户说明做法、原因和验证方法；读取仓库 `AGENTS.md`、GPUI 需求/任务、M2-1 记录，以及当前 Rust Drawing/XComponent/ArkTS 宿主代码。 | 确认 M2-1 的单帧创建、收窄重绘、显式隐藏/显示均已在真机验证；M2-2 的前后台与旋转尚未验证。本阶段记录已建立。 |
| 2 | 在现有签名 HAP 上重启应用，记录前台基线；执行 `uitest uiInput keyEvent Home` 切后台，再 `aa start` 返回，查 `GPUI_M2_1` 日志和截图。 | 基线 `created`、NativeWindow `1260×2109`、Flush 成功。Home 与返回均未触发新的 Surface 创建/变化/销毁回调；返回后截图仍显示深色背景和青色圆角矩形。结论仅限此设备/此轮操作：后台不必然销毁 Surface。 |
| 3 | 查 API 26 SDK `@ohos.window.d.ts` 与[华为窗口方向文档](https://developer.huawei.com/consumer/cn/doc/doccenter-capabilities/window-rotation)，准备主动横竖屏测试。 | `window.getLastWindow(context)` 和 `Window.setPreferredOrientation(Orientation.LANDSCAPE/PORTRAIT)` 可用，API 9 起；计划只在 ArkTS 页面加测试按钮，绘制仍由 Rust 执行。尚未真机验证。 |
| 4 | 在 ArkTS 页面增加 Landscape/Portrait 测试按钮，通过系统窗口 API 请求方向；把本轮日志 tag 更新为 `GPUI_M2_2`。 | 仅改变宿主的测试控制与日志标识，Rust Drawing 绘制逻辑不变；两个方向的按钮均可触达。 |
| 5 | 执行 `cargo fmt --check`、目标 `cargo check --locked --offline`、`scripts/build-native.sh` 和 DevEco `hvigorw ... assembleHap`。 | Rust 格式、交叉检查、原生构建、ArkTS 编译、HAP 打包与签名均通过；ArkTS 仍给出现有原生模块验证的非阻断警告。 |
| 6 | 安装并启动本轮签名 HAP，观察初次 Surface 创建和 Flush。 | `probe bits: 7`，`created` 时窗口缓冲区 `1260×1979`、格式值 `12`（RGBA_8888），Drawing 为 `RGBA_8888(4)/PREMUL(2)`，`generation=1` Flush 成功。尺寸比 M2-1 小，是页面新增方向按钮占用了高度。 |
| 7 | 点击 Landscape、Portrait 并复查 `GPUI_M2_2` 日志及两种方向的真机截图。 | `changed` 在同一代窗口上依次报告 `2720×642 → 1260×1979`，NativeWindow 几何尺寸与之相同；每次都记录 `old_released=true` 和 Flush 成功。横屏、竖屏截图均能看到深蓝背景与青色圆角矩形，无黑屏或裁剪错误。 |
| 8 | 竖屏点击 Hide Surface，再点击 Show Surface，紧跟着读取回调和 Drawing 结果。 | `destroyed: generation=1 ... drawing_released=true`；显示后注册新的 XComponent，`created: generation=2` 且窗口指针改变，`1260×1979` Flush 成功。此段日志未出现对旧窗口的后续 Flush。 |
| 9 | 第二代 Surface 上执行 Home → `aa start` 返回，截图检查；再切横屏并切回竖屏。 | 前后台期间未观察到新的 XComponent 回调；返回截图仍有完整 Drawing 画面。随后 `generation=2` 先后按 `2720×642` 和 `1260×1979` 重建并 Flush，均 `old_released=true`。说明这轮后台返回保留了 Surface；不等同于系统保证后台永不销毁。 |
| 10 | 从最终签名 HAP 提取唯一原生库，执行 `file`、SDK `llvm-readelf -d` 检查。 | HAP 仅有 `libs/arm64-v8a/libgpui_mobile_counter.so`；ELF64/AArch64。动态依赖只有系统 `libnative_drawing.so`、`libnative_window.so`、`libhilog_ndk.z.so`、`libace_napi.z.so`、`libace_ndk.z.so`、`libc.so`；未引入外部渲染库。 |
| 11 | 更新阶段任务、需求与工程 README，并复核本轮边界。 | M2-2 真机完成；未修改 GPUI 平台或 Scene 绘制，也未启用 NativeWindow 缓冲区/EGL 备用路线。 |

真机：`ALN-AL00`、OpenHarmony `7.0.0.105`、API 26、aarch64。测试界面上的 Landscape/Portrait 仅用于触发生命周期；显示内容仍由 Rust 调用 SDK Native Drawing 生成，并非 GPUI Scene。截图仅保存在本次测试的临时目录，不进入仓库。本轮所有观察到的 Drawing 调用均成功，未出现需要切换到 NativeWindow 缓冲区或 EGL 的失败；系统可能采用其他前后台回收时序，后续 GPUI 接入后仍需重复生命周期测试。

复现时从 `gpui/ohos-mobile-counter/` 按 README 构建并签名，再执行：

```sh
hdc install entry/build/default/outputs/default/entry-default-signed.hap
hdc shell aa start -a EntryAbility -b com.example.gpuiohosmobilecounter
hdc shell hilog -x -T GPUI_M2_2
```

依次点击 Landscape、Portrait、Hide Surface、Show Surface，按 Home 后再次用 `aa start` 返回。每一步读取日志中的 `generation`、`native_geometry`、`drawing_released`、`old_released`、`Drawing flush ok`，并检查屏幕上背景和圆角矩形是否可见。
