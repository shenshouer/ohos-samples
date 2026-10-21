# M1-2：最小 HAP 与真机加载验证

日期：2026-10-20。开始前已向用户说明：先建立独立 DevEco 宿主，用系统 Node-API 注册层连接 Rust 静态库，在 ArkTS 中创建空的 XComponent；随后构建、签名、安装，并以真机日志验证原生模块和 SDK 动态库。这样先核对宿主和加载链，再在 M1-3 接入 Surface 生命周期。本任务不做绘制或计数器交互。

**当前状态：M1-2 与 M0-4 已完成。** 2026-10-21 修正产品签名引用后，HarmonyOS 调试签名 HAP 已构建、通过本地校验并在真机安装成功。应用启动日志显示 `loadContent succeeded` 和 `Rust probe bits: 7`：Node-API 模块已从 ArkTS 加载，Rust 入口执行，Native Drawing 错误码探针成功，XComponent 注册符号可寻址。空 XComponent 已创建；Surface 回调、绘制和触摸不属于本任务。先前 SDK 测试签名被拒绝的过程保留在下方记录中。

本记录描述 M1-2 当时的 C++ 注册桥接；该桥接后来由 [M1-2A](m1-2a-rust-napi.md) 的纯 Rust 模块替换，并已在真机复验。

## 实现与验证边界

