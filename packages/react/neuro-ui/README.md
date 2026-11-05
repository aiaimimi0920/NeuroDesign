# @neuro/ui

neuro 的 React 标准实现。提供固定主题、通用控件与可组合 UI 模板，不包含具体应用的业务逻辑或宿主调用。

- [中文声明层](../../../design-md/neuro/DESIGN.zh-CN.md) / [English specification](../../../design-md/neuro/DESIGN.md)
- [真实控件预览](../../../design-md/neuro/index.html) / [网站示例](../../../design-md/neuro/example.html)
- [两层结构与维护边界](../../../docs/ARCHITECTURE.md)
- [应用接入与迁移说明](../../../docs/INTEGRATION.md)

## 构建与安装

在仓库根目录使用 Node 22+、npm 11+：

```powershell
npm ci
npm run check
npm run pack:ui
```

如果系统仍使用 npm 10，可用 `npx --yes npm@11 ci` 安装，不需要修改全局 npm。打包会生成 `neuro-ui-0.1.1.tgz`，将它作为内部依赖安装到应用：

```powershell
npm install C:/Users/Public/nas_home/AwesomeDesignMD/neuro-ui-0.1.1.tgz
```

包内包含 ESM、TypeScript 声明和 CSS；React、React DOM 由应用提供，当前面向 React 19。普通应用安装包不需要本仓库的预览源码和测试依赖。不需要公开到 npm。

当前 Gateway / Loom 使用本地目录依赖。它们在 `apps/desktop` 运行 `npx --yes npm@11 run ui:sync`，会构建本库、按需刷新已安装产物并校验 React 唯一性。完整检出目录与 CI 约定见 [接入说明](../../../docs/INTEGRATION.md)。更新源库后不要假定一次普通 `npm install` 已刷新同版本 `file:` 包。

## 最小用法

```tsx
import { useState } from "react";
import { Button, Field, Input, NeuroProvider, Panel, Stack } from "@neuro/ui";
import "@neuro/ui/styles.css";

export function Settings() {
  const [name, setName] = useState("");
  return (
    <NeuroProvider mode="dark">
      <Panel>
        <Stack>
          <Field label="名称">
            <Input value={name} onChange={event => setName(event.target.value)} />
          </Field>
          <Button variant="primary" onClick={() => console.log(name)}>保存</Button>
        </Stack>
      </Panel>
    </NeuroProvider>
  );
}
```

`mode` 只提供 `dark`、`light` 两个固定预设。通常在应用表现层入口放一个 Provider；它限定样式范围，并为弹窗、菜单、通知提供继承相同主题的 portal 容器。不修改 `body` 的主题或全局颜色。

渐进接入现有应用时可用 `inheritTypography` 保留宿主的基础字体与未迁移标题、段落、链接排版，不在整棵旧树上新增排版 reset。标准控件及 portal 仍使用自己的主题样式；新建应用和标准预览默认使用库的基础排版。

标准控件的字号使用 rem，响应宿主根字号偏好。宿主有字体设置时，通过 Provider 的 `style` 映射 `--n-font-sans` 和 `--n-mono`，不要在应用里重复覆写每一种控件皮肤。

## 组件入口

所有组件从 `@neuro/ui` 导入，类型声明随包提供。业务数据、选项和内容由应用传入。

