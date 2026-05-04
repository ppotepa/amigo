import { Boxes, FileCode2, FileImage, Folder, Grid2X2, Image, Map, Package } from "lucide-react";
import type React from "react";

export type AssetVisualTone =
  | "asset-image"
  | "asset-sprite"
  | "asset-tileset"
  | "asset-tilemap"
  | "asset-raw-image"
  | "asset-generic";

export type AssetVisualDefinition = {
  icon: React.ReactNode;
  label: string;
  tone: AssetVisualTone;
};

export function assetVisualForKind(kind: string): AssetVisualDefinition {
  if (kind === "root") {
    return { icon: <Package size={13} />, label: "Assets", tone: "asset-generic" };
  }
  if (kind.includes("raw") || kind.includes("media") || kind.startsWith("image/")) {
    return { icon: <FileImage size={13} />, label: "Raw Images", tone: "asset-raw-image" };
  }
  if (kind.includes("image")) {
    return { icon: <Image size={13} />, label: "Images", tone: "asset-image" };
  }
  if (kind.includes("sprite")) {
    return { icon: <Boxes size={13} />, label: "Sprites", tone: "asset-sprite" };
  }
  if (kind.includes("tileset") || kind.includes("tile-ruleset")) {
    return { icon: <Grid2X2 size={13} />, label: kind.includes("rule") ? "Rulesets" : "Tilesets", tone: "asset-tileset" };
  }
  if (kind.includes("tilemap")) {
    return { icon: <Map size={13} />, label: "Tilemaps", tone: "asset-tilemap" };
  }
  return { icon: <FileCode2 size={13} />, label: kind || "Asset", tone: "asset-generic" };
}

export function assetFolderVisualForKind(kind: string): AssetVisualDefinition {
  const visual = assetVisualForKind(kind);
  if (visual.tone === "asset-generic" && kind !== "root") {
    return { icon: <Folder size={13} />, label: visual.label, tone: visual.tone };
  }
  return visual;
}
