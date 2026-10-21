# M1-3：XComponent 生命周期与触摸回调

日期：2026-10-21。范围仅为在独立手机工程中取得原生 XComponent、注册 Surface 创建/变化/销毁与触摸回调，并验证指针、尺寸、线程及重复初始化处理；本阶段不绘制。依据为 API 26 SDK 的 `ace/xcomponent/native_interface_xcomponent.h`、`js_native_api.h` 与 `hilog/log.h`。

| 顺序 | 操作与目的 | 实际结果与验证状态 |
| --- | --- | --- |
| 1 | 读取仓库 `AGENTS.md`、GPUI 需求/任务、M1-2A 记录和现有 Rust/ArkTS 宿主，确定本次范围。 | 已确认只修改 `gpui/ohos-mobile-counter/` 的 Rust FFI/宿主和本阶段文档；现有 HAP 仅有 `probe()`，Surface 尚未注册回调。 |
| 2 | 核对 API 26 SDK 的 XComponent 回调、`napi_get_named_property`、`napi_unwrap`、日志声明，并查官方 OpenHarmony/Huawei 文档的模块导出对象获取方式。 | 四个回调均为 `(OH_NativeXComponent*, void*)`；通过导出对象的 `__NATIVE_XCOMPONENT_OBJ__` 属性取包装对象再 `napi_unwrap`。SDK 中未找到对应的注销回调 API，因此回调表须保持进程期有效，销毁时清空应用持有的 Surface/window 状态。尚未在真机验证。 |
| 3 | 在 `src/sdk_ffi/` 增加 Node-API 属性读取/解包与 SDK `OH_LOG_PrintMsg` FFI；在 `src/napi_module.rs` 取得原生 XComponent；新增 `src/surface.rs` 注册四个回调、检查 SDK 返回码并记录线程/窗口/尺寸/触摸。 | 代码已写入，无运行时 C/C++ 桥接；回调表为进程期静态存储，注册相同组件时跳过，销毁时清空窗口和尺寸。 |
| 4 | 执行 `cargo fmt --check` 与 `cargo check --locked --offline --target aarch64-unknown-linux-ohos`。 | 首次格式检查发现 4 处排版差异；执行 `cargo fmt` 后两项通过。 |
| 5 | 执行 `DEVECO_SDK_HOME=<DevEco SDK> scripts/build-native.sh`，检查暂存 `.so` 的 ELF 头和动态依赖。 | Rust `cdylib` 构建成功；ELF64/AArch64；动态依赖包括系统 `libhilog_ndk.z.so`、`libace_napi.z.so`、`libace_ndk.z.so`、`libnative_drawing.so`、`libc.so`。 |
| 6 | 用 DevEco `hvigorw ... assembleHap` 构建签名 HAP，并检查压缩包的原生库条目。 | 首次因沙箱无法写本机 Hvigor daemon 缓存而失败；在获准访问本机缓存后构建/签名成功，HAP 含 `libs/arm64-v8a/libgpui_mobile_counter.so`。 |
| 7 | `hdc list targets`、`hdc install <signed.hap>`、`hdc shell aa force-stop/start ...`、`hdc shell hilog -x -T GPUI_M1_3`。 | 设备连接、安装、启动成功。模块首次普通导入时 `napi_unwrap` 返回 2，随后 XComponent 加载时注册返回成功；收到 `created`，尺寸 `1260×2369`（SDK 声明单位 vp），组件/window 指针非空，回调与注册均为同一线程。实际数值可能随页面布局变化。 |
| 8 | 用 `hdc shell uitest uiInput click 500 700` 点击 XComponent 区域，再读取日志。 | 收到 `type=0` 按下及 `type=1` 抬起；此前真机手动滑动还产生 `type=2` 移动。日志表明触摸回调与创建回调均在同一线程。触摸尚未接入 GPUI。 |
| 9 | 尝试 `uitest uiInput keyEvent Back/Home` 和日志查询，观察销毁回调。 | 未观察到 `destroyed`；随后设备自动锁屏，`aa start` 返回 `10106102`。暂不能据此认定销毁逻辑已实测。 |
| 10 | 为可靠触发销毁/重建，ArkTS 页面加入仅用于 M1-3 的 Hide/Show Surface 按钮；再次运行 `hvigorw ... assembleHap`、安装并启动。 | 构建/签名/安装/启动成功；新布局创建尺寸 `1260×2239`。ArkTS 编译器仍对原生模块验证给出非阻断告警；`.d.ts` 已存在。 |
| 11 | `uitest dumpLayout` 找到按钮区域后尝试点击并查日志。 | 页面随后受锁屏影响，目标窗口布局变为空，未得到销毁/重建证据；已请求用户解锁，待继续。 |
| 12 | 补强注册失败时的状态清理，并增加 ArkTS `Resize Surface` 按钮以主动触发尺寸变化；重新执行目标 `cargo fmt --check`、`cargo check --locked --offline`、原生构建和 Hvigor 签名构建。 | 检查和构建通过；最终 HAP 已安装。设备锁屏期间尺寸变化、销毁与重建仍未真机验证。 |
| 13 | 设备解锁后重装最终签名 HAP、强制停止并重启应用，查询 `GPUI_M1_3`。 | `Rust probe bits: 7`，注册成功并触发 `created`；初始尺寸 `1260×2109`，窗口非空，注册和创建日志均为 `ThreadId(1)`。 |
| 14 | `uitest dumpLayout` 定位本应用两个按钮；点击 `Resize Surface` 并读日志。 | 收到 `changed`，同一窗口宽度 `1260 → 1008`、高度 `2109`；`generation=1`，仍为 `ThreadId(1)`。 |
| 15 | 点击 `Hide Surface`，随后点击 `Show Surface` 并读日志。 | 先收到 `destroyed`（第 1 代窗口）；再注册新的 XComponent 并收到 `created`，窗口地址变化、`generation=2`、尺寸 `1008×2109`，仍为 `ThreadId(1)`。应用未崩溃。 |
| 16 | 在重建的 Surface 区域执行 `uitest uiInput click 500 750` 并读日志。 | 第 2 代窗口收到 `type=0` 按下和 `type=1` 抬起，均 `points=1`、`ThreadId(1)`；触摸重建后仍有效。 |
| 17 | 对最终代码复查 `cargo fmt --check`、`cargo check --locked --offline --target aarch64-unknown-linux-ohos`；从最终 HAP 抽取原生库并用 SDK `llvm-readelf` 查架构/依赖。 | 检查通过；HAP 内库为 ELF64/AArch64，动态依赖 `libnative_drawing.so`、`libhilog_ndk.z.so`、`libace_napi.z.so`、`libace_ndk.z.so`、`libc.so`。HAP 对原生库做 strip，因此与构建暂存的未 strip `.so` 摘要不同。 |
| 18 | 更新 `tasks.md`、`requirements.md` 和工程 README，使任务状态、实施方式、实测结果与限制一致。 | M1-3 标为完成；M2-1 仍待办；未把黑色 Surface 当作绘制成功。 |
| 19 | 检查最终 HAP 原生库清单及本阶段文档中的本机路径、签名材料和设备私有标识。 | HAP 仅有一份 `arm64-v8a` Rust 原生模块；本阶段更新的文档中未发现本机绝对路径、证书文件、口令或设备序列号。 |

真机：`ALN-AL00`、OpenHarmony `7.0.0.105` / API 26、aarch64。所有原生回调在本次观察中均为 `ThreadId(1)`；这仅是实测结果，不能假定 SDK 永远只从该线程回调。`OH_NativeXComponent_GetXComponentSize` 头文件称单位为 vp；本阶段只验证其变化与返回码，密度换算留待后续输入/绘制任务。SDK 没有此回调表的注销函数，本工程只保存 SDK 所有的组件/窗口地址，不取得其所有权；回调表进程期有效，Surface 销毁时清空本工程保留的窗口和尺寸。没有创建 Drawing 资源。重复注册同一组件的跳过路径已经实现，真机本次重建使用了新组件地址，未单独触发相同地址重复初始化分支。
