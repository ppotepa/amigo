export interface WindowEventEnvelope<T> {
  eventId: string;
  eventType: string;
  sourceWindow?: string | null;
  sessionId?: string | null;
  timestampMs: number;
  schemaVersion: number;
  payload: T;
}

export interface ThemeSettingsChangedPayload {
  activeThemeId: string;
}

export interface FontSettingsChangedPayload {
  activeFontId: string;
}

export interface WorkspaceOpenedPayload {
  sessionId: string;
  modId: string;
}

export interface WorkspaceClosedPayload {
  sessionId: string;
}

export interface WindowLifecyclePayload {
  windowLabel: string;
}

export interface SessionClosedPayload {
  sessionId: string;
}

export interface CacheInvalidatedPayload {
  projectCacheId?: string | null;
  modId?: string | null;
  sceneId?: string | null;
  sourceHash?: string | null;
  cacheKind: string;
  reason: string;
}

export interface AssetRegistryChangedPayload {
  modId: string;
}

export interface AssetDescriptorChangedPayload {
  modId: string;
  assetKey: string;
  descriptorRelativePath: string;
  reason: string;
}

export type WindowBusEvent =
  | ({
      type: "ThemeSettingsChanged";
    } & WindowEventEnvelope<ThemeSettingsChangedPayload>)
  | ({
      type: "FontSettingsChanged";
    } & WindowEventEnvelope<FontSettingsChangedPayload>)
  | ({
      type: "WorkspaceOpened";
    } & WindowEventEnvelope<WorkspaceOpenedPayload>)
  | ({
      type: "WorkspaceClosed";
    } & WindowEventEnvelope<WorkspaceClosedPayload>)
  | ({
      type: "WindowCloseRequested";
    } & WindowEventEnvelope<WindowLifecyclePayload>)
  | ({
      type: "WindowFocused";
    } & WindowEventEnvelope<WindowLifecyclePayload>)
  | ({
      type: "SessionClosed";
    } & WindowEventEnvelope<SessionClosedPayload>)
  | ({
      type: "CacheInvalidated";
    } & WindowEventEnvelope<CacheInvalidatedPayload>)
  | ({
      type: "AssetRegistryChanged";
    } & WindowEventEnvelope<AssetRegistryChangedPayload>)
  | ({
      type: "AssetDescriptorChanged";
    } & WindowEventEnvelope<AssetDescriptorChangedPayload>);