| 类别 | 导出 | 关键契约 |
|---|---|---|
| 主题 | `NeuroProvider`, `useNeuroTheme` | 固定明暗预设、同主题弹层作用域 |
| 操作 | `Button`, `ButtonLink`, `IconButton`, `Icon` | primary/secondary/quiet/danger/positive；紧凑和加载状态；图标操作有名称；导航保留真实链接 |
| 表单 | `Field`, `Input`, `Textarea`, `Select`, `Range` | 原生属性与 ref；Field 关联标签和必要错误，不常驻额外说明 |
| 选择 | `Checkbox`, `Switch`, `RadioGroup`, `SegmentedControl`, `PopupSelect` | 分段是 radio，不是 tab；选项较多优先下拉；选择不改变几何 |
| 多选 | `CheckboxGroup`, `MultiSelect` | 横/竖排复选；可搜索多选下拉保留展开，单行摘要不扩展为 chips |
| 搜索联想 | `SearchCombobox` | 输入保留焦点、方向键选择、Enter 提交、Escape 清空并关闭；查询和结果由应用提供 |
| 弹层 | `ActionTooltip`, `Dialog`, `ConfirmDialog`, `DropdownMenu` | Radix 基础、Neuro 外观、焦点/Escape/滚动边界 |
| 分区 | `Tabs`, `TabsRoot`, `TabList`, `Tab`, `TabPanel`, `Disclosure` | 完整或可组合页签，支持方向键；Disclosure 是原生 details |
| 状态 | `Badge`, `Notice`, `Progress`, `EmptyState` | 文本/图标配合语义色，不虚构业务结果 |
| 通知 | `ToastProvider`, `useToast`, `ToastViewport` | 每棵树独立或由应用传入条目；卸载清理所有计时器；不保存后台任务结果 |
| 技术值 | `CopyValue`, `CopyButton`, `NumberField` | 真实复制反馈、固定反馈位；数值范围由应用明确提供 |
| 数据 | `DataTable`, `SelectionBar` | 稳定 key、局部滚动固定表头、全选/混合态、不插入批量操作行 |
| 组合模板 | `Panel`, `Toolbar`, `Stack`, `AppShell`, `NavigationItem` | 可选内容插槽，不包含应用 API 或规定产品结构 |
| 内容 | `ImagePreview` | 适配/百分比缩放，保留图像内容和比例，不染色、不旋转 |

`EmptyState` 默认用于页面或独立区域；内嵌卡片、范围栏使用 `compact`，不应用 140px 的页面最小高度。`title` 只写真实短状态，必要操作作为 children 显式传入；通用 HTML 属性可用于状态语义及布局定位。应用不另写空态皮肤。

## 常用接口

### 按钮与输入

`Button` 默认 `type="button"`，表单提交必须显式传 `type="submit"`。`compact` 使用 28px 控件；默认按钮最小高度 36px。`loading` 禁止重复操作并保留原有标签占位。

确有长操作标签、窄屏或大字号需求时，给 `Button` / `ButtonLink` 传入 `wrap`，允许标签在可用宽度内换行，最小高度和加载占位保持不变。普通短操作仍默认单行，不通过整体缩小字体适配。

导航操作使用 `ButtonLink href`：它是原生 `<a>`，复用相同外观，透传链接属性、事件和 ref，保留键盘、复制链接及浏览器打开方式。不提供伪禁用链接或按钮加载行为；不可用时由应用显示禁用 `Button`。URL 校验、外部链接的 `target` / `rel` 和路由策略由应用负责。

`Field` 接收单个可关联 `id` 的输入组件，以及 `label`、可选 `error` / `description`。没有必要信息时不生成说明行；只读、禁用和错误仍保持不同语义。

### 选择

`RadioGroup` / `SegmentedControl` 接收 `label`、唯一 `name`、`options`、`value`、`onValueChange`。`options` 中每项为 `{ value, label, disabled? }`，值支持字符串联合类型。行外已有可见标签时用 `hideLabel` 隐藏重复 legend，但保留 radiogroup 的可访问名称。必要的组级错误通过 `error` 传入，由组件关联 `aria-invalid` / `aria-describedby`，不为 fieldset 嵌套额外的输入标签。

`Select` 是原生下拉，接收 `<option>` / `<optgroup>`。需要跨平台一致的弹出列表时使用 `PopupSelect`：受控 `value` / `onValueChange`、`options`，支持方向键、选中标记、滚动、禁用和 Escape 返回触发器。每项 `value` 必须是非空字符串；业务上的空 ID 由应用无损编码。`displayValue` 可显示缺失选项的原始标签，`leading` 可放图标；按钮属性和 ref 原样透传。

`CheckboxGroup` / `MultiSelect` 使用 `values` 和 `onValuesChange`。少量多选可设置 `orientation="vertical"`，默认为横排；多选下拉用 `formatSummary(count)` 定义单行摘要，默认显示数量，也可以显示简短名称；不将已选项扩展为会换行的 chips。

`MultiSelect.options` 可提供 `description`、`meta` 和 `searchText`。需要搜索时传入 `search={{ label, placeholder?, emptyMessage }}`；不提供则不增加搜索行。无选项时可传 `empty` 显示应用的空态或不支持原因。选项是否可选、最大数量和已移除项的取消规则由应用决定，不能在库内硬编码。

