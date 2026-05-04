import type { FolderViewDensity, FolderViewGroup, FolderViewThumbnailMode } from "./folderViewTypes";
import "./folder-view.css";

export function FolderView({
  density = "compact",
  emptyMessage = "No items.",
  groups,
  showGroupHeaders = true,
  thumbnailMode = "contain",
}: {
  density?: FolderViewDensity;
  emptyMessage?: string;
  groups: FolderViewGroup[];
  showGroupHeaders?: boolean;
  thumbnailMode?: FolderViewThumbnailMode;
}) {
  const visibleGroups = groups.filter((group) => group.items.length > 0);
  if (visibleGroups.length === 0) {
    return <p className="muted folder-view-empty">{emptyMessage}</p>;
  }

  return (
    <div className={`folder-view density-${density} thumbnail-${thumbnailMode}`}>
      {visibleGroups.map((group) => (
        <section key={group.id} className="folder-view-group">
          {showGroupHeaders ? (
            <header className="folder-view-group-header">
              <span className="folder-view-group-icon">{group.icon}</span>
              <span>
                <strong>{group.title}</strong>
                {group.subtitle ? <small>{group.subtitle}</small> : null}
              </span>
              <em>{group.items.length}</em>
            </header>
          ) : null}
          <div className="folder-view-grid">
            {group.items.map((item) => (
              <button
                key={item.id}
                type="button"
                className={`folder-view-item status-${item.status ?? "muted"} ${item.tone ?? ""} ${item.selected ? "selected" : ""}`}
                title={item.subtitle ? `${item.title}\n${item.subtitle}` : item.title}
                onClick={item.onOpen}
              >
                <span className="folder-view-thumbnail">
                  {item.thumbnailSrc ? <img alt="" draggable={false} src={item.thumbnailSrc} /> : <span>{item.icon}</span>}
                </span>
                <strong>{item.title}</strong>
                {item.subtitle ? <small>{item.subtitle}</small> : null}
                {item.actions?.length ? (
                  <span className="folder-view-actions">
                    {item.actions.map((action) => (
                      <span
                        key={action.id}
                        role="button"
                        tabIndex={0}
                        title={action.label}
                        onClick={(event) => {
                          event.stopPropagation();
                          action.onRun();
                        }}
                        onKeyDown={(event) => {
                          if (event.key !== "Enter" && event.key !== " ") return;
                          event.preventDefault();
                          event.stopPropagation();
                          action.onRun();
                        }}
                      >
                        {action.label}
                      </span>
                    ))}
                  </span>
                ) : null}
              </button>
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}

export type { FolderViewGroup, FolderViewItem } from "./folderViewTypes";
