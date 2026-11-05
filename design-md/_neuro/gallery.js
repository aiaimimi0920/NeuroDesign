/* Interactive atlas: all user-visible mutations stay in this tab. */
(() => {
  "use strict";
  const D = window.NeuroDemo;
  const icon = (name) => D.icon(name);
  const btn = (label, variant, attributes) => '<button type="button" class="btn ' + (variant || "") + '" ' + (attributes || "") + '>' + label + '</button>';
  const heading = (number, title) => '<div class="specimen-heading"><span>' + number + '</span><h2>' + title + '</h2></div>';
  const segments = (name, label, options, selected) => '<fieldset class="choice-group"><legend>' + label + '</legend><div class="segmented">' +
    options.map((option) => '<label class="segment"><input type="radio" name="' + name + '" value="' + option + '"' + (option === selected ? ' checked' : '') + '><span>' + option + '</span></label>').join('') + '</div></fieldset>';
  document.getElementById("app").innerHTML = D.previewBar("gallery") +
    '<main class="atlas" id="main-content" data-compact="' + D.compact + '"><header class="atlas-header"><div>' +
    '<h1>' + D.theme + '<span class="muted"> · Component atlas</span></h1></div>' +
    '<div class="density-label"><strong>' + (D.compact ? '24–28' : '28–36') + '<small> px</small></strong><span>Control heights</span><a href="DESIGN.md">English specification ↗</a></div></header>' +
    '<nav class="atlas-nav" aria-label="Component categories"><a href="#palette">01 Palette</a><a href="#actions">02 Actions</a><a href="#fields">03 Inputs</a><a href="#navigation">04 Navigation</a><a href="#data">05 Data</a><a href="#feedback">06 Feedback</a><a href="#overlays">07 Overlays</a></nav>' +
    '<section class="specimen" id="palette">' + heading("01", "Palette") +
    '<div class="swatches">' +
    [['#d9ff38', 'Signal yellow', 'Active · focus · primary'], ['#22c55e', 'Signal green', 'Brand · online · complete'], ['#06b6d4', 'Information', 'Links · technical state'], ['#f43f5e', 'Danger', 'Errors · destructive actions']].map((item) =>
      '<div class="swatch"><i style="--swatch:' + item[0] + '"></i><strong>' + item[1] + '</strong><code>' + item[0] + '</code><small>' + item[2] + '</small></div>').join("") +
    '</div><div class="neutral-strip">' +
    [['var(--bg)', 'Canvas'], ['var(--rail)', 'Rail'], ['var(--surface)', 'Surface'], ['var(--panel)', 'Panel'], ['var(--control)', 'Control'], ['var(--control-hover)', 'Hover']].map((item) => '<span style="--swatch:' + item[0] + '">' + item[1] + '</span>').join("") +
    '</div></section>' +
    '<section class="specimen" id="actions">' + heading("02", "Actions") +
    '<div class="specimen-grid"><div class="panel"><div class="specimen-label">Action hierarchy <span>' + (D.compact ? '28px · square' : '36px · 4px radius') + '</span></div><div class="row">' +
    btn(icon("plus") + 'Create project', 'primary', 'data-demo-toast="Primary action preview — no project service is connected."') +
    btn('Secondary', '', 'data-demo-toast="Secondary action selected."') + btn('Quiet action', 'ghost', 'data-demo-toast="Quiet action selected."') +
    '</div><div class="row">' + btn(icon("check") + 'Complete', 'positive', 'data-demo-toast="Completion feedback preview."') +
    btn(icon("trash") + 'Delete', 'destructive', 'id="delete-demo"') + btn('Unavailable', '', 'disabled') + '</div></div>' +
    '<div class="panel"><div class="specimen-label">Compact & icon actions</div><div class="row">' +
    btn('Small action', 'small', 'data-demo-toast="Compact action selected."') +
    btn(icon("pin"), 'icon-only tip', 'data-tip="Pin selection" aria-label="Pin selection" aria-pressed="false" data-toggle') +
    btn(icon("star"), 'icon-only tip', 'data-tip="Add to favorites" aria-label="Add to favorites" aria-pressed="true" data-toggle') +
    btn(icon("check") + 'Process sample', '', 'id="process-sample"') + '<span class="sr-only" id="process-result" role="status"></span>' +
    '</div>' +
    (D.compact ? '<div class="specimen-label">Floating toolbar · 9px / 5px variant</div><div class="otter-strip">' +
      btn(icon("edit"), '', 'aria-label="Pen tool" aria-pressed="true" data-tool') + btn(icon("image"), '', 'aria-label="Image tool" aria-pressed="false" data-tool') +
      '<span class="divider"></span>' + btn(icon("pin"), '', 'aria-label="Pin tool" aria-pressed="false" data-tool') + btn(icon("settings"), '', 'aria-label="Properties tool" aria-pressed="false" data-tool') + '</div>' : '') +
    '</div></div></section>' +
    '<section class="specimen" id="fields">' + heading("03", "Inputs & selection") +
    '<div class="specimen-grid three"><div class="panel"><div class="specimen-label">Text fields</div><label class="field"><span>Project name</span><input class="input" id="project-name" placeholder="e.g. Interface studies" maxlength="80"></label>' +
    '<label class="field"><span>Invalid example</span><input class="input" value="not an email" aria-invalid="true" aria-describedby="invalid-help"><small class="error" id="invalid-help">Enter a valid email address.</small></label>' +
    '<label class="field"><span>Read-only value</span><input class="input mono" value="Interface studies" readonly></label>' + D.copyField("Project ID", "project_0042") + '</div>' +
    '<div class="panel"><div class="specimen-label">Choice controls</div>' +
    '<label class="check"><input type="checkbox" checked id="check-demo">Include archived assets</label>' +
    '<label class="check"><input type="checkbox" checked disabled>Managed selection</label>' +
    '<fieldset style="border:0;padding:0;margin:0" class="stack"><legend class="sr-only">Default view</legend><label class="check"><input type="radio" name="default-view" value="list" checked>List view</label><label class="check"><input type="radio" name="default-view" value="grid">Grid view</label></fieldset>' +
    '<label class="switch"><input type="checkbox" role="switch" checked id="switch-demo">Show metadata</label><label class="switch"><input type="checkbox" role="switch" disabled>Unavailable setting</label></div>' +
    '<div class="panel"><div class="specimen-label">Values & longer text</div><label class="field" for="scale-demo"><span>Preview scale <output id="scale-value" for="scale-demo">75%</output></span><input type="range" id="scale-demo" min="25" max="100" value="75"></label>' +
    D.stepper("columns-demo", "Columns", 4, 1, 12, 1, "") +
    D.stepper("stroke-demo", "Stroke width", 1.25, 0.25, 4, 0.25, "px") +
    '<label class="field"><span>Notes</span><textarea class="input" placeholder="Add a short description…"></textarea></label>' +
    '<label class="field"><span>Unavailable field</span><input class="input" value="Managed by the application" disabled></label></div></div>' +
    '<div class="specimen-grid selection-specimens"><div class="panel" id="single-choice-specimens"><div class="specimen-label">Single selection</div>' +
    segments("notifications", "Notifications", ["Off", "On"], "Off") +
    segments("scale-choice", "Scale", ["100%", "110%", "125%", "150%"], "110%") +
    '<fieldset class="choice-group"><legend>Managed mode</legend><div class="segmented"><label class="segment"><input type="radio" name="managed-mode" checked disabled><span>Auto</span></label><label class="segment"><input type="radio" name="managed-mode" disabled><span>Manual</span></label></div></fieldset>' +
    '<label class="field"><span>Asset category</span><select class="input" id="category-demo"><option>All assets</option><option>Interface</option><option>Documents</option><option>Code</option><option>Images</option><option>Audio</option><option>Video</option><option>Archives</option><option>Fonts</option></select></label></div>' +
    '<div class="panel" id="multiple-choice-specimens"><div class="specimen-label">Multiple selection</div>' +
    '<fieldset class="choice-group"><legend>Access methods</legend><div class="choice-options" id="methods-inline">' +
    ["oauth", "api", "web"].map((value) => '<label class="check"><input type="checkbox" name="access-methods" value="' + value + '" checked>' + value + '</label>').join('') + '</div></fieldset>' +
    '<fieldset class="choice-group"><legend>Export formats</legend><div class="choice-options vertical" id="formats-vertical">' +
    ["PNG", "SVG", "PDF"].map((value, index) => '<label class="check"><input type="checkbox" name="export-formats" value="' + value + '"' + (index === 0 ? ' checked' : '') + '>' + value + '</label>').join('') + '</div></fieldset>' +
    '<fieldset class="choice-group"><legend id="capabilities-label">Capabilities</legend><details class="menu multi-select" id="capabilities-select"><summary class="input multi-select-trigger" aria-labelledby="capabilities-label capabilities-summary" aria-controls="capabilities-options"><span id="capabilities-summary" data-selection-summary></span>' + icon("chevron") + '</summary><div class="menu-panel" id="capabilities-options">' +
    ["oauth", "api", "web", "desktop", "automation", "sharing", "versioning", "export"].map((value, index) => '<label class="check"><input type="checkbox" name="capabilities" value="' + value + '"' + (index < 3 ? ' checked' : '') + '>' + value + '</label>').join('') + '</div></details></fieldset></div></div></section>' +
    '<section class="specimen" id="navigation">' + heading("04", "Navigation & disclosure") +
    '<div class="specimen-grid"><div class="panel"><div class="specimen-label">Navigation rail</div><nav class="sample-nav" aria-label="Sample workspace">' +
    btn(icon("grid") + 'Overview', '', 'aria-pressed="true" data-nav') + btn(icon("folder") + 'Assets', 'ghost', 'aria-pressed="false" data-nav') +
    btn(icon("clock") + 'Activity', 'ghost', 'aria-pressed="false" data-nav') + '</nav></div>' +
    '<div class="panel" id="sample-tabs"><div class="specimen-label">Tabs & compact segments</div><div class="tabs" role="tablist" aria-label="Asset information">' +
    '<button type="button" class="tab" role="tab" id="tab-overview" aria-selected="true" aria-controls="panel-overview">Overview</button>' +
    '<button type="button" class="tab" role="tab" id="tab-properties" aria-selected="false" aria-controls="panel-properties" tabindex="-1">Properties</button>' +
    '<button type="button" class="tab" role="tab" id="tab-history" aria-selected="false" aria-controls="panel-history" tabindex="-1">History</button></div>' +
    '<div class="tab-content" id="panel-overview" role="tabpanel" aria-labelledby="tab-overview" tabindex="0">Interface studies</div>' +
    '<div class="tab-content" id="panel-properties" role="tabpanel" aria-labelledby="tab-properties" tabindex="0" hidden><code>Format: SVG · Size: 24 KB</code></div>' +
    '<div class="tab-content" id="panel-history" role="tabpanel" aria-labelledby="tab-history" tabindex="0" hidden><span class="badge info">' + icon("clock") + 'Sample revision 03</span></div>' +
    segments("sample-view", "View", ["List", "Grid"], "List") + '</div></div></section>' +
    '<section class="specimen" id="data">' + heading("05", "Content & data") +
    '<div class="specimen-grid"><div class="panel"><div class="specimen-label">Selectable rows</div><div class="table-wrap"><table class="specimen-table"><thead><tr><th scope="col">Asset</th><th scope="col">Type</th><th scope="col">State</th></tr></thead><tbody>' +
    [['interface-kit.svg', 'SVG', 'Ready'], ['project-notes.md', 'Markdown', 'Review'], ['tokens.css', 'CSS', 'Ready']].map((item, index) =>
      '<tr class="' + (index === 0 ? 'selected' : '') + '"><td><label class="check"><input type="checkbox" data-row-select ' + (index === 0 ? 'checked' : '') + '>' + item[0] + '</label></td><td class="muted mono">' + item[1] + '</td><td><span class="badge ' + (index === 1 ? 'warn' : 'ok') + '">' + item[2] + '</span></td></tr>').join("") +
    '</tbody></table></div></div>' +
    '<div class="panel"><div class="specimen-label">Panel & disclosure</div><h3>Interface studies</h3>' +
    '<details class="accordion"><summary>Advanced properties ' + icon("chevron") + '</summary><label class="check"><input type="checkbox">Keep original filenames</label></details>' +
    '<details class="accordion"><summary>Keyboard shortcuts ' + icon("chevron") + '</summary><p><kbd>Tab</kbd> Move focus · <kbd>Esc</kbd> Close overlays</p></details></div></div>' +
    '<div class="specimen-grid selection-specimens"><div class="panel"><div class="specimen-label">Long table <span>16 sample revisions</span></div><div class="table-wrap long-table" tabindex="0" role="region" aria-label="Sample revision history"><table class="specimen-table"><thead><tr><th scope="col">Revision</th><th scope="col">Size</th><th scope="col">State</th></tr></thead><tbody>' +
    Array.from({ length: 16 }, (_, index) => '<tr><td class="mono">r' + String(16 - index).padStart(2, "0") + '</td><td class="mono">' + (24 + index) + ' KB</td><td><span class="badge ' + (index === 0 ? 'warn' : 'ok') + '">' + (index === 0 ? 'Review' : 'Ready') + '</span></td></tr>').join("") +
    '</tbody></table></div></div><div class="panel"><div class="specimen-label">Content preview</div>' + D.preview("gallery-zoom") + '</div></div></section>' +
    '<section class="specimen" id="feedback">' + heading("06", "State & feedback") +
    '<div class="specimen-grid"><div class="panel"><div class="row"><span class="badge ok"><i class="dot"></i>Ready</span><span class="badge warn">' + icon("warning") + 'Needs review</span><span class="badge info">' + icon("info") + 'Local sample</span><span class="badge error">' + icon("close") + 'Failed</span><span class="badge">Disabled</span></div>' +
    '<div class="notice ok">' + icon("check") + '<span>Sample updated.</span></div><div class="notice warn">' + icon("warning") + '<span>3 assets need review.</span></div><div class="notice error">' + icon("close") + '<span>Filename is required.</span></div></div>' +
    '<div class="panel"><div class="specimen-label">Progress & empty state</div><label class="field" for="progress-demo"><span>Sample progress <output id="progress-value" for="progress-demo">64%</output></span><progress id="progress-demo" value="64" max="100">64%</progress></label>' +
    btn('Advance sample progress', 'small', 'id="advance-progress"') + '<div class="empty-state">' + icon("folder") + '<strong>No matching assets</strong></div></div></div>' +
    '<div class="panel sample-comparison"><div class="specimen-label">Sample comparison</div><dl><div><dt>Storage</dt><dd>28 GB <span class="neutral-delta">+2 GB vs 26 GB</span></dd></div><div><dt>Latency</dt><dd>84 ms <span class="neutral-delta">−12 ms vs 96 ms</span></dd></div></dl></div></section>' +
    '<section class="specimen" id="overlays">' + heading("07", "Overlays") +
    '<div class="panel"><div class="row">' + btn(icon("plus") + 'Open dialog', '', 'id="dialog-demo"') +
    '<details class="menu"><summary class="btn">Asset actions ' + icon("chevron") + '</summary><div class="menu-panel">' +
    '<button type="button" data-demo-toast="Rename is demonstrated in the website example.">' + icon("edit") + 'Rename</button><button type="button" data-demo-toast="Duplicate action preview.">' + icon("copy") + 'Duplicate</button><button type="button" disabled>' + icon("folder") + 'Move to project</button><hr><button type="button" class="danger" id="menu-delete">' + icon("trash") + 'Delete…</button></div></details>' +
    btn('Show notification', '', 'id="toast-demo"') + btn(icon("info"), 'icon-only tip', 'data-tip="A small contextual tooltip" aria-label="Tooltip example: contextual information"') +
    '</div></div></section>' +
    '<footer class="atlas-footer"><span><a href="DESIGN.md">English</a> · <a href="DESIGN.zh-CN.md">中文</a> · <a href="README.md">Preview notes</a> · <a href="example.html">Open the website example ↗</a></span></footer></main>';

  D.setup();
  document.getElementById("process-sample").onclick = (event) => {
    const button = event.currentTarget;
    button.disabled = true;
    button.setAttribute("aria-busy", "true");
    button.innerHTML = '<span class="spinner" aria-hidden="true"></span>Process sample';
    document.getElementById("process-result").textContent = "Processing sample.";
    setTimeout(() => {
      button.disabled = false;
      button.removeAttribute("aria-busy");
      button.innerHTML = icon("check") + 'Process sample';
      document.getElementById("process-result").textContent = "Sample processing complete.";
    }, 900);
  };
  D.tabs(document.getElementById("sample-tabs"));
  document.querySelectorAll("[data-demo-toast]").forEach((button) => button.addEventListener("click", () => {
    D.toast(button.dataset.demoToast);
    const menu = button.closest("details.menu");
    if (menu) menu.open = false;
  }));
  document.querySelectorAll("[data-toggle]").forEach((button) => button.addEventListener("click", () => button.setAttribute("aria-pressed", String(button.getAttribute("aria-pressed") !== "true"))));
  ["data-tool", "data-nav"].forEach((attribute) => {
    document.querySelectorAll("[" + attribute + "]").forEach((button) => button.addEventListener("click", () => {
      button.parentElement.querySelectorAll("[" + attribute + "]").forEach((item) => item.setAttribute("aria-pressed", String(item === button)));
    }));
  });
  document.querySelectorAll("[data-row-select]").forEach((checkbox) => checkbox.addEventListener("change", () => checkbox.closest("tr").classList.toggle("selected", checkbox.checked)));
  document.getElementById("scale-demo").addEventListener("input", (event) => document.getElementById("scale-value").value = event.target.value + "%");
  document.getElementById("advance-progress").onclick = () => {
    const progress = document.getElementById("progress-demo");
    progress.value = progress.value >= 100 ? 0 : Math.min(100, progress.value + 12);
    progress.textContent = progress.value + "%";
    document.getElementById("progress-value").value = progress.value + "%";
  };
  const deletion = () => D.dialog({
    title: "Delete sample asset?",
    description: "This is a destructive-action specimen. It will not delete any file.",
    confirm: "Delete sample", danger: true,
    onConfirm: () => D.toast("Destructive confirmation completed. No file was deleted.")
  });
  document.getElementById("delete-demo").onclick = deletion;
  document.getElementById("menu-delete").onclick = () => {
    document.getElementById("menu-delete").closest("details.menu").open = false;
    deletion();
  };
  document.getElementById("dialog-demo").onclick = () => D.dialog({
    title: "Create a sample project",
    content: '<label class="field"><span>Project name</span><input class="input" name="project" maxlength="80" autofocus aria-describedby="dialog-error"></label>',
    confirm: "Create sample",
    onConfirm: (data, error) => {
      const name = String(data.get("project")).trim();
      if (!name) { error.textContent = "Enter a project name."; return false; }
      D.toast('Sample project "' + name + '" created in this tab.');
    }
  });
  document.getElementById("toast-demo").onclick = () => D.toast("Notification preview.");
})();
