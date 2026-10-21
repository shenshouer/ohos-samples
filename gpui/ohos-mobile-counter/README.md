# GPUI 鸿蒙手机计数器

这是独立于[桌面计数器](../desktop-counter/)的 HarmonyOS 手机工程。页面由 ArkTS 管理 Ability 和 XComponent；Rust 原生模块直接声明并调用 API 26 SDK 的 C ABI，提供 GPUI 平台、绘制、触摸和生命周期适配。屏幕上的 `GPUI Counter`、数值及 `−`、`+`、`Reset` 按钮由 GPUI Scene 生成；计数只在 GPUI 点击回调中更新。运行时没有 C/C++ 桥接，也没有引入额外的平台或渲染项目。

## 版本与设备

| 项目 | 当前配置或已验证环境 |
| --- | --- |
| GPUI | 上游 Zed 仓库提交 `c4a82ab1ab958aa43b0c0b15c0bad4fb77a7ca7f`，关闭默认 feature；以 [`Cargo.lock`](Cargo.lock) 锁定依赖。`futures`、`raw-window-handle`、`uuid` 是实现 GPUI trait 所需的直接依赖。 |
| Rust | edition 2024，目标 `aarch64-unknown-linux-ohos`；已验证工具链 `rustc`/Cargo 1.99.0，目标须预先安装。 |
| SDK / 工程 | API 26 Native SDK；本机验证过独立安装版 `26.0.0.38` 和 DevEco 随附版 `26.0.0.105`。工程 `compatibleSdkVersion`、`targetSdkVersion` 均为 `26.0.0`，运行时为 HarmonyOS。 |
| 真机 | `ALN-AL00`，OpenHarmony `7.0.0.105`，API 26，`aarch64` / `arm64-v8a`；这些信息来自设备查询。其他设备/API 尚未验证。 |
| 应用 | phone entry module，包名 `com.example.gpuiohosmobilecounter`，Ability 为 `EntryAbility`；当前功能不申请权限。 |

## 代码与运行路径

| 位置 | 职责 |
| --- | --- |
| [`entry/src/main/ets/`](entry/src/main/ets/) | ArkTS 页面、XComponent、Ability 前后台与退出；页面定时器每 250 ms 执行一次非绘制安全检查。 |
| [`src/host/`](src/host/) | 纯 Rust Node-API 模块、日志、Surface/触摸事件队列和按需帧调度。 |
| [`src/gpui/`](src/gpui/) | 本工程的 GPUI `Platform`、窗口、文字系统、Atlas、调度器和计数器视图；上游 crate 在代码中称 `gpui_core`。 |
| [`src/render/`](src/render/) | 将 GPUI Scene 绘到 Native Drawing 屏幕 Surface，并管理 GPU/Surface 资源。 |
| [`src/sdk_ffi/`](src/sdk_ffi/) | 按功能拆分的 SDK C ABI：Node-API、XComponent、Native Drawing、Display、日志等。 |
| [`scripts/build-native.sh`](scripts/build-native.sh) | 交叉构建 Rust `cdylib`，复制为 HAP 使用的 `entry/libs/arm64-v8a/libgpui_mobile_counter.so`。 |
| [`probes/`](probes/) | 早期独立 ABI/设备探针；不编入 HAP。`entry/src/main/cpp/types/` 仅是 ArkTS 类型声明。 |

XComponent 的 Surface 回调把窗口变化和触摸事件送入 Rust；宿主在 ArkUI 线程按 Surface 代次处理事件，并按显示密度换算触摸坐标。GPUI 有待绘制内容时请求 XComponent 帧回调，回调到达后在同一线程推进帧并注销本次请求。当前设备采用逐帧重建屏幕 Drawing Surface、复用 GPU context 的策略。页面的 `Hide/Show Surface`、`Resize Surface`、`Landscape`、`Portrait` 可复验 Surface 生命周期；`Exit Demo` 调用系统 `terminateSelf()`，Ability 销毁时调用 Rust `gpuiStop()`。

## 构建与签名

需要 Rust 目标、API 26 Native SDK、DevEco Studio 的 Node/JBR/hvigor 环境，以及可用于目标设备的 HarmonyOS 调试签名。以下命令在本目录执行。把占位符替换成本机安装位置；不要把路径、证书或口令写入仓库。

```sh
rustup target add aarch64-unknown-linux-ohos
export DEVECO_SDK_HOME="<DevEco SDK 根目录>"
export NODE_HOME="<DevEco Node 目录>"
export JAVA_HOME="<DevEco JBR 的 Contents/Home 目录>"
export PATH="<DevEco hvigorw 所在目录>:$PATH"

# 首次构建、Cargo 缓存尚未包含锁定依赖时，先联网获取。
cargo fetch --locked
cargo fmt --all --check
cargo check --locked --offline --target aarch64-unknown-linux-ohos
scripts/build-native.sh
```

`build-native.sh` 使用 `$DEVECO_SDK_HOME/default/openharmony/native` 下的目标 Clang；已有独立 Native SDK 时，也可设置 `OHOS_NATIVE_SDK_HOME` 为其 `native` 根目录并覆盖这一选择。脚本执行 `cargo build --locked --offline --target aarch64-unknown-linux-ohos`，然后暂存原生库。`cargo fetch` 只在缓存缺失时需要；离线环境应预先准备与锁文件匹配的 Cargo 缓存。

