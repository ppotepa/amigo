import { useEffect, useMemo, useRef, useState } from "react";
import type React from "react";
import { AlertTriangle, Grid2X2, Image as ImageIcon, Save } from "lucide-react";
import { loadSheetResource, saveSheetResource } from "../../api/editorApi";
import type { SheetResourceDto } from "../../api/dto";
import { fileSrc } from "../../utils/fileSrc";
import "./sheet-editor.css";

export function SheetEditor({
  onDirtyChange,
  onReveal,
  onSaved,
  resourceUri,
  sessionId,
}: {
  onDirtyChange?: (path: string, dirty: boolean) => void;
  onReveal?: () => void;
  onSaved?: () => void;
  resourceUri: string;
  sessionId: string;
}) {
  const [sheet, setSheet] = useState<SheetResourceDto | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [selectedTileId, setSelectedTileId] = useState<number | null>(null);
  const [zoom, setZoom] = useState(1);
  const [selectedAnimationId, setSelectedAnimationId] = useState<string | null>(null);
  const localDiagnostics = useMemo(() => (sheet ? validateSheetDraft(sheet) : []), [sheet]);

  useEffect(() => {
    let cancelled = false;
    setError(null);
    setSheet(null);
    void loadSheetResource(sessionId, resourceUri)
      .then((loaded) => {
        if (!cancelled) {
          setSheet(loaded);
          setDirty(false);
          onDirtyChange?.(resourceUri, false);
          setSelectedTileId(loaded.tileset?.tiles[0]?.id ?? 0);
          setSelectedAnimationId(loaded.animations?.[0]?.id ?? null);
        }
      })
      .catch((loadError: unknown) => {
        if (!cancelled) setError(loadError instanceof Error ? loadError.message : String(loadError));
      });
    return () => {
      cancelled = true;
    };
  }, [resourceUri, sessionId]);

  async function saveCurrentSheet() {
    if (!sheet) return;
    setSaving(true);
    setError(null);
    try {
      const saved = await saveSheetResource(sessionId, resourceUri, sheet);
      setSheet(saved);
      setDirty(false);
      onDirtyChange?.(resourceUri, false);
      if (saved.relativePath !== resourceUri) {
        onDirtyChange?.(saved.relativePath, false);
      }
      onSaved?.();
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : String(saveError));
    } finally {
      setSaving(false);
    }
  }

  function updateSheetNumber(field: keyof Pick<SheetResourceDto, "cellWidth" | "cellHeight" | "columns" | "rows" | "count" | "marginX" | "marginY" | "spacingX" | "spacingY" | "declaredImageWidth" | "declaredImageHeight">, value: number) {
    setSheet((current) => {
      if (!current) return current;
      const normalized = Number.isFinite(value) ? Math.max(0, Math.floor(value)) : 0;
      setDirty(true);
      onDirtyChange?.(resourceUri, true);
      return { ...current, [field]: normalized };
    });
  }

  function updateSelectedTile(update: TileUpdate) {
    if (selectedTileId == null) return;
    setSheet((current) => {
      if (!current?.tileset) return current;
      setDirty(true);
      onDirtyChange?.(resourceUri, true);
      return {
        ...current,
        tileset: {
          ...current.tileset,
          tiles: current.tileset.tiles.map((tile) => (
            tile.id === selectedTileId ? { ...tile, ...update } : tile
          )),
        },
      };
    });
  }

  function updateDefaults(update: Partial<NonNullable<SheetResourceDto["tileset"]>["defaults"]>) {
    setSheet((current) => {
      if (!current?.tileset) return current;
      setDirty(true);
      onDirtyChange?.(resourceUri, true);
      return {
        ...current,
        tileset: {
          ...current.tileset,
          defaults: {
            ...current.tileset.defaults,
            ...update,
          },
        },
      };
    });
  }

  function adjustZoom(delta: number) {
    setZoom((current) => Math.min(4, Math.max(0.25, Number((current + delta).toFixed(2)))));
  }

  function updateSelectedAnimation(update: Partial<{ id: string; frames: number[]; fps: number | null; looping: boolean | null }>) {
    if (!selectedAnimationId) return;
    setSheet((current) => {
      if (!current?.animations) return current;
      setDirty(true);
      onDirtyChange?.(resourceUri, true);
      return {
        ...current,
        animations: current.animations.map((animation) => (
          animation.id === selectedAnimationId ? { ...animation, ...update } : animation
        )),
      };
    });
  }

  function addAnimation() {
    setSheet((current) => {
      if (!current) return current;
      const nextId = `anim_${(current.animations?.length ?? 0) + 1}`;
      const next = {
        ...current,
        animations: [...(current.animations ?? []), { id: nextId, frames: [0], fps: 12, looping: true }],
      };
      setSelectedAnimationId(nextId);
      setDirty(true);
      onDirtyChange?.(resourceUri, true);
      return next;
    });
  }

  if (error) {
    return (
      <div className="sheet-editor sheet-editor-state">
        <AlertTriangle size={34} />
        <strong>Failed to load sheet resource</strong>
        <span>{error}</span>
      </div>
    );
  }

  if (!sheet) {
    return (
      <div className="sheet-editor sheet-editor-state">
        <Grid2X2 size={34} />
        <strong>Loading sheet resource...</strong>
        <span>{resourceUri}</span>
      </div>
    );
  }

  return (
    <div className="sheet-editor">
      <header className="sheet-editor-toolbar">
        <div className="sheet-editor-title">
          <span className="dock-icon dock-icon-cyan"><Grid2X2 size={14} /></span>
          <strong>{sheet.label}</strong>
          <span>{sheet.relativePath}</span>
          {dirty ? <span className="badge badge-warning">modified</span> : null}
          <span className="badge badge-info">{sheet.sourceSchemaKind}</span>
          <span className="badge badge-valid">{sheet.kind}</span>
        </div>
        <div className="sheet-editor-actions">
          {onReveal ? (
            <button className="button button-tool" type="button" onClick={onReveal}>
              Reveal
            </button>
          ) : null}
          <button className="button button-tool" type="button" disabled={saving || !dirty || localDiagnostics.some((diagnostic) => diagnostic.level === "error")} onClick={() => void saveCurrentSheet()}>
            <Save size={13} />
            {saving ? "Saving" : "Save normalized"}
          </button>
        </div>
      </header>

      <main className="sheet-editor-body">
        <section className="sheet-atlas-stage">
          {sheet.imageExists ? (
            <SheetAtlasPreview
              selectedTileId={selectedTileId}
              sheet={sheet}
              zoom={zoom}
              onZoom={adjustZoom}
              onSelectTile={setSelectedTileId}
            />
          ) : (
            <div className="sheet-editor-state">
              <ImageIcon size={42} />
              <strong>Image not found</strong>
              <span>{sheet.imagePath || "No atlas image declared."}</span>
            </div>
          )}
        </section>

        <aside className="sheet-inspector">
          <section>
            <h3>Atlas</h3>
            <div className="sheet-zoom-controls">
              <button type="button" onClick={() => adjustZoom(-0.25)}>-</button>
              <button className={zoom === 1 ? "active" : ""} type="button" onClick={() => setZoom(1)}>Fit</button>
              <button type="button" onClick={() => adjustZoom(0.25)}>+</button>
              <input
                aria-label="Atlas zoom"
                max={4}
                min={0.25}
                step={0.25}
                type="range"
                value={zoom}
                onChange={(event) => setZoom(Number(event.target.value))}
              />
              <span>{Math.round(zoom * 100)}%</span>
            </div>
            <div className="sheet-form-grid">
              <ReadOnlyField label="Image" value={sheet.imagePath} title={sheet.imageAbsolutePath} />
              <ReadOnlyField label="Actual" value={`${sheet.imageWidth ?? "?"} x ${sheet.imageHeight ?? "?"}`} />
              <NumberField label="Declared W" value={sheet.declaredImageWidth ?? 0} onChange={(value) => updateSheetNumber("declaredImageWidth", value)} />
              <NumberField label="Declared H" value={sheet.declaredImageHeight ?? 0} onChange={(value) => updateSheetNumber("declaredImageHeight", value)} />
              <NumberField label="Cell W" value={sheet.cellWidth} min={1} onChange={(value) => updateSheetNumber("cellWidth", value)} />
              <NumberField label="Cell H" value={sheet.cellHeight} min={1} onChange={(value) => updateSheetNumber("cellHeight", value)} />
              <NumberField label="Columns" value={sheet.columns} min={1} onChange={(value) => updateSheetNumber("columns", value)} />
              <NumberField label="Rows" value={sheet.rows} min={1} onChange={(value) => updateSheetNumber("rows", value)} />
              <NumberField label="Count" value={sheet.count} min={0} onChange={(value) => updateSheetNumber("count", value)} />
              <NumberField label="Margin X" value={sheet.marginX} min={0} onChange={(value) => updateSheetNumber("marginX", value)} />
              <NumberField label="Margin Y" value={sheet.marginY} min={0} onChange={(value) => updateSheetNumber("marginY", value)} />
              <NumberField label="Spacing X" value={sheet.spacingX} min={0} onChange={(value) => updateSheetNumber("spacingX", value)} />
              <NumberField label="Spacing Y" value={sheet.spacingY} min={0} onChange={(value) => updateSheetNumber("spacingY", value)} />
            </div>
          </section>

          <SelectedTilePanel
            selectedTileId={selectedTileId}
            sheet={sheet}
            onUpdateTile={updateSelectedTile}
          />

          <section>
            <h3>Defaults</h3>
            <div className="sheet-form-grid">
              <TextField
                label="Collision"
                value={sheet.tileset?.defaults.collision ?? ""}
                onChange={(value) => updateDefaults({ collision: value.trim() || "none" })}
              />
              <label className="sheet-field sheet-checkbox-field">
                <span>Damageable</span>
                <input
                  checked={sheet.tileset?.defaults.damageable ?? false}
                  type="checkbox"
                  onChange={(event) => updateDefaults({ damageable: event.target.checked })}
                />
              </label>
            </div>
          </section>

          <section>
            <h3>Diagnostics</h3>
            {sheet.diagnostics.length || localDiagnostics.length ? (
              <div className="sheet-diagnostics">
                {localDiagnostics.map((diagnostic, index) => (
                  <div key={`${diagnostic.code}:local:${index}`} className={`sheet-diagnostic diagnostic-${diagnostic.level}`}>
                    <strong>{diagnostic.code}</strong>
                    <span>{diagnostic.message}</span>
                  </div>
                ))}
                {sheet.diagnostics.map((diagnostic, index) => (
                  <div key={`${diagnostic.code}:${index}`} className={`sheet-diagnostic diagnostic-${diagnostic.level}`}>
                    <strong>{diagnostic.code}</strong>
                    <span>{diagnostic.message}</span>
                  </div>
                ))}
              </div>
            ) : (
              <p className="muted">No sheet diagnostics.</p>
            )}
          </section>

          <section>
            <h3>Tiles</h3>
            {selectedTileId != null ? (
              <p className="sheet-selection-readout">
                tile {selectedTileId} · col {selectedTileId % Math.max(1, sheet.columns)} · row {Math.floor(selectedTileId / Math.max(1, sheet.columns))}
              </p>
            ) : null}
            <div className="sheet-tile-list">
              {(sheet.tileset?.tiles ?? []).slice(0, 80).map((tile) => (
                <button
                  key={`${tile.key}:${tile.id}`}
                  className={`sheet-tile-row ${selectedTileId === tile.id ? "selected" : ""}`}
                  type="button"
                  onClick={() => setSelectedTileId(tile.id)}
                >
                  <strong>{tile.id}</strong>
                  <span>{tile.name ?? tile.role ?? tile.key}</span>
                  <em>{tile.collision ?? sheet.tileset?.defaults.collision ?? ""}</em>
                </button>
              ))}
            </div>
          </section>

          {sheet.kind === "spritesheet" ? (
            <section>
              <div className="sheet-section-header">
                <h3>Animations</h3>
                <button type="button" className="button button-tool" onClick={addAnimation}>Add</button>
              </div>
              <div className="sheet-tile-list">
                {(sheet.animations ?? []).map((animation) => (
                  <button
                    key={animation.id}
                    className={`sheet-tile-row ${selectedAnimationId === animation.id ? "selected" : ""}`}
                    type="button"
                    onClick={() => setSelectedAnimationId(animation.id)}
                  >
                    <strong>{animation.id}</strong>
                    <span>{animation.frames.join(", ") || "no frames"}</span>
                    <em>{animation.fps ?? 12} fps</em>
                  </button>
                ))}
              </div>
              {selectedAnimationId ? (
                <div className="sheet-form-grid sheet-section-block">
                  <TextField
                    label="Animation"
                    value={(sheet.animations ?? []).find((animation) => animation.id === selectedAnimationId)?.id ?? ""}
                    onChange={(value) => updateSelectedAnimation({ id: value.trim() || "anim" })}
                  />
                  <NumberField
                    label="FPS"
                    value={(sheet.animations ?? []).find((animation) => animation.id === selectedAnimationId)?.fps ?? 12}
                    min={1}
                    onChange={(value) => updateSelectedAnimation({ fps: value })}
                  />
                  <TextField
                    label="Frames"
                    value={((sheet.animations ?? []).find((animation) => animation.id === selectedAnimationId)?.frames ?? []).join(", ")}
                    onChange={(value) => updateSelectedAnimation({
                      frames: value.split(",").map((item) => Number(item.trim())).filter((item) => Number.isFinite(item) && item >= 0),
                    })}
                    className="wide"
                  />
                  <label className="sheet-field sheet-checkbox-field">
                    <span>Looping</span>
                    <input
                      checked={(sheet.animations ?? []).find((animation) => animation.id === selectedAnimationId)?.looping ?? true}
                      type="checkbox"
                      onChange={(event) => updateSelectedAnimation({ looping: event.target.checked })}
                    />
                  </label>
                </div>
              ) : null}
            </section>
          ) : null}
        </aside>
      </main>
    </div>
  );
}

