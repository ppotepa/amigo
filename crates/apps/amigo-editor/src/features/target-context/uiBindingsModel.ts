import type {
  EditorUiDocumentDto,
  EditorUiNodeDto,
} from "../../api/dto";
import type { ResolvedEditorTarget } from "../../editor-targets/editorTargetTypes";
import type { WorkspaceRuntimeServices } from "../../main-window/workspaceRuntimeServices";

export type UiBindingKind =
  | "theme"
  | "text"
  | "color"
  | "background"
  | "visible"
  | "value"
  | "selected"
  | "options"
  | "enabled"
  | "unknown";

export type UiBindingViewModel = {
  id: string;
  path: string;
  state: string;
  kind: UiBindingKind;
  fallback?: string | null;
  format?: string | null;
  source: "scene-yaml";
  index: number;
};

export type UiBindingNodeStatus =
  | "resolved"
  | "missing-node"
  | "no-document";

export type UiBindingWithStatus = {
  binding: UiBindingViewModel;
  status: UiBindingNodeStatus;
  node: EditorUiNodeDto | null;
};

type PendingUiBinding = {
  id?: string;
  path?: string;
  state?: string;
  kind?: string;
  fallback?: string | null;
  format?: string | null;
};

// @codemap anchor:ui-bindings-model domain:ui-document role:model priority:P1 layer:app tags:bindings,ui-document,editor-target
export function parseUiModelBindingsFromYaml(content: string | null | undefined): UiBindingViewModel[] {
  if (!content) return [];

  const result: UiBindingViewModel[] = [];
  const lines = content.split(/\r?\n/);
  let insideUiModelBindings = false;
  let insideBindings = false;
  let current: PendingUiBinding | null = null;

  function pushCurrent() {
    if (!current?.path || !current.state || !current.kind) {
      current = null;
      return;
    }

    const index = result.length;
    result.push({
      id: current.id ?? `${current.path}:${current.state}:${current.kind}:${index}`,
      path: current.path,
      state: current.state,
      kind: normalizeBindingKind(current.kind),
      fallback: current.fallback ?? null,
      format: current.format ?? null,
      source: "scene-yaml",
      index,
    });
    current = null;
  }

  for (const line of lines) {
    const trimmed = line.trim();

    if (/^-\s+type:\s*UiModelBindings\s*$/.test(trimmed)) {
      pushCurrent();
      insideUiModelBindings = true;
      insideBindings = false;
      continue;
    }

    if (insideUiModelBindings && /^bindings:\s*$/.test(trimmed)) {
      insideBindings = true;
      continue;
    }

    if (!insideBindings) continue;

    if (/^-\s+type:\s+/.test(trimmed)) {
      pushCurrent();
      insideUiModelBindings = false;
      insideBindings = false;
      continue;
    }

    const pathMatch = trimmed.match(/^-\s+path:\s*(.+)$/);
    if (pathMatch) {
      pushCurrent();
      current = {
        path: unquoteYamlScalar(pathMatch[1]),
      };
      continue;
    }

    if (!current) continue;

    const fieldMatch = trimmed.match(/^([A-Za-z_][A-Za-z0-9_]*):\s*(.*)$/);
    if (!fieldMatch) continue;

    const [, key, rawValue] = fieldMatch;
    const value = unquoteYamlScalar(rawValue);

    if (key === "id") current.id = value;
    if (key === "state") current.state = value;
    if (key === "kind") current.kind = value;
    if (key === "fallback") current.fallback = value;
    if (key === "format") current.format = value;
  }

  pushCurrent();
  return result;
}

export function bindingsForUiDocument(
  bindings: UiBindingViewModel[],
  document: EditorUiDocumentDto | null,
): UiBindingWithStatus[] {
  return bindings.map((binding) => {
    if (!document) {
      return { binding, status: "no-document", node: null };
    }

    const node = findBindingTargetNode(document.root, binding.path);
    return {
      binding,
      status: node ? "resolved" : "missing-node",
      node,
    };
  });
}

