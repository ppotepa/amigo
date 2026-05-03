import { Folder, Search } from "lucide-react";
import { useMemo, useState } from "react";
import type { EditorModSummaryDto } from "../api/dto";
import { useEditorStore } from "../app/editorStore";

function statusClass(status: string): string {
  return `badge-${status === "valid" ? "valid" : status === "warning" || status === "missingDependency" || status === "missingSceneFile" ? "warning" : "error"}`;
}

export function ModsPanel() {
  const { state, selectMod } = useEditorStore();
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<"all" | "valid" | "problems">("all");

  const mods = useMemo(() => {
    return state.mods.filter((mod) => {
      const normalizedQuery = query.toLowerCase();
      const matchesQuery = mod.name.toLowerCase().includes(normalizedQuery) || mod.id.toLowerCase().includes(normalizedQuery);
      const matchesFilter = filter === "all" || (filter === "valid" && mod.status === "valid") || (filter === "problems" && mod.status !== "valid");
      return matchesQuery && matchesFilter;
    });
  }, [filter, query, state.mods]);

  return (
    <aside className="panel mods-panel">
      <div className="panel-title-row">
        <h2>Available Mods</h2>
        <span className="count-badge">{mods.length}</span>
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

      <div className="mods-list">
        {mods.map((mod) => (
          <ModRow key={mod.id} mod={mod} selected={state.selectedModId === mod.id} onSelect={() => void selectMod(mod.id)} />
        ))}
      </div>
    </aside>
  );
}

function ModRow({ mod, selected, onSelect }: { mod: EditorModSummaryDto; selected: boolean; onSelect: () => void }) {
  return (
    <button type="button" className={`mod-row interactive ${selected ? "selected" : ""}`} onClick={onSelect} title={mod.rootPath}>
      <Folder size={18} />
      <span className="mod-row-main">
        <strong>{mod.name}</strong>
        <small>
          {mod.version} · {mod.visibleSceneCount}/{mod.sceneCount} scenes · {mod.contentSummary.totalFiles} files
        </small>
      </span>
      <span className={`badge ${statusClass(mod.status)}`}>{mod.status}</span>
    </button>
  );
}
