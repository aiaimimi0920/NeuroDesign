# 维护记录 / 历史调查与预览验证

本文件仅保留维护过程与验证记录，不是主题规范，也不是下游项目实现 UI 时的输入。主题规则以各自的 `DESIGN.md` 或 `DESIGN.zh-CN.md` 为准；历史来源、示例布局与审批过程不构成额外要求。已删除的第三方主题不恢复，以下旧调查名称与行号仅用于追溯。

This note records evidence and adaptations. The independently reusable specifications are
[neuro](../neuro/DESIGN.md) and [neuro-mini](../neuro-mini/DESIGN.md), each with a full Chinese edition.

## 1. 调查日期与来源优先级

调查日期：**2026-11-03**。

| 来源 | 状态 | 用途 |
|---|---|---|
| Gateway | main；HEAD `7969f12c85efceacf2a5e384e143b5c39a550981`；调查开始时 415 个变更/未跟踪条目 | 通用 neuro 的主要实现依据 |
| Loom | main；HEAD `9546704de3ed2c99f89523d6fcc23a7560dcf520`；调查开始时 150 个变更/未跟踪条目 | 补充共享配色、控件状态与可访问性 |
| Otter 独立 main 副本 | `2dfc73d107429c38e25a0c9e0a9fd062d99ab306`，2026-10-16，干净；调查时与远程 main 一致 | neuro-mini 的稳定实现依据 |
| Neuro 既有提示词与设计资料 | 下列共享设计目录 | 意图参考；与当前实现冲突时不覆盖实现 |

Gateway、Loom 读取的是**实际工作树**。上面的 HEAD 不是全部样式的精确源版本，也不代表这些改动已经发布。

来源工作区未被修改、切分支、重置、安装依赖或启动。Otter 的 `feat/windows-native` 及其未提交改动没有用作 mini 视觉定稿来源。

## 2. 本地来源位置

- Gateway：`C:/Users/Public/nas_home/AI/GameEditor/Neuro/Gateway`
- Loom：`C:/Users/Public/nas_home/AI/GameEditor/Neuro/Loom`
- Otter 独立调查副本：`C:/Users/Public/nas_home/AI/GameEditor/linshi/neuro-design-baselines-20261103-183839/otter-main`
- 历史设计参考：`C:/Users/Public/nas_home/AI/GameEditor/Neuro/docs/UI设计与颜色方案`

绝对路径用于本机追溯，不是复用 Markdown 规范或运行网页的依赖。

## 3. 关键实现证据

### Gateway — primary

以下路径相对于 Gateway 根目录。

| 文件及调查时行号 | 支持的结论 |
|---|---|
| `apps/desktop/src/main.tsx:15–45`；`styles.css:1–38` | 真实样式入口与加载顺序；末尾 compact 覆盖不能忽略 |
| `apps/desktop/src/styles/foundation.css:7–64` | 共享配色、语义别名、4/8px 圆角、状态派生、阴影 |
| `apps/desktop/src/styles/foundation.css:74–101` | 浅色不是纯白反相；强调文字需加深 |
| `apps/desktop/src/features/shell/AppShell.tsx:201–235` | 实际挂载 compact 壳层 |
| `apps/desktop/src/styles/compact-shell.css:2–6,58–78,104–109` | 212px rail、38px 顶栏、30px 导航项、46px 主板标题区 |
| `apps/desktop/src/styles/workspace-controls.css:157–205,433–462` | 36px 按钮、12px 水平 padding；常规输入 10/12px padding |
| `apps/desktop/src/styles/theme-overrides.css:203–264` | 最终 primary/secondary/danger/disabled 与 dialog 表面 |
| `apps/desktop/src/styles/compact-settings.css:16–35,63–65` | 设置是 28px 的局部紧凑体系，不是全局所有操作尺寸 |
| `apps/desktop/src/styles/dialogs.css:2–44` | 24px 弹窗 padding、16px gap，仍有局部背景 blur |
| `apps/desktop/src/features/settings/displayPreferences.ts:5–19,42–48` | 实际字体默认值与用户字体/缩放能力 |
| `apps/desktop/src/styles/theme-overrides.css:159–162,328–344` | 反色面实际用于开关滑块与 tooltip，而非通用白卡 |
| `apps/desktop/src/features/console/ProviderBrandIcon.css:1–4` | 商标底板的近白色属于可读性特例 |
| `apps/desktop/src/styles/theme-overrides.css:15–37` | board/rail 直角，无悬浮阴影，board 装饰伪元素取消 |

