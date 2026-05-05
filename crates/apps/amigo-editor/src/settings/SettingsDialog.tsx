import { useEffect, useState } from "react";
import { useEditorStore } from "../app/editorStore";
import { selectedModId } from "../app/selectionSelectors";
import { listenWindowBus } from "../app/windowBus";
import {
  clearAllPreviewCache,
  clearOrphanedProjectCaches,
  clearProjectCache,
  getCacheInfo,
  getCachePolicy,
  getEditorSettings,
  getWindowRegistry,
  pickModsRoot,
  revealCacheFolder,
  runCacheMaintenance,
  setCachePolicy,
  setEditorModsRoot,
} from "../api/editorApi";
import type { CacheInfoDto, CachePolicyDto, EditorSettingsDto, EditorWindowRegistryDto } from "../api/dto";

function formatBytes(bytes: number): string {
  if (bytes === 0) {
    return "0 B";
  }
  const units = ["B", "KB", "MB", "GB"];
  let size = bytes;
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) {
    size = Math.floor(size / 1024);
    unit += 1;
  }
  return `${size} ${units[unit]}`;
}

export function SettingsDialog({
  open,
  onClose,
  onOpenTheme,
}: {
  open: boolean;
  onClose: () => void;
  onOpenTheme: () => void;
}) {
  if (!open) {
    return null;
  }

  return (
    <div className="settings-backdrop">
      <SettingsDialogContent onClose={onClose} onOpenTheme={onOpenTheme} />
    </div>
  );
}

