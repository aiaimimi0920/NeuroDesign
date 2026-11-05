const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const { spawnSync } = require('node:child_process');
const { build } = require('esbuild');
const root = path.resolve(__dirname, '..');
const scripts = name => import(pathToFileURL(path.join(root, 'scripts', name)).href);

// Fixtures are retained for diagnosis; callers may direct them to their temp workspace.
function fixture() {
  const parent = process.env.DESIGN_TEST_TMP || path.join(root, '.tmp');
  fs.mkdirSync(parent, { recursive: true });
  const directory = fs.mkdtempSync(path.join(parent, 'theme-boundaries-'));
  const write = (file, value) => {
    const target = path.join(directory, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, typeof value === 'string' ? value : JSON.stringify(value, null, 2));
  };
  write('package.json', { private: true, workspaces: ['packages/react/*', 'apps/react/*'] });
  for (const theme of ['neuro', 'lucky', 'neuro-mini']) write(`design-md/${theme}/DESIGN.md`, `# ${theme}`);
  function component(folder, name, design, source, dependencies = {}) {
    const dir = `packages/react/${folder}`;
    write(`${dir}/package.json`, { name, version: '1.0.0', type: 'module', designSystem: design, dependencies, exports: { '.': './dist/index.js', './styles.css': './dist/styles.css' } });
    write(`${dir}/src/index.js`, source);
    write(`${dir}/dist/index.js`, source);
    write(`${dir}/src/styles.css`, `.${folder} { color: green; }`);
    write(`${dir}/dist/styles.css`, `.${folder} { color: green; }`);
  }
  component('behavior', '@fixture/behavior', { kind: 'shared' }, 'export const text = value => value;');
  component('neuro-ui', '@neuro/ui', { kind: 'theme', theme: 'neuro' }, 'export const marker = "NEURO_MUST_NOT_LOAD";');
  component('lucky-ui', '@lucky/ui', { kind: 'theme', theme: 'lucky' }, 'import { text } from "@fixture/behavior"; export const label = text("LUCKY_ONLY");', { '@fixture/behavior': '1.0.0' });
  // A native crate has no package.json and must not enter npm discovery/build.
  write('packages/windows-native/neuro-mini-ui/Cargo.toml', '[package]\nname = "neuro-mini-ui"\nversion = "0.1.0"\nedition = "2021"\n');
  write('packages/windows-native/neuro-mini-ui/src/lib.rs', 'pub const THEME: &str = "neuro-mini";');
  return { directory, write };
}

test('theme selection includes only its declared shared packages, never another theme or native crate', async () => {
  const { checkBoundaries } = await scripts('check-boundaries.mjs');
  const { libraryOrder } = await scripts('react-packages.mjs');
  const f = fixture();
  const packages = checkBoundaries(f.directory);
  assert.deepEqual(libraryOrder(packages, 'lucky').map(item => item.name), ['@fixture/behavior', '@lucky/ui']);
  assert.deepEqual(libraryOrder(packages, 'neuro').map(item => item.name), ['@neuro/ui']);
  assert.equal(packages.length, 3);
  assert.throws(() => libraryOrder(packages, 'neuro-mini'), /No React implementation/);
  assert.throws(() => libraryOrder(packages, 'typo'), /No React implementation/);
});

test('a minimal Lucky consumer bundles its real shared dependency and CSS without installing Neuro', async () => {
  const { checkBoundaries } = await scripts('check-boundaries.mjs');
  const f = fixture();
  checkBoundaries(f.directory);
  f.write('consumer/package.json', { private: true, type: 'module', dependencies: { '@lucky/ui': '1.0.0', '@fixture/behavior': '1.0.0' } });
  for (const [name, folder] of [['@lucky/ui', 'lucky-ui'], ['@fixture/behavior', 'behavior']]) {
    const installed = path.join(f.directory, 'consumer/node_modules', name);
    const source = path.join(f.directory, 'packages/react', folder);
    fs.mkdirSync(installed, { recursive: true });
    fs.copyFileSync(path.join(source, 'package.json'), path.join(installed, 'package.json'));
    fs.cpSync(path.join(source, 'dist'), path.join(installed, 'dist'), { recursive: true });
  }
  const result = await build({
    absWorkingDir: f.directory,
    stdin: { contents: 'export { label } from "@lucky/ui"; import "@lucky/ui/styles.css";', resolveDir: path.join(f.directory, 'consumer'), sourcefile: 'consumer.js' },
    outfile: 'consumer/bundle.js', bundle: true, format: 'esm', write: false, metafile: true, treeShaking: false,
  });
  const inputs = Object.keys(result.metafile.inputs).join('\n').replaceAll('\\', '/');
  assert.match(inputs, /@lucky\/ui\/dist\/index\.js/);
  assert.match(inputs, /@fixture\/behavior\/dist\/index\.js/);
  assert.doesNotMatch(inputs, /neuro-ui|windows-native|neuro-mini/);
  const javascript = result.outputFiles.find(file => file.path.endsWith('.js')).text;
  const css = result.outputFiles.find(file => file.path.endsWith('.css')).text;
  assert.match(css, /\.lucky-ui/);
  assert.doesNotMatch(css, /neuro/);
  assert.doesNotMatch(javascript, /NEURO_MUST_NOT_LOAD/);
  const api = await import(`data:text/javascript;base64,${Buffer.from(javascript).toString('base64')}`);
  assert.equal(api.label, 'LUCKY_ONLY');
  assert.ok(!fs.existsSync(path.join(f.directory, 'consumer/node_modules/@neuro/ui')));
});

