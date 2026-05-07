import { Grid2X2, List, ListTree, Search } from "lucide-react";
import type { ReactNode } from "react";

export type ExplorerViewMode = "tree" | "list" | "tiles";

const VIEW_MODE_LABELS: Record<ExplorerViewMode, string> = {
  tree: "Tree",
  list: "List",
  tiles: "Tiles",
};

const VIEW_MODE_ICONS: Record<ExplorerViewMode, ReactNode> = {
  tree: <ListTree size={13} />,
  list: <List size={13} />,
  tiles: <Grid2X2 size={13} />,
};

// @codemap anchor:explorer-shell domain:workspace role:model priority:P1 layer:app tags:explorer,tree,list,tiles
export function ExplorerShell({
  allowedViewModes,
  children,
  className,
  loading,
  loadingLabel = "Loading...",
  onSearchChange,
  onViewModeChange,
  search,
  searchPlaceholder = "Search...",
  subtitle,
  title,
  viewMode,
}: {
  allowedViewModes: ExplorerViewMode[];
  children: ReactNode;
  className?: string;
  loading?: boolean;
  loadingLabel?: string;
  onSearchChange?: (value: string) => void;
  onViewModeChange?: (mode: ExplorerViewMode) => void;
  search?: string;
  searchPlaceholder?: string;
  subtitle?: ReactNode;
  title: ReactNode;
  viewMode: ExplorerViewMode;
}) {
  const canSwitchView = allowedViewModes.length > 1;

  return (
    <div className={["explorer-shell", className].filter(Boolean).join(" ")}>
      <header className="explorer-shell-header">
        <div>
          <strong>{title}</strong>
          {subtitle ? <span>{subtitle}</span> : null}
        </div>

        {canSwitchView ? (
          <div className="explorer-view-toggle" role="group" aria-label="View mode">
            {allowedViewModes.map((mode) => (
              <button
                key={mode}
                type="button"
                className={mode === viewMode ? "active" : ""}
                onClick={() => onViewModeChange?.(mode)}
              >
                {VIEW_MODE_ICONS[mode]}
                <span>{VIEW_MODE_LABELS[mode]}</span>
              </button>
            ))}
          </div>
        ) : null}
      </header>

      {onSearchChange ? (
        <label className="project-tree-searchbar explorer-shell-search">
          <Search size={13} />
          <input
            value={search ?? ""}
            placeholder={searchPlaceholder}
            onChange={(event) => onSearchChange(event.target.value)}
          />
        </label>
      ) : null}

      <div className="project-tree-separator explorer-shell-separator" aria-hidden="true" />
      {loading ? <p className="muted workspace-note">{loadingLabel}</p> : null}

      <div className={`explorer-shell-body explorer-shell-body-${viewMode}`}>{children}</div>
    </div>
  );
}
