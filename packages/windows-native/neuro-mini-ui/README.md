# neuro-mini-ui

`neuro-mini` 的 Rust/Win32 渐进实现：悬浮图标工具条、分组分隔、独立右下角展开按钮、长按展开、tooltip、20 DIP 行高的短菜单，以及附属二级属性行。真实预览见 [原生 Gallery](../../../apps/windows-native/neuro-mini-gallery/README.md)。不依赖 Node、React 或 WebView。

当前实现覆盖深色悬浮工具条、属性行及附着颜色面板，不代表全部主题控件、浅色表单或应用业务已迁移。依赖精确版本 `windows 0.60.0`、`svgtypes 0.16.1`、`roxmltree 0.21.1`；无应用 API、网络或文件读取。

## 独立验证

在 Windows MSVC 开发环境中：

```powershell
cargo fmt --manifest-path packages/windows-native/neuro-mini-ui/Cargo.toml -- --check
cargo test --manifest-path packages/windows-native/neuro-mini-ui/Cargo.toml --locked --offline -- --test-threads=1
cargo clippy --manifest-path packages/windows-native/neuro-mini-ui/Cargo.toml --locked --offline --all-targets -- -D warnings
```

测试创建自有 HWND、内存图像和短菜单，不截取用户桌面、不注册全局输入。CI/无人值守运行应使用独立 desktop；普通可见键盘、DPI 与观感仍需在 Gallery 和实际消费应用中验收。

## 应用消费与所有权

公开 `paint_batch::{Hold, present, control, window}` 供已有原生 owner-draw 控件复用完整帧发布；`prepare_button` 安装标准 BUTTON 的擦底保护。所有调用限定创建 HWND 的 UI 线程，`Hold` 存活时不显示/隐藏该根窗口；最终显隐在 guard 外完成。接口不拥有业务、数据、窗口或异步任务。

`DrawerLayout::apply_region(hwnd, expanded, dpi)` 同时供 Drawer root 和应用顶层宿主使用：收起裁剪到居中的三角，三角外不保留黑色矩形或命中区；展开恢复布局矩形。展开与收起保留相同宽度和水平 caret 槽，避免宿主重新居中引起横向中间位置。组件更新冻结旧帧至控件、裁剪和几何完成，不变投影不重绘，宿主仍负责最终位置及显隐。

`PropertyPanel::update_sections(context, sections)` 在同一面板内排列最多八个连续分组、合计最多 32 个字段；分组强制换行并使用单条分隔线，更新继续保留同身份的真实控件。`PropertyKind::Label` 提供不参与 Tab、不能提交的状态文字/图标（最多 512 UTF-16 单元），不是伪禁用按钮。纵向 Action 使用居中 18 DIP 纯图标和圆角底；窄计数按钮使用独立图标/数字区域。横向属性行仍保持原 16 字段预算。分组业务、页签选择与状态来源由宿主拥有。

`PropertyPanel::update_with_states(context, Vec<(PropertyField, bool)>)` 在同一冻结帧中更新值与各字段最终可用性，避免先全启用再局部禁用造成持续重绘；相同快照不触发布局/绘制，同目标保留真实 HWND、焦点、草稿和滚动。统一可用性仍可用 `update`。`anchor(id)` 返回字段的屏幕物理矩形，随局部滚动同步，字段移除或面板销毁后返回 None；弹层和业务目标复核归宿主。

`Drawer::new(parent, notify, buttons)` 提供下附着设置条的真实 `WS_CHILD`，不另建顶层。最多八个稳定 `DrawerButton`，各有独立图标、可访问名称、可选短文字、active／enabled；`update(title, buttons, expanded, context, dpi)` 原位更新，保留 HWND 与焦点，返回 `DrawerLayout`。ID 和文字 schema 创建后不变，context 必须非零；通知的 `wParam` 低 16 位是 ID、高 32 位是 context，`lParam` 是 Drawer root。`DRAWER_TOGGLE` 仅通知展开意图，宿主负责状态、凭证和业务提交；`translate` 支持标准 Tab／Space 与 Enter。`alive` 可用于宿主退休判断。Drawer 在 parent 前 Drop，禁止外改 userdata；parent 提前销毁时 lifetime 令 wrapper 失效，不触碰复用 HWND。

