import type { ComponentPlacement, EditorSerializedComponentContext } from "./componentTypes";

export type EditorComponentEvent =
  | { type: "ComponentOpenRequested"; componentId: string; context?: EditorSerializedComponentContext }
  | { type: "ComponentOpened"; instanceId: string; componentId: string }
  | { type: "ComponentClosed"; instanceId: string }
  | { type: "ComponentFocused"; instanceId: string }
  | { type: "ComponentMoved"; instanceId: string; placement: ComponentPlacement }
  | { type: "ComponentDocked"; instanceId: string; dock: "left" | "right" | "bottom" }
  | { type: "ComponentUndocked"; instanceId: string }
  | { type: "ComponentOpenedInWindow"; instanceId: string; windowLabel: string };
