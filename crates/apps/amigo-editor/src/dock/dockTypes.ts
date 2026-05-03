import type React from "react";
import type { DockAreaId } from "../main-window/workspaceLayout";

export interface EditorDockContext {
  sessionId: string | null;
  modId: string | null;
  selectedSceneId: string | null;
}

export interface DockPlugin {
  id: string;
  componentId: string;
  title: string;
  icon: React.ReactNode;
  defaultDock: DockAreaId;
  defaultSize?: number;
  canOpen: (context: EditorDockContext) => boolean;
}
