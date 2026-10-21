# M1-1：SDK FFI 声明与链接核对

日期：2026-10-20。范围：独立手机工程 `gpui/ohos-mobile-counter/` 的首批 XComponent 和 Native Drawing C ABI。开始前已告知用户做法、原因和验证方法。本次没有创建 HAP、调用 SDK 函数或操作真机；实际加载与绘制仍分别属于 M1-2、M2-1。

## 已实现的接口边界

- [`src/sdk_ffi/xcomponent.rs`](../../gpui/ohos-mobile-counter/src/sdk_ffi/xcomponent.rs)：绑定 `libace_ndk.z.so`，定义不透明 XComponent、4 个生命周期/触摸回调槽、10 个触点的事件结构、触摸类型常量、状态码检查，以及回调注册、Surface 尺寸、触摸事件读取。头文件 `ace/xcomponent/native_interface_xcomponent.h` 标为 API 8 起。
- [`src/sdk_ffi/drawing.rs`](../../gpui/ohos-mobile-counter/src/sdk_ffi/drawing.rs)：绑定 `libnative_drawing.so`，定义 GPU context、Surface、Canvas、Brush、Rect、RoundRect 的不透明类型，按值传递的 `OH_Drawing_Image_Info`，格式常量、状态码检查，以及屏幕 Surface、Flush、清屏/矩形/圆角矩形、资源销毁和错误码函数。`GpuContextCreate`、`SurfaceCreateOnScreen`、`SurfaceFlush` 为 API 16 起；`ErrorCodeReset` 为 API 18 起；其余已绑定函数为 API 8、11 或 12 起。本机两套 SDK 均为 API 26。
- [`src/lib.rs`](../../gpui/ohos-mobile-counter/src/lib.rs)：仅在 `target_env = "ohos"` 编译 FFI 模块。Node-API 的小型 C/C++ 桥接在 M1-2 编写，因此本任务不引入 Rust Node-API 绑定；XComponent 逐帧回调在 M4-2、有需要时 Native VSync 在其后绑定。
- 所有句柄均为不透明指针；C enum 在 Rust 中用 `i32` 保存，避免未知返回值成为无效 Rust discriminant。外部调用保留为 `unsafe`，由后续实际使用任务建立生命周期/空指针处理；本任务仅提供非零状态码的显式错误转换。不能因静态 ABI 匹配而假定设备可加载。

SDK 头文件说明 `GetXComponentSize` 的宽高单位是 vp，而 Drawing 图像信息的宽高单位是像素；密度与实际 Surface 尺寸换算将在 M1-3/M2-1 实测，不能直接混用。

## 复现检查

`OHOS_SDK_NATIVE` 指向本机 API 26 SDK 的 `native` 目录；以下命令无需把绝对 SDK 路径写进工程。链接探针只取函数地址，不执行系统函数，也不用于设备 shell 运行。

```sh
cd gpui/ohos-mobile-counter
cargo fmt --check
cargo check --locked --offline --target aarch64-unknown-linux-ohos
"$OHOS_SDK_NATIVE/llvm/bin/aarch64-unknown-linux-ohos-clang" -std=c11 -Wall -Wextra -Werror -fsyntax-only probes/m1_ffi_layout.c -I"$OHOS_SDK_NATIVE/sysroot/usr/include"
"$OHOS_SDK_NATIVE/llvm/bin/clang-format" --dry-run --Werror probes/m1_ffi_layout.c
rustfmt --check --edition 2024 probes/m1_ffi_link.rs
rustc --edition=2024 --target aarch64-unknown-linux-ohos -D warnings probes/m1_ffi_link.rs -L native="$OHOS_SDK_NATIVE/sysroot/usr/lib/aarch64-linux-ohos" -C linker="$OHOS_SDK_NATIVE/llvm/bin/aarch64-unknown-linux-ohos-clang" -o /tmp/gpui_m1_ffi_link
"$OHOS_SDK_NATIVE/llvm/bin/llvm-readelf" -h /tmp/gpui_m1_ffi_link
"$OHOS_SDK_NATIVE/llvm/bin/llvm-readelf" -d /tmp/gpui_m1_ffi_link
```

`m1_ffi_layout.c` 在目标 Clang 下断言：回调表 32 字节、触点 56 字节、触摸事件 624 字节、图像信息 16 字节及关键字段偏移；Rust `const` 断言核对相同布局。链接探针生成 ELF64 AArch64，`NEEDED` 含 `libnative_drawing.so`、`libace_ndk.z.so`、`libc.so`。这只说明 SDK 链接桩可解析，不证明设备应用进程内加载成功。