其中 `styles.css` 的完整路径为 `apps/desktop/src/styles.css`。

### Loom — supplementary

以下路径相对于 Loom 根目录。

| 文件及调查时行号 | 支持的结论 |
|---|---|
| `apps/desktop/src/main.tsx:1–15`；`styles.css:2–26` | 真实入口及后置覆盖层 |
| `apps/desktop/src/styles/foundation.css:3–52,137–179` | 与 Gateway 共享主要色值，存在 light/system 模式 |
| `apps/desktop/src/styles/theme-shell-art.css:35–101,133–149` | 名为 glass 的卡实际是实色；输入、只读、禁用状态明确 |
| `apps/desktop/src/styles/accessibility.css:3–35` | 2px focus、reduced motion、forced colors |
| `apps/desktop/src/styles/shell.css:3–18,248–258` | 较早的 186px rail、50px 顶栏和 8px 导航圆角；不覆盖更新 Gateway |
| `apps/desktop/src/styles/theme-shell-art.css:158–170` | 部分 info toast 实际采用绿色，是局部差异 |
| `apps/desktop/src/styles/theme-otter-settings.css:83–85` | 某些画布明确固定 dark，不等于周边控件不支持浅色 |

Loom 没有查到通用白色 focus 卡消费者；不能仅凭 token 的存在宣称效果已经落地。

### Otter main — compact baseline

以下路径相对于独立 main 副本根目录。

| 文件及调查时行号 | 支持的结论 |
|---|---|
| `src/app.css:1–9` | 实际基础主题、primitive、功能样式加载 |
| `src/shared/styles/theme-foundation.css:4–24,26–56` | 与 Neuro 相同主色、Otter 语义映射与零 blur |
| `src/shared/styles/theme-foundation.css:79–108` | 150/200/300ms、四个 0px 圆角 token、light 配色 |
| `src/shared/styles/terminal-primitives.css:28–96,199–234` | primitive 直角、focus、disabled、切换与输入外观 |
| `src/features/stickers/components/stickerTopStripPropertyBarFields.tsx:264–307` | 24px 小工具、42×24px 开关及 14px 方形滑块 |
| `src/features/stickers/styles/sticker-edit-strip.css:2–25,100–145` | 9px 外壳、5px 按钮、4px 菜单、20px 菜单项的局部例外 |
| `src/features/settings/styles/settings-dialog.css:14–48,64–101` | 阅读/设置面保留更宽松 padding，输入至少 36px |

## 4. 明确作出的取舍

1. **实际实现优先于旧提示词。** 不强制大白色焦点卡；不将所有装饰图形铺成背景。
2. **通用 Neuro 也紧凑。** 不用 Loom 较早的大标题和间距去覆盖 Gateway 的新壳层。
3. **Mini 不等于所有地方零圆角。** 默认直角，但明确保留 Otter 悬浮编辑条变体。
4. **不将所有例外写成主规则。** 胶囊开关、局部 blur、品牌图标、不同业务表格有自己的上下文。
5. **区分源实现和网页适配。** 预览的普通文字比例、素材表行高、rail/inspector/frame 尺寸、响应断点、通用 tablist 属于提炼后的演示组织，不假称它们全部是现成产品 token。
6. **调色模式不扩成主题。** dark/light 是每套风格下的配色模式，仍只有 neuro 和 neuro-mini 两套设计。
7. **不使用真实业务数据。** 项目、素材、体积、进度和容量都是夹具。不存在已接通的上传、同步或账号服务。
8. **用户确认的文案修订优先。** 删除装饰性英文眉题、全大写微标签与口号副标题；默认不添加解释性文案，必要标签、可访问名称、错误、风险与非显然限制除外。此修订覆盖来源产品和旧提示词中仍存在的相关写法，不回改来源仓库。
9. **用户确认的选择控件模式。** 少量短选项的单选可用主题色分段 radio；选项较多、名称较长或放不下时优先单选下拉框。多选支持横/竖排复选框组与下拉复选框列表。选择只改变状态，不改变几何或推开列表；这些网页补充模式不假称来源产品已有同款导出组件。

## 5. 演示结构与离线约定

