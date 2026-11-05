// Check source and manifest edges, not whether a bundler happened to tree-shake them.
import { existsSync, readFileSync, readdirSync, realpathSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import { assertDependency, dependencies, inside, libraryOrder, reactPackages, readJson } from "./react-packages.mjs";

export function checkBoundaries(root) {
  const manifest = readJson(path.join(root, "package.json"));
  if (!manifest.private || manifest.main || manifest.exports || Object.keys(manifest.dependencies ?? {}).length) {
    throw new Error("The repository root is tooling, not an all-themes runtime package");
  }
  if (JSON.stringify(manifest.workspaces) !== JSON.stringify(["packages/react/*", "apps/react/*"])) {
    throw new Error("npm workspaces must contain only React implementations and previews");
  }
  const packages = reactPackages(root);
  const findPackage = file => packages.find(item => inside(item.dir, file));
  for (const item of packages) {
    const declared = dependencies(item);
    if (item.kind === "preview") {
      const library = packages.find(candidate => candidate.name === item.library);
      if (!library || library.kind !== "theme" || library.theme !== item.theme || !declared[library.name]) {
        throw new Error(`Preview must declare its own theme library: ${item.name}`);
      }
    }
    for (const [name, version] of Object.entries(declared)) {
      const target = packages.find(candidate => candidate.name === name);
      if (target) assertDependency(item, target);
      if (/^(?:file:|link:)/.test(version)) {
        const directory = path.resolve(item.dir, version.replace(/^(?:file:|link:)/, ""));
        const local = findPackage(directory);
        if (!local || local.dir !== directory || local.name !== name) throw new Error(`Unregistered local dependency: ${item.name} -> ${version}`);
        assertDependency(item, local);
      }
      if (version.startsWith("npm:") && packages.some(candidate => version.startsWith(`npm:${candidate.name}@`))) {
        throw new Error(`Do not alias workspace packages: ${item.name} -> ${version}`);
      }
    }
    function checkImport(file, specifier) {
      if (/^(?:\.|\/|[A-Za-z]:)/.test(specifier)) {
        const resolved = path.resolve(path.dirname(file), specifier.split(/[?#]/)[0]);
        const actual = existsSync(resolved) ? realpathSync(resolved) : resolved;
        if (!inside(item.dir, actual)) throw new Error(`Use a declared package export, not an outside source/asset: ${file} -> ${specifier}`);
        return;
      }
      const target = packages.find(candidate => specifier === candidate.name || specifier.startsWith(`${candidate.name}/`));
      if (target && target !== item) {
        assertDependency(item, target);
        if (!declared[target.name]) throw new Error(`Undeclared component dependency: ${file} -> ${specifier}`);
      }
    }
    function walk(directory) {
      if (!existsSync(directory)) return;
      for (const entry of readdirSync(directory, { withFileTypes: true })) {
        const file = path.join(directory, entry.name);
        if (entry.isSymbolicLink()) throw new Error(`Component sources must not link outside their package: ${file}`);
        if (entry.isDirectory()) { walk(file); continue; }
        if (!/\.(?:[cm]?[jt]sx?|css)$/.test(file)) continue;
        const text = readFileSync(file, "utf8");
        if (file.endsWith(".css")) {
          for (const match of text.matchAll(/@import\s+(?:url\(\s*)?["']([^"']+)["']|url\(\s*["']?([^)'"\s]+)["']?\s*\)/g)) {
            const specifier = match[1] ?? match[2];
            if (!/^(?:data:|https?:|#)/.test(specifier)) checkImport(file, specifier.startsWith(".") || specifier.startsWith("@") ? specifier : `./${specifier}`);
          }
          continue;
        }
        const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true);
        function visit(node) {
          if ((ts.isImportDeclaration(node) || ts.isExportDeclaration(node)) && node.moduleSpecifier) checkImport(file, node.moduleSpecifier.text);
          if (ts.isImportEqualsDeclaration(node) && ts.isExternalModuleReference(node.moduleReference)) checkImport(file, node.moduleReference.expression.text);
          if (ts.isCallExpression(node) && (node.expression.kind === ts.SyntaxKind.ImportKeyword || ts.isIdentifier(node.expression) && node.expression.text === "require")) {
            const value = node.arguments[0];
            if (!value || !ts.isStringLiteralLike(value)) throw new Error(`Component imports must be static: ${file}`);
            checkImport(file, value.text);
          }
          ts.forEachChild(node, visit);
        }
        visit(source);
      }
    }
    walk(path.join(item.dir, "src"));
  }
  libraryOrder(packages);
  return packages;
}

const main = process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (main) {
  const root = fileURLToPath(new URL("../", import.meta.url));
  console.log(`Verified theme boundaries for ${checkBoundaries(root).length} React packages; native stacks are not npm workspaces`);
}
