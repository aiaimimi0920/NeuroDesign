# 多主题、不同技术栈的两层结构

## 声明层

`design-md/<theme>/DESIGN.md` 和 `DESIGN.zh-CN.md` 描述独立主题的视觉与交互契约，不限定应用用途、业务模型、语言或框架，不依赖其他主题或来源历史。

颜色、排版、几何、状态、选择、反馈、键盘与滚动规则属于声明层。React props、Cargo 接口、安装与构建方式属于实现层说明，不混入 DESIGN。

## 标准实现层

一个主题可以选择自己的技术栈，不要求所有主题都是 React，也不要求每个主题同时支持所有框架。同一主题有实际需要时可维护多个独立实现。共享代码和预览仍是实现层内部组织，不是新增设计层。

```text
design-md/<theme>/                 技术无关的中英文声明
packages/<stack>/<package>/        对应技术栈的独立组件包
apps/<stack>/<preview>/            使用真实组件包的预览程序
```

当前 neuro 标准实现位于 [packages/react/neuro-ui](../packages/react/neuro-ui/README.md)，包名 `@neuro/ui` 不变。React 预览位于 `apps/react/neuro-preview`，离线输出继续保留在 `design-md/neuro`，原有 HTML 入口不变。

[packages/windows-native](../packages/windows-native/README.md) 记录 Windows 原生实现的组织约定。未来 Rust/Win32 实现通过 Cargo 交付，不包装 React、不引入 WebView、不通过 npm 构建。neuro-mini 当前仍仅声明层，不存在可引用的原生 crate；旧网页不是原生标准实现。

## 依赖方向与隔离

```text
主题 A 的 React 应用 → A 的 React 包 → 按需使用的 React shared 包
主题 B 的 React 应用 → B 的 React 包 → 按需使用的 React shared 包
主题 C 的 Windows 应用 → C 的原生 crate → 按需使用的原生 shared crate
```

- 主题 A 不依赖主题 B。不得继承另一主题整包后再覆盖样式。
- shared 只能依赖同栈其他 shared，不能反向依赖主题或预览。
- 相同交互、不同外观时只共享合适的行为；行为与外观均相同时才共享对应组件及必要局部资源。无真实需求不建立共享包。
- 不通过相对路径穿透包边界，不从总入口汇总所有主题，不让样式总入口顺带加载其他主题。
- 跨栈共享的是设计契约与确有需要的数据/资源，不强行共享控件代码，不引入跨语言统一运行时。
- 源码共处一个仓库与产物共同交付是两回事。应用直接安装目标主题包，不安装仓库根包或预览工具。

## React 工具边界

根 npm 包是 private 开发工具，workspace 仅匹配 `packages/react/*` 与 `apps/react/*`，不匹配 Windows 原生目录。每个 React 包通过 `package.json.designSystem` 标明归属：

```json
{ "kind": "theme", "theme": "neuro" }
```

shared 使用 `{ "kind": "shared" }`，不指定主题；preview 使用 `kind: preview`、自身主题和 `library`，并显式依赖对应包。无需为了登记未来主题创建空包。

`npm run build` 构建当前全部 React 主题和预览；`npm run build -- --theme neuro` 仅构建指定主题及其真实共享依赖；追加 `--library` 不构建预览。未知主题明确失败，不回退为 Neuro。

`check:boundaries` 检查包清单、静态 import/export、动态 import/require 及 CSS 资源引用，拒绝跨主题、反向 shared、未声明的内部依赖和源码穿透。构建通过 esbuild metafile 再核对实际输入；预览必须消费已构建的公开包入口。测试以独立多主题夹具验证只消费目标主题的 JS/CSS，不依赖 tree shaking 来掩盖非法依赖。

这些门禁目前针对真实存在的 React 实现，不宣称已经验证不存在的原生库。新增其他栈时，必须随实际实现增加其独立构建、依赖边界与消费验证。

## 预览与应用边界

React 控件浏览与网站示例使用真实包，生成离线经典脚本，不需要 CDN、远程字体或服务器。React / React DOM 是组件包的 peer dependencies，运行时由应用提供。

原生控件的真实交互预览应是原生 Gallery 和组合窗口。网页可提供文档、截图、录屏和原生预览入口；浏览器仿制版必须另行标识，不能冒充原生验收。

组件可拥有焦点、展开、输入草稿、通知等短暂界面状态；账户、权限、后台任务、网络、持久化和宿主生命周期仍由应用负责。组合模板可选，不强制产品采用示例业务或页面结构。

## 维护策略

内部引用方统一跟进当前接口，不维护历史兼容层。固定主题配色不等于运行时任意换肤平台；独立主题包也不是一套组件的皮肤集合。

声明与实际标准实现保持一致。已构建应用仍需重新构建和分发才能包含组件更新。下游需要未正式维护的技术栈时，可根据声明与预览自行实现，由下游维护，不自动纳入官方支持。
