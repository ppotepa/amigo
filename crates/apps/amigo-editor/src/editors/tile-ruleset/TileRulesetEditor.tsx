import { useEffect, useMemo, useState, type CSSProperties } from "react";
import { loadSheetResource, loadTileRulesetResource } from "../../api/editorApi";
import type { SheetResourceDto, TileRulesetResourceDto, TileRulesetTerrainDto } from "../../api/dto";
import { fileSrc } from "../../utils/fileSrc";
import "./tile-ruleset-editor.css";

type ToolSymbol = "." | string;

const GRID_WIDTH = 24;
const GRID_HEIGHT = 12;

export function TileRulesetEditor({
  resourceUri,
  sessionId,
  sourceText,
  onReveal,
}: {
  resourceUri: string;
  sessionId: string;
  sourceText?: string;
  onReveal?: () => void;
}) {
  const [ruleset, setRuleset] = useState<TileRulesetResourceDto | null>(null);
  const [tileset, setTileset] = useState<SheetResourceDto | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [drawEnabled, setDrawEnabled] = useState(false);
  const [activeSymbol, setActiveSymbol] = useState<ToolSymbol>("#");
  const [grid, setGrid] = useState(() => createDefaultGrid());
  const [painting, setPainting] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setError(null);
    setTileset(null);
    void loadTileRulesetResource(sessionId, resourceUri)
      .then(async (loadedRuleset) => {
        if (cancelled) return;
        setRuleset(loadedRuleset);
        setActiveSymbol(loadedRuleset.terrains[0]?.symbol ?? "#");
        if (loadedRuleset.tilesetResourceUri) {
          setTileset(await loadSheetResource(sessionId, loadedRuleset.tilesetResourceUri));
        }
      })
      .catch((reason: unknown) => {
        if (cancelled) return;
        const message = reason instanceof Error ? reason.message : String(reason);
        const fallback = sourceText ? fallbackRulesetFromSource(resourceUri, sourceText) : null;
        if (fallback && message.includes("load_tile_ruleset_resource")) {
          setRuleset(fallback);
          setActiveSymbol(fallback.terrains[0]?.symbol ?? "#");
          void loadSheetResource(sessionId, fallback.tilesetResourceUri ?? inferTilesetResourceUri(resourceUri))
            .then((loadedTileset) => {
              if (!cancelled) setTileset(loadedTileset);
            })
            .catch(() => {
              if (!cancelled) setTileset(null);
            });
          return;
        }
        setError(message);
      });
    return () => {
      cancelled = true;
    };
  }, [resourceUri, sessionId, sourceText]);

  const terrainBySymbol = useMemo(() => {
    const map = new Map<string, TileRulesetTerrainDto>();
    for (const terrain of ruleset?.terrains ?? []) {
      map.set(terrain.symbol, terrain);
    }
    return map;
  }, [ruleset]);

  function paintCell(x: number, y: number, symbol = activeSymbol) {
    if (!drawEnabled) return;
    setGrid((current) => current.map((row, rowIndex) => (
      rowIndex === y ? row.map((cell, columnIndex) => (columnIndex === x ? symbol : cell)) : row
    )));
  }

  if (error) {
    return <div className="editor-empty">Failed to load tile ruleset: {error}</div>;
  }

  if (!ruleset) {
    return <div className="editor-empty">Loading tile ruleset...</div>;
  }

  return (
    <div className="tile-ruleset-editor">
      <header className="tile-ruleset-toolbar">
        <div>
          <strong>{ruleset.label}</strong>
          <span>{ruleset.relativePath}</span>
        </div>
        <div className="toolbar-actions">
          {onReveal ? <button className="button button-tool" type="button" onClick={onReveal}>Reveal</button> : null}
          <button
            className={`button button-tool ${drawEnabled ? "active" : ""}`}
            type="button"
            onClick={() => setDrawEnabled((enabled) => !enabled)}
          >
            Draw
          </button>
          <button className="button button-tool" type="button" onClick={() => setGrid(createDefaultGrid())}>Reset</button>
        </div>
      </header>

      <main className="tile-ruleset-main">
        <section className="ruleset-pad">
          <div className="ruleset-grid-head">
            <strong>Test Pad</strong>
            <span>{drawEnabled ? `drawing ${activeSymbol}` : "preview"}</span>
          </div>
          <div
            className={`ruleset-grid ${drawEnabled ? "drawing" : ""}`}
            onMouseLeave={() => setPainting(false)}
            style={{
              "--ruleset-grid-columns": String(GRID_WIDTH),
              "--ruleset-grid-rows": String(GRID_HEIGHT),
            } as CSSProperties}
          >
            {grid.map((row, y) => row.map((symbol, x) => {
              const tileId = resolvePreviewTile(grid, x, y, terrainBySymbol);
              return (
                <button
                  key={`${x}:${y}`}
                  className={`ruleset-cell ${symbol === "." ? "empty" : ""}`}
                  type="button"
                  onMouseDown={(event) => {
                    event.preventDefault();
                    setPainting(true);
                    paintCell(x, y, event.button === 2 ? "." : activeSymbol);
                  }}
                  onMouseEnter={() => {
                    if (painting) paintCell(x, y);
                  }}
                  onContextMenu={(event) => {
                    event.preventDefault();
                    paintCell(x, y, ".");
                  }}
                >
                  {tileId != null && tileset?.imageExists ? (
                    <span className="ruleset-tile-sprite" style={tileSpriteStyle(tileset, tileId)} />
                  ) : (
                    <span className="ruleset-symbol">{symbol}</span>
                  )}
                </button>
              );
            }))}
          </div>
        </section>

        <aside className="ruleset-side">
          <section className="ruleset-panel">
            <h3>Symbols</h3>
            <button
              type="button"
              className={`ruleset-symbol-button ${activeSymbol === "." ? "active" : ""}`}
              onClick={() => setActiveSymbol(".")}
            >
              <span>.</span>
              <strong>empty / erase</strong>
            </button>
            {ruleset.terrains.map((terrain) => (
              <button
                key={terrain.id}
                type="button"
                className={`ruleset-symbol-button ${activeSymbol === terrain.symbol ? "active" : ""}`}
                onClick={() => setActiveSymbol(terrain.symbol)}
              >
                <span>{terrain.symbol}</span>
                <strong>{terrain.id}</strong>
                <small>{terrain.collision ?? "no collision"}</small>
              </button>
            ))}
          </section>

          <section className="ruleset-panel">
            <h3>Tileset</h3>
            <p className="muted">{ruleset.tilesetResourceUri ?? "No tileset inferred."}</p>
            {tileset ? <span className="badge badge-valid">{tileset.columns}x{tileset.rows} · {tileset.count} tiles</span> : null}
          </section>

          <section className="ruleset-panel">
            <h3>Diagnostics</h3>
            {ruleset.diagnostics.length ? ruleset.diagnostics.map((diagnostic) => (
              <p key={`${diagnostic.code}:${diagnostic.message}`} className="muted">{diagnostic.code}: {diagnostic.message}</p>
            )) : <p className="muted">No ruleset diagnostics.</p>}
          </section>
        </aside>
      </main>
    </div>
  );
}