for (const [name, file, source, message] of [
  ['cross-theme manifest dependency', 'packages/react/lucky-ui/package.json', { name: '@lucky/ui', designSystem: { kind: 'theme', theme: 'lucky' }, dependencies: { '@neuro/ui': '1.0.0' } }, /Forbidden theme dependency/],
  ['shared reverse dependency', 'packages/react/behavior/package.json', { name: '@fixture/behavior', designSystem: { kind: 'shared' }, dependencies: { '@neuro/ui': '1.0.0' } }, /Forbidden theme dependency/],
  ['cross-theme re-export', 'packages/react/lucky-ui/src/index.js', 'export * from "@neuro/ui";', /Forbidden theme dependency/],
  ['cross-theme dynamic import', 'packages/react/lucky-ui/src/index.js', 'import("@neuro/ui");', /Forbidden theme dependency/],
  ['cross-theme require', 'packages/react/lucky-ui/src/index.js', 'require("@neuro/ui/styles.css");', /Forbidden theme dependency/],
  ['relative source penetration', 'packages/react/lucky-ui/src/index.js', 'export * from "../../neuro-ui/src/index.js";', /outside source\/asset/],
  ['relative CSS penetration', 'packages/react/lucky-ui/src/styles.css', '@import "../../neuro-ui/src/styles.css";', /outside source\/asset/],
  ['CSS resource penetration', 'packages/react/lucky-ui/src/styles.css', '.icon { background: url("../../neuro-ui/src/icon.svg"); }', /outside source\/asset/],
  ['cross-theme CSS import', 'packages/react/lucky-ui/src/styles.css', '@import "@neuro/ui/styles.css";', /Forbidden theme dependency/],
  ['undeclared internal dependency', 'packages/react/neuro-ui/src/index.js', 'export * from "@fixture/behavior";', /Undeclared component dependency/],
  ['non-static import', 'packages/react/lucky-ui/src/index.js', 'import(window.theme);', /must be static/],
  ['native local dependency from React', 'packages/react/lucky-ui/package.json', { name: '@lucky/ui', designSystem: { kind: 'theme', theme: 'lucky' }, dependencies: { 'neuro-mini-ui': 'file:../../windows-native/neuro-mini-ui' } }, /Unregistered local dependency/],
  ['all-stack npm workspace', 'package.json', { private: true, workspaces: ['packages/**/*'] }, /only React/],
  ['all-theme root runtime', 'package.json', { private: true, workspaces: ['packages/react/*', 'apps/react/*'], exports: './index.js' }, /all-themes runtime/],
]) {
  test(`boundaries reject ${name}`, async () => {
    const { checkBoundaries } = await scripts('check-boundaries.mjs');
    const f = fixture();
    f.write(file, source);
    assert.throws(() => checkBoundaries(f.directory), message);
  });
}

test('a preview cannot select another theme library', async () => {
  const { checkBoundaries } = await scripts('check-boundaries.mjs');
  const f = fixture();
  f.write('apps/react/lucky-preview/package.json', { name: '@lucky/preview', private: true, designSystem: { kind: 'preview', theme: 'lucky', library: '@neuro/ui' }, dependencies: { '@neuro/ui': '1.0.0' } });
  assert.throws(() => checkBoundaries(f.directory), /own theme library/);
});

test('the actual Neuro tarball includes only its independent built implementation', () => {
  const npmCli = process.env.npm_execpath;
  assert.ok(npmCli, 'Run the contract suite through npm test');
  const result = spawnSync(process.execPath, [npmCli, 'pack', '--workspace', '@neuro/ui', '--dry-run', '--ignore-scripts', '--json'], { cwd: root, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  const [packed] = JSON.parse(result.stdout);
  assert.equal(packed.name, '@neuro/ui');
  for (const file of packed.files) {
    assert.match(file.path, /^(?:dist\/|README\.md$|LICENSE$|package\.json$)/, file.path);
    assert.doesNotMatch(file.path, /lucky|windows-native|apps\//);
  }
  assert.ok(packed.files.some(file => file.path === 'dist/index.js'));
  assert.ok(packed.files.some(file => file.path === 'dist/styles.css'));
});
