// Dependency-free contract checks for the local Neuro specifications and previews.
const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const root = path.resolve(__dirname, '..');
const themes = ['neuro', 'neuro-mini'];
const shared = ['base.css', 'gallery.css', 'workbench.css', 'entry.css', 'common.js', 'gallery.js', 'workbench.js', 'SOURCES.md', 'samples/interface-kit.svg'];
const files = ['README.md', 'CONTRIBUTING.md', 'neuro.html', '.github/ISSUE_TEMPLATE/design-md-request.yml', ...shared.map(name => 'design-md/_neuro/' + name),
  ...themes.flatMap(theme => ['DESIGN.md', 'DESIGN.zh-CN.md', 'README.md', 'index.html', 'example.html', 'theme.css'].map(name => 'design-md/' + theme + '/' + name))];
const read = name => fs.readFileSync(path.join(root, name), 'utf8');

test('all deliverables are strict UTF-8 without BOM', () => {
  for (const name of [...files, 'tests/neuro-preview.test.cjs']) {
    const bytes = fs.readFileSync(path.join(root, name));
    assert.notEqual(bytes.subarray(0, 3).toString('hex'), 'efbbbf', name);
    assert.doesNotThrow(() => new TextDecoder('utf-8', { fatal: true }).decode(bytes), name);
  }
});

test('every static HTML and Markdown local link resolves', () => {
  for (const name of files.filter(name => /\.(html|md)$/.test(name))) {
    const text = read(name);
    const targets = name.endsWith('.html')
      ? Array.from(text.matchAll(/(?:href|src)="([^"]+)"/g), match => match[1])
      : Array.from(text.matchAll(/\]\(([^)]+)\)/g), match => match[1]);
    for (const target of targets) {
      if (/^(?:[a-z]+:|#)/i.test(target)) continue;
      const file = target.split('#')[0];
      assert.ok(fs.existsSync(path.resolve(root, path.dirname(name), file)), name + ' → ' + target);
    }
  }
});

