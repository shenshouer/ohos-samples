# M4-1：XComponent 触摸进入 GPUI 计数器

状态：已完成。范围仅为 M4-1，帧回调调度和完整生命周期验收分别留给 M4-2/M4-3。

## 2026-10-21 操作记录

| 顺序 | 操作对象 / 命令与目的 | 实际结果 | 验证状态 |
| --- | --- | --- | --- |
| 1 | 读取 `docs/gpui/requirements.md`、`tasks.md`、当前 Git 状态与手机工程触摸/Surface/GPUI 平台文件，确认 M4-1 边界。 | `main` 工作区干净；XComponent 触摸目前只由 `host/touch.rs` 记录日志，GPUI `PlatformWindow::on_input` 已保存回调但尚未分发；计数器根视图仍固定为 `0`。 | 静态已核对 |
| 2 | 将 M4-1 标为实施中并建立本阶段记录。 | 后续将依次记录接口核对、实现、构建、设备操作和结果。 | 未验证 |
| 3 | 查阅固定 GPUI 提交的 `PlatformInput::Touch`、`TouchEvent`、`TouchId`、窗口手势识别与 `on_input`，以及已安装 SDK 的 XComponent 触摸结构/枚举；用官方触摸文档交叉核对坐标含义。 | SDK `x/y` 相对 XComponent；GPUI 触摸须使用窗口逻辑坐标，且同一平台窗口的 `TouchId` 在后续触摸不可复用。现有宿主只日志化 SDK 事件，需通过队列在 ArkUI pump 分发，避免原生回调重入 GPUI。 | 静态已核对，真机待验证 |
| 4 | 修改宿主触摸适配：回调校验 SDK 事件类型与坐标，只复制事件并按 Surface 代次排队；在 ArkUI pump 将物理像素除以显示密度，分配不复用的 GPUI `TouchId`，调用 `PlatformInput::Touch`。 | 代码覆盖按下、移动、抬起、取消；失配代次和未配对/重复事件被忽略。GPUI 回调在原生回调返回后执行，避免直接重入。 | 已交叉编译，真机待验证 |
| 5 | 将手机计数器根视图改为 `value: i64`，把 `+`、`−`、`Reset` 的 `on_click` 接至 GPUI `cx.listener` 并 `cx.notify()`。 | 数值变化仅在 Rust GPUI 视图中实现；ArkTS 仍只管理宿主页面。 | 已交叉编译，真机待验证 |
| 6 | 将页面和原生日志标签改为 M4-1；执行 `cargo fmt --all` 与 `scripts/build-native.sh`。 | 格式化、aarch64 原生构建通过，无编译警告。 | 构建已通过 |
| 7 | 执行 `hvigorw ... assembleHap`，检查签名包是否可构建。 | 签名 HAP 构建成功；ArkTS 提示原生模块类型暂未验证，但不影响构建。 | 构建已通过 |
| 8 | `hdc list targets`、`hdc install entry/build/default/outputs/default/entry-default-signed.hap`、`hdc shell aa start -a EntryAbility -b com.example.gpuiohosmobilecounter`。 | 设备连接、安装与启动命令均报告成功。 | 安装已通过；交互未验证 |
| 9 | 查询 `uitest uiInput help`，抓取初始屏幕并检查 M4-1 日志。 | 系统提供坐标点击/滑动命令；截图显示手机处于锁屏，`GPUI_M4_1` 暂无运行日志，不能据此判定应用交互。含个人锁屏画面的本机临时截图已删除；已请求用户解锁。 | 真机交互受锁屏阻碍 |
| 10 | 给 GPUI 三个 `on_click` 回调增加数值变化日志，以便结合触摸分发和屏幕数字核验。 | 已修改 Rust 视图；需重新构建 HAP 后复验。 | 待构建 |
| 11 | 重新运行 `cargo fmt --all --check`、`scripts/build-native.sh`、`hvigorw ... assembleHap` 并安装更新后的签名 HAP。 | 三项检查与安装均通过；包内增加 GPUI 按钮数值日志。 | 构建/安装已通过 |
| 12 | 检查 HAP 清单、原生库 ELF 头、动态依赖及 `git diff --check`。 | HAP 含 `libs/arm64-v8a/libgpui_mobile_counter.so`；库为 ELF64 AArch64，依赖系统 Drawing/Node-API 等库，无新增第三方平台库；差异无空白错误。 | 静态已通过 |
| 13 | 再次执行 `hdc shell aa start -a EntryAbility -b com.example.gpuiohosmobilecounter`。 | 系统明确返回 `10106102`：设备锁屏，开发者模式下不能自动解锁；应用无法启动。待用户解锁后继续。 | 真机交互未验证 |
| 14 | 等待后再次启动，再用设备内置 `uitest uiInput swipe` 执行普通上滑并重试启动。 | 上滑命令成功，但启动仍返回同一 `10106102`；需要用户完成设备解锁。未尝试输入密码。 | 真机交互未验证 |
| 15 | 核对手机工程 README 与当前状态并更新。 | README 区分 M3-3 已验证文字和 M4-1 已构建但交互尚未验证；未将安装成功误记为按钮可用。 | 文档已核对 |
| 16 | 设备解锁后执行 `hdc shell aa start -a EntryAbility -b com.example.gpuiohosmobilecounter`、读取 `GPUI_M4_1` 日志并检查初始截图。 | 应用成功启动；设备 `ALN-AL00`、OpenHarmony `7.0.0.105` / API 26、aarch64，密度 `3.25`；原始 GPUI Scene 为 4 个 Quad、19 个单色 Sprite，Flush 成功，画面数字为 `0`。 | 真机已通过 |
| 17 | 用设备内置 `uitest uiInput click 580 1855` 两次点击 `+`，查 SDK、GPUI 与数值日志。 | XComponent 相对坐标为 `(580,1144)` 物理像素，GPUI 换算为 `(178.5,352.0)` 逻辑像素；两次点击分别令数值到 `1`、`2`。SDK 触点编号仍为 `0`，GPUI 分配的新 TouchId 分别为 `1`、`4`（中间还有其他设备触摸），未复用。 | 真机已通过 |
| 18 | 观察设备同时发生的手动触摸，再用 `uitest uiInput click` 执行受控 `Reset(805,1855) → +(580,1855) → −(405,1855)`，逐步核对日志。 | 受控点击分别产生 GPUI TouchId `35`、`36`、`37`；计数日志依次为 `reset value=0`、`increment value=1`、`decrement value=0`，每次触摸抬起后均有 Scene Flush。此序列把其他手动触摸与验收结果区分开。 | 真机已通过 |
| 19 | 再点击 `+` 并抓取/检查本机临时截图。 | 屏幕中心确实由 `0` 显示为 `1`，按钮保持正常；截图未写入仓库。 | 真机可见 |
| 20 | `uitest uiInput swipe 580 1855 900 1500 600`，从 `+` 拖到按钮外，并检查移动/抬起与计数日志。 | GPUI TouchId `39` 收到 `Started → Moved → Ended`；最后的计数回调仍是拖动前的 `increment value=1`，拖动未误点击。真机未产生 `Cancelled` 事件，因此取消映射仅经 SDK 枚举/代码静态核对，尚未动态验证。 | 拖动已通过；Cancel 未实测 |
| 21 | 同步任务、需求与工程 README，核对 M4-1 与后续任务边界。 | M4-1 记录为已完成；M4-2 帧调度和 M4-3 完整生命周期仍未开始。 | 文档已核对 |
| 22 | 再执行 `git diff --check`、`cargo fmt --all --check` 并检索任务/README 当前状态；删除本机临时截图。 | 差异与格式检查通过；文档状态一致，含设备状态栏的截图不留在仓库或本机临时目录。本阶段改动留本地，未提交或推送。 | 已通过 |

