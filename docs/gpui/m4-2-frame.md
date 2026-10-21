# M4-2：按需帧调度

状态：已完成。用 API 26 SDK 的 XComponent 帧回调驱动 GPUI 待绘制帧，已验证点击后的刷新以及空闲、后台时不持续请求。本阶段无需增加 SDK Native VSync FFI；M4-3 的完整生命周期验收仍待执行。

## 2026-10-21 操作记录

| 顺序 | 操作对象 / 命令与目的 | 实际结果 | 验证状态 |
| --- | --- | --- | --- |
| 1 | 用户要求先推送再开始 M4-2；核对 `git status`、M4-1 差异、`AGENTS.md`、任务文档与工程格式。 | M4-1 改动经 `git diff --check`、`cargo fmt --all --check`；提交 `9f24dee` 已推送到 `origin/main`。本次 M4-2 在此提交后开始。 | 版本状态已核对 |
| 2 | 阅读 API 26 SDK `native_interface_xcomponent.h`、现有 XComponent FFI、Surface/触摸宿主、GPUI 平台、定时器与固定 GPUI 源码，确定帧请求关系。 | SDK `RegisterOnFrameCallback` 注册后**每帧**调用，`UnregisterOnFrameCallback` 可停止；它不是单次请求。GPUI 平台提供 `frame_waker()`、`schedule_frame()` 和 `on_request_frame()`，当前实现只靠 ArkTS 50 ms 定时器轮询且未实现 waker。回调线程语义未在头文件中明确，需要真机验证。 | SDK/源码静态已核对；线程与行为待真机验证 |
| 3 | 将 M4-2 标为实施中并建立阶段记录，按需帧方案将先验证回调线程再切换宿主调度。 | 本文件记录后续实现、构建、真机操作和未验证项。 | 文档已建立 |
| 4 | 增加 `sdk_ffi/xcomponent/frame.rs` 最小 Rust FFI、`host/frame.rs` 三帧线程探针，并在 Surface 创建时注册；仍保留 50 ms 定时器。 | `cargo fmt --all`、原生目标构建通过。首次 `scripts/build-native.sh` 因当前 shell 未设置 SDK 环境变量失败，按工程脚本要求设置本机 SDK 后成功；HAP 首次因 `JAVA_HOME` 指向 JBR 外层目录失败，改为 JBR 的 `Contents/Home` 后签名构建通过。 | 静态/构建已通过 |
| 5 | 安装启动首轮探针，查 `GPUI_M4_1` 日志。 | XComponent 帧回调、GPUI pump 均在 `ThreadId(1)`；前三帧日志分别为 `15:18:46.348/.351/.362`，由定时器中的宿主方法在 `15:18:46.398` 注销。 | 真机线程已验证 |
| 6 | 将探针改为第三帧在回调内调用 SDK 注销函数，重新格式化、构建、签名、安装，强制停止并启动应用。 | 新进程 `15:21:27.336` 的第 3 帧在 `ThreadId(1)` 自注销，SDK 返回 `0`，后续没有探针帧日志；说明此机可以从回调内部停止逐帧通知。 | 真机已验证 |
| 7 | 对比受控 `+` 点击、系统 BufferQueue/2DGraphics 日志与临时截图。 | SDK 抬起 `15:21:48.240`、GPUI 计数变为 `1` 在 `.285`；系统 `.292` 报 `NativeWindowFlushBuffer failed ret=41207000`，但项目自己的 `OH_Drawing_SurfaceFlush` 状态检查仍记作成功。SDK 错误码头文件把 `41207000` 定义为 `NATIVE_ERROR_BUFFER_STATE_INVALID`。稍后截图可见 `1`，因而不能据此断定该次画面完全丢失，也不能把 Flush 返回成功当作实际呈现时间。截图仅临时检查，已从本机和设备删除。 | 真机复现底层错误；呈现延迟原因未定 |
| 8 | 用已验证的注册/自注销行为实现 `host/frame.rs` 按需帧状态、GPUI `frame_waker`/`schedule_frame`、帧回调中的 GPUI 绘制；ArkTS 宿主检查改为 250 ms，增加前后台暂停/恢复。 | XComponent 有待处理帧时才注册；每次回调先注销，再在 ArkUI `ThreadId(1)` drain 事件、执行 GPUI 帧。`cargo fmt`、OHOS 原生构建和签名 HAP 构建通过；ArkTS 仅报告现有原生模块类型未验证警告。 | 构建已通过，真机待核对 |
| 9 | 安装启动按需帧 HAP，观察启动、空闲和手动/受控点击。 | 初始 `15:26:29.546` 请求帧，`.553` 回调，`.597` Flush；之后至 `15:26:52.621` 约 23 秒无帧请求，说明空闲停帧。触摸时重新请求，计数回调在抬起后数毫秒内触发。但重复绘制仍出现 `NativeWindowFlushBuffer failed ret=41207000`，按需帧不能单独消除该底层错误。 | 真机按需触发/空闲停帧已验证；Drawing 缓冲问题待处理 |
| 10 | 核对 `41207000` 的 SDK 定义，给 `ScreenDrawing` 加逐帧重建 on-screen Surface 的诊断实验，保持 GPU context 复用；更新原生日志标签并重新格式化、交叉构建、签名。 | SDK 将错误定义为缓冲区状态无效；实验版三项构建均通过。是否能消除系统内部 FlushBuffer 错误尚未验证，不能据此保留为最终方案。 | 构建已通过；实验效果未验证 |
| 11 | 安装实验 HAP 并尝试 `aa start`。 | 安装成功；启动被设备自动锁屏拒绝，系统返回 `10106102`。已请求用户解锁，未尝试输入密码或将安装当作真机功能通过。等待期间继续核对帧状态锁与 Surface 代次。 | 真机实验受锁屏阻碍 |
| 12 | 设备可启动后，在已安装的 Surface 重建实验包上受控点击 `Reset → + → −`，对照 `GPUI_M4_2` 和系统 BufferQueue 日志。 | 三次点击令 GPUI 数值 `0 → 1 → 0`，相应 Scene Flush 约 7–8 ms；当前应用进程日志中没有再出现 `NativeWindowFlushBuffer`、`FlushVulkanSurface` 或 `GPUI Scene draw failed`。此前多次手动点击也未见该错误。逐帧重建是此设备上的有效实验结果，后续还须检查可见画面和 Surface 生命周期。 | 真机受控交互及系统日志已通过；画面/生命周期待核对 |
| 13 | `uitest uiInput keyEvent Home`、`aa start` 返回，随后 Hide/Show Surface，比较帧计数、代次和 Scene。 | Home 后 `15:33:00.682` 记录 `paused requests=823 callbacks=823`，至 `15:33:12.996` 返回前无新帧；返回后恢复并收到下一帧。Hide 时第一代 Drawing 释放、销毁事件在 250 ms 宿主检查中处理；Show 后第二代 Surface `generation=2` 已 Flush 计数器 Scene。完整生命周期压力测试仍属 M4-3。 | 真机后台停帧、返回、基本 Surface 重建已通过 |
| 14 | 在第二代 Surface 受控点击并抓屏；第一次截图仍为 `0`，故启动实时应用日志捕获并重试。 | 第一次 SDK 抬起和 GPUI Touch 均到达，但未产生计数 `on_click`，不能将它写成绘制失败。再次受控点击：SDK Started `15:36:15.503`、Ended `.616`，GPUI `increment value=1` 在 `.622`、Scene Flush `.635`，随后截图确实显示 `1`。此前有同时进行的大量手动触摸，未能断言第一次漏点的原因；本轮复测确认第二代 Surface 可交互、可见。所有临时截图已从设备和本机删除。 | 第二代真机可见交互已通过；偶发未触发点击仍待观察 |
| 15 | 降低高频触摸移动与帧日志量，修正帧代次检查在锁内完成，保留逐帧重建 Drawing Surface 的实验结论，重新 `cargo fmt`、交叉构建、签名并安装。 | 构建、签名与安装通过；最终 HAP 的 arm64-v8a 原生模块为 ELF64/AArch64，动态依赖仅为现有系统库，没有新增第三方库。安装后设备再次锁屏，`aa start` 返回 `10106102`；最终日志降噪包的真机行为尚待解锁复验。 | 构建/架构已通过；最终包真机待验证 |
| 16 | 解锁后启动最终签名 HAP，检查进程 `42453` 的 XComponent 帧回调、空闲间隔与受控 `+` 点击；临时截图核对可见数值，并检查该进程系统日志。 | 启动帧回调于 `15:38:09.273` 到达，第二次于 `.323` 到达；随后约 14 秒无回调。受控点击的 SDK Ended 为 `15:38:23.505`，GPUI `increment value=1` 为 `.514`，Scene Flush 为 `.523`，分别相差约 9 ms 和 18 ms；截图显示 `1`。该进程未检出 `NativeWindowFlushBuffer`、`FlushVulkanSurface` 或 `GPUI Scene draw failed`。临时截图已从设备和本机删除。 | 最终包真机交互、空闲停帧和系统错误日志检查通过 |
| 17 | 对照任务、需求、README 同步 M4-2 结果与限制，运行 `cargo fmt --all --check`、`git diff --check`，查看 `git status --short`。 | 文档说明 XComponent 按需帧、250 ms 非绘制安全检查、逐帧重建 Drawing Surface 的设备实验依据；保留偶发点击未触发和实际屏幕呈现时间未测的限制。格式与差异检查均通过；M4-2 文件仍在本地工作区，未提交或推送。 | 文档/静态检查通过 |

本阶段真机证据表明，GPUI 点击处理与 Scene Flush 可在数十毫秒内完成，按需帧在空闲和后台会停下。先前的 `41207000` 系统缓冲区错误在逐帧重建屏幕 Drawing Surface 的最终进程中未再观察到，但不能据此证明所有设备或长时间运行都不会复现。截图验证了受控点击后的可见数字；尚未测量实际屏幕呈现时间，也未解释先前一次高频触摸期间未触发 `on_click` 的原因。重复进入、退出与更完整的 Surface 生命周期压力验收留给 M4-3。
