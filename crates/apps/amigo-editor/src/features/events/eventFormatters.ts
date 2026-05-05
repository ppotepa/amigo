import type { WindowBusEvent } from "../../app/windowBusTypes";

const EVENT_CATEGORY_MATCHERS: Record<string, (event: WindowBusEvent) => boolean> = {
  all: () => true,
  window: (event) => event.type.includes("Window") || event.type.includes("Workspace") || event.type.includes("Session"),
  asset: (event) => event.type.includes("Asset"),
  workspace: (event) => event.type.includes("Workspace") || event.type.includes("Session"),
  cache: (event) => event.type.includes("Cache") || event.type.includes("Preview"),
  settings: (event) => event.type.includes("Settings") || event.type.includes("Theme") || event.type.includes("Font"),
};

export function windowEventMatchesCategory(event: WindowBusEvent, category: string): boolean {
  return (EVENT_CATEGORY_MATCHERS[category] ?? EVENT_CATEGORY_MATCHERS.all)(event);
}

const EVENT_PAYLOAD_FORMATTERS: Partial<Record<WindowBusEvent["type"], (event: WindowBusEvent) => string>> = {
  ThemeSettingsChanged: (event) => "activeThemeId" in event.payload ? event.payload.activeThemeId : "",
  FontSettingsChanged: (event) => "activeFontId" in event.payload ? event.payload.activeFontId : "",
  WorkspaceOpened: (event) => "modId" in event.payload && "sessionId" in event.payload ? `${event.payload.modId} · ${event.payload.sessionId}` : "",
  WorkspaceClosed: (event) => "sessionId" in event.payload ? event.payload.sessionId : "",
  SessionClosed: (event) => "sessionId" in event.payload ? event.payload.sessionId : "",
  WindowCloseRequested: (event) => "windowLabel" in event.payload ? event.payload.windowLabel : "",
  WindowFocused: (event) => "windowLabel" in event.payload ? event.payload.windowLabel : "",
  CacheInvalidated: (event) => "cacheKind" in event.payload && "reason" in event.payload ? `${event.payload.cacheKind}:${event.payload.reason}` : "",
};

export function formatWindowEventPayload(event: WindowBusEvent): string {
  return EVENT_PAYLOAD_FORMATTERS[event.type]?.(event) ?? "";
}
