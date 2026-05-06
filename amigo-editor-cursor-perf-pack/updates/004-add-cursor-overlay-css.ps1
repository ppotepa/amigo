param([Parameter(Mandatory=$true)][string]$RepoRoot)
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "_common.ps1")

$file = Get-AmigoFile $RepoRoot "crates/apps/amigo-editor/src/features/scenes/editor/scene-editor.css"
$text = Read-AmigoText $file

if ($text -match "scene-editor-cursor-overlay") {
  Write-Host "SKIP scene-editor.css already contains cursor overlay styles"
  return
}

$append = @'

/* Editor viewport cursor.
   The native cursor is hidden over the viewport and replaced by a lightweight Lucide cursor overlay.
   Gizmo geometry still belongs to backend editor mode; this overlay is only for pointer responsiveness. */
.scene-editor-canvas {
  cursor: none;
}

.scene-editor-canvas [data-editor-chrome='true'],
.scene-editor-canvas [data-editor-chrome='true'] *,
.scene-editor-floating-dock,
.scene-editor-floating-dock *,
.scene-editor-toolbar,
.scene-editor-toolbar * {
  cursor: auto;
}

.scene-editor-cursor-overlay {
  position: absolute;
  left: 0;
  top: 0;
  z-index: 35;
  width: 28px;
  height: 28px;
  pointer-events: none;
  color: #f8fafc;
  will-change: transform;
  transform-origin: 0 0;
}

.scene-editor-cursor-icon,
.scene-editor-cursor-shadow {
  position: absolute;
  left: 3px;
  top: 3px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.scene-editor-cursor-shadow {
  color: #020617;
  transform: translate(1.5px, 1.5px);
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.9));
}

.scene-editor-cursor-icon {
  color: #f8fafc;
  filter:
    drop-shadow(0 0 1px rgba(2, 6, 23, 1))
    drop-shadow(0 0 4px rgba(14, 165, 233, 0.35));
}

.scene-editor-cursor-overlay-move .scene-editor-cursor-icon,
.scene-editor-cursor-overlay-scale .scene-editor-cursor-icon,
.scene-editor-cursor-overlay-rotate .scene-editor-cursor-icon,
.scene-editor-cursor-overlay-rect .scene-editor-cursor-icon {
  color: #facc15;
}

.scene-editor-cursor-overlay.is-active .scene-editor-cursor-icon {
  color: #fbbf24;
  filter:
    drop-shadow(0 0 1px rgba(2, 6, 23, 1))
    drop-shadow(0 0 6px rgba(250, 204, 21, 0.65));
}

.scene-editor-cursor-axis-hint {
  position: absolute;
  left: 21px;
  top: 18px;
  display: grid;
  grid-template-columns: 13px 13px;
  gap: 1px;
  color: #facc15;
  opacity: 0.85;
  filter: drop-shadow(0 1px 1px rgba(0, 0, 0, 0.8));
}

.scene-editor-cursor-blocked {
  position: absolute;
  left: 20px;
  top: 18px;
  color: #ef4444;
  filter: drop-shadow(0 1px 1px rgba(0, 0, 0, 0.8));
}

'@

$text = $text.TrimEnd() + $append
Write-AmigoText $file $text
Write-Host "OK updated scene-editor.css"