展开采用 52 DIP 高、28 DIP 按钮、20 DIP 图标、4 DIP 间隔与 11 DIP 名称，收起只保留居中 20×10 DIP caret；文字按钮预留 62 DIP，适合短动作词，不用于长文本。该实现使用稳定近黑不透明底，不依赖桌面透明能力。应用拥有顶层附着位置、层序、截图保护、显示条件及展开同步规则，不在主题库硬编码节点或“执行”等业务。名称宽度只在名称／DPI 变化时测量，稳定投影不继续重绘，无线程、动画或 timer；Gallery `drawer_demo` 直接消费公开 API。

`NoticeStack::new(owner)` 创建一个不激活、不改变宿主层序的附属裁剪视口；`push(title, message, dismiss)` 返回应用持有的 `Notice`，最新项置顶。右上内缩 8 DIP、最大宽 280 DIP、卡高至多 140 DIP，整个栈不超过宿主客户高度减 16 DIP；溢出栈与长卡分别支持即时像素滚轮、8 DIP 轨道、拖动 thumb 和方向/翻页/Home/End，内层到边界继续外层，但不把滚轮传给宿主。空隙透出图片并让出命中，标题、正文与“知道了”沿紧凑同排布局，不丢弃长文本。应用拥有通知条目、数量预算、scope、任务与失败语义，不进入主题库。

整卡真实按下/抬起、Enter/Space/Esc 或五秒超时关闭；拖动轨道不关闭，关闭/Drop 后立刻回收并重排，新项/关闭重置栈到顶。`window()` 在超时、手动或宿主销毁后同步失效，Drop 不触碰复用句柄。宿主的移动/尺寸/DPI/显隐由 UI-thread subclass 同步，隐藏释放自己的内容按下/轨道捕获；`place()` 同步后续捕获排除，只设置顶层视口的 display affinity，子控件继承保护。应用消息泵调用 `translate` 处理本栈 Tab 并滚入焦点；其它键仍正常派发。稳定布局不重新测量全文、不重复失效绘制，不新增轮询/动画/线程。Gallery 的“通知”按钮连续点击消费同一公开栈，非应用业务仿制。

`CapsuleLayout` 提供固定 28 DIP 行高、256 DIP 展开宽度、收起/展开的同一图片侧边缘与外侧连接锚点；Compact 保持 8 DIP，不随展开状态放大。`CapsulePainter` 消费调用方的静态 SVG、标签、可用性和焦点，收起仅绘外侧半图形，展开绘完整图形与圆角底板。调用方提供 DC 和底色并管理命中、层序、行距、滚动、字段控件与业务；组件不创建 HWND、捕获或定时器。绘图器及当前 DPI 的 11px 字体在 owner 释放时回收，Gallery 直接消费同一 API。

展开胶囊可传 `CapsuleAction`：输入名称从主体内缩 4 DIP 开始，16 DIP 动作槽位于 82 DIP；输出动作从对象侧内缩 18 DIP 开始，右对齐的 72 DIP 名称从 44 DIP 开始，不整体镜像输入内容。`inline_action(dpi)` 与 `label(dpi, has_action)` 给出绘制和命中共用的物理矩形。收起和 Compact 不绘制或提供动作命中。图标、active/pressed 状态与点击语义归调用方，不增加主题业务或独立 HWND。

`CapsulePainter::backdrop` 在调用方 DC 上只填连续展开组的圆角底板，保留重叠组透过圆角的已有像素，不清空整块矩形。调用方决定连续可见组、滚动和遮挡裁剪，并将背景放在无输入层；组件不扩展控件命中区，也不创建背景 HWND。

`PropertyPanel::new(toolbar, width)` 创建独立的纵向属性面板，复用属性行的真实 EDIT、草稿、tooltip、步进和资源 owner。Number/ReadOnly 占满内宽并右对齐，Action 按行排列；只读字段不提交，数字按范围设置输入长度。`update` 消费有界字段/context；宿主在 `size/place` 前用 `set_max_height` 传入工作区可用物理高度（至少容纳一行），溢出通过裁剪 viewport、8 DIP 轨道和像素滚轮局部滚动，不缩小控件或字号。键盘/程序焦点滚入整行，数字方向键仍步进；拖动轨道或滚轮不提交值。同 schema 保留 HWND、焦点和滚动位置，context 换代丢弃草稿/旧提交但不因数值刷新卷回顶部。`size/place/show` 仍由应用控制附着和显隐，`take_edit/translate` 接入宿主消息泵，通知为 `PANEL/PANEL_CLOSE`。首次创建继承主条捕获排除，宿主通过 `window()` 同步后续保护；关闭只隐藏本面板，不关闭工具条。

