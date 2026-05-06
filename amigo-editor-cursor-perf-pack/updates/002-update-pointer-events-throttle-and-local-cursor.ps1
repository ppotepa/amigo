param([Parameter(Mandatory=$true)][string]$RepoRoot)
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "_common.ps1")

$file = Get-AmigoFile $RepoRoot "crates/apps/amigo-editor/src/features/scenes/editor/useSceneEditorPointerEvents.ts"
$text = Read-AmigoText $file

if ($text -match "schedulePointerMove") {
  Write-Host "SKIP useSceneEditorPointerEvents.ts already contains schedulePointerMove"
  return
}

$text = $text.Replace(
'import { useState } from "react";',
'import { useEffect, useRef, useState } from "react";'
)

$text = $text.Replace(
'  const [mouseScenePoint, setMouseScenePoint] = useState<SceneEditorPoint | null>(null);',
'  const [mouseScenePoint, setMouseScenePoint] = useState<SceneEditorPoint | null>(null);
  const [mouseFramePoint, setMouseFramePoint] = useState<SceneEditorPoint | null>(null);
  const [pointerActive, setPointerActive] = useState(false);
  const pendingMoveRef = useRef<EditorPointerEventDto | null>(null);
  const moveFrameRef = useRef<number | null>(null);
  const moveInFlightRef = useRef(false);'
)

$text = $text.Replace(
'  function updateMousePoint(event: React.PointerEvent<HTMLDivElement>) {
    setMouseScenePoint(frameToScene(screenToArtboard(localPoint(event), viewport), model));
  }',
'  function updateMousePoint(event: React.PointerEvent<HTMLDivElement>) {
    const framePoint = screenToArtboard(localPoint(event), viewport);
    setMouseFramePoint(framePoint);
    setMouseScenePoint(frameToScene(framePoint, model));
  }

  function flushPendingMove() {
    moveFrameRef.current = null;
    const next = pendingMoveRef.current;
    pendingMoveRef.current = null;
    if (!next || moveInFlightRef.current) {
      if (next) schedulePointerMove(next);
      return;
    }

    moveInFlightRef.current = true;
    void onPointerEvent?.(next).finally(() => {
      moveInFlightRef.current = false;
      if (pendingMoveRef.current) {
        schedulePointerMove(pendingMoveRef.current);
      }
    });
  }

  function schedulePointerMove(event: EditorPointerEventDto) {
    pendingMoveRef.current = event;
    if (moveFrameRef.current !== null) return;
    moveFrameRef.current = window.requestAnimationFrame(flushPendingMove);
  }

  useEffect(() => {
    return () => {
      if (moveFrameRef.current !== null) {
        window.cancelAnimationFrame(moveFrameRef.current);
      }
    };
  }, []);'
)

$text = $text.Replace(
'  async function handlePointerDown(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    updateMousePoint(event);
    event.currentTarget.setPointerCapture(event.pointerId);
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerDown"));
  }',
'  async function handlePointerDown(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    updateMousePoint(event);
    setPointerActive(true);
    event.currentTarget.setPointerCapture(event.pointerId);
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerDown"));
  }'
)

$text = $text.Replace(
'  async function handlePointerMove(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    updateMousePoint(event);
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerMove"));
  }',
'  async function handlePointerMove(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    updateMousePoint(event);

    // Hover-only pointer movement stays local for snappy viewport cursor movement.
    // Drag movement is coalesced to requestAnimationFrame and no longer blocks the UI.
    if (event.buttons === 0) return;

    schedulePointerMove(toEditorPointerEvent(event, "pointerMove"));
  }'
)

$text = $text.Replace(
'  async function handlePointerUp(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    updateMousePoint(event);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerUp"));
  }',
'  async function handlePointerUp(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    updateMousePoint(event);
    setPointerActive(false);
    pendingMoveRef.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerUp"));
  }'
)

$text = $text.Replace(
'  async function handlePointerCancel(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    updateMousePoint(event);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerCancel"));
  }',
'  async function handlePointerCancel(event: React.PointerEvent<HTMLDivElement>) {
    if (isEditorChromeEvent(event)) return;
    updateMousePoint(event);
    setPointerActive(false);
    pendingMoveRef.current = null;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    await onPointerEvent?.(toEditorPointerEvent(event, "pointerCancel"));
  }'
)

$text = $text.Replace(
'    mouseScenePoint,
  };',
'    mouseFramePoint,
    mouseScenePoint,
    pointerActive,
  };'
)

Write-AmigoText $file $text
Write-Host "OK updated useSceneEditorPointerEvents.ts"