复制 [`build-profile.example.json5`](build-profile.example.json5) 为本机 `build-profile.json5`。在 DevEco Studio 的 `File > Project Structure > Project > Signing Configs` 中为 HarmonyOS 配置可用的自动调试签名。检查 `build-profile.json5`：`app.signingConfigs` 中有签名方案，且 `app.products` 里 `name: "default"` 的对象通过 `"signingConfig": "default"` 引用同名方案。模板本身没有签名，因此直接打包只会得到 unsigned HAP。`build-profile.json5`、`entry/libs/`、构建产物和签名材料均被忽略；不要提交证书、profile、密钥、口令或本机路径。

```sh
hvigorw --no-daemon --mode module -p module=entry@default -p product=default -p buildMode=debug assembleHap
```

成功签名后的产物为 `entry/build/default/outputs/default/entry-default-signed.hap`。如果安装报 `9568320: no signature file`，检查签名方案是否被 `default` product 引用，并确认安装的是 `signed.hap`。

可用下列命令检查 HAP 内的原生库，并读取其架构和动态依赖：

```sh
hap=entry/build/default/outputs/default/entry-default-signed.hap
native_sdk_root=${OHOS_NATIVE_SDK_HOME:-$DEVECO_SDK_HOME/default/openharmony/native}
unzip -l "$hap" 'libs/*'
native_lib=$(mktemp)
unzip -p "$hap" libs/arm64-v8a/libgpui_mobile_counter.so > "$native_lib"
"$native_sdk_root/llvm/bin/llvm-readelf" -h -d "$native_lib"
rm "$native_lib"
```

M4-3 的最终包检查结果：只有一份 `arm64-v8a` 的 ELF64/AArch64 原生库，动态依赖均为系统库：`libnative_display_manager.so`、`libnative_drawing.so`、`libhilog_ndk.z.so`、`libace_napi.z.so`、`libnative_window.so`、`libace_ndk.z.so`、`libtime_service_ndk.so`、`libc.so`。

## 安装与真机操作

连接并解锁 API 26 的 arm64 手机，确认 DevEco 签名包含该设备，再执行：

```sh
hdc list targets
hdc install entry/build/default/outputs/default/entry-default-signed.hap
hdc shell aa start -a EntryAbility -b com.example.gpuiohosmobilecounter
hdc shell hilog -x -T GPUI_M4_2
```

若 `aa start` 返回 `10106102`，先在手机上完成解锁后重试；该错误曾在锁屏时出现。`GPUI_M4_2` 是当前代码的日志标签，页面标题中的 `M4-2` 也是沿用的诊断标识。

预期看到页面的 `probe bits: 7`、`GPUI embedded: started`，以及 XComponent 中的 `GPUI Counter`、数字 `0` 和三个 GPUI 按钮。依次点击 `Reset → + → −` 应显示 `0 → 1 → 0`；从按钮拖出不应触发点击。`Hide/Show Surface`、`Resize Surface` 和横竖屏切换后应重新显示计数器；Home 后返回可继续操作。点击 `Exit Demo` 后再启动，应创建新的 Surface 和 GPUI 窗口，并可继续点击。Back/Home 在已测设备上使 Ability 留在后台，不能用作退出清理的检验；应使用 `Exit Demo`。

## 已验证结果与限制

M4-1 至 M4-3 的签名 HAP 已在上述真机安装和操作：文字、图形、`0 → 1 → 0` 交互、拖出按钮不误触、Surface 重建、空闲/后台停帧，以及 `Exit Demo` 时取消待处理帧、释放 Drawing 和关闭 GPUI 窗口均有记录。M4-2 的一次受控 `+` 点击，从 SDK 抬起到 GPUI 计数回调约 9 ms、到 Scene Flush 约 18 ms；约 14 秒空闲期间未连续收到帧回调。这些时间是处理与提交记录，**不是屏幕实际呈现延迟**。M4-3 冷启动复验成功，最终进程未检出此前曾出现的 NativeWindow 缓冲区状态错误。

当前 Scene 绘制范围为纯色 Quad、统一圆角/边框、矩形裁剪和单色字形 Sprite；文字系统只覆盖计数器所需的系统默认字体、逐字符布局和 Alpha8 光栅化。复杂文字 shaping、字体回退、自定义字体及其他 GPUI 图元尚未实现。系统触摸 `Cancel` 已映射，但未在真机触发验证；一次高频触摸中的点击未触发回调，原因未确认。长期压力运行、其他手机/API、Flush 到屏幕实际呈现时间仍未验证，用户报告的偶发秒级视觉延迟不能据当前日志判定已彻底消除。

分阶段要求和任务状态见 [`docs/gpui/requirements.md`](../../docs/gpui/requirements.md)、[`docs/gpui/tasks.md`](../../docs/gpui/tasks.md)。关键操作及证据见 [M0 预检](../../docs/gpui/m0-validation.md)、[M1-2 签名 HAP](../../docs/gpui/m1-2-hap.md)、[M3-3 文字](../../docs/gpui/m3-3-text.md)、[M4-1 触摸](../../docs/gpui/m4-1-touch.md)、[M4-2 按需帧](../../docs/gpui/m4-2-frame.md)、[M4-3 生命周期](../../docs/gpui/m4-3-lifecycle.md)和[M4-4 README 操作记录](../../docs/gpui/m4-4-readme.md)。
