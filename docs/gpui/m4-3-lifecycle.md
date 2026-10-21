# M4-3：GPUI 生命周期与最终技术验收

状态：已完成。本阶段核对重复进入、前后台、Surface 重建及真正终止 Ability 时的帧回调注销和资源释放，完成静态、构建、HAP 与真机验证；M4-4 的 README 全面整理仍待后续任务。

## 2026-10-21 操作记录

| 顺序 | 操作对象 / 命令与目的 | 实际结果 | 验证状态 |
| --- | --- | --- | --- |
| 1 | 阅读仓库 `AGENTS.md`、`docs/gpui/tasks.md`、`docs/gpui/requirements.md`，检查工作区与构建工具环境，明确 M4-3 范围。 | `main` 与 `origin/main` 同步且工作区干净；M4-2 已完成，M4-3 待实施。当前 shell 中 `cargo` 可用，`hdc`、`hvigorw` 和 SDK/JDK 变量未设置，后续需用本机已安装工具路径临时配置。用户先前要求 GPUI 工程不参考或依赖 `ohos-sys`，继续以本工程手写 SDK FFI 为准。 | 范围已确认；构建和设备待验证 |
| 2 | 将 M4-3 标为实施中并建立本记录。 | 后续按顺序记录代码审查、必要修复、构建、HAP 和真机操作。 | 文档已建立 |
| 3 | 阅读 `src/host/{surface,frame,napi}.rs`、`src/gpui/{mod,window}.rs`、`src/render/screen.rs` 与 ArkTS 页面/Ability 生命周期代码，检查资源与事件顺序。 | Surface 销毁先释放 Drawing 并注销帧回调，GPUI Destroyed 事件由 250 ms 安全检查处理；页面消失会暂停帧并停止定时器。若页面销毁事件晚于 `aboutToDisappear`，队列中的 GPUI 窗口关闭可能留到下次进入，需真机观察。静态回调表按进程期保留。 | 静态已审查；时序待真机验证 |
| 4 | 查找本机 DevEco 工具及 API 26 SDK，用于后续构建、HAP 检查和设备操作。 | 找到 DevEco 自带的 `hvigorw`、`hdc`、JBR 和 SDK；仅在 shell 中临时配置，不向仓库写入本机绝对路径。 | 工具已定位；设备待连接 |
| 5 | `hdc list targets` 与 `hdc shell` 查询设备型号、系统/API 版本和架构。 | 默认沙箱中 HDC 服务连接失败；获准访问设备后发现一台设备，型号 `ALN-AL00`、OpenHarmony `7.0.0.105`、API 26、`aarch64`。 | 真机已连接 |
| 6 | 在手机工程运行 `cargo fmt --all --check`、`cargo check --locked --offline --target aarch64-unknown-linux-ohos`、`scripts/build-native.sh` 和 DevEco `hvigorw ... assembleHap`。 | 格式、目标静态检查、Rust 原生库构建和签名 HAP 构建均通过。SDK/JBR 环境仅在当前命令中配置。 | 静态与构建已通过 |
| 7 | `unzip -l` 检查签名 HAP，提取其中原生库到临时文件并用 SDK `llvm-readelf -h -d` 检查后删除临时文件。 | HAP 有且仅有一份 `libs/arm64-v8a/libgpui_mobile_counter.so`，为 ELF64/AArch64；动态依赖为 native_display_manager、native_drawing、hilog_ndk、ace_napi、native_window、ace_ndk、time_service_ndk 和 libc，均为现有系统库。 | 包架构/动态依赖已通过 |
| 8 | `hdc install` 安装本轮签名 HAP，随后 `hdc shell aa start ...` 启动。 | 安装成功；设备当前锁屏，启动被系统拒绝，返回 `10106102`。已请用户解锁并保持亮屏；此时不能记作真机生命周期通过。 | 安装通过；启动待解锁 |
| 9 | `hdc shell hilog -h` 检查日志读取选项。 | `hilog -x` 可非阻塞读取缓冲区，`-z` 限制末尾行数，`-T` 或 `-P` 可筛选标签或进程；后续无需清空设备全局日志。 | 工具用法已核对 |
| 10 | 设备解锁后重新 `aa start`，用 `hilog -x -T GPUI_M4_2` 读取启动日志；`uitest dumpLayout` 定位宿主按钮。 | 应用在进程 `55502` 启动，第一代 Surface 创建、帧回调、GPUI 窗口打开和 4 Quad/19 字形 Scene Flush 均成功。`hilog -x` 与 `-z` 不能组合，改为 `hilog -x -T ...` 后在主机端取末尾行。页面按钮边界已从布局读取；设备临时布局文件后续清理。 | 真机启动已通过 |
| 11 | 用 `uitest uiInput click` 控制 Hide/Show Surface，并按进程日志核对销毁、重建和绘制。 | 第一代 `destroyed` 记录 `drawing_released=true`，约 233 ms 后 GPUI 处理 Destroyed；后续新代 Surface 创建、GPUI 窗口打开和 Scene Flush 成功。期间还出现一次额外 Hide/Show，疑似设备上并发手动操作；最终稳定在第三代 Surface，因此不把这两次都归因于单个受控点击。 | 真机资源释放及重建通过；并发触摸来源未定 |
| 12 | 点击 Resize Surface，检查尺寸变化与旧 Drawing 释放。 | 第三代 Surface 从 `1260×1918` 变为 `1008×1918`，`old_released=true`，GPUI 处理 Changed 后重新 Flush。 | 真机尺寸变更通过 |
| 13 | `uitest uiInput keyEvent Home`、后台日志、再 `aa start` 返回，抓临时截图并受控点击 `+`。 | `15:58:32.316` 记录 `GPUI frame paused requests=211 callbacks=211`；后台间隔无后续帧日志；返回 `15:58:54.460` 记录 resumed。截图显示 GPUI 计数器文字、按钮和 `0`，没有黑屏。受控 `+` 的 GPUI 回调把数值改为 `1` 并于 `15:59:25.220` Flush；另有同时发生的手动触摸，不能归到受控序列。临时截图待清理。 | 真机后台暂停/恢复、画面与交互通过 |
| 14 | 用 `uitest uiInput keyEvent Back` 退出前台，检查同一进程日志和 PID，稍后 `aa start` 重入。 | `15:59:49.373` 帧暂停，`requests=callbacks=601`；旧进程 `55502` 仍存活，未收到 Surface destroyed，重入 `16:00:20.379` 只记录 resumed。这是系统保留的后台 Ability/Surface，不能当成页面销毁或 Drawing 释放。 | 真机 Back→重入通过；不等同于进程退出 |
| 15 | 对进程 `55502` 检查 Drawing/崩溃关键字；`aa force-stop` 后确认 PID 消失，再 `aa start`。 | 旧进程未检出 `NativeWindowFlushBuffer`、`FlushVulkanSurface`、`GPUI Scene draw failed` 或崩溃关键字；force-stop 后 PID 消失。重启为新进程 `57584`，新的 `generation=1` Surface、GPUI 窗口和 Scene Flush 成功。强制停进程不提供原生析构日志，只能确认进程退出和新进程未带入旧状态。 | 真机进程退出/冷启动通过；应用级析构不可由 force-stop 证明 |
| 16 | 审核当前帧状态日志的可观测性，确定是否能证明 Surface 销毁时帧注册已取消。 | 目前仅注销失败会记录日志，Surface 销毁日志不显示注销前是否仍有待处理帧；M4-2 已验证帧回调内 SDK 自注销。拟补仅在生命周期边界输出的状态，随后重新构建与真机复验。 | 诊断缺口已确认；修复待验证 |
| 17 | 修改 `host/frame.rs` 和 `gpui/mod.rs`，在 Surface 丢失、后台暂停和 GPUI 窗口关闭时记录注册注销结果。 | 无待处理帧时记 `None`，有待处理帧时记录 SDK 注销成功与否；不增加逐帧日志，也不更改绘制路径。`cargo fmt --all --check`、目标 `cargo check`、原生库构建和签名 HAP 构建均通过。 | 静态与构建通过；真机待复验 |
| 18 | 检查更新包内 `.so` 架构/动态依赖，安装并启动，在真机控制 Hide/Show 与后台切换。 | 更新包仍只有一份 ELF64/AArch64 原生库，动态依赖仍为上述现有系统库。进程 `58433` Hide 时按顺序记录 `frame surface lost generation=1 pending_cancel_result=None`、`drawing_released=true`、`GPUI window closed generation=1`；Show 创建第二代 Surface 并 Flush。受控 `+` 使值为 `1`，Home 时 `requests=callbacks=8`、`pending_cancel_result=None`，返回后截图显示 `1`。截图已在本机与设备清理。 | 真机注销状态、资源释放、重建和可见交互通过 |
| 19 | 阅读本机 API 26 ArkTS `UIAbilityContext` 类型声明，查找可触发真正 Ability 终止的接口；增加页面 `Exit Demo` 测试按钮和 Ability 销毁日志。 | SDK 声明 `terminateSelf(): Promise<void>`；现有 Back 仅使进程保留并进入后台，不能验证 Ability 销毁。新增按钮调用系统 `terminateSelf()`，`onWindowStageDestroy`/`onDestroy` 输出日志。构建与真机结果待核对。 | SDK 静态已核对；代码未验证 |
| 20 | 构建、安装含 `Exit Demo` 的签名 HAP，按布局坐标点击退出，检查 Ability/Surface/GPUI 日志与 PID。 | ArkTS 编译和签名通过，仅有现有 Node-API 类型“尚未验证”警告。进程 `61625` 的 `terminateSelf` 成功；`16:05:21.898` 暂停时 `requests=275 callbacks=274`，SDK 待处理帧注销结果 `Some(true)`；随后 `drawing_released=true`、WindowStage/Ability 销毁，进程消失。但 GPUI Destroyed 事件在页面定时器停止后未处理，未记录 GPUI 窗口主动关闭。 | 真机退出确认清理缺口 |
| 21 | 增加 Rust Node-API `gpuiStop()`，由 Ability `onDestroy` 调用；它暂停帧、释放仍保留的 Surface/Drawing 状态、关闭 GPUI 窗口并清空宿主事件及弱帧引用。 | 首次补丁因 ArkTS 类型声明实际采用 `export const` 而未应用，核对文件后重新应用。`cargo fmt --all --check` 首次指出两处换行格式，运行 `cargo fmt --all` 修正；目标 `cargo check --locked --offline --target aarch64-unknown-linux-ohos`、`scripts/build-native.sh`、签名 HAP 构建均通过。没有新增库或 SDK 绑定。 | 静态/构建通过；真机待复验 |
| 22 | 安装最终 HAP，重新启动并点击 `Exit Demo`，检查资源清理次序；确认进程退出并再次 `aa start`。 | 进程 `62786` 的 `terminateSelf` 后依次记录帧暂停、`surface lost`、`drawing_released=true`、WindowStage 销毁、`surface shutdown drawing_released=false`（已由 SDK 回调释放）、`GPUI window closed on Ability exit generation=Some(1)`、`ability destroyed gpuiStop=1`；旧 PID 消失。新进程 `63034` 重新创建第一代 Surface 与 GPUI 窗口并 Flush；临时截图可见数字 `0`，受控 `+` 点击产生日志 `value=1` 与后续 Scene Flush。 | 真机正常终止、主动清理、重启及交互通过 |
| 23 | 对最终 HAP 运行 `unzip -l`、SDK `llvm-readelf -h -d`，检查两轮进程错误日志；复核格式、工作区差异并清理临时文件。 | 最终 HAP 仍只有一份 arm64-v8a ELF64/AArch64 原生库，动态依赖仅为已列系统库；进程 `62786`、`63034` 未检出缓冲区 Flush 错误、Scene draw failed 或崩溃关键字。`cargo fmt --all --check` 与 `git diff --check` 通过。设备与本机的临时布局、截图及 ELF 提取文件均已删除。 | 最终静态、包与日志检查通过 |
| 24 | 对照任务、需求和工程 README 同步 M4-3 结果，重跑 Rust 格式及 `git diff --check` 并检查工作区。 | 三份文档已注明真机结果、`Exit Demo` 的用途和未验证的长期运行/呈现耗时；格式和差异检查再次通过。M4-3 改动仅留在本地工作区，未提交或推送。 | 文档与最终差异已核对 |

本阶段在 `ALN-AL00` / API 26 上确认：Hide/Show 会释放旧 Drawing 并关闭旧 GPUI 窗口，重建后可绘制；Resize 会替换 Drawing；Home/Back 将保留的 Ability 放入后台并停止帧，返回后继续交互；`Exit Demo` 真正终止 Ability 时可取消待处理帧、释放 Drawing 并主动关闭 GPUI 窗口，冷启动无旧状态。`aa force-stop` 只能证明进程退出，不能用作应用级析构证据。测试覆盖受控操作和这一台设备，未进行长时间压力测试，也未测量 Flush 到屏幕实际呈现的时间。
