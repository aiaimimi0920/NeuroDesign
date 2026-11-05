# 内部设计系统仓库

本仓库独立维护，不再是外部网站主题集合。统一维护入口不代表统一所有主题的技术栈。

## 两层边界

- `design-md/<theme>/DESIGN.md` 与 `DESIGN.zh-CN.md` 是技术无关、自包含的声明层。保持中英规则一致，不加入来源历史、应用定位、框架接口或另一主题。
- 标准实现位于 `packages/<stack>/<package>`，预览源码位于 `apps/<stack>/<preview>`。每个实现独立发布、依赖与构建；一个主题不必实现所有技术栈。
- `packages/react/neuro-ui` 是 neuro 的 React 标准实现，包名仍为 `@neuro/ui`；`apps/react/neuro-preview` 消费其已构建的公开入口，不另写展示控件。
- `packages/windows-native` 为未来 Windows 原生实现的组织位置。原生 crate 使用 Cargo 与自身验证入口，不依赖 Node、React、WebView 或根 npm 构建。
- neuro-mini 当前只有声明层；旧网页和 `_neuro` 资源冻结为历史示意。不创建空 crate，不抽取尚未稳定的原生控件。
- 本仓库可以正式维护不同技术栈的标准实现；不在正式支持范围的下游自制实现仍由对应项目维护。不建设跨语言统一运行时或自动适配器。

## 包依赖与实现约定

- React 包通过 `package.json` 的 `designSystem` 声明 `kind`（theme/shared/preview）；主题和预览同时声明 `theme`。预览显式声明 `library` 和对应依赖。
- 主题只能依赖自己的实现与同栈 shared 包；shared 不依赖任何主题；实现不依赖预览。禁止跨主题继承、源码相对路径穿透、总包顺带导入全部主题。
- 没有真实共用需求，不建立共享包。共享包独立维护必要局部样式，不把某一主题的整份 CSS 当作公共底座。
- npm workspace 仅包含 `packages/react/*` 与 `apps/react/*`。根包只提供开发工具，不提供运行时组件入口。其他技术栈使用自己的包管理与门禁。
- 只包含通用表现层，不导入应用 API、IPC、业务状态或任务生命周期。引用方统一跟进，不增加历史接口兼容层、任意换肤或跨框架适配。
- 固定主题配色与深浅色预设；状态不只依赖颜色。选择不改变几何、不重建整表、不丢焦点。不加装饰性英文眉题、口号或非必要解释文案。
- Neuro React 样式限定于 `.neuro-root` 和 `n-` 前缀；弹层继承所属 Provider 的主题。React 由宿主通过 peer dependency 提供。
- 此次架构路径调整可以同步已接入应用的依赖和接入测试，不擅自修改其业务代码。
- 保留已有用户改动，不恢复已删除的第三方主题，不擅自提交、推送或改写 remote。文本 UTF-8 无 BOM；命令显式使用本地 C 盘工作目录。

## 验证

React 开发使用 Node 22+、npm 11+；npm 10 环境使用 `npx --yes npm@11`，不绕过 peer dependency 解析。

```powershell
npm run check
npm run build -- --theme neuro
```

`check:boundaries` 检查 React 清单、源码和样式依赖；构建进一步核对实际 bundle 输入。架构测试验证跨主题依赖被拒绝、只选择目标主题及同栈共享包、原生目录不进入 npm 工作区。

保持已生成的 Neuro 离线预览与版权文件，不手改生成物。涉及表现或行为变化时另做真实浏览器验证；原生实现将提供真实原生 Gallery，网页截图/视频或仿制演示不能替代原生交互验收。打包和应用消费需分别验证，不以预览通过冒充产品完成迁移。
