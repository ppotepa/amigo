import { useEffect, useMemo, useRef, useState } from "react";
import type React from "react";
import { AlertTriangle, Brush, Eraser, Grid2X2, Map, PaintBucket, Pipette } from "lucide-react";
import { loadSheetResource, loadTilemapResource, saveTilemapResource } from "../../api/editorApi";
import type { SheetResourceDto, TilemapResourceDto } from "../../api/dto";
import { fileSrc } from "../../utils/fileSrc";
import "./tilemap-editor.css";

type TilemapTool = "paint" | "erase" | "fill" | "picker";

export function TilemapEditor({
  onDirtyChange,
  resourceUri,
  sessionId,
}: {
  onDirtyChange?: (path: string, dirty: boolean) => void;
  resourceUri: string;
  sessionId: string;
}) {
  const [tilemap, setTilemap] = useState<TilemapResourceDto | null>(null);
  const [tileset, setTileset] = useState<SheetResourceDto | null>(null);
  const [selectedTileId, setSelectedTileId] = useState(0);
  const [paintedCells, setPaintedCells] = useState<Record<string, number>>({});
  const [tool, setTool] = useState<TilemapTool>("paint");
  const [zoom, setZoom] = useState(1);
  const [dirty, setDirty] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [paintDragActive, setPaintDragActive] = useState(false);
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const dragRef = useRef<{ active: boolean; left: number; top: number; x: number; y: number }>({
    active: false,
    left: 0,
    top: 0,
    x: 0,
    y: 0,
  });

  useEffect(() => {
    let cancelled = false;
    setError(null);
    setTilemap(null);
    setTileset(null);
    setPaintedCells({});
    setDirty(false);
    onDirtyChange?.(resourceUri, false);

    void loadTilemapResource(sessionId, resourceUri)
      .then(async (loadedTilemap) => {
        if (cancelled) return;
        setTilemap(loadedTilemap);
        setPaintedCells(Object.fromEntries(loadedTilemap.cells.map((cell) => [`${cell.x}:${cell.y}`, cell.tileId])));
        if (loadedTilemap.tilesetResourceUri) {
          setTileset(await loadSheetResource(sessionId, loadedTilemap.tilesetResourceUri));
        }
      })
      .catch((loadError: unknown) => {
        if (!cancelled) setError(loadError instanceof Error ? loadError.message : String(loadError));
      });

    return () => {
      cancelled = true;
    };
  }, [onDirtyChange, resourceUri, sessionId]);

  const cells = useMemo(() => {
    if (!tilemap) return [];
    return Array.from({ length: tilemap.width * tilemap.height }, (_, index) => ({
      x: index % tilemap.width,
      y: Math.floor(index / tilemap.width),
    }));
  }, [tilemap]);

  function markDirty() {
    setDirty(true);
    onDirtyChange?.(resourceUri, true);
  }

  function applyTool(x: number, y: number) {
    if (!tilemap) return;
    const cellKey = `${x}:${y}`;
    if (tool === "picker") {
      const picked = paintedCells[cellKey];
      if (picked != null) setSelectedTileId(picked);
      return;
    }
    if (tool === "fill") {
      const target = paintedCells[cellKey];
      const next = { ...paintedCells };
      const pending = [[x, y]];
      const visited = new Set<string>();
      while (pending.length) {
        const [currentX, currentY] = pending.pop()!;
        const key = `${currentX}:${currentY}`;
        if (visited.has(key)) continue;
        visited.add(key);
        const current = next[key];
        if (current !== target) continue;
        next[key] = selectedTileId;
        for (const [offsetX, offsetY] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
          const neighborX = currentX + offsetX;
          const neighborY = currentY + offsetY;
          if (neighborX < 0 || neighborY < 0 || neighborX >= tilemap.width || neighborY >= tilemap.height) continue;
          pending.push([neighborX, neighborY]);
        }
      }
      setPaintedCells(next);
      markDirty();
      return;
    }
    if (tool === "erase") {
      setPaintedCells((current) => {
        const next = { ...current };
        delete next[cellKey];
        return next;
      });
      markDirty();
      return;
    }
    setPaintedCells((current) => ({ ...current, [cellKey]: selectedTileId }));
    markDirty();
  }

  function applyContinuousTool(x: number, y: number) {
    if (tool === "fill" || tool === "picker") return;
    applyTool(x, y);
  }

  async function saveCurrentTilemap() {
    if (!tilemap) return;
    setSaving(true);
    setError(null);
    try {
      const cells = Object.entries(paintedCells)
        .map(([key, tileId]) => {
          const [x, y] = key.split(":").map(Number);
          return { x, y, tileId };
        })
        .sort((a, b) => a.y - b.y || a.x - b.x || a.tileId - b.tileId);
      const saved = await saveTilemapResource(sessionId, resourceUri, { ...tilemap, cells });
      setTilemap(saved);
      setPaintedCells(Object.fromEntries(saved.cells.map((cell) => [`${cell.x}:${cell.y}`, cell.tileId])));
      setDirty(false);
      onDirtyChange?.(resourceUri, false);
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : String(saveError));
    } finally {
      setSaving(false);
    }
  }

  if (error) {
    return (
      <div className="tilemap-editor tilemap-editor-state">
        <AlertTriangle size={34} />
        <strong>Failed to load tilemap</strong>
        <span>{error}</span>
      </div>
    );
  }

  if (!tilemap) {
    return (
      <div className="tilemap-editor tilemap-editor-state">
        <Map size={34} />
        <strong>Loading tilemap...</strong>
        <span>{resourceUri}</span>
      </div>
    );
  }

  return (
    <div className="tilemap-editor">
      <header className="tilemap-editor-toolbar">
        <div className="tilemap-editor-title">
          <span className="dock-icon dock-icon-cyan"><Map size={14} /></span>
          <strong>{tilemap.label}</strong>
          <span>{tilemap.relativePath}</span>
          {dirty ? <span className="badge badge-warning">modified</span> : null}
          <span className="badge badge-info">{tilemap.width} x {tilemap.height}</span>
          {tilemap.tilesetResourceUri ? <span className="badge badge-valid">palette</span> : <span className="badge badge-warning">no palette</span>}
        </div>
        <div className="tilemap-editor-actions">
          <button className="button button-icon" type="button" title="Paint" aria-pressed={tool === "paint"} onClick={() => setTool("paint")}>
            <Brush size={14} />
          </button>
          <button className="button button-icon" type="button" title="Erase" aria-pressed={tool === "erase"} onClick={() => setTool("erase")}>
            <Eraser size={14} />
          </button>
          <button className="button button-icon" type="button" title="Fill map" aria-pressed={tool === "fill"} onClick={() => setTool("fill")}>
            <PaintBucket size={14} />
          </button>
          <button className="button button-icon" type="button" title="Pick tile" aria-pressed={tool === "picker"} onClick={() => setTool("picker")}>
            <Pipette size={14} />
          </button>
          <span className="toolbar-separator" aria-hidden="true" />
          <button className="button button-icon" type="button" title="Zoom out" onClick={() => setZoom((value) => Math.max(0.5, value - 0.25))}>
            -
          </button>
          <span className="tilemap-zoom-label">{Math.round(zoom * 100)}%</span>
          <button className="button button-icon" type="button" title="Fit zoom" onClick={() => setZoom(1)}>
            Fit
          </button>
          <button className="button button-icon" type="button" title="Zoom in" onClick={() => setZoom((value) => Math.min(4, value + 0.25))}>
            +
          </button>
          <button className="button button-tool" type="button" disabled={!dirty || saving} onClick={() => void saveCurrentTilemap()}>
            {saving ? "Saving" : "Save"}
          </button>
        </div>
      </header>

      <main className="tilemap-editor-body">
        <section
          ref={viewportRef}
          className="tilemap-canvas-stage"
          onWheel={(event) => {
            if (!event.ctrlKey) return;
            event.preventDefault();
            setZoom((value) => Math.min(4, Math.max(0.5, value + (event.deltaY > 0 ? -0.1 : 0.1))));
          }}
          onMouseDown={(event) => {
            if (event.button !== 1 && event.button !== 2) return;
            event.preventDefault();
            dragRef.current = {
              active: true,
              left: viewportRef.current?.scrollLeft ?? 0,
              top: viewportRef.current?.scrollTop ?? 0,
              x: event.clientX,
              y: event.clientY,
            };
          }}
          onMouseMove={(event) => {
            if (!dragRef.current.active || !viewportRef.current) return;
            viewportRef.current.scrollLeft = dragRef.current.left - (event.clientX - dragRef.current.x);
            viewportRef.current.scrollTop = dragRef.current.top - (event.clientY - dragRef.current.y);
          }}
          onMouseUp={() => {
            dragRef.current.active = false;
          }}
          onMouseLeave={() => {
            dragRef.current.active = false;
          }}
          onContextMenu={(event) => event.preventDefault()}
        >
          <div
            className="tilemap-grid"
            onMouseLeave={() => {
              setPaintDragActive(false);
            }}
            style={{
              "--tilemap-columns": String(tilemap.width),
              "--tilemap-rows": String(tilemap.height),
              "--tilemap-cell-size": `${Math.round(30 * zoom)}px`,
            } as React.CSSProperties}
          >
            {cells.map((cell) => {
              const tileId = paintedCells[`${cell.x}:${cell.y}`];
              return (
                <button
                  key={`${cell.x}:${cell.y}`}
                  className={tileId != null ? "painted" : ""}
                  type="button"
                  title={`${cell.x}, ${cell.y}${tileId != null ? ` · tile ${tileId}` : ""}`}
                  onMouseDown={(event) => {
                    if (event.button !== 0) return;
                    setPaintDragActive(true);
                    applyTool(cell.x, cell.y);
                  }}
                  onMouseEnter={() => {
                    if (!paintDragActive) return;
                    applyContinuousTool(cell.x, cell.y);
                  }}
                  onMouseUp={() => {
                    setPaintDragActive(false);
                  }}
                >
                  {tileId != null && tileset?.imageExists ? (
                    <span className="tilemap-tile-sprite" style={tileSpriteStyle(tileset, tileId)} />
                  ) : null}
                </button>
              );
            })}
          </div>
        </section>

        <aside className="tilemap-palette">
          <section>
            <h3>Tileset Palette</h3>
            <p className="muted">{tilemap.tilesetResourceUri ?? "No tileset declared."}</p>
            {tileset ? (
              <div className="tilemap-palette-grid">
                {Array.from({ length: tileset.count }, (_, tileId) => (
                  <button
                    key={tileId}
                    className={selectedTileId === tileId ? "selected" : ""}
                    type="button"
                    title={`Tile ${tileId}`}
                    onClick={() => setSelectedTileId(tileId)}
                  >
                    {tileset.imageExists ? (
                      <span className="tilemap-tile-sprite" style={tileSpriteStyle(tileset, tileId)} />
                    ) : (
                      tileId
                    )}
                  </button>
                ))}
              </div>
            ) : (
              <div className="tilemap-editor-state compact">
                <Grid2X2 size={24} />
                <span>Palette not loaded.</span>
              </div>
            )}
          </section>

          <section>
            <h3>Diagnostics</h3>
            {tilemap.diagnostics.length ? tilemap.diagnostics.map((diagnostic, index) => (
              <div key={`${diagnostic.code}:${index}`} className={`tilemap-diagnostic diagnostic-${diagnostic.level}`}>
                <strong>{diagnostic.code}</strong>
                <span>{diagnostic.message}</span>
              </div>
            )) : <p className="muted">No tilemap diagnostics.</p>}
          </section>
        </aside>
      </main>
    </div>
  );
}

