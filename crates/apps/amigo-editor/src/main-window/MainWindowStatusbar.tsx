import type { EditorModDetailsDto } from "../api/dto";

export function MainWindowStatusbar({
  activeThemeName,
  details,
  editorModeError,
  editorModeOpening,
  runningTaskCount,
}: {
  activeThemeName: string;
  details: EditorModDetailsDto | null;
  editorModeError: string | null;
  editorModeOpening: boolean;
  runningTaskCount: number;
}) {
  return (
    <footer className="workspace-statusbar window-statusbar">
      <span><span className="status-dot" />Ready</span>
      <span>{details?.name ?? "No mod"}</span>
      <span>{details?.sceneCount ?? 0} scenes</span>
      <span>{details?.contentSummary.totalFiles ?? 0} files</span>
      <span>Theme: {activeThemeName}</span>
      <span>{runningTaskCount} tasks running</span>
      {editorModeOpening ? <span>Editor mode opening...</span> : null}
      {editorModeError ? <span>Editor mode error: {editorModeError}</span> : null}
    </footer>
  );
}
