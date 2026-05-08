import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { EditorSerializedComponentContext } from "../editor-components/componentTypes";

interface EditorWindowDescriptor {
  label: string;
  title: string;
  url: string;
  width: number;
  height: number;
  minWidth: number;
  minHeight: number;
  resizable: boolean;
  maximizable: boolean;
}

function workspaceDescriptor(sessionId: string, title = "Workspace"): EditorWindowDescriptor {
  return {
    label: `workspace-${sessionId}`,
    title: `Amigo Editor - ${title}`,
    url: `/index.html?window=workspace&sessionId=${encodeURIComponent(sessionId)}`,
    width: 1440,
    height: 900,
    minWidth: 1200,
    minHeight: 720,
    resizable: true,
    maximizable: true,
  };
}

export type DetachedWorkspaceWindowInput = {
  sessionId: string;
  workspaceId: string;
  title: string;
  componentId: string;
  context?: EditorSerializedComponentContext;
  filePath?: string | null;
  resourceUri?: string | null;
  titleOverride?: string | null;
};

function detachedWorkspaceDescriptor(input: DetachedWorkspaceWindowInput): EditorWindowDescriptor {
  const params = new URLSearchParams({
    window: "workspace",
    sessionId: input.sessionId,
    workspaceId: input.workspaceId,
    componentId: input.componentId,
    title: input.title,
  });

  if (input.context) {
    params.set("context", JSON.stringify(input.context));
  }
  if (input.filePath) {
    params.set("filePath", input.filePath);
  }
  if (input.resourceUri) {
    params.set("resourceUri", input.resourceUri);
  }
  if (input.titleOverride) {
    params.set("titleOverride", input.titleOverride);
  }

  return {
    label: `workspace-${input.sessionId}-${input.workspaceId}`,
    title: `Amigo Editor - ${input.title}`,
    url: `/index.html?${params.toString()}`,
    width: 1440,
    height: 900,
    minWidth: 1200,
    minHeight: 720,
    resizable: true,
    maximizable: true,
  };
}

function themeDescriptor(): EditorWindowDescriptor {
  return {
    label: "theme",
    title: "Theme Controller - Amigo Editor",
    url: "/index.html?window=theme",
    width: 1320,
    height: 940,
    minWidth: 1100,
    minHeight: 760,
    resizable: true,
    maximizable: false,
  };
}

function settingsDescriptor(): EditorWindowDescriptor {
  return {
    label: "settings",
    title: "Settings - Amigo Editor",
    url: "/index.html?window=settings",
    width: 1080,
    height: 780,
    minWidth: 900,
    minHeight: 640,
    resizable: true,
    maximizable: false,
  };
}

function modSettingsDescriptor(sessionId: string): EditorWindowDescriptor {
  return {
    label: `mod-settings-${sessionId}`,
    title: "Mod Settings - Amigo Editor",
    url: `/index.html?window=mod-settings&sessionId=${encodeURIComponent(sessionId)}`,
    width: 1080,
    height: 820,
    minWidth: 900,
    minHeight: 680,
    resizable: true,
    maximizable: false,
  };
}

async function openOrFocusEditorWindow(descriptor: EditorWindowDescriptor): Promise<void> {
  const existing = await WebviewWindow.getByLabel(descriptor.label);
  if (existing) {
    await existing.show();
    await existing.setFocus();
    return;
  }

  await new Promise<void>((resolve, reject) => {
    const webview = new WebviewWindow(descriptor.label, {
      url: descriptor.url,
      title: descriptor.title,
      width: descriptor.width,
      height: descriptor.height,
      minWidth: descriptor.minWidth,
      minHeight: descriptor.minHeight,
      resizable: descriptor.resizable,
      maximizable: descriptor.maximizable,
      center: true,
      focus: true,
      visible: true,
    });

    void webview.once("tauri://created", () => {
      void webview.setFocus().finally(resolve);
    });

    void webview.once("tauri://error", (event) => {
      reject(new Error(String(event.payload)));
    });
  });
}

export async function openWorkspaceWindow(sessionId: string, title?: string): Promise<void> {
  await openOrFocusEditorWindow(workspaceDescriptor(sessionId, title));
}

export async function openDetachedWorkspaceWindow(input: DetachedWorkspaceWindowInput): Promise<void> {
  await openOrFocusEditorWindow(detachedWorkspaceDescriptor(input));
}

export async function openThemeWindow(): Promise<void> {
  await openOrFocusEditorWindow(themeDescriptor());
}

export async function openSettingsWindow(): Promise<void> {
  await openOrFocusEditorWindow(settingsDescriptor());
}

export async function openModSettingsWindow(sessionId: string): Promise<void> {
  await openOrFocusEditorWindow(modSettingsDescriptor(sessionId));
}
