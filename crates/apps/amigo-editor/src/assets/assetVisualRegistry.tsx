import { Boxes, FileCode2, FileImage, Folder, Grid2X2, Image, Map, Music, Package, Play, Type } from "lucide-react";
import { cloneElement } from "react";
import type React from "react";

export type AssetVisualTone =
  | "asset-image"
  | "asset-sprite"
  | "asset-tileset"
  | "asset-tilemap"
  | "asset-audio"
  | "asset-font"
  | "asset-scene"
  | "asset-script"
  | "asset-raw-image"
  | "asset-generic";

export type AssetVisualDefinition = {
  icon: React.ReactNode;
  label: string;
  tone: AssetVisualTone;
};

function assetIcon(icon: React.ReactElement, tone: AssetVisualTone): React.ReactNode {
  return cloneElement(icon, {
    className: `semantic-icon ${tone}`,
  } as React.SVGProps<SVGSVGElement>);
}

export function assetVisualForKind(kind: string): AssetVisualDefinition {
  if (kind === "root") {
    return { icon: assetIcon(<Package size={13} />, "asset-generic"), label: "Assets", tone: "asset-generic" };
  }
  if (kind.includes("raw") || kind.includes("media") || kind.startsWith("image/")) {
    return { icon: assetIcon(<FileImage size={13} />, "asset-raw-image"), label: "Raw Images", tone: "asset-raw-image" };
  }
  if (kind.includes("image")) {
    return { icon: assetIcon(<Image size={13} />, "asset-image"), label: "Images", tone: "asset-image" };
  }
  if (kind.includes("sprite")) {
    return { icon: assetIcon(<Boxes size={13} />, "asset-sprite"), label: "Sprites", tone: "asset-sprite" };
  }
  if (kind.includes("tileset") || kind.includes("tile-ruleset")) {
    return { icon: assetIcon(<Grid2X2 size={13} />, "asset-tileset"), label: kind.includes("rule") ? "Rulesets" : "Tilesets", tone: "asset-tileset" };
  }
  if (kind.includes("tilemap")) {
    return { icon: assetIcon(<Map size={13} />, "asset-tilemap"), label: "Tilemaps", tone: "asset-tilemap" };
  }
  if (kind.includes("audio")) {
    return { icon: assetIcon(<Music size={13} />, "asset-audio"), label: "Audio", tone: "asset-audio" };
  }
  if (kind.includes("font")) {
    return { icon: assetIcon(<Type size={13} />, "asset-font"), label: "Fonts", tone: "asset-font" };
  }
  if (kind.includes("scene")) {
    return { icon: assetIcon(<Play size={13} />, "asset-scene"), label: "Scenes", tone: "asset-scene" };
  }
  if (kind.includes("script")) {
    return { icon: assetIcon(<FileCode2 size={13} />, "asset-script"), label: "Scripts", tone: "asset-script" };
  }
  return { icon: assetIcon(<FileCode2 size={13} />, "asset-generic"), label: kind || "Asset", tone: "asset-generic" };
}

export function assetFolderVisualForKind(kind: string): AssetVisualDefinition {
  const visual = assetVisualForKind(kind);
  if (visual.tone === "asset-generic" && kind !== "root") {
    return { icon: assetIcon(<Folder size={13} />, visual.tone), label: visual.label, tone: visual.tone };
  }
  return visual;
}
