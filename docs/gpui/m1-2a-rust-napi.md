# M1-2A：Rust-only Node-API 模块

日期：2026-10-21。用户确认将运行时 C++ 桥接替换为 Rust FFI。本任务在 M1-3 前独立进行：按 API 26 SDK 头文件手写 Node-API 最小 C ABI，Rust `cdylib` 注册同名模块，删除 HAP 的 C++ 编译链，以相同 ArkTS 页面、签名和真机日志复验。这样可以直接验证语言边界调整是否保持 M1-2 的行为，不混入 Surface 回调或绘制。

状态：**已完成**。Rust-only HAP 已构建、校验、安装并在真机运行；应用日志 `loadContent succeeded`、`Rust probe bits: 7`。执行前已告知做法、原因和验证标准。M1-3 尚未实施。

| 序号 | 操作对象与目的 | 实际结果及验证状态 |
| --- | --- | --- |
| 1 | 读取当前 C++ 桥接、Cargo/DevEco 构建配置、任务文档及 API 26 SDK 的 `node_api.h`、`js_native_api_types.h`，确定替换范围。 | 当前 C++ 只注册 Node-API 并转发 `probe()`；SDK 提供 `napi_module_register`、`napi_create_uint32`、`napi_define_properties` 的 C ABI，可用 Rust 声明。两个 `probes/*.c` 不参与 HAP。 |
| 2 | 查阅 [Huawei 官方构建资料](https://developer.huawei.com/consumer/cn/doc/harmonyos-guides/ide-hmos-oh-package-json5)，确认自带 `.so` 的模块打包路径，再由本工程实际 HAP 内容验证。 | 官方资料说明 `libs` 内可放自行引入的 `.so`；后续实验证明本工程的 `entry/libs/arm64-v8a` 会被打入 HAP。 |
| 3 | 在任务清单新增 M1-2A、明确语言边界并建立本记录。 | M1-3 仍保持未开始；本轮只替换 Node-API 桥接。 |
| 4 | 将 Rust 库改为 `cdylib`，新增 `sdk_ffi/napi.rs` 与 Rust 模块注册/属性导出，并保留原有探针业务逻辑。 | Node-API 的 `napi_module`、`napi_property_descriptor` 和三个所需函数按 SDK C ABI 声明；用 ELF `.init_array` 在共享库加载时注册 `gpui_mobile_counter`。没有新增 Cargo 依赖。 |
| 5 | 首次执行 `cargo fmt` 和 OHOS 目标 `cargo build --locked --offline`。 | 格式整理完成；`cdylib` 链接因 Cargo 默认调用主机 `cc` 失败。该失败仅发生在交叉链接阶段。 |
| 6 | 将 `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_OHOS_LINKER` 指向 API 26 SDK 的目标 Clang，重跑构建，检查 ELF。 | 构建成功；生成 ELF64/AArch64 `.so`，含 `.init_array`；动态依赖仅有 `libnative_drawing.so`、`libace_napi.z.so`、`libace_ndk.z.so`、`libc.so`。目标库留有预期的 Node-API/系统接口导入和 Rust 探针导出。 |
| 7 | 先把 Rust `.so` 暂用不同文件名放在 `entry/libs/arm64-v8a`，保留旧 C++ 构建链并调用 Hvigor，核对打包路径。 | signed HAP 同时包含旧模块及暂名 Rust 模块，证明该目录会被 DevEco 收进 HAP。此实验 HAP 未安装；随后删除暂名库。 |
| 8 | 删除 `napi_init.cpp`、CMakeLists 与模块的 `externalNativeOptions`，新增 `scripts/build-native.sh` 从 SDK 目标 Clang 构建并暂存同名 Rust `.so`，忽略暂存目录。 | 运行时原生源代码只剩 Rust；`cpp/types` 下的 `.d.ts` 仅是 ArkTS 类型声明，两个 `probes/*.c` 仍只供单独验证。脚本不写死本机 SDK 路径。 |
| 9 | 运行 Hvigor `clean`、Rust 构建脚本和 `assembleHap`，检查 HAP。 | 清理旧 CMake 产物后构建和签名成功。HAP 的原生库目录仅有 Rust `libgpui_mobile_counter.so`，无 `libc++_shared.so` 或旧 C++ 库；`cargo fmt --check` 通过。 |
| 10 | 用 SDK 目标 Clang 对 `napi_module` 和 `napi_property_descriptor` 做大小、对齐及关键偏移静态断言，并在 Rust FFI 添加对应编译期断言。 | API 26 SDK C 断言全部通过：模块 72 字节、属性描述符 64 字节；Rust 重新构建、打包和签名成功。未将临时 C 输入文件加入工程。 |
| 11 | 对最终 signed HAP 执行 SDK `verify-app` 并在真机更新安装。 | 代码签名与摘要校验通过，`hdc install` 返回成功；本地签名与安装验证完成。 |
| 12 | 尝试用 `hdc shell aa start` 启动并查询 `GPUI_M1_2` 日志。 | 手机锁屏返回 `10106102`，开发者模式下无法自动解锁；已请求用户手动解锁。Rust 注册与探针在此版本上的真机运行仍**未验证**。 |
| 13 | 再次尝试启动最终版 HAP。 | 手机仍锁屏，同样返回 `10106102`；没有把旧版 M1-2 的日志误当成本版结果。 |
| 14 | 从最终 HAP 提取其中的 Rust `.so`，检查打包后 ELF 的架构、`.init_array`、系统依赖和 Node-API 导入。 | HAP 内库仍为 ELF64/AArch64，保留 `.init_array` 与所需系统依赖、三个 Node-API 导入符号；静态检查通过，运行结果仍待手机解锁。 |
| 15 | 删除旧版被忽略的 `entry/.cxx` 生成目录及本轮临时 HAP 检查、签名校验输出。 | 清理完成；保留可复现脚本、源码和忽略目录中的最终 HAP/暂存 `.so`。 |
| 16 | 手机解锁后运行最终 HAP，首次按标签查询日志，并检查测试应用进程。 | `aa start` 成功且进程存在；首次 `hilog -x -T GPUI_M1_2` 无记录，未以此代替真机验证。 |
| 17 | 仅强制停止本测试应用、重新启动，并立即按 `GPUI_M1_2` 标签读日志。 | 真机新进程记录 `loadContent succeeded` 和 `Rust probe bits: 7`；Rust Node-API 注册、ArkTS 调用、Drawing 错误码探针和 XComponent 符号寻址均与 M1-2 基线一致。M1-2A 验收通过。 |
| 18 | 核对构建脚本语法、Rust 格式、HAP 库清单、运行时源文件和阶段文档。 | `sh -n`、`cargo fmt --check` 通过；HAP 仅有一份 Rust 原生 `.so`；运行时目录无 `.c/.cpp` 源文件；M1-2A 已勾选而 M1-3 仍未勾选；文档未发现本机绝对路径、设备标识或签名口令。 |

复现时先设置 `DEVECO_SDK_HOME` 为 DevEco SDK 根目录，或设置 `OHOS_NATIVE_SDK_HOME` 为 Native SDK 根目录，再在工程目录执行 `scripts/build-native.sh`；之后设置 DevEco 的 `NODE_HOME`、`JAVA_HOME`，运行 `hvigorw --mode module -p module=entry@default -p product=default -p buildMode=debug assembleHap`。`entry/libs/arm64-v8a` 是本机生成的暂存目录，不纳入版本控制。签名仍使用工程本机忽略的 `build-profile.json5`。
