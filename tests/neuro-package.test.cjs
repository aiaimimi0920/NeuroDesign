const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const { spawnSync } = require('node:child_process');
const root = path.resolve(__dirname, '..');
const read = file => fs.readFileSync(path.join(root, file), 'utf8');

function sources(directory) {
  return fs.readdirSync(path.join(root, directory), { withFileTypes: true }).flatMap(entry => {
    const name = `${directory}/${entry.name}`;
    if (entry.isDirectory()) return entry.name === 'dist' ? [] : sources(name);
    return /\.(?:tsx?|css|md|json|mjs|cjs|svg)$/.test(name) ? [name] : [];
  });
}
const owned = [...sources('packages'), ...sources('apps'), ...sources('scripts'), ...sources('docs'), ...sources('tests'), 'package.json', 'package-lock.json', 'tsconfig.json', 'vitest.config.ts', 'AGENTS.md'];

test('Git ignores dependencies and build artifacts but retains lockfiles and offline previews', () => {
  for (const file of ['node_modules/example/index.js', 'packages/react/neuro-ui/dist/index.js', 'packages/windows-native/example-ui/target/debug/app.exe', 'apps/windows-native/example-gallery/target/debug/app.exe', '.tmp/probe.json', 'example.tgz']) {
    const result = spawnSync('git', ['check-ignore', '--no-index', '--quiet', '--', file], { cwd: root, encoding: 'utf8' });
    assert.equal(result.status, 0, `${file}: ${result.stderr}`);
  }
  for (const file of ['package-lock.json', 'design-md/neuro/preview.js', 'design-md/neuro/preview.css']) {
    const result = spawnSync('git', ['check-ignore', '--no-index', '--quiet', '--', file], { cwd: root, encoding: 'utf8' });
    assert.equal(result.status, 1, `${file}: ${result.stderr}`);
  }
  assert.doesNotMatch(read('.github/FUNDING.yml'), /^\s*github\s*:/m);
});

test('new implementation files are strict UTF-8 without BOM and local documentation links resolve', () => {
  for (const name of owned) {
    const bytes = fs.readFileSync(path.join(root, name));
    assert.notEqual(bytes.subarray(0, 3).toString('hex'), 'efbbbf', name);
    assert.doesNotThrow(() => new TextDecoder('utf-8', { fatal: true }).decode(bytes), name);
    if (!name.endsWith('.md')) continue;
    for (const [, target] of read(name).matchAll(/\]\(([^)]+)\)/g)) {
      if (/^(?:[a-z]+:|#)/i.test(target)) continue;
      assert.ok(fs.existsSync(path.resolve(root, path.dirname(name), target.split('#')[0])), `${name} → ${target}`);
    }
  }
});

test('the component package exports built ESM, types and CSS, with host-owned React', async () => {
  const pkg = JSON.parse(read('packages/react/neuro-ui/package.json'));
  assert.equal(pkg.name, '@neuro/ui');
  assert.ok(pkg.peerDependencies.react);
  assert.ok(pkg.peerDependencies['react-dom']);
  assert.ok(!pkg.dependencies.react);
  for (const target of [pkg.exports['.'].import, pkg.exports['.'].types, pkg.exports['./styles.css']]) {
    assert.ok(fs.existsSync(path.resolve(root, 'packages/react/neuro-ui', target)), target);
  }
  const api = await import(pathToFileURL(path.join(root, 'packages/react/neuro-ui/dist/index.js')).href);
  for (const name of ['NeuroProvider', 'Button', 'Field', 'Select', 'PopupSelect', 'SegmentedControl', 'MultiSelect', 'ActionTooltip', 'Dialog', 'ConfirmDialog', 'ToastViewport', 'DataTable', 'SelectionBar', 'CopyValue', 'CopyButton', 'NumberField', 'AppShell', 'ImagePreview']) assert.ok(api[name], name);
  const React = require('react');
  const { renderToString } = require('react-dom/server');
  const rendered = renderToString(React.createElement(api.NeuroProvider, null, React.createElement(api.Button, null, 'Consumer')));
  assert.match(rendered, /Consumer/);
  assert.match(rendered, /type="button"/);
});

test('the tested Radix set is pinned and shares overlay focus and dismissal contexts', async () => {
  const { createRequire } = require('node:module');
  const { verifyRadixRuntime } = await import(pathToFileURL(path.join(root, 'scripts/radix-runtime.mjs')).href);
  const pkg = JSON.parse(read('packages/react/neuro-ui/package.json'));
  for (const [name, version] of Object.entries(pkg.dependencies)) {
    if (name.startsWith('@radix-ui/')) assert.match(version, /^\d+\.\d+\.\d+$/, name);
  }
  const paths = verifyRadixRuntime(createRequire(path.join(root, 'packages/react/neuro-ui/dist/index.js')));
  assert.equal(Object.keys(paths).length, 2);
});

