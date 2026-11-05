import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import {
  ActionTooltip, Badge, Button, ButtonLink, Checkbox, CheckboxGroup, ConfirmDialog, CopyButton, CopyValue, DataTable, Dialog, Disclosure,
  DropdownMenu, EmptyState, Field, Icon, IconButton, ImagePreview, Input, MultiSelect, NavigationItem, Notice,
  NumberField, Panel, PopupSelect, Progress, RadioGroup, Range, SearchCombobox, SegmentedControl, Select, SelectionBar, Stack, Switch, Tabs, Textarea,
  Toolbar, useToast,
} from "@neuro/ui";

const options = ["OAuth", "API", "Web", "Desktop", "Automation", "Sharing", "Versioning", "Export"].map(value => ({ value, label: value }));
const revisions = Array.from({ length: 16 }, (_, index) => ({ id: `r${String(16 - index).padStart(2, "0")}`, size: `${24 + index} KB`, review: index === 0 }));
function Section({ id, title, children }: { id: string; title: string; children: ReactNode }) {
  return <section id={id} className="preview-section"><h2>{title}</h2>{children}</section>;
}
function SampleImage() {
  const [zoom, setZoom] = useState<"fit" | number>("fit");
  return <Stack><ImagePreview src="sample.svg" alt="界面样本，640 × 400" width={640} height={400} zoom={zoom} />
    <Toolbar><NumberField label="缩放" unit="%" value={zoom === "fit" ? 100 : zoom} onValueChange={value => { if (value !== null) setZoom(value); }} min={1} max={200} step={1} />
      <Button compact aria-pressed={zoom === "fit"} onClick={() => setZoom("fit")}>适配</Button></Toolbar>
  </Stack>;
}