export function SettingsDialogContent({
  onClose,
  onOpenTheme,
}: {
  onClose?: () => void;
  onOpenTheme?: () => void;
}) {
  const { state, recordEvent } = useEditorStore();
  const [settings, setSettings] = useState<EditorSettingsDto | null>(null);
  const [cacheInfo, setCacheInfo] = useState<CacheInfoDto | null>(null);
  const [cachePolicy, setCachePolicyState] = useState<CachePolicyDto | null>(null);
  const [windowRegistry, setWindowRegistry] = useState<EditorWindowRegistryDto | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  useEffect(() => {
    void (async () => {
      const loadedSettings = await getEditorSettings();
      setSettings(loadedSettings);
      const info = await getCacheInfo();
      setCacheInfo(info);
      const policy = await getCachePolicy();
      setCachePolicyState(policy);
      const registry = await getWindowRegistry();
      setWindowRegistry(registry);
    })();
  }, []);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listenWindowBus((event) => {
      if (cancelled) {
        return;
      }
      if (event.type === "CacheInvalidated") {
        void getCacheInfo().then((info) => {
          if (!cancelled) {
            setCacheInfo(info);
          }
        });
      }
      void getWindowRegistry().then((registry) => {
        if (!cancelled) {
          setWindowRegistry(registry);
        }
      });
    }).then((dispose) => {
      unlisten = dispose;
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  async function handleChooseModsRoot() {
    const pickedRoot = await pickModsRoot();
    if (!pickedRoot) {
      return;
    }
    const updated = await setEditorModsRoot(pickedRoot);
    setSettings(updated);
    recordEvent({ type: "ModsScanRequested" });
    setMessage(`Mods root set to ${pickedRoot}`);
  }

  async function handleClearProjectCache() {
    const projectCacheId = state.modDetails?.projectCacheId;
    if (!projectCacheId) {
      setMessage("Select a mod first to clear its preview cache.");
      return;
    }
    await clearProjectCache(projectCacheId);
    const info = await getCacheInfo();
    setCacheInfo(info);
    setMessage("Project cache cleared.");
  }

  async function handleClearAllPreviewCache() {
    await clearAllPreviewCache();
    const info = await getCacheInfo();
    setCacheInfo(info);
    setMessage("All preview caches cleared.");
  }

  async function handleSaveCachePolicy() {
    if (!cachePolicy) {
      return;
    }
    const updated = await setCachePolicy(cachePolicy);
    setCachePolicyState(updated);
    setMessage("Cache policy saved.");
  }

  async function handleRunMaintenance() {
    const result = await runCacheMaintenance();
    const info = await getCacheInfo();
    setCacheInfo(info);
    setMessage(`Maintenance removed ${result.removedEntries} entries (${formatBytes(result.removedBytes)}).`);
  }

  async function handleClearOrphans() {
    const result = await clearOrphanedProjectCaches();
    const info = await getCacheInfo();
    setCacheInfo(info);
    setMessage(`Removed ${result.orphanedProjectsRemoved} orphaned project cache(s).`);
  }

  return (
    <section className="settings-dialog">
      <header className="settings-dialog-header">
        <div>
          <h2>Editor Settings</h2>
          <p>Project paths, cache and presentation.</p>
        </div>
        <span className="badge badge-muted">{selectedModId(state.selection) ?? "no mod selected"}</span>
      </header>

      <main className="settings-grid">
        <section className="settings-panel">
          <h3>Mods</h3>
          <label className="settings-row">
            <span>Mods root</span>
            <input value={settings?.modsRoot ?? "(default)"} readOnly />
          </label>
          <button type="button" className="button button-ghost" onClick={() => void handleChooseModsRoot()}>
            Browse...
          </button>
        </section>

        <section className="settings-panel">
          <h3>Cache</h3>
          <p>Root: {cacheInfo?.cacheRoot ?? "(not loaded)"}</p>
          <p>Mode: {cacheInfo?.cacheRootMode ?? "..."}</p>
          <p>Total: {cacheInfo ? formatBytes(cacheInfo.totalSizeBytes) : "..."}</p>
          <p>Projects cached: {cacheInfo?.projectCount ?? 0}</p>
          <label className="settings-row">
            <span>Preview max MB</span>
            <input
              type="number"
              min={0}
              value={cachePolicy?.maxPreviewCacheBytes ? Math.floor(cachePolicy.maxPreviewCacheBytes / 1024 / 1024) : ""}
              onChange={(event) => {
                const value = Number(event.target.value);
                setCachePolicyState((policy) => ({
                  ...(policy ?? { autoCleanupEnabled: false }),
                  maxPreviewCacheBytes: value > 0 ? value * 1024 * 1024 : null,
                }));
              }}
            />
          </label>
          <label className="settings-check">
            <input
              type="checkbox"
              checked={cachePolicy?.autoCleanupEnabled ?? false}
              onChange={(event) =>
                setCachePolicyState((policy) => ({
                  ...(policy ?? { maxPreviewCacheBytes: null, maxAgeDays: null }),
                  autoCleanupEnabled: event.target.checked,
                }))
              }
            />
            <span>Auto cleanup</span>
          </label>
          <div className="settings-cache-actions">
            <button type="button" className="button button-ghost" onClick={() => void handleSaveCachePolicy()}>
              Save Policy
            </button>
            <button type="button" className="button button-ghost" onClick={() => void handleRunMaintenance()}>
              Maintain
            </button>
            <button type="button" className="button button-ghost" onClick={() => void handleClearOrphans()}>
              Orphans
            </button>
            <button type="button" className="button button-ghost" onClick={() => void revealCacheFolder()}>
              Reveal Cache
            </button>
            <button type="button" className="button button-ghost" onClick={() => void handleClearProjectCache()}>
              Clear Project
            </button>
            <button type="button" className="button button-ghost" onClick={() => void handleClearAllPreviewCache()}>
              Previews
            </button>
          </div>
        </section>

        <section className="settings-panel settings-projects-panel">
          <h3>Project Index</h3>
          <div className="settings-project-list">
            {(cacheInfo?.projects ?? []).map((project) => (
              <div className="settings-project-row interactive" key={project.projectCacheId}>
                <strong>{project.displayName}</strong>
                <span className="badge badge-info">{project.modId}</span>
                <small title={project.rootPath}>{project.rootPath}</small>
                <em>{formatBytes(project.projectSizeBytes)}</em>
              </div>
            ))}
          </div>
        </section>

        <section className="settings-panel">
          <h3>Theme</h3>
          <p>Change visual theme from the dedicated theme controller.</p>
          <button type="button" className="button button-ghost" onClick={onOpenTheme}>
            Open Theme Controller
          </button>
        </section>

        <section className="settings-panel settings-projects-panel">
          <h3>Window Registry</h3>
          <div className="settings-project-list">
            {(windowRegistry?.windows ?? []).map((window) => (
              <div className="settings-project-row interactive" key={window.label}>
                <strong>{window.label}</strong>
                <span className={window.focused ? "badge badge-valid" : "badge badge-muted"}>{window.kind}</span>
                <small>{window.sessionId ?? "global"}</small>
                <em>{window.focused ? "focused" : "open"}</em>
              </div>
            ))}
          </div>
        </section>
      </main>

      {message ? <p className="settings-message">{message}</p> : null}

      <footer className="settings-footer">
        <button className="button button-primary" type="button" onClick={onClose}>
          Close
        </button>
      </footer>
    </section>
  );
}
