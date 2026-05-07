import {
  Box,
  CircleHelp,
  Columns3,
  FileText,
  Image,
  Layers3,
  MousePointerClick,
  Rows3,
  Square,
  Type,
} from "lucide-react";
import type { EditorUiNodeKindDto } from "../../api/dto";

// @codemap anchor:ui-node-kind-icons domain:ui-document role:tree-icon-resolver priority:P1 layer:app tags:tree,icons,node-kind
export function UiNodeKindIcon({
  kind,
  size = 14,
}: {
  kind: EditorUiNodeKindDto | string;
  size?: number;
}) {
  switch (kind) {
    case "column":
      return <Columns3 size={size} className="ui-node-kind-icon kind-layout" />;
    case "row":
      return <Rows3 size={size} className="ui-node-kind-icon kind-layout" />;
    case "panel":
    case "group-box":
      return <Square size={size} className="ui-node-kind-icon kind-container" />;
    case "stack":
    case "tab-view":
      return <Layers3 size={size} className="ui-node-kind-icon kind-container" />;
    case "text":
      return <Type size={size} className="ui-node-kind-icon kind-text" />;
    case "button":
      return <MousePointerClick size={size} className="ui-node-kind-icon kind-control" />;
    case "image":
      return <Image size={size} className="ui-node-kind-icon kind-media" />;
    case "progress-bar":
    case "slider":
    case "toggle":
    case "option-set":
    case "dropdown":
    case "color-picker-rgb":
    case "curve-editor":
      return <Rows3 size={size} className="ui-node-kind-icon kind-control" />;
    case "spacer":
      return <Box size={size} className="ui-node-kind-icon kind-spacer" />;
    case "unknown":
      return <CircleHelp size={size} className="ui-node-kind-icon kind-unknown" />;
    default:
      return <FileText size={size} className="ui-node-kind-icon kind-node" />;
  }
}

export function uiNodeKindLabel(kind: EditorUiNodeKindDto | string): string {
  switch (kind) {
    case "column":
    case "row":
    case "panel":
    case "group-box":
    case "stack":
      return "layout";
    case "text":
      return "text";
    case "button":
      return "button";
    case "image":
      return "image";
    case "progress-bar":
    case "slider":
    case "toggle":
    case "option-set":
    case "dropdown":
    case "color-picker-rgb":
    case "curve-editor":
      return "control";
    case "spacer":
      return "spacer";
    default:
      return kind;
  }
}