方向键在可用选项间移动，搜索框保留文本编辑语义；勾选不关闭弹层、不重建选项。Tab / Shift+Tab 从边界离开选择器，Escape 只关闭当前弹层并返回触发器，不能连带关闭父弹窗。弹层继承所属 Provider 的主题。

### 搜索联想

`SearchCombobox` 接收 `label`、受控 `value` / `onValueChange`、`options`、`onSelect` 和 `emptyLabel`。选项为 `{ value, label, description?, disabled? }`；查询索引、结果上限和选中后的路由或定位都由应用负责。弹层不抢走输入焦点，方向键跳过禁用项，Enter 选择，Tab 离开后关闭。Escape 清空查询并关闭当前联想，不连带关闭父弹窗；无查询时不拦截父层 Escape。`onExpandedChange` 可通知宿主暂时隐藏遮挡联想层的原生子视图，不在库内调用宿主 API。

### 可组合页签

简单场景使用 `Tabs` 的 `items`。需要把工具栏放在页签和面板之间时，组合 `TabsRoot`、`TabList label`、`Tab value` 与 `TabPanel value`；`TabsRoot asChild` 可复用应用的语义容器。页签仍由同一个 Radix Root 管理关联 ID、选中态和方向键，不再由应用手写一套键盘处理。需要保留未激活面板的 DOM 或局部状态时，可使用 `forceMount`，同时按当前值设置 `hidden`。

`Disclosure` 使用原生 `details` / `summary`，支持 Enter、Space 和展开状态关联。`defaultOpen` 只初始化非受控状态；需要互斥展开或由路由决定分区时，传 `open` / `onOpenChange`，应用持有展开规则。关闭时是否卸载业务内容由调用方决定，不在库内重置草稿。

### 弹窗与确认

`Dialog` 可传 `trigger` 自动管理打开，也可使用 `open` / `onOpenChange`。`title` 必须提供，`description` 只在确有必要时提供；表单和末端操作由调用方组合。

表单需要聚焦指定输入时传入 `initialFocusRef`，无可用目标时使用 Radix 默认焦点。`busy` 禁用关闭按钮、Escape 和遮罩取消，应用仍负责真实保存与字段可用性。额外的 `aria-describedby` 会与必要的上下文说明合并，可关联应用自己的校验信息。`className` 用于应用特有的宽度和滚动布局，不复制弹窗皮肤。

`Dialog.dismissOnOutside` 默认为 true；容易丢失重要草稿的表单可设为 false，仍保留取消按钮与非忙碌时的 Escape。

业务按键输入器可使用 `onEscapeKeyDown(event)`，仅在确需捕获的范围调用 `preventDefault()`，例如输入器正在记录带修饰键的 Escape。普通 Escape 仍交给标准弹窗关闭；`busy` 始终阻止关闭，宿主回调不能绕过。不要为此另建 document 键盘监听或焦点圈定。

`ConfirmDialog` 使用受控 `open`，必须提供 `title`、风险说明 `description`、`onOpenChange`、`onConfirm`。`onConfirm` 不自动关闭：应用完成真实操作后再设 `open=false`，失败时可保留弹窗和就近错误。`busy` 禁止重复确认和取消/Escape，等待应用决定完成结果。

仅禁用确认而保留取消时用 `confirmDisabled`。`children` 可放必要风险说明或可选复选框。默认不允许点击遮罩取消；`dismissOnOutside` 显式开启，忙碌时仍禁止取消。队列继续显示下一个请求时改变 `focusResetKey`，焦点回到取消按钮，不卸载重建整个弹窗。

两种弹窗通常返回打开前的焦点；如果由关闭后会卸载的菜单项等临时控件打开，传入 `returnFocusRef` 指向持续存在的菜单按钮等合理位置，避免尝试聚焦已经移除的节点。

如果操作后原入口可能变为禁用状态，使用 `resolveReturnFocus(target)` 在关闭时按应用的最新状态解析合理返回目标。`target` 是 `returnFocusRef` 指向的元素或打开前的元素；返回另一个 `HTMLElement` 或 `null`。库只聚焦仍连接、未禁用且不在 inert 区域中的目标；返回 `null` 或无效目标时不拦截 Radix 默认返回行为。组件库不识别卡片、路由或业务 CSS，应用负责选择正确位置。

