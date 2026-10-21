# GPUI 鸿蒙手机迁移：M0 操作与验证记录

日期：2026-10-20。对应工程：[`gpui/ohos-mobile-counter/`](../../gpui/ohos-mobile-counter/)；桌面基线 [`gpui/desktop-counter/`](../../gpui/desktop-counter/) 保持 `gpui = 0.2.2` 发布版及原有代码。

## 1. 固定官方 GPUI 来源

用 `git ls-remote https://github.com/zed-industries/zed.git HEAD` 得到提交 `c4a82ab1ab958aa43b0c0b15c0bad4fb77a7ca7f`。只读取该官方仓库的 `crates/gpui/`、`crates/gpui_platform/` 和工作区清单来核对 API。`crates/gpui/src/app.rs` 中存在 `Application::with_platform`、`run_embedded` 和 `ApplicationHandle`；`gpui_platform` 负责选择桌面后端，手机工程直接依赖 GPUI 核心，今后自行实现 `Platform`。

官方提交中 `gpui` 包版本仍标为 `0.2.2`，不能仅凭包版本区分发布包与此源码提交。手机工程以完整 Git SHA 和 `Cargo.lock` 固定来源。该提交的 GPUI 默认 feature 包含 `font-kit`、`wayland`、`x11`、`windows-manifest`；手机工程设置 `default-features = false`。清单中 GPUI 有 56 个直接依赖条目，含可选项；锁文件的具体解析结果以本工程 `Cargo.lock` 为准。

## 2. 核对主机 SDK、Rust 目标与真机

读取两套 `native/oh-uni-package.json`：独立安装的 Native SDK 为 API 26、`26.0.0.38`；DevEco Studio 随附 Native SDK 为 API 26、`26.0.0.105`。`rustup target list --installed` 含 `aarch64-unknown-linux-ohos`，Rust/Cargo 均为 `1.99.0`。

使用独立 SDK 的 `toolchains/hdc list targets` 得到设备序列号；随后运行：

```sh
hdc shell param get const.product.model
hdc shell param get const.ohos.apiversion
hdc shell param get const.ohos.fullname
hdc shell uname -m
hdc shell param get const.product.cpu.abilist
```

实际返回依次为 `ALN-AL00`、`26`、`OpenHarmony-7.0.0.105`、`aarch64`、`arm64-v8a`。`hdc` 在当前受限执行环境下需要允许访问设备服务；未授权运行时会返回 `Connect server failed`，不能把它解释为设备未连接。

读取 API 26 Native SDK 的 `native_drawing/drawing_gpu_context.h` 与 `drawing_surface.h`，确认 `OH_Drawing_GpuContextCreate`、`OH_Drawing_SurfaceCreateOnScreen`、`OH_Drawing_SurfaceFlush` 均标注 API 16 起。检查了 SDK sysroot 中的 aarch64 `libnative_drawing.so`、`libace_ndk.z.so`、`libace_napi.z.so`。真机上 `libace_ndk.z.so` 路径可列出；直接列举 `libnative_drawing.so` 返回 `Permission denied`，`libace_napi.z.so` 在所试的 `/system/lib64/` 路径不存在。**因此尚未确认真机动态库加载和上述符号的运行能力**，M1/M2 仍须通过 HAP 实测。

## 3. 验证 GPUI 核心的目标编译

先在系统临时目录的最小 Cargo 工程，仅依赖上述 GPUI Git 提交并关闭默认 feature，执行 `cargo generate-lockfile` 和 `cargo check --locked --target aarch64-unknown-linux-ohos`。锁文件解析出 312 个包，目标检查通过；其中包含官方 Zed 仓库的 GPUI 核心及其工作区组件。随后在正式的独立手机工程执行：

```sh
cd gpui/ohos-mobile-counter
cargo generate-lockfile
cargo fmt --check
cargo check --locked --offline --target aarch64-unknown-linux-ohos
```

正式工程的锁文件也是 312 个包。实际重复检查时用 `CARGO_TARGET_DIR` 指向临时工程的已编译缓存，以免重复编译；结果通过。该检查只验证 GPUI 核心的编译，不含自建 `Platform`、SDK FFI、链接、HAP 或设备执行。

另用 `cargo tree --locked --offline --target aarch64-unknown-linux-ohos --edges normal` 核对目标的普通依赖，未出现 `gpui_platform`、`gpui_wgpu`、`wgpu`、`winit`、`x11` 或 `wayland`。`Cargo.lock` 仍会记录跨平台解析出的包，不能把锁文件条目都视作手机端实际链接依赖。

