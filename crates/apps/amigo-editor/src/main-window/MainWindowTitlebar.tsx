import { ArrowLeft, PanelTopOpen, Settings } from "lucide-react";
import type { EditorModDetailsDto } from "../api/dto";
import { DebugSourceToggleButton } from "../debug/debugSource";
import type { EditorComponentDefinition } from "../editor-components/componentTypes";
import { ThemeButton } from "../theme/ThemeButton";
import { ComponentMenu } from "./ComponentMenu";

export function MainWindowTitlebar({
  componentMenuOpen,
  details,
  onCloseWorkspace,
  onOpenComponent,
  onOpenModSettings,
  onOpenTheme,
  onAttachWorkspace,
  onToggleComponentMenu,
  onToggleDebugSources,
  session,
  showDebugSources,
}: {
  componentMenuOpen: boolean;
  details: EditorModDetailsDto | null;
  onCloseWorkspace: () => Promise<void>;
  onOpenComponent: (component: EditorComponentDefinition<any>) => void;
  onOpenModSettings: () => void;
  onOpenTheme: () => void;
  onAttachWorkspace?: () => void;
  onToggleComponentMenu: () => void;
  onToggleDebugSources: () => void;
  session: { sessionId: string; modId: string; rootPath: string } | null;
  showDebugSources: boolean;
}) {
  return (
    <header className="main-titlebar window-titlebar">
      <div className="main-brand window-brand">
        <div className="brand-mark">A</div>
        <strong>Amigo Editor</strong>
        <span>{session ? `workspace session ${session.sessionId}` : "workspace"}</span>
      </div>
      <nav className="main-menu" aria-label="Application menu">
        <button type="button">File</button>
        <button type="button">Edit</button>
        <button type="button">View</button>
        <span className="main-menu-popover-anchor">
          <button type="button" onClick={onToggleComponentMenu}>Window</button>
          {componentMenuOpen ? (
            <ComponentMenu
              onOpen={(component) => {
                onOpenComponent(component);
              }}
            />
          ) : null}
        </span>
        <button type="button" onClick={onToggleComponentMenu}>Tools</button>
      </nav>
      <div className="titlebar-project-context">
        <span className="titlebar-project-summary">
          <strong>{details?.name ?? session?.modId ?? "No mod"}</strong>
          <small>{details ? `${details.id} · ${details.version}` : session?.rootPath ?? "No active session"}</small>
          <span className={`titlebar-status-dot status-${details?.status ?? "warning"}`} aria-label={details?.status ?? "session"} />
        </span>
        <span className="titlebar-separator" aria-hidden="true" />
        <ThemeButton onClick={onOpenTheme} />
        <DebugSourceToggleButton showDebugSources={showDebugSources} onToggle={onToggleDebugSources} />
        {onAttachWorkspace ? (
          <button className="button button-ghost" type="button" onClick={onAttachWorkspace}>
            <PanelTopOpen size={15} />
            Attach
          </button>
        ) : null}
        <button className="button button-ghost" type="button" onClick={onOpenModSettings}>
          <Settings size={15} />
          Settings
        </button>
        <span className="titlebar-separator" aria-hidden="true" />
        <button className="titlebar-action-button" type="button" onClick={() => void onCloseWorkspace()}>
          <ArrowLeft size={15} />
          Close Workspace
        </button>
      </div>
    </header>
  );
}