### 通知

在 `NeuroProvider` 内放置 `ToastProvider`，子组件通过 `useToast()` 获取 `notify`：

```tsx
notify({ message: "保存成功", tone: "success" });
```

只能在操作真正成功后调用。`tone` 支持 success/warning/info/danger，`duration` 可指定毫秒。持久错误保留在页面，不只显示短暂通知。

应用已有通知发布入口时，直接使用受控 `ToastViewport entries={entries} onRemove={remove}`。条目为 `{ id, message, tone?, duration? }`；发布总线留在应用。默认 info/success 3200ms、warning 4200ms、danger 5200ms，另有 220ms 退出过渡。通知整条可点击/键盘关闭，error 使用 assertive alert，其余使用 polite status；卸载时清理等待和退出计时器。

### 复制与数值

`CopyValue` 接收显式 `value`、按钮可访问名称 `label`，可选 `displayValue` 仅控制显示；调用方必须明确复制权限和复制范围。默认调用真实 Clipboard API，不支持或被拒绝时显示失败，不伪造成功。可通过 `writeText` 注入宿主提供的异步剪贴板能力，库不导入 Tauri。

仅需复制操作而不展示值时使用 `CopyButton`。必须提供可访问名称 `label` 和 `onCopy(): Promise<void>`，可通过 `children` 设置简短可见标签。回调 resolve 才显示成功，reject 显示失败；复制过程中禁止重复操作。`copiedLabel`、`errorLabel` 和 `feedbackDuration` 控制反馈文案与持续时间，默认 1800ms。标签和失败占位同时预留几何，成功仅更换图标并通过 status 宣告，不推动邻近操作。权限校验、敏感值读取、卸载后是否仍允许写入剪贴板等业务生命周期由调用方负责。

`NumberField` 必须提供 `label`、`value`、`onValueChange`、`min`、`max`、`step`，可提供 `unit`。范围和步长必须有效，数值草稿在失焦或 Enter 时提交；提交时夹到范围内并从 min 起对齐步长。默认空草稿恢复上次已提交值；`allowEmpty` 时提交 `null`。空值不被擅自替换为某个数字，未提供值时增减按钮禁用。演示中的列数 1–12、描边 0.25–4px 只是夹具范围，不是主题要求。

### 表格与模板

`DataTable<T>` 使用 `rows`、`columns`、`getRowKey`；列为 `{ key, header, cell, width?, rowHeader? }`。key 必须来自数据的稳定标识，不得使用随排序变化的行号。传 `selectedKeys` / `onSelectedKeysChange` 开启选择；`getRowLabel` 用于行选择的名称。全选只改变当前列表内的 key，保留其他已有选择。`rowHeader` 将该列单元格输出为 `th scope="row"`，不会随列头吸顶。

需要逐行展开时，传入 `rowDetails(row) => ({ id, open, content })`。库负责独立详情行、正确跨列和 `hidden`；数据行与详情行共享稳定 key 分组。调用方提供页面内唯一且稳定的 `id`，在自己的标准 Button 上关联 `aria-controls` / `aria-expanded`，并维护展开状态和内容。折叠不会卸载库拥有的详情行；内容是否延迟挂载由调用方决定，读取结果等业务操作不由表格自动触发。

`SelectionBar` 同时保留摘要与操作区域，未激活区域 `inert`，不因第一次选择插入新行；从批量操作区清空选择时，焦点回到摘要，不留在已隐藏的按钮上。`AppShell` 支持 navigation、actions、toolbar、inspector、status 和 children 插槽，小屏可通过导航弹窗访问侧栏。它不是强制布局。

`ImagePreview.zoom` 是 `"fit"` 或百分比数字；必须明确提供图像尺寸和 alt。格式解码、资源权限、加载失败等产品状态由应用负责。

## 内部维护策略

所有引用方统一跟进当前接口，不维护历史接口兼容层。改动组件时同步改调用方、文档和真实预览。固定主题不代表复制两套深浅色组件，也不建设多品牌换肤平台。

组件包与预览都不拥有应用流程。关闭/销毁界面后，业务是否继续运行由应用核心负责，不能依赖 React 组件的计时器或 Effect。