test('the standard implementation owns no host, application API, business queue or network client', () => {
  for (const file of sources('packages/react/neuro-ui/src').filter(file => /\.(?:ts|tsx)$/.test(file))) {
    assert.doesNotMatch(read(file), /@tauri-apps|loomApi|gatewayApi|\bfetch\s*\(|XMLHttpRequest|WebSocket|localStorage|sessionStorage|\.\.[/\\].*apps[/\\]/, file);
  }
});

test('both previews consume the real package and offline outputs, not the frozen handwritten controls', () => {
  for (const file of ['apps/react/neuro-preview/src/Gallery.tsx', 'apps/react/neuro-preview/src/Website.tsx']) {
    assert.match(read(file), /from "@neuro\/ui"/);
    assert.doesNotMatch(read(file), /_neuro|innerHTML|replaceChildren/);
  }
  assert.match(read('scripts/build.mjs'), /Preview must consume the built package/);
  for (const file of ['index.html', 'example.html']) {
    const html = read(`design-md/neuro/${file}`);
    assert.match(html, /src="preview\.js"/);
    assert.match(html, /href="preview\.css"/);
    assert.doesNotMatch(html, /type="module"|https?:\/\/|_neuro/);
  }
  for (const asset of ['preview.js', 'preview.css', 'sample.svg']) assert.ok(fs.statSync(path.join(root, 'design-md/neuro', asset)).size > 0);
  assert.match(read('design-md/neuro-mini/README.md'), /只维护声明层/);
  assert.doesNotMatch(read('neuro.html'), /href="design-md\/neuro-mini\/(?:index|example)\.html"/);
});

test('fixed palette matches the declaration and state geometry stays stable', () => {
  const css = read('packages/react/neuro-ui/src/styles.css');
  for (const color of ['#d9ff38', '#22c55e', '#06b6d4', '#f43f5e', '#06080d', '#090c11', '#0e1218', '#070a0f', '#111720', '#18222c', '#f7f8ef', '#929a9f', '#edf1e4', '#f8faf3', '#ffffff', '#10150e']) {
    assert.ok(css.includes(color), color);
    assert.ok(read('design-md/neuro/DESIGN.md').includes(color), color);
  }
  assert.match(css, /prefers-reduced-motion/);
  assert.match(css, /forced-colors/);
  const geometry = /(?:^|;)\s*(?:padding(?:-[\w-]+)?|margin(?:-[\w-]+)?|border(?:-width)?|font(?:-[\w-]+)?|(?:min-|max-)?(?:width|height)|display)\s*:/;
  for (const [, selector, body] of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
    if (/input:checked \+ span|tr\[data-selected\]|aria-pressed="true"/.test(selector)) assert.doesNotMatch(body, geometry, selector);
  }
  assert.match(css, /\.n-table thead th \{[^}]*position: sticky[^}]*background: var\(--n-surface\)/s);
  assert.doesNotMatch(css, /\.n-table (?:tbody )?th \{[^}]*position: sticky/s);
  assert.match(css, /\.n-table tbody tr:hover > :is\(td, th\)/);
  assert.doesNotMatch(css, /\.n-table tr:hover/);
  assert.match(css, /\.n-selection-summary, \.n-selection-actions \{[^}]*grid-area: 1 \/ 1/s);
  assert.match(css, /\.n-button \{[^}]*vertical-align: middle/s);
  assert.match(css, /\.n-loading \.n-button-label \{[^}]*opacity: 0;/);
  assert.doesNotMatch(css, /\.n-loading \.n-button-label \{[^}]*(?:visibility: hidden|display: none)/);
  assert.doesNotMatch(css, /text-transform:\s*uppercase/);
});

test('notices keep long diagnostics inside their available width without clipping', () => {
  const css = read('packages/react/neuro-ui/src/styles.css');
  assert.match(css, /\.n-notice \{[^}]*min-width: 0;[^}]*overflow-wrap: anywhere;/);
  assert.match(css, /\.n-notice > div \{[^}]*min-width: 0;/);
  assert.doesNotMatch(css.match(/\.n-notice \{([^}]+)\}/)[1], /overflow: hidden|text-overflow: ellipsis/);
});

test('embedded empty states use the standard compact recipe instead of a page-sized minimum', () => {
  const css = read('packages/react/neuro-ui/src/styles.css');
  assert.match(css, /\.n-empty--compact \{[^}]*grid-template-columns: auto minmax\(0, 1fr\);[^}]*min-height: 0;/);
  assert.match(css, /\.n-empty \{[^}]*min-width: 0;[^}]*overflow-wrap: anywhere;/);
  assert.match(css, /\.n-empty--compact > :nth-child\(n \+ 3\) \{[^}]*grid-column: 1 \/ -1;/);
  assert.ok(read('apps/react/neuro-preview/src/Gallery.tsx').includes('<EmptyState compact'));
});

test('scrollable multi-select keeps wrapped option rows at intrinsic height', () => {
  const css = read('packages/react/neuro-ui/src/styles.css');
  assert.match(css, /\.n-multi-list \{[^}]*grid-auto-rows: max-content;/);
  assert.match(css, /\.n-multi-list \{[^}]*overflow-y: auto;/);
  assert.match(css, /\.n-multi-options \{[^}]*max-height:/);
});
