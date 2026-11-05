/* Comparable project/asset fixture. No credentials, persistence, or backend simulation. */
(() => {
  "use strict";
  const D = window.NeuroDemo;
  const E = (value) => D.escape(value);
  const I = (name) => D.icon(name);
  const projects = ["Interface studies", "Release kit", "Research"];
  const assets = [
    { id: 1, name: "interface-kit.svg", type: "Interface", format: "SVG", size: "24 KB", project: "Interface studies", status: "Ready", revision: "r08", favorite: true, tags: ["interface", "components"], note: "Shared building blocks for the next interface review.", archived: false },
    { id: 2, name: "workspace-layout.fig", type: "Interface", format: "FIG", size: "1.8 MB", project: "Interface studies", status: "Review", revision: "r03", favorite: false, tags: ["layout", "workbench"], note: "Check the navigation hierarchy before approval.", archived: false },
    { id: 3, name: "design-tokens.css", type: "Code", format: "CSS", size: "8 KB", project: "Interface studies", status: "Ready", revision: "r12", favorite: true, tags: ["tokens", "theme"], note: "Semantic colors and component measurements.", archived: false },
    { id: 4, name: "review-notes.md", type: "Documents", format: "MD", size: "12 KB", project: "Research", status: "Review", revision: "r02", favorite: false, tags: ["notes", "review"], note: "Open questions from the latest design review.", archived: false },
    { id: 5, name: "release-cover.png", type: "Interface", format: "PNG", size: "420 KB", project: "Release kit", status: "Ready", revision: "r04", favorite: false, tags: ["release", "image"], note: "Cover image placeholder for the sample release.", archived: false },
    { id: 6, name: "component-inventory.md", type: "Documents", format: "MD", size: "18 KB", project: "Interface studies", status: "Ready", revision: "r06", favorite: false, tags: ["inventory", "components"], note: "Inventory of controls and their states.", archived: false },
    { id: 7, name: "interaction-map.svg", type: "Interface", format: "SVG", size: "36 KB", project: "Research", status: "Review", revision: "r01", favorite: true, tags: ["interaction", "flow"], note: "A sample map of the main task flow.", archived: false },
    { id: 8, name: "release-manifest.json", type: "Code", format: "JSON", size: "4 KB", project: "Release kit", status: "Ready", revision: "r05", favorite: false, tags: ["release", "metadata"], note: "Metadata fixture only; not a real release manifest.", archived: false },
    { id: 9, name: "old-navigation.svg", type: "Interface", format: "SVG", size: "20 KB", project: "Interface studies", status: "Ready", revision: "r01", favorite: false, tags: ["archive"], note: "Archived sample, available from the Archive view.", archived: true }
  ];
  // Enough fixture rows to exercise a real local scroll, rather than forcing a short list into a viewport.
  ["button-states.svg", "form-layout.svg", "color-audit.md", "navigation-notes.md", "spacing-tokens.css", "focus-states.css", "review-checklist.md", "export-settings.json", "panel-layout.svg", "contrast-report.md", "menu-states.svg", "type-scale.css"].forEach((name, index) => {
    const format = name.split(".").pop().toUpperCase();
    assets.push({ id: index + 10, name, type: format === "SVG" ? "Interface" : format === "MD" ? "Documents" : "Code", format, size: (6 + index * 2) + " KB", project: projects[index % projects.length], status: index % 4 === 0 ? "Review" : "Ready", revision: "r02", favorite: false, tags: ["sample"], note: "", archived: false });
  });
  const state = { section: "all", project: "", query: "", category: "all", status: "all", view: "list", activeId: 1, selected: new Set(), nextId: 22 };
  document.getElementById("app").innerHTML = D.previewBar("workbench") +
    '<div class="workbench" id="workbench"><aside class="app-rail" aria-label="Workspace navigation"><div class="app-brand">' + I("mark") + 'neuro</div>' +
    '<div class="rail-section"><nav class="rail-nav" id="workspace-nav"></nav></div>' +
    '<div class="rail-section"><div class="row spread" style="padding:0 8px 6px"><span class="section-label">Projects</span><button class="btn ghost icon-only small" type="button" aria-label="Create project" id="new-project">' + I("plus") + '</button></div><nav class="rail-nav" id="project-nav" aria-label="Projects"></nav></div>' +
    '<div class="rail-bottom"><div class="row spread muted"><span>Sample storage</span><span class="mono">28 / 100 GB</span></div><progress class="capacity" value="28" max="100" aria-label="Illustrative storage usage: 28 percent"></progress><span class="neutral-delta">+2 GB vs 26 GB</span></div></aside>' +
    '<main class="app-body" id="main-content" tabindex="-1"><header class="workspace-heading"><div class="row grow"><button class="btn ghost icon-only mobile-rail" type="button" id="mobile-nav-toggle" aria-label="Toggle workspace navigation" aria-expanded="false">' + I("menu") + '</button><h1 id="workspace-title">Project & asset workbench</h1></div><button class="btn primary" type="button" id="add-asset">' + I("plus") + 'Add asset</button></header>' +
    '<div class="workspace-toolbar"><label class="search"><span class="sr-only">Search assets</span>' + I("search") + '<input class="input" id="search-assets" type="search" placeholder="Search assets…" autocomplete="off"></label>' +
    '<label class="sr-only" for="type-filter">Asset type</label><select class="input" id="type-filter"><option value="all">All types</option><option>Interface</option><option>Documents</option><option>Code</option></select>' +
    '<label class="sr-only" for="status-filter">Asset status</label><select class="input" id="status-filter"><option value="all">All statuses</option><option>Ready</option><option>Review</option></select>' +
    '<div class="view-toggle" role="group" aria-label="Asset view"><button class="btn icon-only" id="view-list" type="button" aria-label="List view" aria-pressed="true">' + I("list") + '</button><button class="btn icon-only" id="view-grid" type="button" aria-label="Grid view" aria-pressed="false">' + I("grid") + '</button></div>' +
    '<details class="menu"><summary class="btn icon-only" aria-label="Workspace actions">' + I("more") + '</summary><div class="menu-panel"><button type="button" id="export-metadata">' + I("download") + 'Export sample metadata</button><button type="button" id="clear-filters">' + I("close") + 'Clear filters</button></div></details></div>' +
    '<div class="work-area"><section class="asset-browser" aria-label="Assets"><div class="browser-header"><div class="browser-summary"><h2 id="browser-title">All assets</h2><span id="results-count" role="status"></span></div><div class="selection-strip" id="selection-strip"><span id="selection-count" role="status"></span><div class="row"><button class="btn small" id="clear-selection" type="button">Clear</button><button class="btn small destructive" id="delete-selection" type="button">' + I("trash") + 'Delete…</button></div></div></div><div id="assets-view" tabindex="0" role="region" aria-label="Asset browser"></div></section><aside class="inspector" id="asset-inspector" aria-label="Asset details"></aside></div>' +
    '<footer class="app-statusbar"><span id="status-summary"></span></footer></main></div>';
  D.setup();

  function visibleAssets() {
    return assets.filter((asset) => {
      if (state.section === "archive" ? !asset.archived : asset.archived) return false;
      if (state.section === "favorites" && !asset.favorite) return false;
      if (state.section === "review" && asset.status !== "Review") return false;
      if (state.project && asset.project !== state.project) return false;
      if (state.category !== "all" && asset.type !== state.category) return false;
      if (state.status !== "all" && asset.status !== state.status) return false;
      return (asset.name + " " + asset.project + " " + asset.tags.join(" ")).toLowerCase().includes(state.query.toLowerCase());
    });
  }
  const stateBadge = (asset) => '<span class="badge ' + (asset.status === "Ready" ? 'ok' : 'warn') + '">' + I(asset.status === "Ready" ? "check" : "warning") + E(asset.status) + '</span>';
  const fileIcon = (asset) => I(asset.type === "Interface" ? "image" : asset.type === "Code" ? "code" : "file");
  const currentAsset = () => assets.find((asset) => asset.id === state.activeId);
  function renderNavigation() {
    const sections = [["all", "grid", "All assets"], ["favorites", "star", "Favorites"], ["review", "warning", "Needs review"], ["archive", "folder", "Archive"]];
    document.getElementById("workspace-nav").innerHTML = sections.map((item) => {
      const count = assets.filter((asset) => item[0] === "archive" ? asset.archived : !asset.archived && (item[0] === "all" || item[0] === "favorites" && asset.favorite || item[0] === "review" && asset.status === "Review")).length;
      return '<button type="button" class="rail-link" data-section="' + item[0] + '"' + (!state.project && state.section === item[0] ? ' aria-current="page"' : '') + '>' + I(item[1]) + item[2] + '<span class="count">' + count + '</span></button>';
    }).join("");
    document.getElementById("project-nav").innerHTML = projects.map((project) => '<button type="button" class="rail-link" data-project="' + E(project) + '"' + (state.project === project ? ' aria-current="page"' : '') + '>' + I("folder") + '<span class="grow" style="overflow-wrap:anywhere">' + E(project) + '</span></button>').join("");
  }
  function renderSelection() {
    const visible = visibleAssets();
    document.querySelector(".browser-header").dataset.hasSelection = String(state.selected.size > 0);
    document.getElementById("selection-count").textContent = state.selected.size + " selected";
    document.getElementById("status-summary").textContent = assets.filter((asset) => !asset.archived).length + " active assets / " + projects.length + " projects / " + state.selected.size + " selected";
    const selectAll = document.getElementById("select-all");
    if (selectAll) {
      const count = visible.filter((asset) => state.selected.has(asset.id)).length;
      selectAll.checked = visible.length > 0 && count === visible.length;
      selectAll.indeterminate = count > 0 && count < visible.length;
    }
    document.querySelectorAll("#assets-view [data-select]").forEach((checkbox) => {
      const id = Number(checkbox.dataset.select);
      checkbox.checked = state.selected.has(id);
      checkbox.closest("tr").dataset.selected = String(id === state.activeId || state.selected.has(id));
    });
  }
  function renderBrowser() {
    const focusId = document.activeElement && document.activeElement.id;
    const visible = visibleAssets();
    document.getElementById("browser-title").textContent = state.project || ({ all: "All assets", favorites: "Favorites", review: "Needs review", archive: "Archive" })[state.section];
    document.getElementById("results-count").textContent = visible.length + " asset" + (visible.length === 1 ? "" : "s");
    let content;
    if (!visible.length) {
      content = '<div class="empty-state">' + I("folder") + '<strong>No matching assets</strong><button class="btn small" id="empty-clear" type="button">Clear filters</button></div>';
    } else if (state.view === "grid") {
      content = '<div class="asset-grid">' + visible.map((asset) => '<button type="button" class="asset-card" data-inspect="' + asset.id + '" id="asset-' + asset.id + '" aria-pressed="' + (asset.id === state.activeId) + '">' + D.thumbnail(asset.id === 1) + '<strong>' + E(asset.name) + '</strong><small>' + E(asset.project) + '</small>' + stateBadge(asset) + '</button>').join("") + '</div>';
    } else {
      content = '<table class="asset-table"><thead><tr><th scope="col"><label><span class="sr-only">Select all visible assets</span><input type="checkbox" id="select-all"></label></th><th scope="col">Name</th><th scope="col" class="optional">Type</th><th scope="col">Status</th><th scope="col" class="size-col">Size</th></tr></thead><tbody>' +
        visible.map((asset) => '<tr data-selected="' + (asset.id === state.activeId || state.selected.has(asset.id)) + '"><td><label><span class="sr-only">Select ' + E(asset.name) + '</span><input type="checkbox" id="select-' + asset.id + '" data-select="' + asset.id + '" ' + (state.selected.has(asset.id) ? 'checked' : '') + '></label></td><td><button type="button" class="asset-name" data-inspect="' + asset.id + '" id="asset-' + asset.id + '"' + (asset.id === state.activeId ? ' aria-current="true"' : '') + '><span class="file-icon">' + fileIcon(asset) + '</span><span><strong>' + E(asset.name) + '</strong><small>' + E(asset.project) + '</small></span></button></td><td class="optional muted mono">' + E(asset.format) + '</td><td>' + stateBadge(asset) + '</td><td class="size-col muted mono">' + E(asset.size) + '</td></tr>').join("") + '</tbody></table>';
    }
    document.getElementById("assets-view").innerHTML = content;
    renderSelection();
    if (focusId) {
      const target = document.getElementById(focusId);
      if (target && target.closest("#assets-view, #selection-strip")) target.focus({ preventScroll: true });
    }
  }
  function renderInspector() {
    const asset = currentAsset();
    const inspector = document.getElementById("asset-inspector");
    if (!asset) { inspector.innerHTML = '<div class="empty-state">' + I("file") + '<strong>No asset selected</strong></div>'; return; }
    inspector.innerHTML = '<div class="section-line"><h2>Details</h2><button class="btn ghost icon-only small" id="favorite-asset" type="button" aria-label="Favorite selected asset" aria-pressed="' + asset.favorite + '">' + I("star") + '</button></div>' +
      (asset.id === 1 ? D.preview("inspector-zoom") : '<div class="empty-state preview-unavailable">' + I("file") + '<strong>No preview available</strong></div>') + '<h3>' + E(asset.name) + '</h3><div class="tag-row" id="inspector-state">' + stateBadge(asset) + (asset.archived ? '<span class="badge">Archived</span>' : '') + '</div>' +
      D.copyField("Asset ID", "asset_" + String(asset.id).padStart(4, "0")) +
      '<dl class="property-list"><dt>Project</dt><dd>' + E(asset.project) + '</dd><dt>Format</dt><dd>' + E(asset.format) + '</dd><dt>Sample size</dt><dd>' + E(asset.size) + '</dd><dt>Revision</dt><dd>' + E(asset.revision) + '</dd></dl>' +
      '<div class="row"><button class="btn small" id="rename-asset" type="button">' + I("edit") + 'Rename</button><button class="btn small" id="archive-asset" type="button">' + I("folder") + (asset.archived ? 'Restore' : 'Archive') + '</button></div>' +
      '<label class="field"><span>Review status</span><select class="input" id="asset-status"><option' + (asset.status === "Ready" ? ' selected' : '') + '>Ready</option><option' + (asset.status === "Review" ? ' selected' : '') + '>Review</option></select></label>' +
      '<div class="tag-row">' + asset.tags.map((tag) => '<span class="badge">' + E(tag) + '</span>').join("") + '</div>' +
      '<label class="field"><span>Notes</span><textarea class="input" id="asset-note" maxlength="500">' + E(asset.note) + '</textarea></label><div class="row" style="margin-top:8px"><button class="btn small" id="save-note" type="button">Save note</button></div>';
    D.controls(inspector);
  }
  function renderAll() { renderNavigation(); renderBrowser(); renderInspector(); }
  function clearFilters() {
    state.query = ""; state.category = "all"; state.status = "all";
    document.getElementById("search-assets").value = "";
    document.getElementById("type-filter").value = "all";
    document.getElementById("status-filter").value = "all";
    renderBrowser();
  }
  function deleteSamples(ids) {
    D.dialog({
      title: "Delete " + ids.length + " sample asset" + (ids.length === 1 ? "?" : "s?"),
      description: "Only the in-memory fixture will change. No file on your computer will be deleted.",
      confirm: "Delete samples", danger: true, returnFocus: "#search-assets",
      onConfirm: () => {
        for (let index = assets.length - 1; index >= 0; index--) if (ids.includes(assets[index].id)) assets.splice(index, 1);
        ids.forEach((id) => state.selected.delete(id));
        if (ids.includes(state.activeId)) state.activeId = visibleAssets()[0]?.id || null;
        renderAll();
        D.toast("Samples deleted.");
      }
    });
  }
  document.getElementById("search-assets").oninput = (event) => { state.query = event.target.value; renderBrowser(); };
  document.getElementById("type-filter").onchange = (event) => { state.category = event.target.value; renderBrowser(); };
  document.getElementById("status-filter").onchange = (event) => { state.status = event.target.value; renderBrowser(); };
  ["list", "grid"].forEach((view) => document.getElementById("view-" + view).onclick = () => {
    state.view = view;
    ["list", "grid"].forEach((name) => document.getElementById("view-" + name).setAttribute("aria-pressed", String(view === name)));
    renderBrowser();
  });
  document.getElementById("workbench").addEventListener("click", (event) => {
    const sectionButton = event.target.closest("[data-section]");
    const projectButton = event.target.closest("[data-project]");
    const inspectButton = event.target.closest("[data-inspect]");
    if (sectionButton || projectButton) {
      state.section = sectionButton ? sectionButton.dataset.section : "all";
      state.project = projectButton ? projectButton.dataset.project : "";
      state.selected.clear();
      clearFilters();
      state.activeId = visibleAssets()[0]?.id || null;
      renderAll();
      document.getElementById("workbench").classList.remove("mobile-nav-open");
      document.getElementById("mobile-nav-toggle").setAttribute("aria-expanded", "false");
      const navigationVisible = getComputedStyle(document.querySelector(".app-rail")).display !== "none";
      const focusTarget = navigationVisible ? document.querySelector('.rail-link[aria-current="page"]') : document.getElementById("mobile-nav-toggle");
      if (focusTarget) focusTarget.focus();
    }
    if (inspectButton) { state.activeId = Number(inspectButton.dataset.inspect); renderBrowser(); renderInspector(); }
    if (event.target.closest("#clear-selection")) {
      state.selected.clear(); renderSelection();
      (document.getElementById("select-all") || document.getElementById("view-" + state.view)).focus();
    }
    if (event.target.closest("#delete-selection")) deleteSamples(Array.from(state.selected));
    if (event.target.closest("#empty-clear")) clearFilters();
  });
  document.getElementById("assets-view").addEventListener("change", (event) => {
    if (event.target.matches("[data-select]")) {
      const id = Number(event.target.dataset.select);
      if (event.target.checked) state.selected.add(id); else state.selected.delete(id);
    } else if (event.target.id === "select-all") {
      visibleAssets().forEach((asset) => { if (event.target.checked) state.selected.add(asset.id); else state.selected.delete(asset.id); });
    } else return;
    renderSelection();
  });
  document.getElementById("asset-inspector").addEventListener("click", (event) => {
    const asset = currentAsset();
    if (!asset) return;
    if (event.target.closest("#favorite-asset")) {
      asset.favorite = !asset.favorite; renderAll();
      document.getElementById("favorite-asset").focus();
    }
    if (event.target.closest("#save-note")) {
      asset.note = document.getElementById("asset-note").value;
      D.toast("Note saved.");
    }
    if (event.target.closest("#archive-asset")) {
      asset.archived = !asset.archived; state.selected.delete(asset.id);
      D.toast(asset.archived ? "Sample archived." : "Sample restored.");
      state.activeId = visibleAssets()[0]?.id || null; renderAll();
      document.getElementById("search-assets").focus();
    }
    if (event.target.closest("#rename-asset")) D.dialog({
      title: "Rename sample asset",
      content: '<label class="field"><span>Filename</span><input class="input" name="filename" value="' + E(asset.name) + '" maxlength="100" autofocus aria-describedby="dialog-error"></label>',
      confirm: "Rename", returnFocus: "#rename-asset",
      onConfirm: (data, error) => {
        const name = String(data.get("filename")).trim();
        if (!name) { error.textContent = "Enter a filename."; return false; }
        asset.name = name; renderAll(); D.toast("Sample filename updated.");
      }
    });
  });
  document.getElementById("asset-inspector").addEventListener("change", (event) => {
    if (event.target.id === "asset-status") {
      const asset = currentAsset();
      asset.status = event.target.value; renderNavigation(); renderBrowser();
      // Update the badge without replacing the form and losing an unsaved note.
      document.querySelector("#inspector-state .badge").outerHTML = stateBadge(asset);
    }
  });
  document.getElementById("new-project").onclick = () => D.dialog({
    title: "Create sample project",
    content: '<label class="field"><span>Project name</span><input class="input" name="project" maxlength="60" autofocus aria-describedby="dialog-error"></label>',
    confirm: "Create project", returnFocus: "#new-project",
    onConfirm: (data, error) => {
      const name = String(data.get("project")).trim();
      if (!name) { error.textContent = "Enter a project name."; return false; }
      if (projects.some((project) => project.toLowerCase() === name.toLowerCase())) { error.textContent = "A sample project already has this name."; return false; }
      projects.push(name); state.project = name; state.section = "all"; state.activeId = null; clearFilters(); renderAll();
      D.toast("Sample project created.");
    }
  });
  document.getElementById("add-asset").onclick = () => D.dialog({
    title: "Add sample asset", description: "Metadata only; no file is uploaded.",
    content: '<div class="stack"><label class="field"><span>Filename</span><input class="input" name="filename" placeholder="new-asset.svg" maxlength="100" autofocus aria-describedby="dialog-error"></label><label class="field"><span>Project</span><select class="input" name="project">' +
      projects.map((project) => '<option' + (state.project === project ? ' selected' : '') + '>' + E(project) + '</option>').join("") +
      '</select></label><label class="field"><span>Category</span><select class="input" name="type"><option>Interface</option><option>Documents</option><option>Code</option></select></label></div>',
    confirm: "Add sample", returnFocus: "#add-asset",
    onConfirm: (data, error) => {
      const name = String(data.get("filename")).trim();
      if (!name) { error.textContent = "Enter a filename."; return false; }
      const asset = { id: state.nextId++, name, type: String(data.get("type")), format: name.includes(".") ? name.split(".").pop().toUpperCase().slice(0, 8) : "FILE", size: "Sample", project: String(data.get("project")), status: "Review", revision: "r01", favorite: false, tags: ["sample"], note: "", archived: false };
      assets.unshift(asset); state.section = "all"; state.project = asset.project; state.activeId = asset.id; clearFilters(); renderAll();
      D.toast("Sample asset added.");
    }
  });
  document.getElementById("clear-filters").onclick = () => { document.querySelector("details.menu").open = false; clearFilters(); };
  document.getElementById("export-metadata").onclick = () => {
    document.querySelector("details.menu").open = false;
    const payload = { demo: true, theme: D.theme, description: "Illustrative metadata; no real asset files are included.", assets: visibleAssets() };
    const url = URL.createObjectURL(new Blob([JSON.stringify(payload, null, 2)], { type: "application/json" }));
    const link = document.createElement("a");
    link.href = url; link.download = D.theme + "-sample-metadata.json"; link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
    D.toast("Exported visible sample metadata as JSON.");
  };
  document.getElementById("mobile-nav-toggle").onclick = () => {
    const open = document.getElementById("workbench").classList.toggle("mobile-nav-open");
    document.getElementById("mobile-nav-toggle").setAttribute("aria-expanded", String(open));
  };
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && !document.querySelector("dialog[open]") && document.getElementById("workbench").classList.contains("mobile-nav-open")) {
      document.getElementById("workbench").classList.remove("mobile-nav-open");
      document.getElementById("mobile-nav-toggle").setAttribute("aria-expanded", "false");
      document.getElementById("mobile-nav-toggle").focus();
    }
  });
  renderAll();
})();
