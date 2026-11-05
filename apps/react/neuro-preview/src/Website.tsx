import { useMemo, useRef, useState } from "react";
import {
  AppShell, Badge, Button, Checkbox, ConfirmDialog, CopyValue, DataTable, Dialog, DropdownMenu, EmptyState,
  Field, Icon, IconButton, ImagePreview, Input, MultiSelect, NavigationItem, Notice, NumberField, Panel,
  Progress, SegmentedControl, Select, SelectionBar, Stack, Switch, Tabs, Textarea, Toolbar, useToast,
  type TableColumn,
} from "@neuro/ui";

interface Asset { id: string; name: string; type: string; size: string; project: string; review: boolean; favorite: boolean; archived: boolean; note: string }
const initialAssets: Asset[] = [
  { id: "asset_001", name: "interface-kit.svg", type: "SVG", size: "24 KB", project: "界面研究", review: false, favorite: true, archived: false, note: "" },
  { id: "asset_002", name: "workspace-layout.fig", type: "FIG", size: "1.8 MB", project: "界面研究", review: true, favorite: false, archived: false, note: "" },
  { id: "asset_003", name: "design-tokens.css", type: "CSS", size: "8 KB", project: "界面研究", review: false, favorite: true, archived: false, note: "" },
  { id: "asset_004", name: "review-notes.md", type: "MD", size: "12 KB", project: "调研", review: true, favorite: false, archived: false, note: "" },
  { id: "asset_005", name: "release-cover.png", type: "PNG", size: "420 KB", project: "发布资料", review: false, favorite: false, archived: false, note: "" },
  { id: "asset_006", name: "old-navigation.svg", type: "SVG", size: "20 KB", project: "界面研究", review: false, favorite: false, archived: true, note: "" },
  ...["button-states.svg", "form-layout.svg", "color-audit.md", "navigation-notes.md", "spacing-tokens.css", "focus-states.css", "review-checklist.md", "export-settings.json", "panel-layout.svg", "contrast-report.md", "menu-states.svg", "type-scale.css"].map((name, index) => ({
    id: `asset_${String(index + 7).padStart(3, "0")}`, name, type: name.split(".").pop()!.toUpperCase(), size: `${6 + index * 2} KB`,
    project: ["界面研究", "发布资料", "调研"][index % 3], review: index % 4 === 0, favorite: false, archived: false, note: "",
  })),
];
const sectionLabels: Record<string, string> = { all: "全部资源", favorites: "收藏", review: "待检查", archive: "归档" };

