# GPUI 桌面计数器

这是鸿蒙 GPUI 移植前的桌面交互基线：用 GPUI 自身创建窗口、绘制计数，并通过点击事件修改状态和重绘。初始值为 `0`；`+` 加一，`−` 减一，`Reset` 清零。

## 依赖与环境

- 官方 [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) crate `=0.2.2`，关闭默认 feature，启用 `font-kit` 和 `runtime_shaders`；具体传递依赖由 `Cargo.lock` 固定。`runtime_shaders` 使此机缺少 `metal` 命令时仍能编译，着色器在应用启动时由 Metal 加载。
- 已验证目标：macOS 26.6.1 arm64 桌面，Rust 1.99.0，Xcode Command Line Tools。无需应用权限、签名或 HarmonyOS SDK。
- 本例不构建 HAP，也不验证 `XComponent`。它只验证桌面上的 GPUI 视图、事件和状态更新。后续鸿蒙示例将根据目标 SDK 头文件在本项目中自建所需 FFI。
- 此已发布版本使用 `Application::new()` 选择桌面后端；[当前 GPUI 源码](https://github.com/zed-industries/zed/blob/main/crates/gpui/src/app.rs)中的 `Application::with_platform` 是后续鸿蒙适配计划采用的接口。两者不可未经版本核对直接混用。

## 构建与运行

在此目录执行：

```sh
cargo fmt --check
cargo check --locked
cargo run --locked
```

窗口打开后依次点击 `+`、`−` 和 `Reset`，预期数字分别变化为 `1`、`0`、`0`；连续点击应持续更新。源码只有 `src/main.rs`，构建产物位于被忽略的 `target/`。

## 验证记录

2026-10-20 在上述 macOS arm64 主机执行：`cargo fmt --check`、`cargo check --locked`、`cargo build --locked` 均通过；产物确认为 Mach-O arm64。运行后可见 GPUI 窗口，实测点击 `Reset`、`+`、`−`，数字依次显示 `0 → 1 → 0`。为方便 GUI 验证，曾在被忽略的 `target/` 中临时封装 `.app`，正式示例仍直接使用 `cargo run --locked`。桌面运行时出现过 macOS `com.apple.hiservices-xpcservice` 日志，但窗口与按钮交互正常。

已发布依赖中的 `block 0.1.6` 和 `proc-macro-error2 2.0.1` 有 Rust future-incompat 提示，本次未阻断构建。该桌面结果不能代表鸿蒙设备上的编译、渲染或触摸结果。
