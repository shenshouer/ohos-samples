# M3-3：系统文字与 GPUI 字形

状态：已完成。范围仅为 M3-3，不包括 M4 交互和帧调度。

## 2026-10-21 操作记录

| 顺序 | 操作对象 / 目的 | 实际结果 | 验证状态 |
| --- | --- | --- | --- |
| 1 | 读取 `docs/gpui/tasks.md`，确认任务边界与前置状态。 | M3-2 已完成，M3-3 为当前待办；已有 M3-2 未提交改动需保留。 | 已核对 |
| 2 | 读取 GPUI 固定提交的 `PlatformTextSystem` 接口、手机平台实现及源码结构，确定接入点。 | 当前 `text_system()` 每次返回 `NoopTextSystem`；需替换为系统字体实现；现有 M3-2 Atlas 可复用。 | 已核对 |
| 3 | 将 M3-3 标记为实施中并建立本记录。 | 待实现、构建和真机验证。 | 未验证 |
| 4 | 查阅安装的 API 26 Native Drawing 字体、TextBlob、Canvas/Bitmap 头文件及固定提交 GPUI 的文本类型与绘制流程；辅以官方接口文档核对。 | 确认默认字型、字体度量、UTF-8→字形、字形宽度、TextBlob 边界、Alpha8 离屏 Canvas 的 C ABI；GPUI 使用字体单位与物理像素两套坐标。 | 静态已核对，真机未验证 |
| 5 | 新增 `sdk_ffi/drawing/text.rs`、`gpui/text.rs`，平台共享文本系统；删除 M3-2 固定诊断 A 字形。 | 代码已接入 GPUI 原始 Scene；仅支持系统默认字型、简单从左到右逐字符布局；高级 shaping、fallback、自定义字体暂未实现。 | 待构建 |
| 6 | 执行 `cargo fmt --all` 与 `scripts/build-native.sh`。 | 格式化通过；首次编译失败：`Pixels.0` 为私有字段（E0616），同时有两处未使用导入。 | 失败，已定位 |
| 7 | 将字号读取改为 `f32::from(Pixels)`，删除未使用导入。 | 已修正首次编译错误，等待复编。 | 待构建 |
| 8 | 删除 `OhosWindow` 的诊断 Scene。 | 首次补丁因固定 A 掩码上下文不匹配而失败；重新读取该函数后，用精确字符串边界删除并让 `draw()` 直接提交 GPUI 原始 Scene。 | 代码已核对 |
| 9 | 再运行 `scripts/build-native.sh`。 | aarch64 构建通过，出现 `_face` 持有字段未读警告；同时审查 RAII，修正创建对象后若 SDK 错误码非零时的释放路径。 | 已构建，警告已处理 |
| 10 | 将 ArkTS 页面与 Rust hiLog 标签改为 `GPUI_M3_3`，以区分 M3-2 记录。 | 标题显示 M3-3，后续真机日志使用新标签。 | 真机已验证 |
| 11 | 执行 `cargo fmt --all --check`、`scripts/build-native.sh` 和 `hvigorw ... assembleHap`。 | 格式检查、aarch64 原生构建及签名 HAP 均通过；ArkTS 报原生模块类型验证提示，未阻止构建或运行。 | 已通过 |
| 12 | `hdc list targets`，`hdc install entry/build/default/outputs/default/entry-default-signed.hap`，`hdc shell aa start -a EntryAbility -b com.example.gpuiohosmobilecounter`。 | 检测到原测试设备，安装与启动均成功。 | 真机已验证 |
| 13 | `hdc shell 'hilog -x -T GPUI_M3_3 | tail -60'`，核对原始 Scene 与 Flush。 | 设备 `ALN-AL00`、OpenHarmony `7.0.0.105` / API 26、aarch64；本次窗口 `1260×1918` 物理像素。原始 Scene 为 `quads (4), monochrome sprites (19) on atlas 0`，Native Drawing Flush 记录 `quads=4 monochrome_glyphs=19`，无文字或绘制错误。 | 真机已通过 |
| 14 | `hdc shell snapshot_display -f /data/local/tmp/gpui_m3_3.jpeg` 并取回临时目录检查画面。 | 画面可见 `GPUI Counter`、`0`、`−`、`+`、`Reset`；按钮文字位于各自形状内，标题、数字、按钮居中。截图只保存在本机临时目录，未写入仓库。 | 真机可见 |
| 15 | 对照 `gpui/desktop-counter/src/main.rs` 与手机根视图，检查布局。 | 两端使用相同字号、`gap_4`、按钮内边距、圆角与颜色；手机标题→数字→按钮纵向顺序一致。窗口/Surface 尺寸和系统字体不同，因此未做同分辨率像素差或精确字距一致性断言。 | 静态与画面比较完成 |
| 16 | 检查签名 HAP 清单及包内原生库 ELF 头和动态依赖。 | HAP 含 `libs/arm64-v8a/libgpui_mobile_counter.so`；库为 ELF64 AArch64，动态依赖包括系统 `libnative_drawing.so`、`libace_napi.z.so` 等，无新增第三方平台库。 | 已通过 |
| 17 | 同步更新任务状态、需求状态与手机工程 README。 | M3-3 结果和限制一致；M4-1 触摸、M4-2 帧调度仍未验证。 | 文档已核对 |
| 18 | 执行 `git diff --check`、检索当前状态措辞与 `git status --short`。 | Diff 无空白错误；检索到的 `NoopTextSystem` / M3-2 未完成措辞仅在历史记录中；已有 M3-2 与其他未提交改动均保留。本次未创建提交、未推送。 | 已通过 |

## 实现边界

本阶段仅面向计数器的拉丁字母、数字与 U+2212。字形映射、度量和光栅化由系统 Native Drawing 完成，GPUI 负责文本布局与 Scene Sprite，现有自有 Atlas 保存单通道掩码。`font_id` 当前统一指向系统默认字型；逐字符布局不做字距调整或复杂文本 shaping。系统字体指标失败时使用保守的 1000 单位后备值；复杂文字、字体回退、外部字体和文本输入没有在本阶段验证。

参考接口：[HarmonyOS Native Drawing 字体管理官方文档](https://developer.huawei.com/consumer/cn/doc/doccenter-capabilities/api/capi-drawing-font-mgr-h)；实际 C ABI 以安装的 API 26 SDK 头文件为准。
