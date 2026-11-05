# NeuroDesign

供内部程序开发使用的独立仓库，维护自己的设计规范和标准实现，不再作为上游的网站主题集合。

独立仓库：[aiaimimi0920/NeuroDesign](https://github.com/aiaimimi0920/NeuroDesign)。

一个主题分为两层：**声明层**定义技术无关的视觉与交互规则；**标准实现层**提供可直接引用的组件、UI 模板与真实预览。不同主题可以使用不同技术栈，各自独立构建和分发，不是一个 React 组件库中的多套皮肤。

## 主题与预览

**[打开主题预览入口](neuro.html)**。也可以直接选择下表中的文件：

| 主题 | 当前范围 | 设计规范 | 标准实现 |
|---|---|---|---|
| **neuro** | 声明层 + React 标准实现层 | [中文](design-md/neuro/DESIGN.zh-CN.md) · [English](design-md/neuro/DESIGN.md) | [@neuro/ui](packages/react/neuro-ui/README.md) · [控件浏览](design-md/neuro/index.html) · [网站示例](design-md/neuro/example.html) |
| **neuro-mini** | 仅声明层，标准实现暂不开发 | [中文](design-md/neuro-mini/DESIGN.zh-CN.md) · [English](design-md/neuro-mini/DESIGN.md) | 暂无；旧网页仅作为历史示意保留 |

## 在项目中使用

React 应用使用 [`@neuro/ui`](packages/react/neuro-ui/README.md) 提供的真实组件和可选模板，再按照声明层完成产品独有内容。完整接入步骤与迁移边界见 [INTEGRATION.md](docs/INTEGRATION.md)。

有匹配的标准实现时直接使用对应包；本仓库允许正式维护 React、Windows 原生等不同技术栈的实现。不在当前支持范围的技术栈，可结合 DESIGN 与标准预览自行实现，由下游维护；不要求接受 React 接口，也不自动生成适配器。

两种情况下都保留现有业务字段、权限、流程和功能范围，不要求产品复制网站示例的业务或布局。

例如：

> 按项目中的 DESIGN.md 应用主题。保留当前产品定义和功能，只调整视觉与必要的交互表达。根据实际内容选择布局与控件，不添加示例中的业务模块。

每份设计规范自包含，不依赖其他主题、来源项目或维护历史；两种语言表达相同规则。技术安装说明与组件接口放在实现层文档，不混入 DESIGN。

## 代码组织

```text
design-md/<theme>/               技术无关的中英文声明
packages/react/neuro-ui/          @neuro/ui：独立 React 标准实现
apps/react/neuro-preview/         通过真实包入口运行的网页预览
packages/windows-native/         原生实现组织约定（暂未实现组件）
scripts/build.mjs                仅管理 React 构建与离线预览
scripts/check-boundaries.mjs     React 主题、共享包和预览的依赖边界
docs/                            两层边界与应用接入说明
```

详见 [ARCHITECTURE.md](docs/ARCHITECTURE.md)。主题包只依赖自身与必要的同栈共享包，shared 不反向依赖主题；不提供顺带导入全部主题的总包。根 npm workspace 仅管理 React，原生实现将独立使用 Cargo，不要求 Node 或 WebView。原 `_neuro` 资料仅供历史参考。

## 打开离线预览

用现代浏览器直接打开 `neuro.html` 或 neuro 的两个 HTML 文件，无需安装依赖、启动服务器、账号、CDN 或网络字体。深浅色可在预览页切换。仓库保留构建后的脚本和样式；修改源码后执行构建即可更新它们。

预览使用样本数据，增删改仅在当前标签页有效，刷新重置，不接入真实账号、文件或后端。数值范围、样本数量、网站业务和窗口构图都只是演示选择，不是主题的通用要求。

neuro 预览只需本主题目录中的 HTML、`preview.js`、`preview.css`、版权文件与 `sample.svg`。控件页和网站示例都从已构建的 `@neuro/ui` 包生成，不另维护一套展示用控件。具体演示边界见 [neuro 说明](design-md/neuro/README.md)。

## 维护与检查

使用 Node 22+、npm 11+，在仓库根目录执行：

```powershell
npm ci
npm run check
npm run build -- --theme neuro
npm run pack:ui
```

如果系统 npm 为 10，可使用 `npx --yes npm@11 ci` 安装。`check` 执行依赖边界、库/预览构建、类型检查、文档契约与组件交互测试；浏览器另验布局与实际键盘行为。指定 `--theme` 只构建该 React 主题及真实共享依赖，未知主题明确失败。`pack:ui` 仅打包当前 `@neuro/ui`，不是全部主题合集。新增原生实现不进入 npm 构建链。

只维护当前接口，内部引用方统一跟进，不添加历史接口兼容层；配色固定，不建设任意换肤平台。修改声明时同步中英文，修改实现时同步真实预览和实际调用点。维护约定见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可

保留 [LICENSE](LICENSE) 中的 MIT 许可与版权声明。