`PropertyKind::Number::suffix` 为最多四个非控制字符的真实单位（无单位用空串）。单位以 10px 次级文字在 EDIT 与步进按钮之间独立显示，预留稳定宽度，不进入数字草稿或提交值；应用的字段名称应保留必要的单位语义。Gallery 的窄面板通过公开高度约束直接演示同一滚动组件，不仿制产品控件。局部滚动能力不代表应用完整长设置栈或业务已迁移。

`PropertyKind::Action` 同时提供图标与短文字时，文字从图标区右侧开始，不互相覆盖；无图标的反馈文字仍在原按钮内居中。Gallery 的“开关”示例直接消费同一布局，应用拥有具体文字与反馈期限。

通过 Cargo 精确版本和明确 path 引用本 crate，不复制源码或依赖 Gallery。`create` 创建隐藏的 owner-owned HWND；首次 `prepare` 或显示时才分配完整子控件、tooltip 和软件 D2D 绘图器。失败回滚；重复准备保持句柄与焦点。全部 API 仅在创建窗口的 UI 线程使用，禁止修改库窗口的 `GWLP_USERDATA`。

应用持有并销毁工具条 HWND，owner 销毁时 Win32 同步销毁附属窗口。组件通知宿主，`wParam` 是按钮 ID，`lParam`：0 主按钮、1 展开、2 请求关闭、3 完成首次准备、4 属性提交、5 独立面板提交、6 独立面板关闭。应用审核模态、会话、工作区和目标身份后执行命令或关闭；主题库不拥有产品任务和持久化。同步通知中不得销毁组件。

分体按钮主体短按只选择工具；右下角小三角或主体长按 450ms 展开。移动超过 6 DIP、失去捕获/焦点、禁用和销毁撤销长按；展开前交还系统 BUTTON 的捕获，且不补发普通点击。菜单只使用虚拟行焦点，不激活新窗口，允许同一 root-owner 家族中的前台窗口，外部应用仍拒绝。

`update` 独立投影选中、主按钮/展开可用性和真实可访问名称，变化不改变几何；未变化不重绘。`resize` 接收权威 DPI 和工作区宽度，窄屏换行，不缩小 28 DIP 命中区。宿主位置、层级、显隐以及工作区约束由应用管理。

`Item::Slot(id)` 声明可选文字工具槽，`set_text_slots` 一次投影当前槽内容、名称、分组箭头与真实状态。空槽不占宽；更换内容、显隐和 DPI 保留所有真实 BUTTON 及属性编辑器，不重建主条。文字使用 9px 等宽信号黄，工具仍为 28 DIP 命中区；可访问名称独立保留完整有界单行值。`TextSlot.context` 随异步通知放在 64 位 `wParam` 的高 32 位，低 16 位为原 id；应用必须复核自己的目标和目录凭证。组件不知道 slot 的业务命令、分组或权限。

`track_options` 与 `track_list` 共用有界长列表和固定可见 BUTTON 池，增加 `ListOption.enabled`。禁用行保留名称、次级文字且不可鼠标/键盘提交，Tab 跳过禁用项；全部禁用仍可取消和滚动。每个标签最多 512 UTF-16 单元，数量仍最多 16384，不截断条目或为每项创建 HWND。Gallery 的文字工具组通过这一公开 API 演示混合可用状态，应用负责列表内容及最终执行。

`track_menu` 只消费 1–12 项有界快照并返回 ID，主干小工具菜单无需级联。调用方先建立应用模态门禁，不跨调用借用模型，返回后复核原目标。菜单不抢前台、不接管已有捕获；Esc/外部点击/失焦/owner 移动关闭，返回前释放窗口、字体、捕获和计时器。空间不足明确报错，不静默丢选项。长设置/业务菜单仍由应用现有设施管理。

