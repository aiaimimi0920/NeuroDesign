import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync, readFileSync, readdirSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { verifyRadixRuntime } from "./radix-runtime.mjs";
import { reactPackages } from "./react-packages.mjs";

const repository = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const consumer = process.cwd();
const manifest = JSON.parse(readFileSync(path.join(consumer, "package.json"), "utf8"));
const args = process.argv.slice(2);
if (args.length !== 2 || args[0] !== "--package") throw new Error("Choose one React theme package: ui:sync --package <name>");
const item = reactPackages(repository).find(candidate => candidate.kind === "theme" && candidate.name === args[1]);
if (!item) throw new Error(`Unknown React theme package: ${args[1]}`);
const source = item.dir;
const spec = manifest.dependencies?.[item.name];
if (!spec?.startsWith("file:") || realpathSync(path.resolve(consumer, spec.slice(5))) !== realpathSync(source)) {
  throw new Error(`Run ui:sync from an application depending on this local ${item.name} package`);
}
const npmCli = process.env.npm_execpath;
if (!npmCli) throw new Error("Run this command through npm run ui:sync");
const npmVersion = JSON.parse(readFileSync(path.resolve(path.dirname(npmCli), "../package.json"), "utf8")).version;
if (Number(npmVersion.split(".")[0]) < 11) throw new Error("UI synchronization requires npm 11+: npx --yes npm@11 run ui:sync");

function run(args, cwd = consumer) {
  const result = spawnSync(process.execPath, args, { cwd, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`Command failed (${result.status}): ${args.join(" ")}`);
}

run([path.join(repository, "scripts/build.mjs"), "--library", "--theme", item.theme], repository);
const installed = path.join(consumer, "node_modules", ...item.name.split("/"));
const digest = file => createHash("sha256").update(readFileSync(file)).digest("hex");
function distFiles(directory, prefix = "dist") {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => entry.isDirectory()
    ? distFiles(path.join(directory, entry.name), `${prefix}/${entry.name}`)
    : [`${prefix}/${entry.name}`]);
}
const artifacts = [...distFiles(path.join(source, "dist")), "README.md", "LICENSE"];
const expected = JSON.parse(readFileSync(path.join(source, "package.json"), "utf8"));
function metadataMatches() {
  try {
    const current = JSON.parse(readFileSync(path.join(installed, "package.json"), "utf8"));
    return ["name", "version", "type", "main", "types", "sideEffects", "dependencies", "peerDependencies", "exports", "designSystem"].every(key => JSON.stringify(current[key]) === JSON.stringify(expected[key]));
  } catch (error) {
    if (error.code === "ENOENT") return false;
    throw error;
  }
}
if (!metadataMatches()) {
  run([npmCli, "install", "--install-links", "--ignore-scripts", "--prefer-offline", "--no-audit", "--no-fund", `${item.name}@${spec}`]);
  if (!metadataMatches()) throw new Error(`Local package metadata changed: increment its version and reinstall ${item.name} before synchronizing`);
}
if (realpathSync(installed) === realpathSync(source)) throw new Error("Expected install-links=true, not a source-directory symlink");
// npm may cache a same-version file: package. Refresh built artifacts only; never
// copy source files or uninstall/prune the application's dependency graph.
let copied = 0;
for (const file of artifacts) {
  const destination = path.join(installed, file);
  let same = false;
  try { same = digest(path.join(source, file)) === digest(destination); }
  catch (error) { if (error.code !== "ENOENT") throw error; }
  if (same) continue;
  mkdirSync(path.dirname(destination), { recursive: true });
  copyFileSync(path.join(source, file), destination);
  if (digest(path.join(source, file)) !== digest(destination)) throw new Error(`Artifact synchronization failed: ${file}`);
  copied++;
}
console.log(`Synchronized ${copied} built artifacts; unchanged dependencies were not reinstalled`);
const appRequire = createRequire(path.join(consumer, "package.json"));
const uiRequire = createRequire(path.join(installed, "dist/index.js"));
for (const peer of ["react", "react-dom"]) {
  if (appRequire.resolve(peer) !== uiRequire.resolve(peer)) throw new Error(`Duplicate ${peer} instance`);
}
if (expected.dependencies?.["@radix-ui/react-dialog"]) verifyRadixRuntime(uiRequire);
console.log(`${item.name} synchronized: current built files and host-owned React runtime`);