- `neuro.html` 是总入口。
- 每套主题包含 `DESIGN.md`、`DESIGN.zh-CN.md`、`index.html`、`example.html`、`theme.css`。
- 本目录 `_neuro` 保存共享的中性色/控件、控件陈列和同一工作台夹具。
- 两套主题由各自 `theme.css` 与明确的结构变体控制，不是分别复制后独立漂移的两份应用。
- 无 framework runtime、模块加载器、fetch、远程字体、CDN、构建与后端依赖，支持直接打开 `file://` 页面。
- 项目/素材新增、重命名、收藏、归档、审阅与备注仅改变标签页内存；刷新重置。
- 导出功能只下载明确标记为 demo 的 JSON 元数据，不读取素材文件。
- 复制网页预览时保留 `_neuro` 与相对目录结构；只复用设计指导时可单独复制任意一份 DESIGN 文档。

## 6. 证据边界

来源产品采用静态源码核查，未启动其日常实例、未读取真实账号、未用其发布包作本次视觉验收。
新建预览的浏览器检查不能代替来源产品的原生验收，也不能证明目前重构中的 Otter UI 稳定。

The source products were inspected statically. Browser validation applies to these new offline examples only, not to released Gateway/Loom/Otter binaries.

## 7. 首版预览验证

验证日期：**2026-11-03**。环境：Windows、Node `v22.22.2`、专用无持久用户配置的 Chrome `156.0.8078.80` 会话。

- **静态检查 8/8 通过**：UTF-8 无 BOM、本地链接、双语规范的关键契约/色值/章节对齐、脚本语法、本地依赖、配色和密度差异。复验命令：在仓库根目录运行 `node --test tests/neuro-preview.test.cjs`。
- **浏览器断言 163 项通过**（主流程 143 项、补充布局 20 项）：直接通过 `file://` 打开两套控件页及网站页，并将浏览器设为离线；主流程无页面脚本错误、无外部 HTTP(S) 请求。
- **布局范围**：1440 × 1000 深/浅色，375 × 812 深/浅色，另检查 320px 宽浅色页面；没有文档级横向溢出。窄屏菜单与弹窗边界、背景滚动锁定及外部交互关闭均已检查。
- **交互范围**：选择、开关、键盘滑杆、方向键/Home/End 标签页、菜单、弹窗校验/Escape/焦点恢复、减少动效、移动导航、搜索/组合筛选/空态、列表/网格、重命名、收藏、审阅状态、备注、归档/恢复、新增、确认删除、多选及刷新重置。
- **文字与导出**：长名称可布局，用户输入的 HTML 作为字面文字显示；两个实际下载的 JSON 均解析成功，包含 `demo: true`、对应主题和 8 条示例元数据，没有素材文件内容。
- **视觉检查**：查看了两套完整控件页、工作台及代表性的浅色/窄屏/弹窗截图。相同 1440px 视口下，通用工作台宽 1440px，mini 外框宽 1160px；差异来自布局与密度，而非全局缩放。

验证中修复了通知层遮挡操作，以及审阅状态修改后详情标签未同步的问题；更新详情标签不会清空未保存的备注。原生 Chrome 在模态 Tab 循环边界可将焦点交给浏览器工具栏，背景页面控件仍不可获得焦点；没有为此添加与浏览器重叠的手写焦点陷阱。

一次性浏览器脚本、日志、截图及下载样本保存在本机 `C:/Users/Public/nas_home/AI/GameEditor/linshi/neuro-preview-validation` 及其相邻调查脚本中，不是网页运行依赖，也没有为此给仓库安装浏览器框架。静态测试不是逐句翻译校验；上述浏览器结果也不是跨浏览器、屏幕阅读器或来源产品原生验收的证明。

## 8. 文案规则修订与针对性验证

用户在 **2026-11-03** 确认：不要装饰性英文标题；默认不添加非必要的解释型文本。本次同步更新两套中英文规范、实现提示词、验收清单和网页预览，移除相关排版规则，而不只是替换两句示例字符串。

工作台删除装饰眉题、理念副标题、重复说明与不再需要的顶栏；移动导航按钮并入任务标题区。控件页及总入口也移除口号、全大写栏目样式和多余说明，保留控件名称、必要规格、错误、破坏性确认与非显然限制。演示边界集中为一处提示，窄屏也可见。弹窗说明变为按需提供；省略说明时不生成空段落或悬空的 `aria-describedby`。

