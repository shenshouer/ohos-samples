# M0-5：GPUI 在 OHOS 上的 Linux 条件编译审计

日期：2026-10-20。范围仅为固定官方提交 `c4a82ab1ab958aa43b0c0b15c0bad4fb77a7ca7f` 的 `crates/gpui/`、`crates/gpui_platform/`、当前桌面计数器和独立手机工程。开始前已向用户说明检查方法、原因与验证范围。本次没有实现平台层、修改 GPUI 源码或构建 HAP。

## 结论与后续处理

`aarch64-unknown-linux-ohos` 同时命中 `target_os = "linux"` 和 `target_env = "ohos"`。官方 `gpui_platform::current_platform` 因此选择 `gpui_linux`，会进入桌面后端；手机工程必须继续只依赖 GPUI 核心，自行提供 `Platform` 并以 `Application::with_platform(...).run_embedded(...)` 接入宿主。目标普通依赖树中没有 `gpui_platform`、`gpui_wgpu`、`wgpu`、`winit`、`x11` 或 `wayland`。

GPUI 核心有若干 Linux 分支，但已查到的计数器必需路径没有强制启动桌面图形后端。首版决定**不修改固定 GPUI 源码**。手机平台实现必须提供 `read_from_primary` / `write_to_primary` 两个 Linux 条件下的 trait 方法；计数器不使用 primary selection，可先返回 `None` / 忽略写入，并在 README 明确未支持。`GraphicalEnvironment`、桌面标题栏按钮布局不用于手机宿主。可在 M3-1 实现平台层时通过编译验证这些接口选择；真机运行语义仍未验证。

| 固定源码位置 | OHOS 命中与影响 | 决定或待验证项 |
| --- | --- | --- |
| `gpui_platform/src/gpui_platform.rs` 的 `current_platform` | Linux 分支选择 `gpui_linux::current_platform`。 | 不依赖该 crate；M3-1 自建手机 `Platform`。 |
| `gpui/src/platform.rs` 的 `Platform` trait、`GraphicalEnvironment`、`WindowingModes` | Linux 条件新增 primary selection 方法及桌面显示环境/模式类型。`graphical_environment` 默认返回 `None`；不启用 X11/Wayland feature 时 compositor 猜测为 `Headless`。 | 实现两个必需方法；不调用桌面环境切换；不把 `Headless` 猜测当作手机显示状态。 |
| `gpui/src/window.rs` 的 `is_window_hovered` | OHOS 会读取独立的 `hovered` 状态；回调来自 `PlatformWindow::on_hover_status_change`，目前在 `reset_cursor_style` 使用。 | M3-1 按宿主窗口激活状态维护回调；M4-1 检查触摸后焦点/光标行为。触摸 Tap 本身走 `dispatch_touch_event` 与手势识别器，不以 `is_window_hovered` 为前置条件。 |
| `gpui/src/app.rs` 的 primary selection API | OHOS 会公开 Linux 条件的 `App::read_from_primary` / `write_to_primary`。 | 首版不调用；与平台方法的桩行为一致。 |
| `gpui/src/keymap/context.rs`、`platform/keystroke.rs` | 默认键位上下文把 OHOS 标为 `os=linux`，平台修饰键格式化为 Linux `super-` / `❖`。 | 首版没有键盘/输入法；后续支持键位时需检查，并按实际键盘语义决定是否给固定源码打最小补丁。 |
| `gpui/src/svg_renderer.rs` | Emoji 字体候选使用 Linux 列表。 | 计数器的拉丁文字、数字和 `−` 由 M3-3 系统文字层处理；Emoji 回退暂不在首版范围，后续真机检查。 |
| `gpui/src/profiler/journal.rs` | Linux 路径用 `libc::clock_gettime` 读取 `CLOCK_MONOTONIC` 和 `CLOCK_BOOTTIME`；失败返回 `None`。 | 交叉编译通过；真机时钟能力未验证，若性能记录异常再做运行探针。 |
| `gpui/src/scene.rs`、`gpui/src/platform.rs` 多处 `cfg_attr` | 针对无 X11/Wayland 构建添加 `allow(dead_code)` 等 lint 标记，`Scene::batches` 仍存在。 | 不影响图元接口；M3-2 实际接入时检查批次。 |
| `gpui/src/gpui.rs` 的 `queue`、`hot-reload`、`platform.rs` 的 `wayland` 分支 | `queue` 在 Linux 目标导出；热重载及 Wayland 条件需对应 feature。 | 当前未启用默认 feature，相关桌面功能不参与此计数器；`queue` 导出本身不启动桌面后端。 |

