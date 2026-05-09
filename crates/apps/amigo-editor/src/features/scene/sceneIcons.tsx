import {
  AlertTriangle,
  Boxes,
  Camera,
  CircleDot,
  Cuboid,
  FileCode2,
  FileText,
  FolderOpen,
  Gauge,
  Grid3X3,
  Image,
  Keyboard,
  ListTree,
  Music,
  Package,
  Play,
  Sparkles,
  Square,
  Type,
  Wand2,
} from "lucide-react";
import type { SemanticTone } from "../../theme/semanticColorRegistry";
import { semanticIconClass, toneForFileKind } from "../../theme/semanticColorRegistry";

export function sceneIcon(name: string, size = 14) {
  const className = semanticIconClass(toneForSceneIcon(name));
  switch (name) {
    case "scene":
      return <FileText size={size} className={className} />;
    case "script":
      return <FileCode2 size={size} className={className} />;
    case "assets":
      return <Boxes size={size} className={className} />;
    case "spritesheet":
      return <Grid3X3 size={size} className={className} />;
    case "tilemap":
      return <Grid3X3 size={size} className={className} />;
    case "audio":
      return <Music size={size} className={className} />;
    case "font":
      return <Type size={size} className={className} />;
    case "image":
      return <Image size={size} className={className} />;
    case "entity":
      return <Package size={size} className={className} />;
    case "entities":
      return <ListTree size={size} className={className} />;
    case "entity-ui":
      return <Square size={size} className={className} />;
    case "entity-camera":
      return <Camera size={size} className={className} />;
    case "entity-physics":
      return <CircleDot size={size} className={className} />;
    case "entity-motion":
      return <Gauge size={size} className={className} />;
    case "entity-render":
      return <Image size={size} className={className} />;
    case "entity-behavior":
      return <Play size={size} className={className} />;
    case "entity-input":
      return <Keyboard size={size} className={className} />;
    case "entity-particles":
      return <Sparkles size={size} className={className} />;
    case "entity-3d":
      return <Cuboid size={size} className={className} />;
    case "entity-other":
      return <Package size={size} className={className} />;
    case "diagnostics":
      return <AlertTriangle size={size} className={className} />;
    case "folder":
      return <FolderOpen size={size} className={className} />;
    case "package":
      return <Package size={size} className={className} />;
    case "component":
      return <Wand2 size={size} className={className} />;
    default:
      return <Package size={size} className={className} />;
  }
}

function toneForSceneIcon(name: string): SemanticTone {
  if (name === "scene") return toneForFileKind("sceneDocument");
  if (name === "script") return toneForFileKind("sceneScript");
  if (name === "assets") return "domain-assets";
  if (name === "spritesheet") return "asset-sprite";
  if (name === "tilemap") return "asset-tilemap";
  if (name === "audio") return "asset-audio";
  if (name === "font") return "asset-font";
  if (name === "image") return "asset-image";
  if (name === "diagnostics") return "domain-diagnostics";
  if (name === "entity-physics") return "domain-physics_2d";
  if (name === "entity-motion") return "domain-motion_2d";
  if (name === "entity-particles") return "domain-particles_2d";
  if (name === "entity-render") return "domain-rendering_2d";
  if (name === "entity-3d") return "domain-rendering_2d";
  if (name === "entity-behavior" || name === "entity-input") return "domain-scripting";
  if (name === "folder") return "domain-project";
  if (name === "package") return "domain-modding";
  return "domain-scene";
}