本次静态检查 **9/9 通过**；针对上述改动的 Chrome 离线浏览器断言 **79 项通过**，包括双主题控件页/网站页在 1440、900、375、320px 宽度的布局、文案删除、必要提示保留、移动导航、无说明/有说明弹窗、校验、焦点恢复及备注/状态反馈。无页面脚本错误和外部请求，代表性深/浅色和窄屏截图已查看。首版的 163 项断言是上一节记录的历史验证，不重复计入本次结果。

本次一次性脚本为 `C:/Users/Public/nas_home/AI/GameEditor/linshi/verify-neuro-copy.js`，日志及截图在 `C:/Users/Public/nas_home/AI/GameEditor/linshi/neuro-copy-validation`。没有修改来源产品，也没有提交或推送 Git。

## 9. 选择稳定性修复与控件补充

用户在 **2026-11-03** 要求修复勾选引起的重排，并补充分段单选、横/竖排多选和下拉多选。本次同步更新两套中英文规范、实现提示词、验收清单和共享控件页，不改来源项目。

### 根因与修复

- 控件页选中行把首列左 padding 从 0 改为 8px，导致浏览器重新分配自动表格列宽。修复前 1440px 的 neuro 首列宽度在 `272.921875` / `275.734375`px 间变化；修复后两种状态均为 `281.3125`px，checkbox 左偏移均为 8px。现在各状态 padding 一致，选中线仅使用 inset box-shadow。
- 工作台每次勾选调用完整列表渲染，并插入批量操作条，导致列表下移 56px、原表节点被替换。现在只原地更新 checkbox、行状态、数量和全选混合态；批量操作与标题共用同一已占位区域，验证位移为 0px，原表节点保留。清空选择后，列表焦点回到全选，网格焦点回到当前视图按钮。
- 新分段控件使用原生 radio，固定边框、padding 和字重，以主题信号黄派生底/边线与内侧标记表达选择。多选保留原生 checkbox；下拉勾选不自动关闭，数量摘要不扩展高度，面板浮层独立滚动并按可用空间上下放置。neuro 保留 4px 控件圆角，mini 保持直角。

### 本次验证

- 永久回归检查 **12/12 通过**，在原有检查上增加选择态几何约束、原地选择更新行为，以及新控件语义和四份规范的契约检查。命令仍为 `node --test tests/neuro-preview.test.cjs`。
- Chrome 离线浏览器主检查 **882 项断言通过**：两套主题 × 深/浅模式 × 1440/375/320px 宽度 × 控件页/网站页，共 24 个场景。各场景逐行勾选、全选、取消、清空均保持表格/单元格/复选框几何和节点身份不变，焦点保留，全选混合态正确。
- 新控件覆盖鼠标、原生 radio 方向键与单个 Tab 入口、复选框 Tab/Space、0/多项选择、单选下拉框、下拉多选数量更新/连续勾选/Escape 焦点恢复/外部关闭/上翻。另在 320×260 短视口检查面板内部滚动。搜索、列表/网格切换、跨项目选择清理、批量删除取消/确认、长名称、普通菜单和标签页通过相关回归；主检查无页面脚本错误和外部 HTTP(S) 请求。
- 最后补充 **16 项断言通过**，检查两主题下拉向下展开、触发器几何稳定，以及列表/网格清空后的 DOM、位置和焦点。共计 898 项本次浏览器断言，不包含上两节历史结果。
- 已查看两主题新增选择控件的深色、代表性浅色与 320px 窄屏截图，以及两主题展开下拉多选的截图。没有文档级横向溢出。验证仍限于本机 Chrome 的离线演示，不是跨浏览器、屏幕阅读器或来源产品原生验收。

一次性脚本为 `C:/Users/Public/nas_home/AI/GameEditor/linshi/verify-neuro-selection.js` 和 `verify-neuro-selection-final.js`，截图及结果摘要在 `C:/Users/Public/nas_home/AI/GameEditor/linshi/neuro-selection-validation`。未增加浏览器框架依赖，未提交或推送 Git。

## 10. 人工批准的设计借鉴补充（仅文档）

用户于 **2026-11-03** 明确批准将候选 **R01–R06** 纳入两套设计文档，**R07 命令搜索、R08 资源获取行不纳入**。本轮同步更新 neuro / neuro-mini 的中英文规范、各自实现提示词及验收清单，不修改控件页、网站示例、主题、脚本或来源产品。

