// React build entry only. Native implementations build with their own toolchain.
import { build } from "esbuild";
import { copyFile, mkdir } from "node:fs/promises";
import { existsSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { checkBoundaries } from "./check-boundaries.mjs";
import { inside, libraryOrder } from "./react-packages.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const args = process.argv.slice(2);
let theme, libraryOnly = false;
for (let index = 0; index < args.length; index++) {
  if (args[index] === "--library") libraryOnly = true;
  else if (args[index] === "--theme" && args[index + 1] && !args[index + 1].startsWith("--")) theme = args[++index];
  else throw new Error(`Unknown or incomplete build argument: ${args[index]}`);
}
const packages = checkBoundaries(root);
const libraries = libraryOrder(packages, theme);
function checkInputs(result, allowed) {
  for (const input of Object.keys(result.metafile.inputs)) {
    const file = path.resolve(root, input);
    if (inside(path.join(root, "packages"), file) || inside(path.join(root, "apps"), file)) {
      if (!allowed.some(item => inside(item.dir, file))) throw new Error(`Unexpected implementation in bundle: ${input}`);
    }
  }
}
for (const item of libraries) {
  await mkdir(path.join(item.dir, "dist"), { recursive: true });
  const result = await build({
    absWorkingDir: root,
    entryPoints: [path.join(item.dir, "src/index.ts")],
    outfile: path.join(item.dir, "dist/index.js"),
    bundle: true, packages: "external", format: "esm", platform: "browser",
    tsconfigRaw: { compilerOptions: { jsx: "react-jsx" } },
    jsx: "automatic", target: "es2022", metafile: true,
  });
  checkInputs(result, [item]);
  const declarations = spawnSync(process.execPath, [path.join(root, "node_modules/typescript/bin/tsc"), "-p", path.join(item.dir, "tsconfig.build.json")], { cwd: root, stdio: "inherit" });
  if (declarations.error) throw declarations.error;
  if (declarations.status !== 0) process.exit(declarations.status ?? 1);
  if (existsSync(path.join(item.dir, "src/styles.css"))) await copyFile(path.join(item.dir, "src/styles.css"), path.join(item.dir, "dist/styles.css"));
  await copyFile(path.join(root, "LICENSE"), path.join(item.dir, "LICENSE"));
  console.log(`Built ${item.name}: independent ESM, types and local styles`);
}
if (!libraryOnly) {
  for (const preview of packages.filter(item => item.kind === "preview" && (!theme || item.theme === theme))) {
    const library = packages.find(item => item.name === preview.library);
    const output = path.join(root, "design-md", preview.theme);
    const result = await build({
      absWorkingDir: root,
      entryPoints: [path.join(preview.dir, "src/main.tsx")],
      outfile: path.join(output, "preview.js"),
      bundle: true, format: "iife", platform: "browser",
      tsconfigRaw: { compilerOptions: { jsx: "react-jsx" } },
      jsx: "automatic", target: "es2022", minify: true,
      define: { "process.env.NODE_ENV": '"production"' },
      legalComments: "linked", metafile: true,
      banner: { js: `/* Generated from ${path.relative(root, preview.dir).replaceAll("\\", "/")} and the built ${library.name} package. Run npm run build; do not edit. */` },
    });
    checkInputs(result, [preview, ...libraryOrder(packages, preview.theme)]);
    if (!Object.keys(result.metafile.inputs).some(file => path.resolve(root, file) === path.join(library.dir, "dist/index.js"))) {
      throw new Error(`Preview must consume the built package: ${library.name}`);
    }
    if (existsSync(path.join(preview.dir, "assets/interface-kit.svg"))) await copyFile(path.join(preview.dir, "assets/interface-kit.svg"), path.join(output, "sample.svg"));
    console.log(`Built ${preview.theme} offline previews from ${library.name}`);
  }
}
