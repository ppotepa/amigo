import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const forbidden = [
  String.raw`component\.typeName\s*===\s*["']`,
  String.raw`component\.typeName\s*!==\s*["']`,
  String.raw`component\.typeName\.includes\(`,
  String.raw`switch\s*\(\s*component\.typeName\s*\)`,
];

const allowed = [
  "editorTargetResolver",
  "sceneTargetAdapter",
  "ItemContextNavigator",
  "GenericPropertiesPanel",
  "editorMetadataTypes",
  ".test.",
  "__fixtures__",
];

const ignoredPaths = [
  "src/components/",
  "src/data/",
  "src/types/",
  "src/ui/folder-view/",
  "src/features/editor-targets/",
  "src/features/scenes/editor/live/",
  "src/features/tools/pixel-labs/",
];

let failed = false;
const sourceFiles = execFileSync("git", ["ls-files", "src"], {
  cwd: new URL("..", import.meta.url),
  encoding: "utf8",
})
  .split("\n")
  .filter((path) => path.endsWith(".ts") || path.endsWith(".tsx"))
  .filter((path) => !ignoredPaths.some((ignoredPath) => path.startsWith(ignoredPath)));

for (const pattern of forbidden) {
  const expression = new RegExp(pattern, "u");
  const violations = [];

  for (const file of sourceFiles) {
    const text = readFileSync(new URL(`../${file}`, import.meta.url), "utf8");
    const lines = text.split("\n");

    lines.forEach((line, index) => {
      if (!expression.test(line)) return;
      const location = `${file}:${index + 1}:${line.trim()}`;
      if (allowed.some((allowedPart) => location.includes(allowedPart))) return;
      violations.push(location);
    });
  }

  if (violations.length) {
    failed = true;
    console.error(`Forbidden metadata-driven UI pattern: ${pattern}`);
    console.error(violations.join("\n"));
  }
}

if (failed) {
  process.exit(1);
}
