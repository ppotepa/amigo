import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";

const repoRoot = execFileSync("git", ["rev-parse", "--show-toplevel"], {
  encoding: "utf8",
}).trim();

const descriptorPath = path.join(repoRoot, "crates/engine/scene/src/component_descriptors.rs");
const descriptorSource = readFileSync(descriptorPath, "utf8");
const descriptors = parseRegisteredDescriptors(descriptorSource);
const usage = parseYamlComponentUsage();
const ignoredComponentTypes = new Set(["button", "column", "dropdown", "panel", "row", "slider", "spacer", "stack", "text", "toggle"]);

const report = [];
let failed = false;

for (const [typeName, details] of [...usage.entries()].sort(([left], [right]) => left.localeCompare(right))) {
  if (ignoredComponentTypes.has(typeName)) continue;
  if (!/^[A-Z]/.test(typeName)) continue;

  const descriptor = descriptors.get(typeName);
  const fields = [...details.fields.keys()].sort();

  if (!descriptor) {
    failed = true;
    report.push(`${typeName}\t${details.count}\tmissing descriptor\t${fields.join(", ")}`);
    continue;
  }

  const describedFields = new Set([...descriptor.properties, ...descriptor.assetRefs]);
  const missingFields = fields.filter((field) => !describedFields.has(field));
  const genericFields = fields.filter((field) => descriptor.genericGroups.has(field));

  if (missingFields.length || genericFields.length) {
    failed = true;
  }

  report.push([
    typeName,
    details.count,
    missingFields.length ? `missing fields: ${missingFields.join(", ")}` : "fields covered",
    genericFields.length ? `generic groups: ${genericFields.join(", ")}` : "specific groups",
  ].join("\t"));
}

console.log("Component\tOccurrences\tCoverage\tGroups");
console.log(report.join("\n"));

if (failed) {
  process.exit(1);
}

function parseRegisteredDescriptors(source) {
  const registryMatch = source.match(/pub fn default_component_registry\(\) -> ComponentRegistry \{[\s\S]*?ComponentRegistry::new\(\[([\s\S]*?)\]\)[\s\S]*?\}/u);
  if (!registryMatch) {
    throw new Error("default_component_registry() not found");
  }

  const registeredFunctions = new Set(
    [...registryMatch[1].matchAll(/([a-z0-9_]+_descriptor)\(\)/gu)].map((match) => match[1]),
  );

  const functions = new Map();
  const functionMatches = [...source.matchAll(/pub fn ([a-z0-9_]+_descriptor)\(\) -> ComponentTypeDescriptor \{/gu)];
  for (let index = 0; index < functionMatches.length; index += 1) {
    const name = functionMatches[index][1];
    const start = functionMatches[index].index;
    const end = functionMatches[index + 1]?.index ?? source.indexOf("pub fn default_component_registry", start);
    functions.set(name, source.slice(start, end > start ? end : undefined));
  }

  const descriptors = new Map();
  for (const functionName of registeredFunctions) {
    const body = functions.get(functionName);
    if (!body) continue;

    const typeName =
      body.match(/type_name:\s*"([^"]+)"/u)?.[1] ??
      body.match(/ComponentKind::[A-Za-z0-9_]+,\s*"([^"]+)"/u)?.[1];
    if (!typeName) continue;

    const propertyGroups = parsePropertyGroups(body);
    const properties = new Set(propertyGroups.keys());
    const assetRefs = new Set([...body.matchAll(/field_path:\s*"([^"]+)"/gu)].map((match) => match[1]));
    const genericGroups = new Set();

    for (const [field, group] of propertyGroups) {
      if (group === "generic.properties") {
        genericGroups.add(field);
      }
    }

    descriptors.set(typeName, { properties, assetRefs, genericGroups });
  }

  return descriptors;
}

function parseYamlComponentUsage() {
  const files = execFileSync("git", ["ls-files", "mods/*.yml", "mods/*.yaml"], {
    cwd: repoRoot,
    encoding: "utf8",
  })
    .split("\n")
    .filter(Boolean);
  const usage = new Map();

  for (const file of files) {
    const lines = readFileSync(path.join(repoRoot, file), "utf8").split(/\r?\n/u);

    for (let index = 0; index < lines.length; index += 1) {
      const match = lines[index].match(/^(\s*)-\s+type:\s*([A-Za-z0-9_]+)\s*$/u);
      if (!match) continue;

      const indent = match[1].length;
      const typeName = match[2];
      const details = usage.get(typeName) ?? { count: 0, fields: new Map() };
      details.count += 1;

      for (let cursor = index + 1; cursor < lines.length; cursor += 1) {
        const line = lines[cursor];
        if (!line.trim() || line.trimStart().startsWith("#")) continue;

        const currentIndent = line.length - line.trimStart().length;
        if (currentIndent <= indent) break;
        if (currentIndent !== indent + 2) continue;

        const fieldMatch = line.match(/^\s*([A-Za-z0-9_-]+):/u);
        if (fieldMatch) {
          details.fields.set(fieldMatch[1], (details.fields.get(fieldMatch[1]) ?? 0) + 1);
        }
      }

      usage.set(typeName, details);
    }
  }

  return usage;
}

function parsePropertyGroups(body) {
  const groups = new Map();

  for (const match of body.matchAll(/EditorPropertyDescriptor\s*\{([\s\S]*?)\n\s*\}/gu)) {
    const field = match[1].match(/\bpath:\s*"([^"]+)"/u)?.[1];
    const group = match[1].match(/\bgroup:\s*"([^"]+)"/u)?.[1] ?? "unknown";
    if (field) groups.set(field, group);
  }

  for (const call of macroCalls(body, "p!")) {
    const strings = [...call.matchAll(/"([^"]+)"/gu)].map((match) => match[1]);
    if (strings.length >= 2) {
      groups.set(strings[0], strings[strings.length - 1]);
    }
  }

  return groups;
}

function macroCalls(body, name) {
  const calls = [];
  let searchIndex = 0;

  while (searchIndex < body.length) {
    const start = body.indexOf(`${name}(`, searchIndex);
    if (start === -1) break;

    let depth = 0;
    let cursor = start + name.length;
    let inString = false;
    let escaped = false;

    for (; cursor < body.length; cursor += 1) {
      const char = body[cursor];
      if (inString) {
        escaped = char === "\\" && !escaped;
        if (char === '"' && !escaped) inString = false;
        if (char !== "\\") escaped = false;
        continue;
      }
      if (char === '"') {
        inString = true;
      } else if (char === "(") {
        depth += 1;
      } else if (char === ")") {
        depth -= 1;
        if (depth === 0) {
          calls.push(body.slice(start, cursor + 1));
          cursor += 1;
          break;
        }
      }
    }

    searchIndex = cursor;
  }

  return calls;
}
