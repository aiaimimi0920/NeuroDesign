import { createRequire } from "node:module";

// These composable overlays must share the same focus stack and dismissal context.
export function verifyRadixRuntime(uiRequire) {
  const owners = ["@radix-ui/react-dialog", "@radix-ui/react-popover", "@radix-ui/react-select"];
  const result = {};
  for (const shared of ["@radix-ui/react-focus-scope", "@radix-ui/react-dismissable-layer"]) {
    const paths = owners.map(owner => ({ owner, path: createRequire(uiRequire.resolve(owner)).resolve(shared) }));
    if (new Set(paths.map(entry => entry.path)).size !== 1) {
      throw new Error(`Neuro overlays resolved different ${shared} instances: ${JSON.stringify(paths)}`);
    }
    result[shared] = paths[0].path;
  }
  return result;
}