`rustc --print cfg --target aarch64-unknown-linux-ohos` 同时输出 `target_os="linux"` 和 `target_env="ohos"`。在固定提交的 GPUI 核心文件中存在多处 `target_os="linux"` 条件，M1 要逐处检查是否错误纳入桌面行为；本次目标 `cargo check` 通过，不代表运行语义已正确。`gpui_platform` 的 Linux 路径会选桌面后端，手机工程不引入它。

曾临时尝试把原桌面示例改为该提交的 `gpui` 加 `gpui_platform`，Cargo 在 `gpui_platform` 的 `windows-manifest` feature 解析处报错；这次尝试已完全撤回，原桌面示例 `cargo check --locked --offline` 再次通过。它不影响手机工程的 GPUI 核心解析，但说明不能直接把桌面启动 crate 作为手机入口。

## M0 结论与边界

目标设备、API、架构、SDK 和 GPUI 核心的交叉编译已确认。平台层实现、Native Drawing 真机绘制、动态库实际加载、签名打包与 GPUI 计数器交互均未验证。后续每一步将只推进一个可检查的子任务，并在任务文档和本工程 README 留下结果。

## M0-4：签名与真机动态库加载预检

日期：2026-10-20。开始前已向用户说明检查方法、原因和验证边界。此任务仍**未完成**：没有 HAP 时，不能验证本工程的签名配置或应用进程内的库加载。[OpenHarmony HAP 文档](https://github.com/openharmony/docs/blob/master/en/application-dev/quick-start/hap-package.md)说明 HAP 是安装运行的基本单元，调试安装也要求有效且一致的签名；[签名验证文档](https://github.com/openharmony/docs/blob/master/en/device-dev/subsystems/subsys-security-sigverify.md)说明调试 profile 与设备标识、证书的核对关系。以下是本次逐项操作和结果。

| 顺序 | 操作与目的 | 实际结果 |
| --- | --- | --- |
| 1 | 读取 `requirements.md`、`tasks.md`，列出 `gpui/ohos-mobile-counter/` 文件，确定 M0-4 的当前条件。 | 工程只有 Rust 核心依赖清单和空库入口；没有 `build-profile.json5`、`module.json5`、HAP 或工程签名配置。 |
| 2 | 在 DevEco SDK 中查找签名工具，用系统 `java` 运行 `hap-sign-tool.jar -help`。 | 找到 `hap-sign-tool.jar`；系统 `java` 报未安装 Java Runtime，首次命令失败。 |
| 3 | 查找 DevEco 自带 Java，再运行该签名工具的 `-help`。 | DevEco 自带 JBR 可运行工具；列出 `sign-profile`、`sign-app`、`verify-app` 等命令。仅证明工具可启动。 |
| 4 | 只检查本机已有 `.p12`、`.p7b`、`.cer` 文件是否存在，不读取或复制密钥。 | 找到另一应用的本地签名材料；文件名和内容未写入仓库，不能据此断言适用于本工程。 |
| 5 | 核对 SDK Native Drawing 头文件、aarch64 库及导出符号。 | SDK 标注 Drawing 屏幕 Surface 接口 API 16 起；`llvm-readelf -Ws` 找到 `OH_Drawing_GpuContextCreate`、`OH_Drawing_SurfaceCreateOnScreen`、`OH_Drawing_SurfaceFlush`、`OH_NativeXComponent_RegisterCallback`、`napi_module_register`。这是 SDK 链接桩检查，不是真机加载。 |
| 6 | 编写 `gpui/ohos-mobile-counter/probes/m0_library_load.c`，尝试在设备上 `dlopen` 三个系统库并用 `dlsym` 查上述符号。 | 探针只打印加载和符号结果，不绘图、不调用 GPUI；源码留在独立手机工程供复现。 |
| 7 | 用 DevEco aarch64 OHOS Clang 以 `-std=c11 -Wall -Wextra -Werror -O2 -ldl` 编译探针，用 `llvm-readelf -h` 检查。 | 编译通过；生成 ELF64 AArch64 可执行文件，产物仅位于系统临时目录，未提交工程。 |
| 8 | 通过 `hdc file send` 传到设备临时目录并执行 `chmod 700`。 | 文件传输成功，`ls -l` 显示执行位已设置。 |
| 9 | 用 `hdc shell` 执行设备上的探针。 | `/bin/sh` 返回 `Permission denied`，没有进入 `main`，因此没有任何 `dlopen`/`dlsym` 结果。没有尝试绕过设备执行策略。 |
| 10 | 删除设备上的临时探针；复核 SDK 符号和本工程 HAP 文件状态。 | `hdc shell rm` 成功；SDK 符号存在，但本工程仍无 HAP，M0-4 保持未勾选。 |
| 11 | 查阅 OpenHarmony 官方 HAP 打包和签名验证文档，核对应用签名依赖。 | 确认应用要以 HAP 安装验证，调试签名的 profile 与设备、证书相关；当前另一应用的本地材料不能直接证明本工程可签名。 |
| 12 | 对探针执行 `clang-format --dry-run --Werror`，修正格式后重新格式检查和交叉编译。 | 首次格式检查失败；`clang-format -i` 后格式检查及 `-Wall -Wextra -Werror` 交叉编译均通过。 |
| 13 | 再次检查设备临时路径，并删除本机临时编译产物。 | 设备返回 `No such file or directory`，确认探针已清理；本机临时 ELF 已删除，工程只保留可复现源码。 |

探针构建和设备尝试可按以下形式复现，`SDK` 填为本机 API 26 SDK 根目录，不把实际路径提交仓库：

```sh
"$SDK/native/llvm/bin/aarch64-unknown-linux-ohos-clang" -std=c11 -Wall -Wextra -Werror -O2 -o /tmp/gpui_m0_library_load gpui/ohos-mobile-counter/probes/m0_library_load.c -ldl
"$SDK/native/llvm/bin/llvm-readelf" -h /tmp/gpui_m0_library_load
"$SDK/toolchains/hdc" file send /tmp/gpui_m0_library_load /data/local/tmp/gpui_m0_library_load
"$SDK/toolchains/hdc" shell chmod 700 /data/local/tmp/gpui_m0_library_load
"$SDK/toolchains/hdc" shell /data/local/tmp/gpui_m0_library_load
"$SDK/toolchains/hdc" shell rm /data/local/tmp/gpui_m0_library_load
```

后续依赖：M1-2 建立最小宿主后，配置**本工程**调试签名，在设备上安装 HAP，从应用进程记录 Native Drawing、Node-API 和原生模块的加载结果；完成后再勾选 M0-4。不得把这次 shell 探针的权限失败解释成系统库缺失。

2026-10-20 更新：M1-2 已构建最小 HAP，ELF 动态依赖包含上述系统库；使用 SDK 测试 CA 生成的签名通过本地校验，但真机安装返回 `9568257: fail to verify pkcs7 file`。应用进程尚未创建，M0-4 仍未完成；详细操作见 [M1-2 记录](m1-2-hap.md)。

2026-10-21 更新：修正 `default` 产品的签名方案引用后，HarmonyOS 调试签名 HAP 已在真机安装并启动。应用日志 `loadContent succeeded`、`Rust probe bits: 7` 证实 Node-API 模块、Rust 入口和 Native Drawing 错误码探针运行，XComponent 注册符号可寻址；M0-4 已完成。屏幕 Surface 和绘制仍属后续任务。

### M0-4 执行拒绝的原因核对（2026-10-20）

| 顺序 | 只读操作与目的 | 实际结果 |
| --- | --- | --- |
| 1 | 复核此前的 `ls -l`：判断是否忘记设置执行位。 | 探针执行前为 `-rwx------`、所有者 `shell:shell`；普通文件模式不解释这次拒绝。 |
| 2 | 读取 `/proc/mounts`，查看 `/data` 和 `/data/local/tmp` 的挂载选项。 | `/data` 为 `rw,nosuid,nodev,...`，没有 `noexec`；没有发现 `/data/local/tmp` 独立的 `noexec` 挂载。 |
| 3 | 执行 `getenforce`，尝试确认 SELinux 工作模式。 | 命令返回 `security_getenforce() failed: Permission denied`，模式未能读取。 |
| 4 | 执行 `ls -ldZ /data/local/tmp` 和 `cat /proc/self/attr/current`，读取目录与 shell 安全上下文。 | 目录标记为 `u:object_r:data_local_tmp:s0`，shell 为 `u:r:sh:s0`；这些标签本身不能证明具体策略拒绝。 |
| 5 | 尝试从 `dmesg` 查找 `avc: denied` 和探针名。 | 未取得拒绝记录；直接运行 `dmesg` 返回 `klogctl: Operation not permitted`。 |
| 6 | 阅读 [OpenHarmony Toybox 常见权限错误说明](https://github.com/openharmony/docs/blob/master/zh-cn/application-dev/tools/toybox.md)和[系统安全指南](https://github.com/openharmony/docs/blob/master/en/device-dev/security/security-guidelines-overall.md)。 | 官方资料说明 `Permission denied` 可能来自文件权限或 SELinux 拦截，确诊 SELinux 需拒绝日志；本设备目前无法读取该日志。 |

**判断：** 现有证据排除了普通执行位缺失和 `/data` 的 `noexec` 挂载。更可能是设备对 `sh` 域执行临时文件的安全策略限制，但未取得审计日志，不能确定是 SELinux、代码完整性检查还是其他设备策略。该拒绝发生在程序启动前，与 Native Drawing/Node-API 库能否在 HAP 中加载没有直接证据关系；用 M1-2 的已签名 HAP 做应用进程内验证。