function fallbackRulesetFromSource(resourceUri: string, source: string): TileRulesetResourceDto | null {
  const terrainsBlock = source.match(/\nterrains:\s*\n([\s\S]*)/);
  if (!terrainsBlock) return null;
  const terrainsOffset = terrainsBlock.index ?? 0;
  const terrainMatches = [...terrainsBlock[1].matchAll(/^  ([A-Za-z0-9_-]+):\s*$/gm)];
  const terrains = terrainMatches.map((match, index) => {
    const id = match[1];
    const start = terrainsOffset + (match.index ?? 0);
    const end = index + 1 < terrainMatches.length
      ? terrainsOffset + (terrainMatches[index + 1].index ?? terrainsBlock[1].length)
      : source.length;
    const block = source.slice(start, end);
    return {
      id,
      symbol: textValue(block, "symbol") ?? id.charAt(0),
      collision: textValue(block, "collision"),
      variants: {
        single: numberValue(block, "single"),
        leftCap: numberValue(block, "left_cap"),
        middle: numberValue(block, "middle"),
        rightCap: numberValue(block, "right_cap"),
        sideLeft: numberValue(block, "side_left"),
        sideRight: numberValue(block, "side_right"),
        center: numberValue(block, "center"),
        topCap: numberValue(block, "top_cap"),
        bottomCap: numberValue(block, "bottom_cap"),
        verticalMiddle: numberValue(block, "vertical_middle"),
        outerCornerTopLeft: numberValue(block, "outer_corner_top_left"),
        outerCornerTopRight: numberValue(block, "outer_corner_top_right"),
        outerCornerBottomLeft: numberValue(block, "outer_corner_bottom_left"),
        outerCornerBottomRight: numberValue(block, "outer_corner_bottom_right"),
        innerCornerTopLeft: numberValue(block, "inner_corner_top_left"),
        innerCornerTopRight: numberValue(block, "inner_corner_top_right"),
        innerCornerBottomLeft: numberValue(block, "inner_corner_bottom_left"),
        innerCornerBottomRight: numberValue(block, "inner_corner_bottom_right"),
      },
    };
  });
  if (!terrains.length) return null;

  return {
    resourceUri,
    absolutePath: "",
    relativePath: resourceUri,
    schemaVersion: numberValue(source, "schema_version") ?? 1,
    id: textValue(source, "id") ?? resourceUri.split("/").pop()?.replace(/\.ya?ml$/i, "") ?? "tile-ruleset",
    label: textValue(source, "label") ?? "Tile Ruleset",
    tileWidth: numberValue(source, "width") ?? 128,
    tileHeight: numberValue(source, "height") ?? 128,
    tilesetResourceUri: textValue(source, "tileset") ?? inferTilesetResourceUri(resourceUri),
    terrains,
    diagnostics: [{
      level: "warning",
      code: "frontend_ruleset_fallback",
      message: "Using frontend fallback parser. Restart Tauri dev process to enable the backend ruleset loader.",
    }],
  };
}