## 按执行顺序的操作记录

| 顺序 | 操作与目的 | 实际结果与验证状态 |
| --- | --- | --- |
| 1 | 读取 `AGENTS.md`、`docs/gpui/requirements.md`、`tasks.md`、现有手机工程清单与入口，确定本项边界。 | 工程仍只有 GPUI 依赖和空库入口；M1-2/M1-3/M2-1 均未实施。 |
| 2 | 在用户目录做一次广域 SDK `find`，再尝试 zsh 通配符定位。 | 广域搜索过慢，已中止；zsh 对不存在的路径报 `no matches found`。随后限定 `Library` 与 DevEco 安装目录，找到独立 `26.0.0.38` 和 DevEco 随附 `26.0.0.105` Native SDK。 |
| 3 | 读取两套 SDK 元数据、XComponent 与 Drawing 相关头文件，核对签名、结构体、常量、API 级别及 `@library`。 | 选独立 API 26 SDK 作为基准，DevEco 版复核。XComponent 使用 `libace_ndk.z.so`；Drawing 使用 `libnative_drawing.so`。确认触摸数组容量 10、图像信息按值传递、`GetXComponentSize` 为 vp。 |
| 4 | 在临时 C 文件中用目标 Clang 首次验证 `sizeof` 与 `offsetof`，随后把断言移入 `probes/m1_ffi_layout.c`。 | 独立 SDK 的全部断言通过；DevEco 随附 SDK 的相同断言也通过。临时文件不是工程依赖。 |
| 5 | 新增 `src/sdk_ffi/{mod.rs,xcomponent.rs,drawing.rs}`，在 `src/lib.rs` 按 OHOS 目标导出。 | 声明 3 个 XComponent 与 20 个 Drawing 函数、对应系统库、状态检查和布局断言；未引入任何新的 Cargo 依赖。 |
| 6 | `cargo fmt --check`、`cargo check --locked --offline --target aarch64-unknown-linux-ohos`；用 `llvm-readelf --dyn-syms` 检查 SDK aarch64 库。 | 格式与交叉编译通过；23 个绑定符号均在对应 SDK 库导出表内。该库为 SDK 链接桩，并非真机加载证据。 |
| 7 | 新增 `probes/m1_ffi_link.rs` 并用目标 `rustc` 链接 23 个函数地址。 | 首次链接成功且 ELF `NEEDED` 包含两个目标库，但函数项直接转整数产生 23 个警告。将转换改为函数指针后，以 `-D warnings` 重编译通过。 |
| 8 | 对仓库内 C 布局探针运行目标 Clang 和 `clang-format --dry-run --Werror`。 | 编译通过；首次格式检查指出长断言换行不符合格式。运行 `clang-format -i` 后格式与目标 Clang 检查均通过。 |
| 9 | 用 DevEco 随附 API 26 头文件再次编译仓库内 C 探针，检查 Rust probe 格式和 OHOS 目标编译。 | 两套 SDK 的 C 布局检查均通过；Rust 格式和 OHOS 目标 `cargo check` 通过。主机目标 `cargo check` 仅用于确认条件编译隔离，不验证 OHOS FFI。 |
| 10 | 更新 `tasks.md`、`requirements.md` 和手机工程 README，并扫描新增源码/文档是否包含本机 SDK 绝对路径；复查任务勾选和构建检查。 | M1-1 已勾选，M0-4/M1-2 仍未勾选；未发现本机 SDK 绝对路径或新增绑定项目引用，C/Rust 格式与 OHOS 目标静态检查通过。 |
| 11 | 用 `llvm-readelf` 复核链接探针 ELF 头、动态依赖和未定义 SDK 符号数；删除临时 C 源和 ELF。 | ELF64 AArch64，动态依赖包含两个目标库，23 个 SDK 符号留待装载器解析；临时产物已删除，工程内保留可复现探针源码。 |

## 尚未验证

本任务没有调用任何绑定函数；空指针、SDK 错误码、线程归属、回调注销、Surface 生命周期、vp/px 换算、实际渲染与真机库加载都需要 M1-2、M1-3、M2-1 后续实测。设备曾拒绝执行 shell 临时 ELF，因此本链接探针不会作为真机探针使用。