for (const theme of themes) {
  test(theme + ': English and Chinese specifications preserve the same contract', () => {
    const english = read('design-md/' + theme + '/DESIGN.md');
    const chinese = read('design-md/' + theme + '/DESIGN.zh-CN.md');
    const frontmatter = text => text.split('---')[1].split('\n').filter(line => !line.startsWith('description:')).join('\n');
    assert.equal(frontmatter(english), frontmatter(chinese));
    const colors = text => Array.from(new Set(text.match(/#[0-9a-f]{6}\b/gi))).sort();
    assert.deepEqual(colors(english), colors(chinese));
    assert.equal((english.match(/^## \d+\./gm) || []).length, 12);
    assert.equal((chinese.match(/^## \d+\./gm) || []).length, 12);
    assert.equal((english.match(/^- \[ \]/gm) || []).length, (chinese.match(/^- \[ \]/gm) || []).length);
    assert.match(english, /\*\*No decorative English headings\.\*\*/);
    assert.match(english, /\*\*No explanatory copy by default\.\*\*/);
    assert.match(chinese, /\*\*不使用装饰性英文标题。\*\*/);
    assert.match(chinese, /\*\*默认不添加解释性文案。\*\*/);
    for (const text of [english, chinese]) {
      assert.match(text, /prefers-reduced-motion/);
      assert.match(text, /#d9ff38/);
      assert.match(text, /#22c55e/);
    }
  });

  test(theme + ': specification is standalone, product-neutral, and free of provenance or sibling-theme dependencies', () => {
    for (const language of ['DESIGN.md', 'DESIGN.zh-CN.md']) {
      const text = read('design-md/' + theme + '/' + language);
      assert.doesNotMatch(text, /Gateway|Loom|Otter|SOURCES\.md|README\.md|index\.html|example\.html|theme\.css|_neuro|feat\/windows-native/i);
      assert.doesNotMatch(text, /\b20\d\d-\d\d-\d\d\b|implementation-led|source-grounded|user-approved|historical prompts|control plane|操作中台|来源|提炼|调查|获批|审批|重构分支/i);
      assert.doesNotMatch(text, /\]\([^)]*\)/, 'design rules need no companion files');
      const withoutOwnName = text.replaceAll(theme, 'selected-theme');
      assert.doesNotMatch(withoutOwnName, /\bneuro(?:-mini)?\b/i, 'no other theme named');
      assert.match(text, /Theme definition|主题定义/);
      assert.match(text, /does not prescribe|不限定/);
      assert.match(text, /business fields|业务字段/);
      assert.match(text, /0\.25|step|步长/);
    }
    const notes = read('design-md/' + theme + '/README.md');
    for (const target of ['DESIGN.md', 'DESIGN.zh-CN.md', 'index.html', 'example.html']) assert.ok(notes.includes('](' + target + ')'));
    assert.match(notes, /1–12/);
    assert.match(notes, /Clipboard API/);
  });

  test(theme + ': pages load classic scripts and local styles only', () => {
    for (const name of ['index.html', 'example.html']) {
      const html = read('design-md/' + theme + '/' + name);
      assert.match(html, /<meta charset="utf-8">/);
      assert.match(html, /<noscript>/);
      assert.doesNotMatch(html, /type="module"|https?:\/\//);
      if (theme === 'neuro') {
        assert.match(html, /src="preview\.js"/);
        assert.match(html, /href="preview\.css"/);
        assert.doesNotMatch(html, /_neuro\/(?:common|gallery|workbench)\.js/);
      } else {
        assert.match(html, /src="\.\.\/_neuro\/common\.js"/);
        assert.match(html, /href="theme\.css"/);
      }
      assert.match(html, new RegExp('data-theme="' + theme + '"'));
    }
  });
}

test('shared browser scripts parse without framework or network dependencies', () => {
  for (const name of ['common.js', 'gallery.js', 'workbench.js']) {
    const text = read('design-md/_neuro/' + name);
    assert.doesNotThrow(() => new vm.Script(text, { filename: name }));
    assert.doesNotMatch(text, /\b(?:fetch|XMLHttpRequest|WebSocket|localStorage|sessionStorage)\b/);
    assert.doesNotMatch(text, /https?:\/\//);
  }
});

test('repository entry points describe only the maintained themes, without old collection or source labels', () => {
  const readme = read('README.md');
  assert.doesNotMatch(readme, /getdesign\.md|cdn\.voltagent|Collection|curated collection|DESIGN\.md%20count|Gateway|Loom|Otter/i);
  for (const theme of themes) assert.ok(readme.includes('design-md/' + theme + '/DESIGN.md'));
  assert.doesNotMatch(read('.github/ISSUE_TEMPLATE/design-md-request.yml'), /getdesign\.md|Delivery Email|Website URL/);
  const ui = read('neuro.html') + read('design-md/_neuro/common.js') + read('design-md/_neuro/gallery.js');
  assert.doesNotMatch(ui, /Gateway|Loom|Otter|Source-grounded|Evidence & scope|Compare /);
  const D = primitives();
  for (const theme of themes) {
    D.theme = theme; D.compact = theme === 'neuro-mini';
    const navigation = D.previewBar('gallery');
    const other = theme === 'neuro' ? 'neuro-mini' : 'neuro';
    assert.doesNotMatch(navigation, new RegExp('href="\\.\\./' + other + '/'));
  }
});

test('interface copy stays task-focused without decorative headings or redundant explanations', () => {
  const ui = ['common.js', 'gallery.js', 'workbench.js'].map(name => read('design-md/_neuro/' + name)).join('\n') + read('neuro.html');
  assert.doesNotMatch(ui, /PERSONAL \/ DESIGN OPERATIONS|Keep the work visible|NEURO FAMILY|ASSET WORKSPACE|SAMPLE \/ UI|class="kicker"/);
  assert.doesNotMatch(ui, /A visible label, even when|Try Tab to see|Select an asset to inspect|Illustrative preview ·|This tab only/);
  for (const name of ['base.css', 'gallery.css', 'workbench.css', 'entry.css']) {
    assert.doesNotMatch(read('design-md/_neuro/' + name), /text-transform:\s*uppercase|\.kicker\b/);
  }
});

test('palette, control density, and floating-toolbar variant remain explicit', () => {
  const palette = read('design-md/_neuro/base.css');
  for (const color of ['#d9ff38', '#22c55e', '#06b6d4', '#f43f5e', '#06080d', '#f7f8ef']) assert.ok(palette.includes(color));
  assert.match(palette, /prefers-reduced-motion/);
  assert.match(palette, /forced-colors/);
  assert.match(palette, /\.toast-stack \{[^}]*pointer-events: none/s);
  assert.match(palette, /\.toast button \{[^}]*pointer-events: auto/s);
  assert.match(read('design-md/neuro/theme.css'), /--radius-control: 4px/);
  assert.match(read('design-md/neuro/theme.css'), /--control-height: 36px/);
  assert.match(read('design-md/neuro-mini/theme.css'), /--radius-control: 0px/);
  assert.match(read('design-md/neuro-mini/theme.css'), /--control-height: 28px/);
  assert.match(read('design-md/_neuro/gallery.css'), /\.otter-strip \{[^}]*border-radius: 9px/s);
});

test('selection styles do not change the geometry of rows or segmented controls', () => {
  const geometry = /(?:^|;)\s*(?:padding(?:-[\w-]+)?|margin(?:-[\w-]+)?|border(?:-(?:width|style|left|right|top|bottom))?|font(?:-[\w-]+)?|(?:min-|max-)?(?:width|height)|display|grid-[\w-]+)\s*:/;
  for (const name of ['base.css', 'gallery.css', 'workbench.css']) {
    const css = read('design-md/_neuro/' + name);
    const selected = Array.from(css.matchAll(/([^{}]+)\{([^{}]*)\}/g))
      .filter(([, selector]) => /\.selected|\[data-selected="true"\]|\.segment input:checked/.test(selector));
    assert.ok(selected.length, name + ': selected styles exist');
    for (const [, selector, body] of selected) assert.doesNotMatch(body, geometry, name + ': ' + selector.trim());
  }
  assert.match(read('design-md/_neuro/base.css'), /\.segment span \{[^}]*border: 1px solid transparent/s);
  assert.match(read('design-md/_neuro/workbench.css'), /\.browser-summary, \.selection-strip \{[^}]*grid-area: 1 \/ 1/s);
  assert.match(read('design-md/_neuro/workbench.css'), /\.selection-strip \{[^}]*visibility: hidden/s);
});

test('selection updates existing checkboxes without replacing table DOM', () => {
  const source = read('design-md/_neuro/workbench.js');
  const selection = source.slice(source.indexOf('function renderSelection()'), source.indexOf('function renderBrowser()'));
  const header = { dataset: {} }, count = {}, status = {}, all = {};
  const rows = [1, 2, 3].map(id => ({ dataset: { select: String(id) }, row: { dataset: {} }, closest() { return this.row; } }));
  const state = { activeId: 1, selected: new Set([2]) };
  const elements = { 'selection-count': count, 'status-summary': status, 'select-all': all };
  const document = {
    querySelector(selector) { assert.equal(selector, '.browser-header'); return header; },
    getElementById(id) { assert.ok(id in elements, id); return elements[id]; },
    querySelectorAll(selector) { assert.equal(selector, '#assets-view [data-select]'); return rows; }
  };
  const context = vm.createContext({ document, state, visibleAssets: () => [{ id: 1 }, { id: 2 }, { id: 3 }], assets: [{ archived: false }], projects: ['Sample'] });
  vm.runInContext(selection + ';renderSelection();', context);
  assert.equal(header.dataset.hasSelection, 'true');
  assert.equal(count.textContent, '1 selected');
  assert.equal(status.textContent, '1 active assets / 1 projects / 1 selected');
  assert.deepEqual(rows.map(row => row.checked), [false, true, false]);
  assert.deepEqual(rows.map(row => row.row.dataset.selected), ['true', 'true', 'false']);
  assert.equal(all.checked, false);
  assert.equal(all.indeterminate, true);
  state.selected = new Set([1, 2, 3]);
  vm.runInContext('renderSelection();', context);
  assert.equal(all.checked, true);
  assert.equal(all.indeterminate, false);
  state.selected.clear();
  vm.runInContext('renderSelection();', context);
  assert.equal(header.dataset.hasSelection, 'false');
  assert.equal(all.checked, false);
  assert.equal(all.indeterminate, false);
  assert.deepEqual(rows.map(row => row.checked), [false, false, false]);
  assert.doesNotMatch(selection, /innerHTML|replaceChildren|renderBrowser\(/);
  assert.match(source, /state\.selected\.clear\(\); renderSelection\(\)/);
  assert.match(source, /else return;\s*renderSelection\(\);/);
});

test('selection patterns use native semantics and appear in all four specifications', () => {
  const gallery = read('design-md/_neuro/gallery.js');
  assert.match(gallery, /class="segment"><input type="radio" name=/);
  assert.match(gallery, /segments\("notifications", "Notifications", \["Off", "On"\]/);
  assert.match(gallery, /segments\("scale-choice", "Scale", \["100%", "110%", "125%", "150%"\]/);
  assert.match(gallery, /<select class="input" id="category-demo">/);
  for (const pattern of ['choice-options', 'choice-options vertical', 'menu multi-select', 'data-selection-summary', 'name="capabilities"']) assert.ok(gallery.includes(pattern), pattern);
  assert.doesNotMatch(gallery, /role="(?:listbox|option)"/);
  const common = read('design-md/_neuro/common.js');
  assert.match(common, /menu\.addEventListener\("change", update\)/);
  assert.match(common, /menu\.addEventListener\("toggle", place\)/);
  for (const theme of themes) {
    assert.match(read('design-md/' + theme + '/DESIGN.md'), /Choosing selection controls/);
    assert.match(read('design-md/' + theme + '/DESIGN.zh-CN.md'), /选择控件的取舍/);
    for (const language of ['DESIGN.md', 'DESIGN.zh-CN.md']) {
      const text = read('design-md/' + theme + '/' + language);
      assert.match(text, /radio/);
      assert.match(text, /select/);
      assert.match(text, /padding/);
      assert.match(text, /Escape/);
    }
  }
});

function primitives(navigator = {}) {
  const window = {};
  const context = vm.createContext({ window, document: { body: { dataset: { theme: 'neuro' } } }, navigator, clearTimeout() {}, setTimeout() { return 1; } });
  vm.runInContext(read('design-md/_neuro/common.js'), context);
  return window.NeuroDemo;
}

test('numeric commits clamp, snap from min, retain the previous value when empty, and avoid decimal drift', () => {
  const D = primitives();
  for (const [input, expected] of [['', 4], ['invalid', 4], ['-3', 1], ['999', 12], ['9.6', 10], ['12', 12]]) {
    assert.equal(D.numberValue(input, 1, 12, 1, 4), expected, input);
  }
  for (const [input, expected] of [['1.13', 1.25], ['0', 0.25], ['9', 4], ['', 1.25]]) {
    assert.equal(D.numberValue(input, 0.25, 4, 0.25, 1.25), expected, input);
  }
  let value = 0.1;
  for (let index = 0; index < 8; index++) value = D.numberValue(value + 0.1, 0.1, 2, 0.1, value);
  assert.equal(value, 0.9);
  const html = D.stepper('weight', 'Weight', 1.25, 0.25, 4, 0.25, 'px');
  assert.match(html, /type="number"[^>]*min="0.25" max="4" step="0.25" required/);
  assert.match(html, /aria-label="Weight \(px\)"/);
});

test('copy feedback reflects the actual clipboard promise, blocks duplicate writes, and handles rejection', async () => {
  let resolve, writes = [];
  const clipboard = { writeText(value) { writes.push(value); return new Promise(done => { resolve = done; }); } };
  const D = primitives({ clipboard });
  const message = { textContent: '' };
  const field = { dataset: {}, querySelector() { return message; } };
  const attrs = {};
  const button = { dataset: { copyValue: 'asset_0001' }, closest() { return field; },
    getAttribute(name) { return attrs[name]; }, setAttribute(name, value) { attrs[name] = value; }, removeAttribute(name) { delete attrs[name]; } };
  const pending = D.copy(button);
  assert.equal(field.dataset.state, 'copying');
  assert.equal(message.textContent, '');
  await D.copy(button);
  assert.deepEqual(writes, ['asset_0001']);
  resolve(); await pending;
  assert.equal(field.dataset.state, 'copied');
  assert.equal(message.textContent, 'Copied to clipboard.');
  assert.equal(attrs['aria-busy'], undefined);
  clipboard.writeText = async () => { throw new Error('Permission denied'); };
  await D.copy(button);
  assert.equal(field.dataset.state, 'error');
  assert.match(message.textContent, /Copy failed/);
  assert.doesNotMatch(button.innerHTML, /m3 8 3 3 7-7/);
  const unsupported = primitives();
  await unsupported.copy(button);
  assert.equal(field.dataset.state, 'error');
  assert.match(D.copyField('Asset ID', '<not a secret>'), /&lt;not a secret&gt;/);
});

test('new previews use a local untransformed fixture and real scrollable sticky headers', () => {
  const D = primitives();
  const preview = D.preview('zoom');
  assert.match(preview, /src="\.\.\/_neuro\/samples\/interface-kit.svg" width="640" height="400"/);
  assert.match(preview, /tabindex="0" role="region"/);
  assert.match(read('design-md/_neuro/samples/interface-kit.svg'), /viewBox="0 0 640 400"/);
  assert.doesNotMatch(read('design-md/_neuro/workbench.css'), /rotate\(/);
  assert.match(read('design-md/_neuro/workbench.css'), /#assets-view \{[^}]*overflow: auto/s);
  assert.match(read('design-md/_neuro/workbench.css'), /\.asset-table th \{[^}]*position: sticky[^}]*background: var\(--surface\)/s);
  assert.match(read('design-md/_neuro/gallery.css'), /\.long-table th \{[^}]*position: sticky[^}]*background: var\(--panel\)/s);
  assert.match(read('design-md/_neuro/base.css'), /\.neutral-delta \{[^}]*color: var\(--muted\)/s);
  const gallery = read('design-md/_neuro/gallery.js');
  assert.match(gallery, /button\.disabled = true;\s*button\.setAttribute\("aria-busy", "true"\)/);
  assert.match(read('design-md/_neuro/base.css'), /\.segment input:disabled:checked \+ span \{[^}]*box-shadow:/s);
});