function tileSpriteStyle(tileset: SheetResourceDto, tileId: number): React.CSSProperties {
  const columns = Math.max(1, tileset.columns);
  const rows = Math.max(1, tileset.rows);
  const column = tileId % columns;
  const row = Math.floor(tileId / columns);
  const imageWidth = Math.max(1, tileset.imageWidth ?? tileset.declaredImageWidth ?? columns * tileset.cellWidth);
  const imageHeight = Math.max(1, tileset.imageHeight ?? tileset.declaredImageHeight ?? rows * tileset.cellHeight);
  const tileLeft = tileset.marginX + column * (tileset.cellWidth + tileset.spacingX);
  const tileTop = tileset.marginY + row * (tileset.cellHeight + tileset.spacingY);
  const x = imageWidth <= tileset.cellWidth ? 0 : (tileLeft / (imageWidth - tileset.cellWidth)) * 100;
  const y = imageHeight <= tileset.cellHeight ? 0 : (tileTop / (imageHeight - tileset.cellHeight)) * 100;
  return {
    backgroundImage: `url("${fileSrc(tileset.imageAbsolutePath)}")`,
    backgroundPosition: `${x}% ${y}%`,
    backgroundSize: `${(imageWidth / Math.max(1, tileset.cellWidth)) * 100}% ${(imageHeight / Math.max(1, tileset.cellHeight)) * 100}%`,
  };
}
