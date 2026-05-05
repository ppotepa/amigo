import type { AssetVisualTone } from "../assets/assetVisualRegistry";
import type { EditorComponentDomain } from "../editor-components/componentTypes";

export type SemanticTone =
  | `domain-${EditorComponentDomain}`
  | AssetVisualTone
  | "status-success"
  | "status-warning"
  | "status-error"
  | "status-info"
  | "status-running"
  | "action-danger"
  | "action-success"
  | "action-refresh"
  | "action-preview"
  | "neutral";

export function toneForComponentDomain(domain: EditorComponentDomain): SemanticTone {
  return `domain-${domain}`;
}

export function toneForAssetKind(kind: string): SemanticTone {
  const normalized = kind.toLowerCase();
  if (normalized.includes("raw") || normalized.includes("media") || normalized.startsWith("image/")) return "asset-raw-image";
  if (normalized.includes("sprite")) return "asset-sprite";
  if (normalized.includes("tileset") || normalized.includes("tile-ruleset")) return "asset-tileset";
  if (normalized.includes("tilemap")) return "asset-tilemap";
  if (normalized.includes("audio")) return "asset-audio";
  if (normalized.includes("font")) return "asset-font";
  if (normalized.includes("scene")) return "asset-scene";
  if (normalized.includes("script")) return "asset-script";
  if (normalized.includes("image")) return "asset-image";
  return "asset-generic";
}

export function toneForFileKind(kindOrPath: string | null | undefined): SemanticTone {
  const normalized = (kindOrPath ?? "").toLowerCase();
  if (!normalized) return "neutral";
  if (normalized.includes("scene") && (normalized.includes("document") || normalized.endsWith(".yml") || normalized.endsWith(".yaml"))) return "asset-scene";
  if (normalized.includes("script") || normalized.endsWith(".rhai") || normalized.endsWith(".ts") || normalized.endsWith(".js")) return "asset-script";
  if (normalized.includes("manifest") || normalized.endsWith(".toml")) return "domain-modding";
  return toneForAssetKind(normalized);
}

export function toneForActionId(actionId: string): SemanticTone {
  const normalized = actionId.toLowerCase();
  if (normalized.includes("delete") || normalized.includes("remove") || normalized.includes("close")) return "action-danger";
  if (normalized.includes("save") || normalized.includes("apply") || normalized.includes("commit") || normalized.includes("validate")) return "action-success";
  if (normalized.includes("refresh") || normalized.includes("regenerate") || normalized.includes("rescan")) return "action-refresh";
  if (normalized.includes("play") || normalized.includes("preview") || normalized.includes("open")) return "action-preview";
  if (normalized.includes("settings") || normalized.includes("configure")) return "domain-settings";
  if (normalized.includes("problem") || normalized.includes("diagnostic")) return "domain-diagnostics";
  return "neutral";
}

export function toneForStatus(status: string | null | undefined): SemanticTone {
  switch (status) {
    case "valid":
    case "ok":
    case "success":
      return "status-success";
    case "warning":
    case "orphan":
    case "dirty":
    case "missingSource":
      return "status-warning";
    case "error":
    case "invalid":
    case "missing":
      return "status-error";
    case "running":
    case "busy":
      return "status-running";
    case "cached":
    case "info":
      return "status-info";
    default:
      return "neutral";
  }
}

export function semanticIconClass(tone: SemanticTone): string {
  return `semantic-icon ${tone}`;
}