`set_properties` 接收最多 16 个字段及业务 context；Number/Text 使用真实 EDIT，支持提交、取消和步进，Action/Color 只通知宿主。主条上方优先零间隙附着，接触边为直边且不重复描边，外侧圆角保留；显隐/移动/销毁跟随主条。首次创建继承主条截图排除，后续宿主通过 `property_window` 一起更新。schema 换代保留同 id/类型的共用控件，相同 context 保留草稿；新增控件隐藏准备后发布，无变化刷新不重绘。`take_property_edit` 单槽取出字段提交，宿主仍须复核原目标。工具范围和文档历史不属于组件。

宿主有同批移动的图框时，可用 `attached_position(toolbar, proposed)` 只读取得二级行的目标 HWND/物理矩形，随主条放入带 `SWP_NOREDRAW` 的 `DeferWindowPos` 批次；主条不会再逐级追随二级行。宿主提交后调用 `sync_attached` 同步接边和屏幕边界翻转，无变化不重新移动；不在移动中重复更新工具数据，不新增窗口、timer 或业务状态。

主条和属性条在离屏帧内完成底色、图标、GDI 文字及真实 child 树绘制后一次提交；属性切换在控件更新和最终定位期间保留旧画面，首次显示返回前完成绘制。标准 owner-draw BUTTON 在系统擦底阶段保留旧帧，直到完整自绘帧完成再提交，不以关闭捕获排除换取防闪烁，不增加过渡动画或异步重绘队列。真实窗口和内存像素回归不代替普通桌面观感验收。

`PropertyKind::Ratio` 将约束开关、宽/高两个真实 EDIT 和外侧步进组合为 103 DIP 字段，冒号不混入单个输入框。提交中的 `value` 为比例文本，`PropertyEdit.active` 为最终约束状态；宿主应原子应用两者，避免失焦提交与开关点击互相覆盖。普通字段的 `active` 为空。

`track_color` 提供 248 DIP 的 HSV、独立 alpha、6/8 位 HEX、百分比和八列局部滚动调色板。真实 EDIT 保留不完整草稿；只有“应用”返回 `Apply(Rgba)`，Esc/取消/外部关闭不提交颜色。添加和删除调色板独立保留在 `ColorResult`，调色板及去重删除记录各限 4096 项；宿主拥有跨窗口共享、删除默认值副作用和保存语义。小工作区滚动整个内容并滚入键盘焦点，不缩放控件。预览显式左键复制 HEX、右键复制 RGB/rgba；复制失败传回错误，不读剪贴板或桌面。吸管仅返回 `ScreenPick` 意图，必须由宿主在模态门释放后授权采样。面板继承 owner 的截图排除，关闭清理窗口、捕获和 timer 并按同前台家族恢复焦点。Gallery 的颜色字段调用同一公开 API，禁用屏幕采样。

`track_list` 消费 1–16384 个、每项至多 256 UTF-8 字节的标签快照并返回索引或取消。196 DIP 宽、至多 220 DIP 内容高、28 DIP 行、11px 文本、8 DIP 轨道和 18 DIP 最小 thumb；空间不足向上并约束在工作区，窄工作区只截断标签，不缩小字号。固定可见 BUTTON 池与裁剪 viewport 支持像素滚动，不为每项分配窗口。滚轮、轨道跳转、thumb 拖动、Tab 进入选项和滚动条均可用；方向/翻页/Home/End 只在滚动条焦点时滚动，不暗改选中值或增加首字母搜索。打开不强制滚入当前项；Esc、外部点击、失焦、失捕获或 owner 移动取消，关闭释放所有短暂资源并继承截图排除。`PropertyKind::Dropdown` 显示图标/10px 值/箭头，宿主通过 `set_property_expanded` 在模态前后投影展开状态。字体枚举、排序、缓存和业务提交不属于主题组件。

`Icon::svg` 直接消费静态 SVG，保留 path/rect/circle/ellipse 的逐元素填充、描边、透明度、圆角、rotate 和两段 dash；颜色支持 `none`、`currentColor` 和精确六位 `#RRGGBB`，子元素可以继承固定色或显式恢复调用方颜色。拒绝脚本、外部资源及未支持语法。`Icon::stroke` 保留纯路径入口；`validate` 可用软件 D2D 验证完整资源。颜色和几何固定遵循 [声明层](../../../design-md/neuro-mini/DESIGN.zh-CN.md)，不提供任意皮肤平台。