function textValue(source: string, key: string): string | null {
  const match = source.match(new RegExp(`^\\s*${key}:\\s*([^\\n#]+)`, "m"));
  return match?.[1]?.trim().replace(/^["']|["']$/g, "") || null;
}

function numberValue(source: string, key: string): number | null {
  const value = textValue(source, key);
  if (!value) return null;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) ? parsed : null;
}

function inferTilesetResourceUri(resourceUri: string): string {
  const normalized = resourceUri.replace(/\\/g, "/");
  const fileName = normalized.split("/").pop() ?? "";
  const assetId = fileName
    .replace(/\.tile-ruleset\.ya?ml$/i, "")
    .replace(/[-_]rules$/i, "");
  const parent = normalized.slice(0, Math.max(0, normalized.length - fileName.length)).replace(/\/$/, "");
  return `${parent}/${assetId}.tileset.yml`;
}

function createDefaultGrid(): string[][] {
  return Array.from({ length: GRID_HEIGHT }, (_, y) => (
    Array.from({ length: GRID_WIDTH }, (_, x) => {
      if (y === 4 && x >= 4 && x <= 10) return "=";
      if (y === 7 && x >= 13 && x <= 19) return "#";
      if (y === 8 && x >= 14 && x <= 18) return "#";
      return ".";
    })
  ));
}

function resolvePreviewTile(
  grid: string[][],
  x: number,
  y: number,
  terrainBySymbol: Map<string, TileRulesetTerrainDto>,
): number | null {
  const symbol = grid[y]?.[x] ?? ".";
  if (symbol === ".") return null;
  const terrain = terrainBySymbol.get(symbol);
  if (!terrain) return null;

  const left = grid[y]?.[x - 1] === symbol;
  const right = grid[y]?.[x + 1] === symbol;
  const up = grid[y - 1]?.[x] === symbol;
  const down = grid[y + 1]?.[x] === symbol;
  const variants = terrain.variants;

  if (!left && !right && !up && !down) return firstTile(variants.single, variants.center, variants.middle);
  if (!left && right) return firstTile(variants.leftCap, variants.sideLeft, variants.middle, variants.single);
  if (left && !right) return firstTile(variants.rightCap, variants.sideRight, variants.middle, variants.single);
  if (up && down) return firstTile(variants.verticalMiddle, variants.center, variants.middle, variants.single);
  if (left && right && down) return firstTile(variants.topCap, variants.center, variants.middle, variants.single);
  if (left && right) return firstTile(variants.middle, variants.center, variants.single);
  return firstTile(variants.center, variants.middle, variants.single);
}

function firstTile(...values: Array<number | null | undefined>): number | null {
  return values.find((value): value is number => typeof value === "number") ?? null;
}

function tileSpriteStyle(tileset: SheetResourceDto, tileId: number): CSSProperties {
  const columns = Math.max(1, tileset.columns);
  const rows = Math.max(1, tileset.rows);
  const column = tileId % columns;
  const row = Math.floor(tileId / columns);
  const x = columns <= 1 ? 0 : (column / (columns - 1)) * 100;
  const y = rows <= 1 ? 0 : (row / (rows - 1)) * 100;
  return {
    backgroundImage: `url("${fileSrc(tileset.imageAbsolutePath)}")`,
    backgroundPosition: `${x}% ${y}%`,
    backgroundSize: `${columns * 100}% ${rows * 100}%`,
  };
}