export function Website() {
  const notify = useToast();
  const [assets, setAssets] = useState(initialAssets);
  const [projects, setProjects] = useState(["界面研究", "发布资料", "调研"]);
  const [section, setSection] = useState("all");
  const [project, setProject] = useState("");
  const [query, setQuery] = useState("");
  const [type, setType] = useState("all");
  const [status, setStatus] = useState("all");
  const [view, setView] = useState("list");
  const [activeId, setActiveId] = useState("asset_001");
  const [selected, setSelected] = useState<string[]>([]);
  const [editor, setEditor] = useState<"asset" | "project" | "rename" | null>(null);
  const [editorName, setEditorName] = useState("");
  const [editorError, setEditorError] = useState("");
  const addResourceButton = useRef<HTMLButtonElement | null>(null);
  const editorReturnFocus = useRef<HTMLElement | null>(null);
  const [confirmIds, setConfirmIds] = useState<string[]>([]);
  const [settings, setSettings] = useState(false);
  const [metadata, setMetadata] = useState(true);
  const [capabilities, setCapabilities] = useState(["API", "Web"]);
  const [zoom, setZoom] = useState<"fit" | number>("fit");
  const [inspectorTab, setInspectorTab] = useState("properties");
  const active = assets.find(asset => asset.id === activeId);
  const visible = useMemo(() => assets.filter(asset =>
    (section === "archive" ? asset.archived : !asset.archived)
    && (section !== "favorites" || asset.favorite)
    && (section !== "review" || asset.review)
    && (!project || asset.project === project)
    && (type === "all" || asset.type === type)
    && (status === "all" || (status === "review" ? asset.review : !asset.review))
    && `${asset.name} ${asset.project}`.toLowerCase().includes(query.toLowerCase()),
  ), [assets, section, project, type, status, query]);

  function edit(kind: "asset" | "project" | "rename") {
    // New-project navigation may unmount on mobile; return to its next persistent action.
    editorReturnFocus.current = kind === "project" ? addResourceButton.current : null;
    setEditorName(kind === "rename" ? active?.name ?? "" : "");
    setEditorError("");
    setEditor(kind);
  }
  function patchActive(patch: Partial<Asset>) {
    setAssets(current => current.map(asset => asset.id === activeId ? { ...asset, ...patch } : asset));
  }
  function saveEditor(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const name = editorName.trim();
    if (!name) { setEditorError("名称不能为空"); return; }
    if (editor === "project") {
      if (projects.includes(name)) { setEditorError("项目名称已存在"); return; }
      setProjects(current => [...current, name]);
      setProject(name);
      setSection("all");
    } else if (editor === "rename") {
      patchActive({ name });
    } else {
      const id = `asset_${crypto.randomUUID()}`;
      setAssets(current => [...current, { id, name, type: name.includes(".") ? name.split(".").pop()!.toUpperCase() : "FILE", size: "—", project: project || projects[0], review: true, favorite: false, archived: false, note: "" }]);
      setActiveId(id);
      setSection("all"); setType("all"); setStatus("all"); setQuery("");
    }
    setEditor(null);
    notify({ message: "演示内容已更新", tone: "success" });
  }
  function clearFilters() { setQuery(""); setType("all"); setStatus("all"); }
  function exportMetadata() {
    const blob = new Blob([JSON.stringify(visible, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url; anchor.download = "neuro-sample-assets.json"; anchor.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  const tableColumns: TableColumn<Asset>[] = [
    { key: "name", header: "名称", width: "44%", cell: asset => <Button className="preview-asset-name" compact variant="quiet" onClick={() => setActiveId(asset.id)}><Icon name={asset.type === "SVG" || asset.type === "PNG" ? "grid" : "folder"} /><span>{asset.name}</span></Button> },
    ...(metadata ? [{ key: "type", header: "类型", width: "15%", cell: (asset: Asset) => <span className="n-muted n-mono">{asset.type}</span> }] : []),
    { key: "status", header: "状态", width: "23%", cell: asset => <Badge tone={asset.review ? "warning" : "success"}>{asset.review ? "待检查" : "就绪"}</Badge> },
    { key: "size", header: "大小", width: "18%", cell: asset => <span className="n-muted n-mono">{asset.size}</span> },
  ];
  const navigation = <Stack><div className="preview-nav-brand"><Icon name="grid" /><strong>neuro</strong></div>
    <nav aria-label="资源分组">{Object.entries(sectionLabels).map(([key, label]) => <NavigationItem key={key} active={section === key && !project} onClick={() => { setSection(key); setProject(""); }}><Icon name={key === "favorites" ? "star" : key === "review" ? "warning" : "folder"} />{label}</NavigationItem>)}</nav>
    <div className="preview-nav-heading"><h2>项目</h2><IconButton label="新建项目" compact variant="quiet" onClick={() => edit("project")}><Icon name="plus" /></IconButton></div>
    <nav aria-label="项目">{projects.map(value => <NavigationItem key={value} active={project === value} onClick={() => { setProject(value); setSection("all"); }}><Icon name="folder" /><span>{value}</span></NavigationItem>)}</nav>
    <div className="preview-storage"><span>样本存储 <code>28 / 100 GB</code></span><Progress label="样本存储使用率" value={28} /><span className="n-muted">+2 GB / 对比 26 GB</span></div>
  </Stack>;
  const inspector = active ? <Stack>
    <div className="preview-inspector-heading"><h2>详情</h2><IconButton label="收藏当前资源" compact variant="quiet" aria-pressed={active.favorite} onClick={() => patchActive({ favorite: !active.favorite })}><Icon name="star" /></IconButton></div>
    {active.id === "asset_001" ? <><ImagePreview src="sample.svg" alt="界面样本，640 × 400" width={640} height={400} zoom={zoom} /><Toolbar><Button compact onClick={() => setZoom("fit")} aria-pressed={zoom === "fit"}>适配</Button><NumberField label="缩放" unit="%" value={zoom === "fit" ? 100 : zoom} min={1} max={200} step={1} onValueChange={value => { if (value !== null) setZoom(value); }} /></Toolbar></> : <EmptyState title="无可用预览" />}
    <h3 className="preview-filename">{active.name}</h3><CopyValue label="复制资源 ID" value={active.id} />
    <Tabs label="详情分区" value={inspectorTab} onValueChange={setInspectorTab} items={[
      { value: "properties", label: "属性", content: <Stack><dl className="preview-properties"><dt>类型</dt><dd>{active.type}</dd><dt>大小</dt><dd>{active.size}</dd><dt>项目</dt><dd>{active.project}</dd></dl><Field label="备注"><Textarea value={active.note} onChange={event => patchActive({ note: event.target.value })} /></Field><Switch label="待检查" checked={active.review} onChange={event => patchActive({ review: event.target.checked })} /></Stack> },
      { value: "actions", label: "操作", content: <Stack><Button onClick={() => edit("rename")}>重命名</Button><Button onClick={() => patchActive({ archived: !active.archived })}>{active.archived ? "取消归档" : "归档"}</Button><Button variant="danger" onClick={() => setConfirmIds([active.id])}>删除…</Button></Stack> },
    ]} />
  </Stack> : <EmptyState title="没有选中的资源" />;

  return <div id="main-content" className="preview-website">
    <AppShell title="资源" navigation={navigation} actions={<Button ref={addResourceButton} variant="primary" onClick={() => edit("asset")}><Icon name="plus" />添加资源</Button>}
      toolbar={<Toolbar><Field label="搜索资源"><Input compact type="search" placeholder="搜索资源…" value={query} onChange={event => setQuery(event.target.value)} /></Field>
        <Field label="类型"><Select compact value={type} onChange={event => setType(event.target.value)}><option value="all">全部类型</option>{[...new Set(assets.map(asset => asset.type))].sort().map(value => <option key={value}>{value}</option>)}</Select></Field>
        <Field label="状态"><Select compact value={status} onChange={event => setStatus(event.target.value)}><option value="all">全部状态</option><option value="ready">就绪</option><option value="review">待检查</option></Select></Field>
        <SegmentedControl label="视图" name="asset-view" value={view} onValueChange={setView} options={[{ value: "list", label: "列表" }, { value: "grid", label: "网格" }]} />
        <div className="preview-toolbar-end"><IconButton label="显示设置" compact onClick={() => setSettings(true)}><Icon name="grid" /></IconButton>
          <DropdownMenu label="工作区操作" trigger={<IconButton label="工作区操作" compact><Icon name="more" /></IconButton>} items={[{ id: "export", label: "导出演示元数据", onSelect: exportMetadata }, { id: "clear", label: "清除筛选", onSelect: clearFilters }]} /></div>
      </Toolbar>}
      inspector={inspector} status={<span>{assets.filter(asset => !asset.archived).length} 项活动资源 · {projects.length} 个项目 · 已选 {selected.length} 项 <span className="preview-demo-label">演示数据，刷新重置</span></span>}>
      <SelectionBar count={selected.length} summary={<><h2>{project || sectionLabels[section]}</h2><span className="n-muted">{visible.length} 项</span></>}>
        <Button compact onClick={() => setSelected([])}>清除</Button><Button compact variant="danger" onClick={() => setConfirmIds(selected)}>删除…</Button>
      </SelectionBar>
      {view === "list" ? <DataTable label="资源列表" rows={visible} columns={tableColumns} getRowKey={asset => asset.id} getRowLabel={asset => asset.name} activeKey={activeId} selectedKeys={selected} onSelectedKeysChange={setSelected}
        empty={<EmptyState title="没有匹配的资源"><Button compact onClick={clearFilters}>清除筛选</Button></EmptyState>} />
        : <div className="preview-asset-grid">{visible.map(asset => <Panel key={asset.id} className="preview-asset-card"><Checkbox aria-label={`选择 ${asset.name}`} label={asset.name} checked={selected.includes(asset.id)} onChange={event => setSelected(current => event.target.checked ? [...current, asset.id] : current.filter(id => id !== asset.id))} />
          {asset.id === "asset_001" ? <img src="sample.svg" alt="界面样本缩略图" /> : <Icon name="folder" />}
          <Badge tone={asset.review ? "warning" : "success"}>{asset.review ? "待检查" : "就绪"}</Badge><Button compact aria-pressed={activeId === asset.id} onClick={() => setActiveId(asset.id)}>查看</Button></Panel>)}
          {!visible.length ? <EmptyState title="没有匹配的资源"><Button compact onClick={clearFilters}>清除筛选</Button></EmptyState> : null}</div>}
    </AppShell>
    <Dialog title={editor === "project" ? "新建项目" : editor === "rename" ? "重命名" : "添加演示资源"} open={editor !== null} onOpenChange={open => { if (!open) setEditor(null); }} returnFocusRef={editorReturnFocus}>
      <form onSubmit={saveEditor}><Field label="名称" error={editorError}><Input value={editorName} onChange={event => setEditorName(event.target.value)} maxLength={120} /></Field>
        <div className="n-dialog-actions"><Button onClick={() => setEditor(null)}>取消</Button><Button type="submit" variant="primary">保存</Button></div></form>
    </Dialog>
    <ConfirmDialog title={`删除 ${confirmIds.length} 项资源？`} description="将从当前演示数据中移除，刷新页面可恢复。" open={confirmIds.length > 0} onOpenChange={open => { if (!open) setConfirmIds([]); }} confirmLabel="删除" onConfirm={() => {
      setAssets(current => current.filter(asset => !confirmIds.includes(asset.id)));
      setSelected(current => current.filter(id => !confirmIds.includes(id)));
      if (confirmIds.includes(activeId)) setActiveId("");
      setConfirmIds([]); notify({ message: "已从演示中移除", tone: "success" });
    }} />
    <Dialog title="显示设置" open={settings} onOpenChange={setSettings}><Stack><Switch label="显示类型列" checked={metadata} onChange={event => setMetadata(event.target.checked)} />
      <MultiSelect label="演示能力选择" options={["API", "Web", "Desktop", "Sharing", "Versioning", "Export"].map(value => ({ value, label: value }))} values={capabilities} onValuesChange={setCapabilities} />
      <Notice tone="info">这些设置仅影响本页演示。</Notice></Stack></Dialog>
  </div>;
}