type SheetDraftDiagnostic = {
  level: "warning" | "error";
  code: string;
  message: string;
};

type TileUpdate = {
  name?: string | null;
  role?: string | null;
  category?: string | null;
  collision?: string | null;
  damageable?: boolean | null;
  tags?: string[];
};

function SheetAtlasPreview({
  onSelectTile,
  onZoom,
  selectedTileId,
  sheet,
  zoom,
}: {
  onSelectTile: (tileId: number) => void;
  onZoom: (delta: number) => void;
  selectedTileId: number | null;
  sheet: SheetResourceDto;
  zoom: number;
}) {
  const viewportRef = useRef<HTMLDivElement | null>(null);
  const dragRef = useRef<{ active: boolean; left: number; top: number; x: number; y: number }>({
    active: false,
    left: 0,
    top: 0,
    x: 0,
    y: 0,
  });
  const imageWidth = sheet.imageWidth ?? sheet.declaredImageWidth ?? (
    sheet.marginX + sheet.columns * sheet.cellWidth + Math.max(0, sheet.columns - 1) * sheet.spacingX
  );
  const imageHeight = sheet.imageHeight ?? sheet.declaredImageHeight ?? (
    sheet.marginY + sheet.rows * sheet.cellHeight + Math.max(0, sheet.rows - 1) * sheet.spacingY
  );
  const gridStyle = useMemo(() => {
    return {
      "--sheet-image-width": `${imageWidth * zoom}px`,
      "--sheet-image-height": `${imageHeight * zoom}px`,
    } as React.CSSProperties;
  }, [imageHeight, imageWidth, zoom]);
  const hitAreas = useMemo(
    () => Array.from({ length: sheet.count }, (_, tileId) => {
      const column = tileId % Math.max(1, sheet.columns);
      const row = Math.floor(tileId / Math.max(1, sheet.columns));
      const left = (sheet.marginX + column * (sheet.cellWidth + sheet.spacingX)) * zoom;
      const top = (sheet.marginY + row * (sheet.cellHeight + sheet.spacingY)) * zoom;
      const width = sheet.cellWidth * zoom;
      const height = sheet.cellHeight * zoom;
      return {
        tileId,
        style: {
          left: `${left}px`,
          top: `${top}px`,
          width: `${width}px`,
          height: `${height}px`,
        } as React.CSSProperties,
      };
    }),
    [sheet.cellHeight, sheet.cellWidth, sheet.columns, sheet.count, sheet.marginX, sheet.marginY, sheet.spacingX, sheet.spacingY, zoom],
  );

  return (
    <div
      ref={viewportRef}
      className="sheet-atlas-viewport"
      onWheel={(event) => {
        if (!event.ctrlKey) return;
        event.preventDefault();
        onZoom(event.deltaY > 0 ? -0.25 : 0.25);
      }}
      onMouseDown={(event) => {
        if (event.button !== 1 && event.button !== 0) return;
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
    >
      <div className="sheet-atlas-frame" style={gridStyle}>
        <img src={fileSrc(sheet.imageAbsolutePath)} alt={sheet.label} draggable={false} />
        <div className="sheet-atlas-hit-grid" aria-label="Tiles">
          {hitAreas.map(({ tileId, style }) => (
            <button
              key={tileId}
              className={selectedTileId === tileId ? "selected" : ""}
              style={style}
              type="button"
              title={`Tile ${tileId}`}
              onClick={() => onSelectTile(tileId)}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

function ReadOnlyField({ label, title, value }: { label: string; title?: string; value: string }) {
  return (
    <label className="sheet-field readonly">
      <span>{label}</span>
      <input readOnly title={title ?? value} value={value} />
    </label>
  );
}

function NumberField({
  label,
  min = 0,
  onChange,
  value,
}: {
  label: string;
  min?: number;
  onChange: (value: number) => void;
  value: number;
}) {
  return (
    <label className="sheet-field">
      <span>{label}</span>
      <input
        min={min}
        type="number"
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </label>
  );
}

function SelectedTilePanel({
  onUpdateTile,
  selectedTileId,
  sheet,
}: {
  onUpdateTile: (update: TileUpdate) => void;
  selectedTileId: number | null;
  sheet: SheetResourceDto;
}) {
  const tile = sheet.tileset?.tiles.find((candidate) => candidate.id === selectedTileId);
  const tagsText = tile?.tags.join(", ") ?? "";
  return (
    <section>
      <h3>Selected Tile</h3>
      <dl className="kv-list">
        <dt>ID</dt>
        <dd>{selectedTileId ?? "none"}</dd>
      </dl>
      <div className="sheet-form-grid">
        <TextField label="Name" value={tile?.name ?? ""} onChange={(value) => onUpdateTile({ name: emptyToNull(value) })} />
        <TextField label="Role" value={tile?.role ?? ""} onChange={(value) => onUpdateTile({ role: emptyToNull(value) })} />
        <TextField label="Category" value={tile?.category ?? ""} onChange={(value) => onUpdateTile({ category: emptyToNull(value) })} />
        <TextField label="Collision" value={tile?.collision ?? sheet.tileset?.defaults.collision ?? ""} onChange={(value) => onUpdateTile({ collision: emptyToNull(value) })} />
        <TextField className="wide" label="Tags" value={tagsText} onChange={(value) => onUpdateTile({ tags: parseTags(value) })} />
        <label className="sheet-field sheet-checkbox-field">
          <span>Damageable</span>
          <input
            checked={tile?.damageable ?? sheet.tileset?.defaults.damageable ?? false}
            type="checkbox"
            onChange={(event) => onUpdateTile({ damageable: event.target.checked })}
          />
        </label>
      </div>
    </section>
  );
}

function TextField({
  className,
  label,
  onChange,
  value,
}: {
  className?: string;
  label: string;
  onChange: (value: string) => void;
  value: string;
}) {
  return (
    <label className={`sheet-field ${className ?? ""}`}>
      <span>{label}</span>
      <input value={value} onChange={(event) => onChange(event.target.value)} />
    </label>
  );
}

function emptyToNull(value: string): string | null {
  const trimmed = value.trim();
  return trimmed.length ? trimmed : null;
}

function parseTags(value: string): string[] {
  return value
    .split(",")
    .map((tag) => tag.trim())
    .filter(Boolean);
}

function validateSheetDraft(sheet: SheetResourceDto): SheetDraftDiagnostic[] {
  const diagnostics: SheetDraftDiagnostic[] = [];
  if (sheet.cellWidth <= 0 || sheet.cellHeight <= 0) {
    diagnostics.push({
      level: "error",
      code: "invalid_cell_size",
      message: "Cell width and height must be greater than zero.",
    });
  }
  if (sheet.columns <= 0 || sheet.rows <= 0) {
    diagnostics.push({
      level: "error",
      code: "invalid_grid_size",
      message: "Columns and rows must be greater than zero.",
    });
  }
  if (sheet.count > sheet.columns * sheet.rows) {
    diagnostics.push({
      level: "warning",
      code: "tile_count_overflow",
      message: "Tile count is larger than columns * rows.",
    });
  }

  const imageWidth = sheet.imageWidth ?? sheet.declaredImageWidth ?? 0;
  const imageHeight = sheet.imageHeight ?? sheet.declaredImageHeight ?? 0;
  const gridWidth = sheet.marginX + sheet.columns * sheet.cellWidth + Math.max(0, sheet.columns - 1) * sheet.spacingX;
  const gridHeight = sheet.marginY + sheet.rows * sheet.cellHeight + Math.max(0, sheet.rows - 1) * sheet.spacingY;
  if (imageWidth > 0 && imageHeight > 0 && (gridWidth > imageWidth || gridHeight > imageHeight)) {
    diagnostics.push({
      level: "error",
      code: "grid_exceeds_image_bounds",
      message: `Grid ${gridWidth}x${gridHeight} exceeds image bounds ${imageWidth}x${imageHeight}.`,
    });
  }
  return diagnostics;
}
