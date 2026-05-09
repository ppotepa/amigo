import { execFileSync } from "node:child_process";

const forbidden = [
  String.raw`component\.typeName\s*===\s*["']`,
  String.raw`component\.typeName\s*!==\s*["']`,
  String.raw`component\.typeName\.includes\(`,
  String.raw`switch\s*\(\s*component\.typeName\s*\)`,
  String.raw`descriptor\.capabilities`,
  String.raw`traitEditorSections`,
  String.raw`editorSections`,
  String.raw`editor_sections`,
];

const allowed = [
  ".test.",
  "__fixtures__",
  "editorMetadataTypes.ts",
];

let failed = false;
const scanDirs = [
  "src/app",
  "src/api",
  "src/debug",
  "src/editor-components",
  "src/editor-targets",
  "src/features/metadata",
  "src/features/target-context",
  "src/features/inspector",
  "src/main-window",
  "src/properties",
  "src/theme",
];

for (const pattern of forbidden) {
  let output = "";

  try {
    output = execFileSync("rg", [pattern, ...scanDirs], {
      cwd: new URL("..", import.meta.url),
      encoding: "utf8",
    });
  } catch {
    continue;
  }

  const violations = output
    .split("\n")
    .filter(Boolean)
    .filter((line) => !allowed.some((allowedPart) => line.includes(allowedPart)));

  if (violations.length) {
    failed = true;
    console.error(`Forbidden metadata-driven UI pattern: ${pattern}`);
    console.error(violations.join("\n"));
  }
}

if (failed) {
  process.exit(1);
}
