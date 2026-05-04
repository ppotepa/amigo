import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type { FontId } from "../theme/fontRegistry";
import { WINDOW_BUS_SCHEMA_VERSION } from "./windowBusContract";
import type {
  CacheInvalidatedPayload,
  AssetDescriptorChangedPayload,
  AssetRegistryChangedPayload,
  FontSettingsChangedPayload,
  SessionClosedPayload,
  ThemeSettingsChangedPayload,
  WindowLifecyclePayload,
  WindowBusEvent,
  WindowEventEnvelope,
  WorkspaceClosedPayload,
  WorkspaceOpenedPayload,
} from "./windowBusTypes";

export const WINDOW_BUS_EVENTS = {
  themeSettingsChanged: "theme-settings-changed",
  fontSettingsChanged: "font-settings-changed",
  editorSettingsChanged: "editor-settings-changed",
  workspaceOpened: "workspace-opened",
  workspaceClosed: "workspace-closed",
  windowCloseRequested: "window-close-requested",
  windowFocused: "window-focused",
  sessionClosed: "session-closed",
  cacheInvalidated: "cache-invalidated",
  assetRegistryChanged: "asset-registry-changed",
  assetDescriptorChanged: "asset-descriptor-changed",
  scenePreviewCompleted: "scene-preview-completed",
  diagnosticsUpdated: "diagnostics-updated",
} as const;

const WINDOW_BUS_EVENT_NAMES = [
  WINDOW_BUS_EVENTS.themeSettingsChanged,
  WINDOW_BUS_EVENTS.fontSettingsChanged,
  WINDOW_BUS_EVENTS.workspaceOpened,
  WINDOW_BUS_EVENTS.workspaceClosed,
  WINDOW_BUS_EVENTS.windowCloseRequested,
  WINDOW_BUS_EVENTS.windowFocused,
  WINDOW_BUS_EVENTS.sessionClosed,
  WINDOW_BUS_EVENTS.cacheInvalidated,
  WINDOW_BUS_EVENTS.assetRegistryChanged,
  WINDOW_BUS_EVENTS.assetDescriptorChanged,
] as const;

export async function listenWindowBus(handler: (event: WindowBusEvent) => void): Promise<() => void> {
  const unlistenFns = await Promise.all(
    WINDOW_BUS_EVENT_NAMES.map((eventName) =>
      listen<WindowEventEnvelope<unknown>>(eventName, (event) => {
        const mapped = mapRawEvent(event.payload);
        if (mapped) {
          handler(mapped);
        }
      }),
    ),
  );

  return () => {
    for (const unlisten of unlistenFns) {
      unlisten();
    }
  };
}

export function listenThemeSettingsChanged(
  handler: (settings: ThemeSettingsChangedPayload) => void,
): Promise<() => void> {
  return listenTypedWindowEvent<ThemeSettingsChangedPayload>(WINDOW_BUS_EVENTS.themeSettingsChanged, (event) => {
    handler(event.payload);
  });
}

export function listenFontSettingsChanged(
  handler: (payload: FontSettingsChangedPayload) => void,
): Promise<() => void> {
  return listenTypedWindowEvent<FontSettingsChangedPayload>(WINDOW_BUS_EVENTS.fontSettingsChanged, (event) => {
    handler(event.payload);
  });
}

export async function emitFontSettingsChanged(activeFontId: FontId): Promise<void> {
  await emitWindowEvent(WINDOW_BUS_EVENTS.fontSettingsChanged, { activeFontId });
}

export async function emitWorkspaceOpened(sessionId: string, modId: string): Promise<void> {
  await emitWindowEvent(WINDOW_BUS_EVENTS.workspaceOpened, { sessionId, modId }, sessionId);
}

export async function emitWorkspaceClosed(sessionId: string): Promise<void> {
  await emitWindowEvent(WINDOW_BUS_EVENTS.workspaceClosed, { sessionId }, sessionId);
}

export async function emitWindowCloseRequested(sessionId?: string | null): Promise<void> {
  await emitWindowEvent(
    WINDOW_BUS_EVENTS.windowCloseRequested,
    { windowLabel: getCurrentWindow().label },
    sessionId,
  );
}

export async function emitWindowFocused(sessionId?: string | null): Promise<void> {
  await emitWindowEvent(WINDOW_BUS_EVENTS.windowFocused, { windowLabel: getCurrentWindow().label }, sessionId);
}

function listenTypedWindowEvent<T>(
  eventName: string,
  handler: (event: WindowEventEnvelope<T>) => void,
): Promise<() => void> {
  return listen<WindowEventEnvelope<T>>(eventName, (event) => {
    handler(event.payload);
  });
}

async function emitWindowEvent<T>(eventName: string, payload: T, sessionId?: string | null): Promise<void> {
  const envelope: WindowEventEnvelope<T> = {
    eventId: newEventId(),
    eventType: eventName,
    sourceWindow: getCurrentWindow().label,
    sessionId: sessionId ?? null,
    timestampMs: Date.now(),
    schemaVersion: WINDOW_BUS_SCHEMA_VERSION,
    payload,
  };

  await emit(eventName, envelope);
}

function mapRawEvent(raw: WindowEventEnvelope<unknown>): WindowBusEvent | null {
  switch (raw.eventType) {
    case WINDOW_BUS_EVENTS.themeSettingsChanged:
      return {
        ...raw,
        type: "ThemeSettingsChanged",
        payload: raw.payload as ThemeSettingsChangedPayload,
      };
    case WINDOW_BUS_EVENTS.fontSettingsChanged:
      return {
        ...raw,
        type: "FontSettingsChanged",
        payload: raw.payload as FontSettingsChangedPayload,
      };
    case WINDOW_BUS_EVENTS.workspaceOpened:
      return {
        ...raw,
        type: "WorkspaceOpened",
        payload: raw.payload as WorkspaceOpenedPayload,
      };
    case WINDOW_BUS_EVENTS.workspaceClosed:
      return {
        ...raw,
        type: "WorkspaceClosed",
        payload: raw.payload as WorkspaceClosedPayload,
      };
    case WINDOW_BUS_EVENTS.windowCloseRequested:
      return {
        ...raw,
        type: "WindowCloseRequested",
        payload: raw.payload as WindowLifecyclePayload,
      };
    case WINDOW_BUS_EVENTS.windowFocused:
      return {
        ...raw,
        type: "WindowFocused",
        payload: raw.payload as WindowLifecyclePayload,
      };
    case WINDOW_BUS_EVENTS.sessionClosed:
      return {
        ...raw,
        type: "SessionClosed",
        payload: raw.payload as SessionClosedPayload,
      };
    case WINDOW_BUS_EVENTS.cacheInvalidated:
      return {
        ...raw,
        type: "CacheInvalidated",
        payload: raw.payload as CacheInvalidatedPayload,
      };
    case WINDOW_BUS_EVENTS.assetRegistryChanged:
      return {
        ...raw,
        type: "AssetRegistryChanged",
        payload: raw.payload as AssetRegistryChangedPayload,
      };
    case WINDOW_BUS_EVENTS.assetDescriptorChanged:
      return {
        ...raw,
        type: "AssetDescriptorChanged",
        payload: raw.payload as AssetDescriptorChangedPayload,
      };
    default:
      return null;
  }
}

function newEventId(): string {
  return `evt-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}