## 发布版与固定提交的已知 API 差异

桌面工程使用 crates.io `gpui = 0.2.2`，入口为 `Application::new().run(...)`。固定 Git 提交的核心提供 `Application::with_platform(Rc<dyn Platform>)` 和 `run_embedded(...) -> ApplicationHandle`；`gpui_platform::application()` 才负责选择默认平台。两份源码的包版本字段都写 `0.2.2`，不能据此认为 API 相同。桌面 `Counter::render`、`on_click` 等视图代码尚未在固定提交上迁移或编译；**视图级 API 差异未验证**，应在实际移植视图的任务里逐项记录，不把本次核心 `cargo check` 视为视图兼容性证明。

## 按执行顺序的操作记录

| 顺序 | 操作、目的 | 实际结果与状态 |
| --- | --- | --- |
| 1 | 读取 `docs/gpui/requirements.md`、`tasks.md`、`m0-validation.md`、工程 README 和 `AGENTS.md`，确定 M0-5 边界与记录要求。 | 确认本次只做条件编译审计；M0-4 因缺 HAP 仍未完成。 |
| 2 | 在固定源码仓库执行 `git rev-parse HEAD`、`git status --short`；用 `rustc --print cfg --target aarch64-unknown-linux-ohos` 核对目标 cfg。 | 提交为上述固定 SHA，源码临时工作树无改动；目标同时命中 `linux`、`ohos`。 |
| 3 | 用 `rg -n 'target_os = "linux"|target_env = "ohos"|target_os = "freebsd"' crates/gpui/src crates/gpui_platform/src` 枚举分支，再按位置读取 `platform.rs`、`gpui_platform.rs`、`window.rs`、`app.rs`、`scene.rs`、`svg_renderer.rs`、`keymap/context.rs`、`platform/keystroke.rs`、`profiler/journal.rs` 和 GPUI Cargo feature。 | 得到上表逐项判断；部分条件仅影响 lint，必需 primary selection 方法和桌面平台选择需特别处理。 |
| 4 | 用 `rg` 搜索 `graphical_environment`、primary selection、窗口悬停、触摸分派以及相关事件回调的使用点。 | `is_window_hovered` 在核心内用于重置光标样式；触摸 Tap 由手势识别后分派按下/抬起事件。运行效果待 M4-1 真机验证。 |
| 5 | 读取桌面 `src/main.rs`、crates.io 发布版 `gpui-0.2.2/src/app.rs` 与固定源码 `app.rs`，比较入口。 | 找到上述启动 API 差异；视图尚未移植，故未声称所有视图 API 兼容。 |
| 6 | 执行 `cargo tree --locked --offline --target aarch64-unknown-linux-ohos --edges normal`，筛查桌面后端依赖。 | 筛查命令无匹配项，`rg` 退出码 1 表示没有目标关键字；Cargo tree 本身成功。 |
| 7 | 在手机工程执行 `cargo fmt --check` 和 `cargo check --locked --offline --target aarch64-unknown-linux-ohos`。 | 均通过，后者完成 GPUI 核心及 312 包锁文件解析下的目标静态检查；不证明链接、HAP 或运行。 |
| 8 | 用 `git status --short` 尝试检查示例目录变更。 | 该目录不是 Git 仓库，命令报 `fatal: not a git repository`；改用文件清单与文档核对，本次仅改本审计文档、任务状态和手机 README。 |
| 9 | 首次从示例目录执行针对固定源码相对路径的 `rg`。 | 路径不在当前目录，返回 `No such file or directory`；随后在固定源码目录重跑并取得结果。 |
| 10 | 写入本审计文档，勾选 `tasks.md` 的 M0-5，并更新手机工程 README；检查文件清单。 | 只有文档变化；手机 Rust 清单和入口、桌面工程均未修改。Cargo 构建产生的 `target/` 在本工程忽略规则内。 |
| 11 | 初次在手机工程目录运行文档路径 `rg`，随后在仓库根目录重跑，并核对 M0-1 至 M0-5 勾选状态。 | 初次因相对路径不匹配报 `No such file or directory`；根目录复查通过，M0-4 未勾选、M0-5 已勾选，README 链接目标存在。重复 `cargo fmt --check` 与静默目标 `cargo check` 均通过。 |

本次没有设备操作。M0-5 是**静态审计完成**；真实平台生命周期、文字、触摸和渲染的 Linux 分支效果仍须分别在 M3/M4 验证。M0-4 保持未勾选，待 M1-2 在已签名 HAP 内验证动态库加载。