export function Gallery() {
  const notify = useToast();
  const [enabled, setEnabled] = useState(true);
  const [notification, setNotification] = useState("off");
  const [scale, setScale] = useState("110%");
  const [role, setRole] = useState("");
  const [view, setView] = useState("list");
  const [category, setCategory] = useState("全部");
  const [methods, setMethods] = useState(["OAuth", "API"]);
  const [formats, setFormats] = useState(["PNG"]);
  const [capabilities, setCapabilities] = useState(["OAuth", "API", "Web"]);
  const [columns, setColumns] = useState<number | null>(4);
  const [stroke, setStroke] = useState<number | null>(1.25);
  const [tab, setTab] = useState("overview");
  const [expanded, setExpanded] = useState<string | null>(null);
  const [selected, setSelected] = useState<string[]>([]);
  const [expandedRevision, setExpandedRevision] = useState<string | null>(null);
  const revisionDetailsId = useId();
  const [confirm, setConfirm] = useState(false);
  const confirmReturnFocus = useRef<HTMLElement | null>(null);
  const menuTrigger = useRef<HTMLButtonElement | null>(null);
  const [dialog, setDialog] = useState(false);
  const projectNameInput = useRef<HTMLInputElement | null>(null);
  const [name, setName] = useState("");
  const [search, setSearch] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState(64);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => () => clearTimeout(timer.current), []);
  const [pinned, setPinned] = useState(false);
  const [navigation, setNavigation] = useState("概览");
  return <main id="main-content" className="preview-gallery">
    <header className="preview-heading"><h1>neuro 控件</h1><span className="n-muted">React 标准实现 · @neuro/ui</span></header>
    <nav className="preview-categories" aria-label="控件分类">{[["palette", "配色"], ["actions", "操作"], ["inputs", "输入与选择"], ["navigation", "导航"], ["data", "数据与内容"], ["feedback", "反馈"], ["overlays", "弹层"]].map(([id, title]) => <a key={id} href={`#${id}`}>{title}</a>)}</nav>
    <Section id="palette" title="配色"><div className="preview-swatches">{[["#d9ff38", "信号黄"], ["#22c55e", "成功绿"], ["#06b6d4", "信息蓝"], ["#f43f5e", "危险红"]].map(([color, label]) => <Panel key={color}><i style={{ background: color }} /><strong>{label}</strong><code>{color}</code></Panel>)}</div></Section>
    <Section id="actions" title="操作"><div className="preview-grid">
      <Panel><Stack><h3>操作层级</h3><Toolbar><Button variant="primary" onClick={() => setDialog(true)}><Icon name="plus" />新建项目</Button><Button onClick={() => notify({ message: "次级操作示例" })}>次级操作</Button><Button variant="quiet" onClick={() => notify({ message: "安静操作示例" })}>安静操作</Button></Toolbar>
        <Toolbar><Button variant="positive" onClick={() => notify({ message: "完成状态示例", tone: "success" })}><Icon name="check" />完成</Button><Button variant="danger" onClick={event => { confirmReturnFocus.current = event.currentTarget; setConfirm(true); }}><Icon name="trash" />删除</Button><Button disabled>不可用</Button><ButtonLink href="example.html">网站示例</ButtonLink></Toolbar></Stack></Panel>
      <Panel><Stack><h3>紧凑与加载状态</h3><Toolbar><Button compact onClick={() => notify({ message: "紧凑操作示例" })}>紧凑操作</Button>
        <ActionTooltip label="收藏"><IconButton label="收藏" compact aria-pressed={pinned} onClick={() => setPinned(!pinned)}><Icon name="star" /></IconButton></ActionTooltip>
        <Button loading={busy} onClick={() => { setBusy(true); timer.current = setTimeout(() => { setBusy(false); notify({ message: "演示处理完成", tone: "success" }); }, 900); }}>处理样本</Button>
        <CopyButton label="复制样本 ID" compact onCopy={() => navigator.clipboard.writeText("project_0042")}>复制 ID</CopyButton></Toolbar><Button wrap>继续导入当前选定的资源</Button></Stack></Panel>
    </div></Section>
    <Section id="inputs" title="输入与选择"><div className="preview-grid three">
      <Panel><Stack><Field label="项目名称"><Input placeholder="界面研究" /></Field><Field label="无效输入" error="请输入有效的邮箱地址"><Input defaultValue="not-an-email" /></Field><Field label="只读值"><Input value="界面研究" readOnly /></Field><CopyValue label="复制项目 ID" value="project_0042" /><Field label="不可用字段"><Input value="由应用管理" disabled readOnly /></Field></Stack></Panel>
      <Panel><Stack><Checkbox label="包含归档资源" defaultChecked /><Checkbox label="固定选择" defaultChecked disabled />
        <RadioGroup label="默认视图" name="default-view" value={view} onValueChange={setView} options={[{ value: "list", label: "列表" }, { value: "grid", label: "网格" }]} />
        <Switch label="显示元数据" checked={enabled} onChange={event => setEnabled(event.target.checked)} /><Switch label="不可用设置" disabled />
        <Field label="预览比例"><Range min={25} max={100} defaultValue={75} /></Field></Stack></Panel>
      <Panel><Stack><NumberField label="列数" value={columns} onValueChange={setColumns} min={1} max={12} step={1} /><NumberField label="描边宽度" value={stroke} onValueChange={setStroke} min={0.25} max={4} step={0.25} unit="px" /><Field label="备注"><Textarea placeholder="备注" /></Field></Stack></Panel>
    </div><div className="preview-grid">
        <Panel><Stack><SegmentedControl label="通知" name="notifications" value={notification} onValueChange={setNotification} options={[{ value: "off", label: "关闭" }, { value: "on", label: "开启" }]} />
          <SegmentedControl label="缩放比例" name="scale" value={scale} onValueChange={setScale} options={["100%", "110%", "125%", "150%"].map(value => ({ value, label: value }))} />
          <SegmentedControl label="固定模式" name="managed" value="auto" onValueChange={() => { }} disabled options={[{ value: "auto", label: "自动" }, { value: "manual", label: "手动" }]} />
          <SegmentedControl label="执行角色" name="execution-role" value={role} onValueChange={setRole} error={!role ? "请选择执行角色" : undefined} options={[{ value: "local", label: "本地" }, { value: "remote", label: "远程" }]} />
          <Field label="资源分类"><Select defaultValue="全部">{["全部", "界面", "文档", "代码", "图像", "音频", "视频", "压缩包", "字体"].map(value => <option key={value}>{value}</option>)}</Select></Field>
          <Field label="弹出式分类"><PopupSelect value={category} onValueChange={setCategory} options={["全部", "界面", "文档", "代码", "图像", "音频", "视频", "压缩包", "字体"].map(value => ({ value, label: value }))} /></Field></Stack></Panel>
        <Panel><Stack><CheckboxGroup label="访问方式" options={options.slice(0, 3)} values={methods} onValuesChange={setMethods} />
          <CheckboxGroup label="导出格式" options={["PNG", "SVG", "PDF"].map(value => ({ value, label: value }))} values={formats} onValuesChange={setFormats} orientation="vertical" />
          <MultiSelect label="能力" options={options} values={capabilities} onValuesChange={setCapabilities} search={{ label: "搜索能力", emptyMessage: "没有匹配的能力" }} />
          <SearchCombobox label="搜索样本" placeholder="搜索样本" value={search} onValueChange={setSearch}
            options={options.filter(option => option.label.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase()))}
            onSelect={value => { setSearch(""); notify({ message: `已选择样本 ${value}` }); }} /></Stack></Panel>
      </div></Section>
    <Section id="navigation" title="导航与展开"><div className="preview-grid">
      <Panel><nav aria-label="样本导航">{["概览", "资源", "活动"].map(value => <NavigationItem key={value} active={navigation === value} onClick={() => setNavigation(value)}><Icon name={value === "资源" ? "folder" : "grid"} />{value}</NavigationItem>)}</nav></Panel>
      <Panel><Tabs label="资源信息" value={tab} onValueChange={setTab} items={[{ value: "overview", label: "概览", content: "界面研究" }, { value: "properties", label: "属性", content: <code>SVG · 24 KB</code> }, { value: "history", label: "历史", content: <Badge tone="info">样本修订 r03</Badge> }]} />
        <Disclosure title="高级属性"><Checkbox label="保留原始文件名" /></Disclosure>
        <Disclosure title="资源权限" open={expanded === "permissions"} onOpenChange={open => setExpanded(open ? "permissions" : null)}><Checkbox label="允许编辑" /></Disclosure>
        <Disclosure title="导出选项" open={expanded === "export"} onOpenChange={open => setExpanded(open ? "export" : null)}><Checkbox label="包含元数据" /></Disclosure></Panel>
    </div></Section>
    <Section id="data" title="数据与内容"><div className="preview-grid">
      <Panel><SelectionBar count={selected.length} summary={<h3>修订记录</h3>}><Button compact onClick={() => setSelected([])}>清除</Button></SelectionBar>
        <div className="preview-long-table"><DataTable label="样本修订记录" rows={revisions} getRowKey={row => row.id} selectedKeys={selected} onSelectedKeysChange={setSelected}
          columns={[{
            key: "revision", header: "修订", rowHeader: true, cell: row => <Button compact variant="quiet" aria-label={`展开修订 ${row.id}`} aria-expanded={expandedRevision === row.id}
              aria-controls={`${revisionDetailsId}-${row.id}`} onClick={() => setExpandedRevision(expandedRevision === row.id ? null : row.id)}><code>{row.id}</code></Button>
          },
          { key: "size", header: "大小", cell: row => <code>{row.size}</code> }, { key: "state", header: "状态", cell: row => <Badge tone={row.review ? "warning" : "success"}>{row.review ? "待检查" : "就绪"}</Badge> }]}
          rowDetails={row => ({ id: `${revisionDetailsId}-${row.id}`, open: expandedRevision === row.id, content: <CopyValue label="修订标识" value={row.id} /> })} /></div></Panel>
      <Panel><Stack><h3>内容预览</h3><SampleImage /></Stack></Panel>
    </div></Section>
    <Section id="feedback" title="状态与反馈"><div className="preview-grid">
      <Panel><Stack><Toolbar><Badge tone="success">就绪</Badge><Badge tone="warning">待检查</Badge><Badge tone="info">本地样本</Badge><Badge tone="danger">失败</Badge><Badge>不可用</Badge></Toolbar>
        <Notice tone="success">样本已更新</Notice><Notice tone="warning">3 项资源待检查</Notice><Notice tone="danger">名称不能为空</Notice></Stack></Panel>
      <Panel><Stack><h3>样本进度 · {progress}%</h3><Progress label="样本进度" value={progress} /><Button compact onClick={() => setProgress(value => value >= 100 ? 0 : Math.min(100, value + 12))}>推进样本进度</Button><EmptyState title="没有匹配的资源" /><EmptyState compact title="当前范围为空" /></Stack></Panel>
    </div><Panel><Toolbar><span>存储 <code>28 GB</code> <span className="n-muted">+2 GB / 对比 26 GB</span></span><span>延迟 <code>84 ms</code> <span className="n-muted">−12 ms / 对比 96 ms</span></span></Toolbar></Panel></Section>
    <Section id="overlays" title="弹层"><Panel><Toolbar>
      <Dialog title="新建项目" open={dialog} onOpenChange={value => { setDialog(value); setError(""); }} initialFocusRef={projectNameInput} trigger={<Button>打开弹窗</Button>}>
        <form onSubmit={event => { event.preventDefault(); if (!name.trim()) { setError("请输入项目名称"); return; } notify({ message: `已在演示中创建“${name.trim()}”`, tone: "success" }); setDialog(false); setName(""); }}>
          <Field label="项目名称" error={error}><Input ref={projectNameInput} value={name} onChange={event => setName(event.target.value)} /></Field><div className="n-dialog-actions"><Button onClick={() => setDialog(false)}>取消</Button><Button type="submit" variant="primary">创建</Button></div>
        </form>
      </Dialog>
      <DropdownMenu label="资源操作" trigger={<Button ref={menuTrigger}>资源操作<Icon name="chevron" /></Button>} items={[{ id: "rename", label: "重命名", onSelect: () => notify({ message: "重命名示例见网站页面" }) }, { id: "duplicate", label: "创建副本", onSelect: () => notify({ message: "副本操作示例" }) }, { id: "move", label: "移动", disabled: true, onSelect: () => { } }, { id: "delete", label: "删除…", danger: true, onSelect: () => { confirmReturnFocus.current = menuTrigger.current; setConfirm(true); } }]} />
      <Button onClick={() => notify({ message: "通知示例", tone: "info" })}>显示通知</Button>
      <ActionTooltip label="补充信息"><IconButton label="补充信息"><Icon name="info" /></IconButton></ActionTooltip>
    </Toolbar></Panel></Section>
    <ConfirmDialog title="删除样本？" description="这只演示确认流程，不会删除文件。" open={confirm} onOpenChange={setConfirm} returnFocusRef={confirmReturnFocus} confirmLabel="删除样本" onConfirm={() => { setConfirm(false); notify({ message: "删除确认流程演示完成", tone: "success" }); }} />
    <footer className="preview-footer"><a href="DESIGN.md">English specification</a><a href="README.md">预览边界</a><span>数据仅在当前页面生效，刷新重置。</span></footer>
  </main>;
}