| 已批准补充 | 本地参考与调查时行号 | 纳入后的规则与边界 |
|---|---|---|
| R01 技术值就地复制 | Mintlify（历史调查稿，已移除）:713–720；OpenCode（历史调查稿，已移除）:407–409 | 按需低强调复制，真实就地反馈、稳定操作位，保留权限、键盘及宿主剪贴板验证；不是所有文字都带复制按钮 |
| R02 紧凑数值步进器 | Starbucks（历史调查稿，已移除）:323–327 | 减/值/加组合，可按需直接输入并合组单位；范围、步长、精度、空值、边界和宽度有明确语义。直接输入及单位组合属于获批适配，不假称原稿已有；neuro 保持既有小圆角，mini 保持直角 |
| R03 长表格表头 | Airtable（历史调查稿，已移除）:530 | 真实长表比较时在局部滚动中保留列标题；短列表不增加无谓外壳，不顺带要求冻结首列 |
| R04 内容预览保真 | Cal（历史调查稿，已移除）:519–522 | 检查预览保持比例与关键内容，区分缩略图裁切；不染色/扭曲等规则属于面向截图工具的获批适配，主题作用于外壳 |
| R05 数据变化语义 | Binance（历史调查稿，已移除）:529、625 | 区分方向与成功/失败；给出单位和比较基准，不照搬交易涨跌色或新增竞争主色 |
| R06 状态配方 | Resend（历史调查稿，已移除）:573–576；Raycast（历史调查稿，已移除）:535–538 | 以实际控件和获批目标整理六类控件状态矩阵，细化已有交互契约；不照搬来源“不记录 hover”的政策，不新建组件框架或 token 平台 |

外部参考仅为当前仓库中的第三方网站分析稿，本轮未联网核实其现行官方设计；不能把营销页面片段提升为完整产品交互证据。Neuro 的实现来源优先级、既有配色/几何、无装饰性标题、无非必要解释文案及选择几何稳定规则均不变。

**文档批准不等于运行实现。** 本节记录的是仅文档阶段，当时尚未补入离线预览；随后按用户“继续修改控件和网站示例”的要求完成的实现与验证见第 11 节。上节 898 项断言属于此前的选择控件修订，不用于证明新增配方已经可用，不因为新增验收项就自动扩展来源产品功能。

原始调查和人工决定保存在本机 `C:/Users/Public/nas_home/AI/GameEditor/linshi/neuro-design-reference-review-20261103.md`，用于追溯，不是独立规范的运行或阅读依赖。

## 11. 已批准配方的离线实现（2026-11-03）

按用户后续要求，将 R01–R06 实际应用于两主题共用的控件页与网站工作台，不加入 R07、R08，不修改 Gateway、Loom、Otter 源码，不引入框架、依赖、后端或持久化。

| 配方 | 控件页 | 网站示例与具体边界 |
|---|---|---|
| R01 就地复制 | Project ID 旁的低强调复制操作 | Details 中复制完整、已显示的非秘密 Asset ID；等待真实 Clipboard API 结果，成功原位勾号及可访问通告，不弹成功 toast；拒绝或 API 不可用时原位警告及短错误浮层，保留可选取值和焦点，不推动布局 |
| R02 数值步进器 | Columns 1–12 / 1；Stroke width 0.25–4 px / 0.25；可直接输入、使用原生方向键及增减按钮 | 图像 Zoom 1–200% / 1；越界提交夹到边界，非整步值吸附，空/无效值恢复上次提交值；仅禁用到达边界的对应按钮，位数和反馈变化不改变宽度 |
| R03 长表表头 | 16 行修订夹具，局部纵向滚动；窄宽允许表内横向滚动 | 20 个活动资源及 1 个归档资源的元数据夹具，列表独立滚动、表头不透明且列对齐；窄宽隐藏非关键列，滚动区域有可访问名称和键盘入口；不冻结首列，不给三行样本额外套长表容器 |
| R04 内容保真 | 随附 640 × 400 SVG 样本，四角编号与边界可检查，Fit / Zoom 可操作 | 初始 interface-kit.svg 使用同一样本检查器；缩放按原始像素尺寸，可滚动查看边缘；Fit 还原完整画面，显示取整后的有效百分比，窗口变化时重新适配；图像不染色、不拉伸、不旋转。只有该随附夹具有实际预览，其余条目明确显示不可预览，不假装解码 FIG、PNG 等文件 |
| R05 数据变化 | 存储 +2 GB / 延迟 −12 ms 均用中性色并保留比较基准 | 导航底部 Sample storage 的 +2 GB vs 26 GB 为中性、静态夹具，不冒充真实存储统计，不与绿/红结果状态混同 |
| R06 状态配方 | 可实际触发的 sample 加载/完成、无效/只读/禁用输入、选中/禁用选择项、菜单禁用项及既有焦点状态 | 延续原地勾选更新和稳定批量操作位；缩放/复制不重建详情表单，修改 review status 不丢未保存 Notes；不为凑状态矩阵虚构业务加载能力 |

