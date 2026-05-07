import { Circle, Folder, FolderOpen, Plus, RefreshCcw, Search, ShieldCheck } from "lucide-react";
import { useMemo, useState } from "react";
import type { EditorModSummaryDto } from "../api/dto";
import { useEditorStore } from "../app/editorStore";
import { selectedModId } from "../app/selectionSelectors";
import { DebugSourceOverlay, useDebugSourceEnabled } from "../debug/debugSource";

function statusClass(status: string): string {
  return status === "valid" ? "is-valid" : "is-warning";
}

function isHiddenStartupProject(mod: EditorModSummaryDto): boolean {
  const id = mod.id.toLowerCase();
  const name = mod.name.toLowerCase();
  return (
    id === "ink-wars" ||
    name.includes("ink wars") ||
    id.includes("core-runtime") ||
    name.includes("core runtime") ||
    id.includes("developer-tools") ||
    id.includes("developers-tools") ||
    id.includes("dev-tools") ||
    name.includes("dev tools") ||
    name.includes("developer tools") ||
    name.includes("developers tools")
  );
}

export function ModsPanel({ onNewProject, onRescan }: { onNewProject: () => void; onRescan: () => void }) {
  const { state, openSelectedMod, revealSelectedModFolder, selectMod, validateSelectedMod } = useEditorStore();
  const showDebugSources = useDebugSourceEnabled();
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<"all" | "valid" | "problems">("all");

  const mods = useMemo(() => {
    return state.mods.filter((mod) => {
      if (isHiddenStartupProject(mod)) return false;
      const normalizedQuery = query.toLowerCase();
      const matchesQuery = mod.name.toLowerCase().includes(normalizedQuery) || mod.id.toLowerCase().includes(normalizedQuery);
      const matchesFilter =
        filter === "all" || (filter === "valid" && mod.status === "valid") || (filter === "problems" && mod.status !== "valid");
      return matchesQuery && matchesFilter;
    });
  }, [filter, query, state.mods]);
  const activeModId = selectedModId(state.selection);

  return (
    <DebugSourceOverlay enabled={showDebugSources} source="src/startup/ModsPanel.tsx" contentClassName="debug-source-fill">
      <aside className="panel mods-panel">
      <div className="panel-title-row">
        <h2>Available Mods</h2>
        <div className="mods-title-actions">
          <span className="count-badge">{mods.length}</span>
          <button type="button" className="mods-rescan-button" title="Rescan mods" aria-label="Rescan mods" onClick={onRescan}>
            <RefreshCcw size={13} />
          </button>
        </div>
      </div>

      <label className="search-box">
        <Search size={15} />
        <input value={query} placeholder="Search mods..." onChange={(event) => setQuery(event.target.value)} />
      </label>

      <div className="segmented-control">
        <button className={filter === "all" ? "active" : ""} onClick={() => setFilter("all")} type="button">All</button>
        <button className={filter === "valid" ? "active" : ""} onClick={() => setFilter("valid")} type="button">Valid</button>
        <button className={filter === "problems" ? "active" : ""} onClick={() => setFilter("problems")} type="button">Problems</button>
      </div>

      <div className="mods-actions-row">
        <button className="button button-primary" type="button" onClick={onNewProject}>
          <Plus size={14} />
          New
        </button>
        <button className="button button-tool" type="button" disabled={!activeModId} onClick={() => void validateSelectedMod()}>
          <ShieldCheck size={14} />
          Validate
        </button>
        <button className="button button-tool" type="button" disabled={!activeModId} onClick={() => void revealSelectedModFolder()}>
          <Folder size={14} />
          Reveal
        </button>
      </div>

      <div className="mods-list">
        {mods.map((mod) => (
          <ModRow
            key={mod.id}
            mod={mod}
            selected={selectedModId(state.selection) === mod.id}
            onSelect={() => void selectMod(mod.id)}
            onOpen={() => {
              void selectMod(mod.id).then(() => openSelectedMod(mod.id));
            }}
          />
        ))}
      </div>
      </aside>
    </DebugSourceOverlay>
  );
}

function ModRow({
  mod,
  selected,
  onSelect,
  onOpen,
}: {
  mod: EditorModSummaryDto;
  selected: boolean;
  onSelect: () => void;
  onOpen: () => void;
}) {
  return (
    <div className={`mod-row ${selected ? "selected" : ""}`} title={mod.rootPath}>
      <button type="button" className="mod-row-main-button interactive" onClick={onSelect}>
        <Folder size={18} />
        <span className="mod-row-main">
          <strong>{mod.name}</strong>
          <small>
            {mod.version} · {mod.visibleSceneCount}/{mod.sceneCount} scenes · {mod.contentSummary.totalFiles} files
          </small>
        </span>
      </button>
      <div className="mod-row-actions">
        <button type="button" className="mod-row-open-button" title="Open mod" aria-label="Open mod" onClick={onOpen}>
          <FolderOpen size={14} />
        </button>
        <span className={`mod-status-dot ${statusClass(mod.status)}`} title={mod.status}>
          <Circle size={9} fill="currentColor" />
        </span>
      </div>
    </div>
  );
}
