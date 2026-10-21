# M3-2：GPUI Scene 到 Native Drawing

日期：2026-10-21。范围仅为 `Scene::batches()` 的图元转换、自有单色 Atlas 和当前页面的绘制验证。字体布局/光栅化属于 M3-3，触摸交互与按需帧调度属于 M4。

| 日期 | 顺序 | 操作与目的 | 实际结果与验证状态 |
| --- | --- | --- | --- |
| 2026-10-21 | 1 | 开始前告知 M3-2 的做法、原因与验证方式；读取任务、需求、M3-1 记录、手机工程源码和锁定的 GPUI `Scene` 接口。 | 确认窗口 `draw(&Scene)` 仍为空、Surface 创建时仍绘制 M2 探针；锁定 GPUI 的批次含 Quad、阴影、路径、下划线、三种 Sprite 与 Surface，当前空根视图不产生待验收的计数器图元。工作区还有上次文档规范修改，保留不覆盖。 |
| 2026-10-21 | 2 | 核对本机 API 26 SDK 的 Canvas、圆角、裁剪、位图和 Pen 声明，并检索 GPUI 官方源码。 | SDK 声明了保存/恢复、矩形裁剪、位图绘制和逐角圆角 API；GPUI `Background::as_solid()` 可明确拒绝未支持的渐变。具体 FFI 与真机行为待实现验证。 |
| 2026-10-21 | 3 | 在 `sdk_ffi/drawing/` 补齐 Canvas 裁剪/画笔/位图最小 FFI；实现自有 `MonoAtlas`、`Scene::batches()` 绘制器，并让 GPUI 平台窗口在 ArkUI 宿主 pump 中触发帧。 | 首轮目标 `cargo check --locked --offline --target aarch64-unknown-linux-ohos` 通过；批次按 GPUI 顺序绘制，未实现的批次返回带名称的错误，不静默跳过。Atlas 限单色掩码，纯色 Quad 支持矩形裁剪、统一圆角和统一宽度实线边框。 |
| 2026-10-21 | 4 | 执行 `cargo fmt --check`、目标检查及 `scripts/build-native.sh`。 | 格式和目标检查通过；首次直接执行构建脚本因 shell 未设置 SDK 环境变量失败，指定本机 API 26 Native SDK 后 Rust aarch64 原生库构建成功。仓库未记录本机绝对 SDK 路径。 |
| 2026-10-21 | 5 | 用 DevEco `hvigorw --no-daemon ... assembleHap` 打包签名，安装/启动首轮形状验证 HAP，读取 `GPUI_M3_1` 日志并检查临时截图。 | 构建、签名、安装成功；真机显示 GPUI 生成的背景和圆角带边框卡片，日志 `GPUI Scene flush generation=1 quads=6 monochrome_glyphs=0`。这一轮仍保留旧探针首帧，随后移除。 |
| 2026-10-21 | 6 | 用 `ScreenDrawing` 替换旧 `ProbeDrawing` 首帧绘制；追加固定 A 形单色掩码，经自有 Atlas、GPUI `MonochromeSprite` 与 SDK 位图绘制；重新构建、签名、安装并检查截图。 | 真机日志记录 `quads=6 monochrome_glyphs=1`，卡片中央可见 A 形字形；页面文字仍由 ArkTS 宿主显示，GPUI 真实字体尚未实现。旧探针不再负责当前画面。 |
| 2026-10-21 | 7 | 在真机点击 Resize，检查新尺寸的 Drawing Surface 与 GPUI Flush。 | 首轮发现 NativeWindow `1260×1918 → 1008×1918` 后只创建了新 Drawing Surface，没有 GPUI 新帧；锁定 GPUI 的 `Window::resize` 只更新平台尺寸。补充 `window.refresh()`，并按当前 Surface 尺寸定位诊断字形；重建 HAP 复测后新尺寸有 `quads=6 monochrome_glyphs=1` Flush。 |
| 2026-10-21 | 8 | 在 SDK 位图 FFI 增加 API 26 行跨度查询，逐行写入 RGBA/PREMUL 掩码；重新执行格式、目标检查、原生构建、HAP 构建签名、安装启动。 | 全部通过；真机再次记录 `quads=6 monochrome_glyphs=1`，未出现 Drawing 错误。按行跨度写入避免假定设备位图每行紧密排列。 |
| 2026-10-21 | 9 | 把形状验证视图改为桌面计数器的布局与三个按钮外观，暂不接触摸；记录 GPUI 原始 Scene 批次，重新构建/安装/截图。 | 最终真机原始 Scene 日志为 `quads (4)`，对应页面背景和三个圆角按钮；追加诊断 A 后 Flush 为 `quads=4 monochrome_glyphs=1`。截图显示按钮形状及 A，`GPUI Counter`、`0`、`−`、`+`、`Reset` 仍不可见，因为当前 `NoopTextSystem` 不产出字形；这部分留给 M3-3。 |
| 2026-10-21 | 10 | 在最终计数器布局 HAP 上点击 Resize、Hide/Show，读取 Surface/Scene 日志；解包 HAP 检查原生库目标与动态依赖。 | Resize 后 `1008×1918` 重新 Flush `quads=4 monochrome_glyphs=1`；第一代 Surface 销毁并释放 Drawing，第二代 Surface 创建后再次 Flush 同样图元。HAP 只含一份 `arm64-v8a` ELF64/AArch64 Rust 库，动态依赖仅为既有系统库。 |
| 2026-10-21 | 11 | 核对任务、需求、README 与阶段记录，移除未使用的圆角 FFI 声明；执行最终 `cargo fmt --check`、OHOS 目标 `cargo check --locked --offline` 与 `git diff --check`。 | 三项检查通过；任务状态与 README 均明确 M3-2 图形/诊断字形已真机验证、系统字体和交互未实现。此前保留的仓库级文档规范改动未被覆盖；本轮不提交或推送。 |

M3-2 完成。当前计数器布局的**原始**图元只有 4 个 Quad；固定 A 字形是为验证 Atlas 而在平台窗口追加的 1 个单色 Sprite，不是实际计数器文字。纯色填充、统一圆角/边框、矩形裁剪和单色掩码已在真机显示；渐变、非统一圆角/边框、阴影、路径、下划线、彩色/子像素图像及变换 Sprite 尚未实现，出现时绘制器会报错。真实系统字体、文字度量与字形光栅化属于 M3-3；触摸到 GPUI 的输入映射和按需帧调度分别属于 M4-1、M4-2。诊断截图只留在本机临时目录，没有加入工程。