- Rust `staticlib` 导出 `gpui_mobile_probe`：位 0 表示进入 Rust，位 1 表示 `OH_Drawing_ErrorCodeReset/Get` 成功，位 2 表示 XComponent 注册符号在本地库装载后可寻址。真机日志已观察到 `Rust probe bits: 7`；用户提供的真机截图也显示页面文字 `probe bits: 7`。
- SDK Node-API 的 C++ 注册层只导出 `probe()`，调用 Rust 入口。`XComponent(type: 'surface')` 目前仅创建可显示区域，不注册 Surface/触摸回调；后者属于 M1-3。
- 构建使用 DevEco Studio 26.0.0 的 SDK/API 26 和 `aarch64-unknown-linux-ohos` Rust 目标。未新增平台或渲染库，也未使用现成 Rust SDK 绑定。
- Huawei 官方[自动签名说明](https://developer.huawei.com/consumer/cn/doc/doccenter-deveco-studio/ide-signing-auto)区分 OpenHarmony 与 HarmonyOS 签名，并要求 HarmonyOS 调试签名使用“Support HarmonyOS”及“Automatically generate signature”。本机 SDK 测试 CA 的本地验证成功不能代替真机信任校验。

## 按顺序的操作记录

| 序号 | 操作与目的 | 实际结果及验证状态 |
| --- | --- | --- |
| 1 | 读取 `AGENTS.md`、`requirements.md`、`tasks.md`、M1-1 记录和手机工程，确定 M1-2 范围。 | 确认需独立 HAP、Rust/Node-API 桥接和真机加载证据；未进入 M1-3。 |
| 2 | 检查 DevEco SDK 头文件、库和项目模板，核对 Node-API、XComponent、Drawing 的链接名与宿主结构。 | 选用 API 26 SDK；使用系统 `libace_napi.z.so`、`libace_ndk.z.so`、`libnative_drawing.so`。 |
| 3 | 修改 `Cargo.toml` 和 `src/lib.rs`，生成 Rust `staticlib` 与按位返回的 SDK 探针。 | `cargo build --locked --offline --target aarch64-unknown-linux-ohos` 成功；Rust 导出符号进入静态库。 |
| 4 | 新建 AppScope、entry、Hvigor、资源、ArkTS 页面、CMake 和 Node-API 模块注册文件。 | 页面创建空 XComponent 并调用 `probe()`；C++ 共享库链接 Rust 静态库及三个系统库。仅完成构建侧接线。 |
| 5 | 执行 Rust 与 C++ 格式检查。 | 首次格式检查各发现排版差异；运行 `cargo fmt`、`clang-format -i` 后复查通过。 |
| 6 | 用 Hvigor 构建 HAP。 | 首次受本机 Hvigor 缓存写权限影响；改用获准的缓存目录后发现 SDK 根目录和版本配置错误，调整为 SDK 根目录与 `26.0.0`。随后根据打包报错补齐应用图标、启动窗口图标和背景。最终 `assembleHap` 成功，产出未签名 HAP。 |
| 7 | 检查 HAP 压缩包与共享库 ELF。 | HAP 含 `ets/modules.abc`、`libs/arm64-v8a/libgpui_mobile_counter.so`、`libc++_shared.so`；共享库为 ELF64/AArch64，`DT_NEEDED` 包含三个预期系统库、C++ 运行库和 libc。仅为静态证据。 |
| 8 | 使用 SDK `hap-sign-tool` 和测试 CA，为本工程包名及已连接设备生成临时调试 profile、证书和独立密钥，再执行 `sign-profile`、`sign-app`。 | 初始 SDK 应用叶证书无法形成链；改为由 SDK 测试 Application CA 签发新叶证书后签名成功。临时密钥和设备标识仅在私有临时目录，未写入工程。一次失败命令的临时口令曾出现在工具输出中，随即废弃该密钥并生成新的口令和密钥。 |
| 9 | 执行 `hap-sign-tool verify-app`，检查签名文件。 | 返回 `Digest verify result: true`、`verify-app success`。这只验证签名格式和内容摘要，不证明 HarmonyOS 手机信任测试 CA。 |
| 10 | 使用 `hdc install` 将签名 HAP 安装到已连接的 `ALN-AL00`（API 26）。 | 安装失败：`code:9568257 error: fail to verify pkcs7 file`。因此没有启动应用，也没有 `GPUI_M1_2` 真机日志。 |
| 11 | 用 DevEco Studio 打开当前工程，准备检查官方 HarmonyOS 自动签名入口。 | IDE 显示“信任并打开项目”。点击“信任项目”被自动审批审查拒绝，理由是信任会允许 IDE 执行工程代码，而当前指令未明确授权这一安全设置变更。未绕过拒绝；改用安全模式预览，确认安全模式不加载项目配置，无法完成签名配置。 |
| 12 | 查阅 Huawei 官方自动签名与 `build-profile.json5` 文档，并审阅本工程 Hvigor 文件及依赖声明。 | 当前项目仅声明官方 `@ohos/hvigor` 和 `@ohos/hvigor-ohos-plugin` 6.26.8，两个 Hvigor 入口均无自定义插件。真机所需 HarmonyOS 调试签名仍须通过 DevEco/开发者签名流程生成；不复用其他应用的证书和 profile。 |
| 13 | 更新 `.gitignore`、任务状态、需求概况、M0-4 记录和工程 README。 | 忽略 IDE/CMake 生成目录及本地签名材料；明确 HAP 已构建但手机未安装，M1-2/M0-4 保持未勾选。 |
| 14 | 删除本次失败测试签名的临时目录。 | 临时口令、私钥、设备 profile 和被拒绝的测试签名 HAP 已删除；可供复核的未签名 HAP 仍留在工程忽略的构建目录。 |
| 15 | 扫描本阶段文档及 README 的本机绝对路径、设备标识和旧绑定项目引用；复核任务勾选状态。 | 未发现私有路径、设备标识或被禁止的绑定引用；M1-2、M0-4 均保持未勾选。此次最后只修改文档和忽略规则，未重复运行已通过且代码未变的构建。 |

## 复现命令与待续步骤

在 `gpui/ohos-mobile-counter/` 中，先用本机 DevEco 提供的 Node、JBR、SDK 设置 `NODE_HOME`、`JAVA_HOME`、`DEVECO_SDK_HOME`，再运行：

```sh
cargo fmt --check
cargo build --locked --offline --target aarch64-unknown-linux-ohos
hvigorw --mode module -p module=entry@default -p product=default -p buildMode=debug assembleHap
```

生成的 `entry/build/default/outputs/default/entry-default-unsigned.hap` 不能直接安装。需在 DevEco 的 `File > Project Structure > Project > Signing Configs` 为**本工程包名**生成 HarmonyOS 调试签名，并确认 `products` 中当前产品的 `signingConfig` 指向 `signingConfigs` 中的方案名称。2026-10-21 已按此修正，本机产出并安装 `entry-default-signed.hap`。本机含口令的 `build-profile.json5` 已忽略；可分享模板是 `build-profile.example.json5`。解锁设备并启动后，已在 `GPUI_M1_2` 真机日志观察到实际位值 `7`。

截至 2026-10-20，本次代理操作 DevEco 的信任项目动作被自动审批审查拒绝，签名步骤停在安全预览状态。后续签名状态见下一节。

## 2026-10-21：DevEco 签名失败排查

用户报告 DevEco Studio 中签名一直失败。本次只继续 M1-2，先读取报错和本地配置，再修正并以 signed HAP/真机运行核对；不进入 M1-3。

| 序号 | 操作与目的 | 实际结果及验证状态 |
| --- | --- | --- |
| 1 | 读取任务、需求、上一轮 M1-2 记录及当前工程配置，确定排查边界。 | 工程已有 DevEco 生成的 HarmonyOS `default` 签名方案；`default` 产品未设置 `signingConfig`。配置含本机路径和签名口令，不能纳入版本控制。 |
| 2 | 只检查本机签名证书、profile、密钥文件是否存在，不输出路径或内容。 | 三类文件均存在；不能仅凭存在性判定证书/profile 有效。 |
| 3 | 读取 DevEco 当前运行控制台的失败信息。 | IDE 报“未检测到签名配置”；部署的是 `entry-default-unsigned.hap`，手机返回 `9568320: no signature file`。这与上一轮测试 CA 签名的 `9568257` 是不同错误。 |
| 4 | 对照 Huawei 官方 `build-profile.json5` 文档，核对产品到签名方案的引用。 | 官方文档明确：`products[].signingConfig` 应指向 `signingConfigs[].name`；缺失时默认不签名。故本次 unsigned HAP 的直接原因已定位。 |
| 5 | 将本机 `default` 产品的 `signingConfig` 设为现有 `default` HarmonyOS 签名方案，并将含本机口令的 `build-profile.json5` 加入 `.gitignore`，提供无口令的 `build-profile.example.json5`。 | 本机签名配置仍可供 DevEco 构建；可分享模板没有密钥、口令、设备标识或个人绝对路径。 |
| 6 | 运行 DevEco Hvigor 的 `assembleHap`，验证修正后的签名产物。 | `SignHap` 和构建均成功，产出 `entry-default-signed.hap`。外层 shell 在构建后因使用 zsh 只读变量名报错，但 Hvigor 日志已明确构建成功；后续通过产物和独立验证确认。 |
| 7 | 对 signed HAP 执行 SDK `hap-sign-tool verify-app`。 | 签名块、原生库代码签名、摘要和权限签名均验证成功；调试 profile 被识别。本机校验通过。 |
| 8 | 通过 `hdc install` 安装 signed HAP。 | 真机返回 `install bundle successfully`；签名和安装问题已解决。 |
| 9 | 通过 `hdc shell aa start` 启动应用，准备核对探针。 | 设备锁屏，返回 `10106102`：开发者模式下无法自动解锁。已请求用户手动解锁；应用进程和探针位值仍未验证。 |
| 10 | 删除临时签名校验输出和构建日志，保留工程忽略目录内的 signed HAP 供复核。 | 临时证书链、profile 和日志已清理；HAP 和本机签名材料仍在本机，不进入文档。 |
| 11 | 检查本机签名方案引用、`.gitignore`、可分享模板和阶段文档。 | 产品已关联 `default` 方案；本机含口令配置被忽略。文档与模板未发现个人绝对路径、设备标识或口令。 |
| 12 | 设备解锁后重试 `hdc shell aa start`。 | 返回 `start ability successfully`；应用进程存在。 |
| 13 | 初次用 `hilog -d` 查询探针日志。 | 未得到日志；查帮助发现 `-d` 是生成字典的参数，正确的一次性读取参数为 `-x`。 |
| 14 | 改用 `hilog -x -T GPUI_M1_2` 并查询应用 PID 日志。 | 首次按标签查询无结果，应用进程仍存在且设备日志显示 XComponent 布局；旧日志可能已滚出缓冲区，不能据此判定探针未运行。 |
| 15 | 仅强制停止当前测试应用、重新启动，并立即用 `hilog -x -T GPUI_M1_2` 查询。 | 真机日志出现 `loadContent succeeded` 与 `Rust probe bits: 7`；确认 ArkTS → Node-API → Rust 与 Native Drawing 探针在应用进程内运行。M1-2/M0-4 验收通过。 |
| 16 | 更新任务、需求、M0-4 记录与工程 README，并复核本机配置、signed HAP、任务勾选和文档隐私扫描。 | signed HAP 存在，`default` 产品关联本地 `default` 签名方案；M0-4/M1-2 已勾选，M1-3 未勾选；本阶段文档与模板未发现本机绝对路径、设备标识或口令。 |

## 2026-10-21：真机画面核对

用户提供真机截图并询问大面积黑屏原因。本次只核对 M1-2 页面表现并补记视觉证据；不实施 M1-3 或 M2-1。

| 序号 | 操作与目的 | 实际结果及验证状态 |
| --- | --- | --- |
| 1 | 查看用户提供的真机截图，核对页面可见内容。 | 顶部白色区域显示 `GPUI mobile host: M1-2` 和 `probe bits: 7`；其下由 XComponent 占据的区域呈黑色。页面文字已视觉核验，用户未报告崩溃。 |
| 2 | 读取 `Index.ets`、Node-API 桥接、Rust 探针及 M1-2/M1-3/M2-1 任务边界，定位黑色区域对应的实现。 | `XComponent(type: 'surface')` 设置了 `width('100%')` 和 `layoutWeight(1)`，占据剩余页面；桥接只暴露 `probe()`，没有 Surface 回调或向窗口提交画面。当前黑色区域与空 Surface 的实现相符，不是 GPUI 已绘制内容。 |
| 3 | 更新 M1-2 记录和工程 README，检查任务状态。 | 已补充截图可见结果与后续绘制边界；M1-2/M0-4 保持完成，M1-3/M2-1 仍待实施。仅改文档，无需重复构建。 |

## 2026-10-21：C/C++ 文件用途核对

用户询问为何工程中仍有 C/C++ 文件，以及这是否违反直接编写 Rust FFI 的约定。本次仅核对用途，不改变已验证的 M1-2 构建链，也不提前实施 M1-3。

| 序号 | 操作与目的 | 实际结果及验证状态 |
| --- | --- | --- |
| 1 | 列出工程内手写 `.c`、`.cpp` 和 CMake 文件，读取内容及 Cargo 配置。 | `entry/src/main/cpp/napi_init.cpp` 参与 HAP：仅注册 Node-API 模块并把 `probe()` 转发给 Rust `staticlib`；SDK Drawing/XComponent 的 FFI 声明与探针逻辑在 Rust。`probes/m0_library_load.c` 是旧设备预检，`probes/m1_ffi_layout.c` 只用于编译期 ABI 断言；这两个 C 文件均不打入 HAP。 |
| 2 | 对照本项目需求和 SDK `node_api.h` 的 `napi_module`、`napi_module_register` 定义，判断 C++ 是否强制。 | SDK 提供 C ABI；Rust 可用 `#[repr(C)]`、`extern "C"` 及模块加载时注册来实现同一桥接。现有 C++ 是 M1-2 的过渡实现，并非 SDK 或 GPUI 的硬性要求；若选择全 Rust，可另设明确子任务把原生模块改为 Rust `cdylib` 并复测真机，避免与 M1-3 混做。 |
| 3 | 记录语言边界和现阶段原因。 | 已明确 C++ 只用于 Node-API 注册/转发，C 探针只供验证；没有新增 SDK 第三方绑定。此轮无源码改动，既有真机验证状态不变。 |