export function bindingsForUiNode(
  bindings: UiBindingViewModel[],
  document: EditorUiDocumentDto | null,
  nodePath: string | null | undefined,
): UiBindingWithStatus[] {
  if (!nodePath) return [];
  return bindingsForUiDocument(bindings, document).filter((entry) =>
    bindingTargetsNode(entry.binding.path, nodePath),
  );
}

export function currentUiDocumentForTarget(
  target: ResolvedEditorTarget,
  services: WorkspaceRuntimeServices,
): EditorUiDocumentDto | null {
  const ref = target.ref;
  if (ref.kind !== "uiDocument" && ref.kind !== "uiNode") return null;

  return services.hierarchy?.uiDocuments.find(
    (document) =>
      document.entityId === ref.entityId &&
      document.componentIndex === ref.componentIndex,
  ) ?? null;
}

export function currentUiNodePathForTarget(
  target: ResolvedEditorTarget,
): string | null {
  return target.ref.kind === "uiNode" ? target.ref.nodePath : null;
}

export function uiBindingKindLabel(kind: UiBindingKind): string {
  if (kind === "theme") return "Theme";
  if (kind === "text") return "Text";
  if (kind === "color") return "Color";
  if (kind === "background") return "Background";
  if (kind === "visible") return "Visible";
  if (kind === "value") return "Value";
  if (kind === "selected") return "Selected";
  if (kind === "options") return "Options";
  if (kind === "enabled") return "Enabled";
  return "Unknown";
}

export function uiBindingStatusLabel(status: UiBindingNodeStatus): string {
  if (status === "resolved") return "OK";
  if (status === "missing-node") return "Missing node";
  return "No document";
}

export function collectUiBindingsFromServices(
  services: WorkspaceRuntimeServices,
  document: EditorUiDocumentDto | null,
): UiBindingViewModel[] {
  if (document?.bindings.length) {
    return document.bindings.map((binding, index) => ({
      id: binding.id,
      path: binding.path,
      state: binding.state,
      kind: normalizeBindingKind(binding.kind),
      fallback: null,
      format: binding.format ?? null,
      source: "scene-yaml",
      index,
    }));
  }

  return parseUiModelBindingsFromYaml(services.selectedFileContent?.content);
}

function findBindingTargetNode(
  root: EditorUiNodeDto,
  bindingPath: string,
): EditorUiNodeDto | null {
  const nodes = flattenUiNodes(root);
  return nodes.find((node) => bindingTargetsNode(bindingPath, node.path)) ?? null;
}

function bindingTargetsNode(bindingPath: string, nodePath: string): boolean {
  const binding = normalizeBindingPath(bindingPath);
  const node = normalizeBindingPath(nodePath);

  return binding === node || binding.endsWith(`.${node}`);
}

function flattenUiNodes(root: EditorUiNodeDto): EditorUiNodeDto[] {
  return [root, ...root.children.flatMap(flattenUiNodes)];
}

function normalizeBindingKind(value: string): UiBindingKind {
  if (value === "theme") return "theme";
  if (value === "text") return "text";
  if (value === "color") return "color";
  if (value === "background") return "background";
  if (value === "visible") return "visible";
  if (value === "value") return "value";
  if (value === "selected") return "selected";
  if (value === "options") return "options";
  if (value === "enabled") return "enabled";
  return "unknown";
}

function normalizeBindingPath(path: string): string {
  return path.trim().replace(/\[/g, ".").replace(/\]/g, "").replace(/\.+/g, ".").toLowerCase();
}

function unquoteYamlScalar(value: string): string {
  const trimmed = value.trim();
  if (
    (trimmed.startsWith('"') && trimmed.endsWith('"')) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    return trimmed.slice(1, -1);
  }
  return trimmed;
}