实现位于 `common.js`、`gallery.js`、`workbench.js` 及对应共享 CSS，样本图为 [samples/interface-kit.svg](samples/interface-kit.svg)。两个主题仍从各自 `theme.css` 获取既有密度及几何，mini 保持直角；四份中英文规范已同步实际能力与数值行为。

### 本轮验证

- `node --test tests/neuro-preview.test.cjs`：**15/15 通过**。新增检查覆盖数值夹取/吸附/空值/小数精度、真实异步复制结果与拒绝/不可用路径、样本资源与滚动表头契约；保留选择不重建 DOM、编码、链接、主题及中英对齐检查。
- Chrome 直接以 `file://` 打开，不启服务器。两主题 × 深浅色 × 1440 / 375 / 320px × 控件页/网站，共 **24 场景、922 项断言通过**；减少动效开启。覆盖真实剪贴板写入/读回、错误反馈、数值边界与宽度、固定表头、预览比例/边缘/自然像素缩放、选择 DOM 身份/焦点、搜索/空态/视图切换、重命名/删除取消、Notes 保留和窄屏导航。该轮无页面异常、无外部页面请求。
- 补充 **64 项断言通过**，恢复普通动效设置：深浅色下检查禁用但已选中的值仍可辨、数字无效与键盘焦点并存、不完整指数输入恢复、横向滚动列对齐、列表键盘滚动、缩放不丢 Notes，以及窗口缩小时 Fit 重算。
- 剪贴板另外在两主题深浅色共 4 次清除测试权限覆盖后，以键盘 Enter 触发 `file://` 真实写入，均成功；写入后授予读取权限再读回验证。拒绝及 API 不可用路径以显式注入验证。此结果仅证明本机 Chrome 下这些路径，不声称所有浏览器/宿主的权限策略相同。
- 浏览器截图经过实际查看；`git diff --check` 及新增交付文件空白检查通过。全部修改文本为 UTF-8 无 BOM，未提交或推送。

一次性浏览器脚本：`C:/Users/Public/nas_home/AI/GameEditor/linshi/verify-neuro-r01-r06.js`、`verify-neuro-r01-r06-final.js`；本轮截图以该目录下 `neuro-r01-r06-` 为前缀。上述 922 + 64 项是本轮新增实现的验证，不复用此前 898 项或文档阶段 101 项作为运行证据。样本是离线演示，不代表来源产品的新功能或跨浏览器认证。

## 12. 主题规范独立化（2026-11-04）

按用户要求，四份中英文设计文档改为独立的视觉与交互规范，移除产品定位、来源优先级、历史调查、审批过程、跨主题引用及预览实现报告。保留配色、排版、几何、控件配方和已确认的交互规则；页面结构按目标内容决定，不强制复制示例业务或布局。

预览操作方法、数值夹具与能力限制移至各主题的 `README.md`。仓库首页、维护约定和反馈模板改为自用主题仓库的说明，预览入口与页尾不再展示来源产品、提交号或跨主题比较。用户删除的第三方主题保持删除；旧调查引用改为历史文字，不保留失效文件链接。本文件仅留作维护记录，不是下游设计输入。

本次静态与契约检查 **18/18 通过**，包括新增的规范独立性、产品中立性、无来源/跨主题依赖、中英关键规则对齐与入口检查。Chrome 直接打开 `file://`，在 1440 / 375px 检查入口、两主题控件页和网站示例，**64 项针对性断言通过**；链接、明暗切换、勾选 DOM 身份和焦点正常，无页面异常、无外部页面请求。截图在本机查看；一次性脚本为 `C:/Users/Public/nas_home/AI/GameEditor/linshi/verify-neuro-standalone.js`。本次未修改控件业务逻辑，不以此前的全面验证冒充新的完整回归。
