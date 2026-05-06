param([Parameter(Mandatory=$true)][string]$RepoRoot)
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "_common.ps1")

$file = Get-AmigoFile $RepoRoot "crates/apps/amigo-editor/src/features/scenes/editor/SceneEditorCanvas.tsx"
$text = Read-AmigoText $file

if ($text -match "SceneEditorCursorOverlay") {
  Write-Host "SKIP SceneEditorCanvas.tsx already uses SceneEditorCursorOverlay"
  return
}

$text = $text.Replace(
'import { SceneEditorArtboard } from "./SceneEditorArtboard";',
'import { SceneEditorArtboard } from "./SceneEditorArtboard";
import { SceneEditorCursorOverlay } from "./SceneEditorCursorOverlay";'
)

$text = $text.Replace(
'    mouseScenePoint,
  } = useSceneEditorPointerEvents({',
'    mouseFramePoint,
    mouseScenePoint,
    pointerActive,
  } = useSceneEditorPointerEvents({'
)

$text = $text.Replace(
'      <SceneEditorArtboard frame={frame} resolution={model.resolution} viewport={viewport} />',
'      <SceneEditorArtboard frame={frame} resolution={model.resolution} viewport={viewport} />
      <SceneEditorCursorOverlay
        framePoint={mouseFramePoint}
        pointerActive={pointerActive}
        tool={tool}
      />'
)

Write-AmigoText $file $text
Write-Host "OK updated SceneEditorCanvas.tsx"
