# Windows 原生标准实现的组织约定

此目录用于未来的 Windows 原生主题实现，不是 npm workspace。目前没有可引用的原生组件 crate，也不在本次架构调整中从 Otter 抽取代码；neuro-mini 仍只维护声明层。

## 有实际实现后采用的结构

组件放在 `packages/windows-native/<theme>-ui`，原生控件浏览与组合窗口放在 `apps/windows-native/<theme>-gallery`。组件使用自己的 `Cargo.toml` 和明确的 crate 导出；应用通过 Cargo path、内部发布源或其他明确的 Cargo 分发方式依赖它，不依赖仓库根 npm 包。

每个可交付实现必须提供可直接使用的 Cargo 构建与测试命令，以及自身的依赖边界、消费和资源生命周期检查。不能把不存在的 crate 或尚未执行的门禁标成已经支持。未创建的目录名只是约定，不是可运行路径。

## 边界

- Windows 原生可采用系统控件、Win32 原生自绘或二者组合，不因统一主题仓库而引入 React、DOM、CSS、Node 或 WebView。
- 主题 crate 只依赖本主题与确有需求的同栈共享 crate；共享 crate 不依赖具体主题。没有实际共享内容时不创建公共底座。
- 本地 npm 安装、React 构建、React 类型检查和 React 包分发不进入此目录；原生消费与构建也不要求先执行根 npm 命令。
- 只抽离通用表现和交互；捕获、图形业务、IPC、授权、后台任务与应用窗口调度仍由应用维护。
- 原生 Gallery 使用真实组件。网页仅作为文档、截图、录屏和程序入口；额外 Web 演示不能替代原生交互、DPI、键盘或输入法验证。

设计声明继续放在 `design-md/<theme>`，保持技术无关，不将 Rust/Win32 接口混入 DESIGN。
