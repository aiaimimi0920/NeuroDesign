// React-only package discovery. Other stacks use their own manifests and tools.
import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";

export const readJson = file => JSON.parse(readFileSync(file, "utf8"));
export const inside = (directory, file) => file === directory || file.startsWith(`${directory}${path.sep}`);

export function reactPackages(root) {
  const packages = [];
  for (const group of ["packages", "apps"]) {
    const directory = path.join(root, group, "react");
    if (!existsSync(directory)) continue;
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (!entry.isDirectory()) continue;
      const dir = path.join(directory, entry.name);
      const manifest = readJson(path.join(dir, "package.json"));
      const design = manifest.designSystem;
      if (!design || !["theme", "shared", "preview"].includes(design.kind)) throw new Error(`Missing designSystem kind: ${dir}`);
      if ((group === "apps") !== (design.kind === "preview")) throw new Error(`Wrong package location: ${dir}`);
      if (design.kind === "shared") {
        if (design.theme !== undefined) throw new Error(`Shared packages cannot own a theme: ${dir}`);
      } else if (!/^[a-z][a-z0-9-]*$/.test(design.theme ?? "") || !existsSync(path.join(root, "design-md", design.theme, "DESIGN.md"))) {
        throw new Error(`Missing theme declaration: ${dir}`);
      }
      if (design.kind === "preview" && !manifest.private) throw new Error(`Previews must be private: ${dir}`);
      if (!manifest.name || packages.some(item => item.name === manifest.name)) throw new Error(`Missing or duplicate package name: ${dir}`);
      packages.push({ dir, name: manifest.name, manifest, ...design });
    }
  }
  return packages;
}

export function dependencies(item) {
  return { ...item.manifest.dependencies, ...item.manifest.peerDependencies, ...item.manifest.optionalDependencies, ...item.manifest.devDependencies };
}

export function assertDependency(owner, target) {
  if (target.kind === "preview" || (owner.kind === "shared" ? target.kind !== "shared" : target.kind !== "shared" && owner.theme !== target.theme)) {
    throw new Error(`Forbidden theme dependency: ${owner.name} -> ${target.name}`);
  }
}

// Build only the selected themes and their declared shared dependencies, in order.
export function libraryOrder(packages, theme) {
  const selected = packages.filter(item => item.kind === "theme" && (!theme || item.theme === theme));
  if (theme && !selected.length) throw new Error(`No React implementation for theme: ${theme}`);
  const done = new Set(), visiting = new Set(), result = [];
  function visit(item) {
    if (done.has(item.name)) return;
    if (visiting.has(item.name)) throw new Error(`Cyclic component dependency: ${item.name}`);
    visiting.add(item.name);
    for (const name of Object.keys(dependencies(item))) {
      const target = packages.find(candidate => candidate.name === name);
      if (target) { assertDependency(item, target); visit(target); }
    }
    visiting.delete(item.name);
    done.add(item.name);
    result.push(item);
  }
  selected.forEach(visit);
  return result;
}