## 结果与限制

SDK 回调只获取并复制触摸数据；按 Surface 代次排队后，ArkUI 线程在现有 pump 中检查窗口代次，按密度换算坐标并分发 `PlatformInput::Touch`。GPUI 自身识别点击，三个 Rust `on_click` 回调更新计数并调用 `cx.notify()`。本阶段沿用 M3 的 50 ms 宿主轮询；按需帧回调替换属 M4-2。`Cancelled` 分支已实现，但本次真机操作未触发该系统事件。

## 2026-10-21 点击延迟追查

用户反馈真机点击后数字可能隔数秒才变化。本节只追查 M4-1 现状，不提前实施 M4-2。

| 顺序 | 操作对象 / 命令与目的 | 实际结果 | 验证状态 |
| --- | --- | --- | --- |
| 1 | 读取 `docs/gpui/requirements.md`、`tasks.md`、本阶段记录、`AGENTS.md` 和当前 `git status`，确认范围及现场。 | M4-1 已完成，M4-2 待开始；工作区仍有未提交的 M4-1 改动。需要用事件、回调和 Flush 时间定位延迟。 | 静态已核对 |
| 2 | `rg` 检索 ArkTS `setInterval`、Rust `pump`、GPUI 帧回调与触摸/绘制日志，确认可观测点。 | ArkTS 每 50 ms 调一次 `gpuiPump()`；Rust pump 最多取 64 个宿主事件后执行 GPUI 任务及帧处理；现有日志可比较 SDK 触摸、GPUI 点击与 Scene Flush。 | 静态已核对，真机待复测 |
| 3 | 读取 `src/gpui/mod.rs`、`window.rs`、`dispatcher.rs`、`src/render/screen.rs`、`src/host/surface.rs`、ArkTS 页面与工程 README，检查完整处理链。 | GPUI `on_click` 在抬起事件处理时变更数值并通知刷新；当前帧回调由每 50 ms 的 ArkTS 定时器调用，Drawing 的 `SurfaceFlush` 在同一轮绘制中执行。每次触摸移动在宿主与 GPUI 两处都输出 INFO 日志，密集滑动会增加主线程工作；仅凭代码不能判定数秒延迟。 | 静态已核对 |
| 4 | 查阅本机 API 26 SDK `native_drawing/drawing_surface.h` 中 `OH_Drawing_SurfaceFlush` 的契约，并检索官方资料。 | SDK 说明 Flush 将绘制内容推送到 GPU；返回成功不是屏幕像素已经显示的时间证明。未找到能由现有日志直接给出屏幕呈现时间的接口记录。 | SDK 静态已核对；实际呈现时间未验证 |
| 5 | `hdc list targets` 检查连接，再读取 `GPUI_M4_1` 的最近日志。 | 默认沙箱内 HDC 服务连接失败；获准访问设备后连接成功。近期手动滑动产生大量 Move 事件；一段触摸位于按钮区域外，没有对应计数回调。 | 真机已读取 |
| 6 | `hdc shell uitest uiInput click 580 1855`，随后读取该触摸的 SDK、GPUI、计数与 Scene 日志。 | 受控 `+` 点击：SDK Started `15:11:14.499`、Ended `15:11:14.613`，GPUI `increment value=3` 为 `15:11:14.658`，Drawing Flush 返回为 `15:11:14.666`。抬起到回调 45 ms，到 Flush 53 ms；按下到 Flush 167 ms，其中设备注入的按压约 114 ms。 | 真机处理/提交时间已验证；屏幕呈现时间未验证 |
| 7 | 过滤 `15:08` 的手动点击日志，逐次比较 SDK 抬起、计数回调与 Flush。 | 七次手动 `+`/`−` 点击均匹配回调和 Flush；抬起到 Flush 分别约 53、35、55、48、37、23、26 ms。相邻点击间隔为用户输入间隔，不是同一次事件的处理耗时。 | 真机处理/提交时间已验证 |
| 8 | 将延迟追查结论同步到工程 README；执行 `git diff --check`、`git status --short --branch` 并核对 M4-2 状态。 | README 记下实测处理时间和屏幕呈现时间未测的限制；差异检查通过；M4-2 仍为待办，本次没有更改代码、HAP 或真机安装。 | 文档已核对；屏幕呈现时间未验证 |

现有真机记录不支持“点击已被识别，但 GPUI 几秒后才更新计数”的判断。计数器当前只在抬起后触发 `on_click`；按钮外触摸或拖出按钮不会改变数字。50 ms 轮询确实带来可感知延迟和主线程负载时的不确定性，但本次实测未出现秒级排队。Drawing Flush 到屏幕实际呈现之间尚无时间戳，用户报告的偶发视觉延迟仍需在后续帧调度/呈现专项中测量，不能据此断言已经找到根因。
