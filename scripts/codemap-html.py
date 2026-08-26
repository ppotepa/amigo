#!/usr/bin/env python3
from __future__ import annotations

import argparse
import html
import tomllib
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

COLUMNS = {
    "plugin": 20,
    "capability": 320,
    "target": 620,
    "contribution": 920,
    "evidence": 1220,
}
NODE_WIDTH = 260
NODE_HEIGHT = 26
ROW_HEIGHT = 34
TOP = 100


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate a standalone HTML/SVG plugin architecture map")
    parser.add_argument("--plugins", default="plugins", help="plugin tree relative to repo root")
    parser.add_argument("--output", default="target/amigo-codemap.html", help="output HTML path relative to repo root")
    return parser.parse_args()


def contribution_label(item: object) -> str:
    if not isinstance(item, dict):
        return str(item)
    domain = str(item.get("domain", "?"))
    kind = str(item.get("type", item.get("contribution_type", item.get("contribution-type", "?"))))
    return f"{domain}::{kind}"


def manifests(root: Path) -> list[tuple[Path, dict]]:
    result = []
    for path in sorted(root.rglob("plugin.toml")):
        with path.open("rb") as handle:
            result.append((path, tomllib.load(handle)))
    return result


def collect_graph(plugin_root: Path):
    nodes: dict[str, set[str]] = defaultdict(set)
    edges: list[tuple[str, str, str, str]] = []

    for path, manifest in manifests(plugin_root):
        plugin_id = str(manifest.get("id", path.parent.as_posix()))
        nodes["plugin"].add(plugin_id)

        caps = manifest.get("capabilities", {})
        for relation, values in (("provides", caps.get("provides", [])), ("requires", caps.get("requires", []))):
            for value in values:
                label = str(value)
                nodes["capability"].add(label)
                edges.append((plugin_id, "capability", label, relation))

        targets = manifest.get("targets", {})
        for relation in ("reads", "writes", "contributes"):
            for value in targets.get(relation, []):
                label = str(value)
                nodes["target"].add(label)
                edges.append((plugin_id, "target", label, relation))

        contributions = manifest.get("contributions", {})
        for relation in ("emits", "consumes"):
            for value in contributions.get(relation, []):
                label = contribution_label(value)
                nodes["contribution"].add(label)
                edges.append((plugin_id, "contribution", label, relation))

        for channel in manifest.get("diagnostics", {}).get("channels", []):
            label = f"diagnostic: {channel}"
            nodes["evidence"].add(label)
            edges.append((plugin_id, "evidence", label, "diagnostic"))

        for key, value in manifest.get("tests", {}).items():
            if value:
                label = f"test:{key}: {plugin_id} / {value}"
                nodes["evidence"].add(label)
                edges.append((plugin_id, "evidence", label, "test"))

        for key, value in manifest.get("docs", {}).items():
            if value:
                label = f"doc:{key}: {plugin_id} / {value}"
                nodes["evidence"].add(label)
                edges.append((plugin_id, "evidence", label, "doc"))

    ordered = {category: sorted(values) for category, values in nodes.items()}
    return ordered, edges


def svg_for(nodes: dict[str, list[str]], edges: list[tuple[str, str, str, str]]) -> tuple[str, set[str]]:
    positions: dict[tuple[str, str], tuple[int, int]] = {}
    max_rows = 0
    parts = []

    titles = {
        "plugin": "Plugins",
        "capability": "Capabilities",
        "target": "Targets",
        "contribution": "Contributions",
        "evidence": "Diagnostics / docs / tests",
    }
    for category, x in COLUMNS.items():
        values = nodes.get(category, [])
        max_rows = max(max_rows, len(values))
        parts.append(f'<text class="column-title" x="{x}" y="46">{html.escape(titles[category])}</text>')
        for index, label in enumerate(values):
            y = TOP + index * ROW_HEIGHT
            positions[(category, label)] = (x, y)
            tooltip = html.escape(label, quote=True)
            display = label if len(label) <= 40 else label[:37] + "…"
            parts.append(
                f'<g class="node node-{category}"><title>{tooltip}</title>'
                f'<rect x="{x}" y="{y}" width="{NODE_WIDTH}" height="{NODE_HEIGHT}" rx="5" />'
                f'<text x="{x + 8}" y="{y + 18}">{html.escape(display)}</text></g>'
            )

    relation_names = {relation for *_, relation in edges}
    edge_parts = []
    for plugin, category, label, relation in edges:
        source = positions.get(("plugin", plugin))
        target = positions.get((category, label))
        if source is None or target is None:
            continue
        sx, sy = source[0] + NODE_WIDTH, source[1] + NODE_HEIGHT // 2
        tx, ty = target[0], target[1] + NODE_HEIGHT // 2
        bend = (sx + tx) // 2
        edge_parts.append(
            f'<path class="edge rel-{html.escape(relation)}" data-relation="{html.escape(relation)}" '
            f'd="M {sx} {sy} C {bend} {sy}, {bend} {ty}, {tx} {ty}"><title>{html.escape(plugin)} {html.escape(relation)} {html.escape(label)}</title></path>'
        )

    width = 1510
    height = TOP + max_rows * ROW_HEIGHT + 80
    svg = (
        f'<svg viewBox="0 0 {width} {height}" width="{width}" height="{height}" '
        'xmlns="http://www.w3.org/2000/svg">'
        + "".join(edge_parts)
        + "".join(parts)
        + "</svg>"
    )
    return svg, relation_names


def document(nodes: dict[str, list[str]], edges: list[tuple[str, str, str, str]]) -> str:
    svg, relations = svg_for(nodes, edges)
    controls = " ".join(
        f'<label><input type="checkbox" data-toggle="{html.escape(relation)}" checked> {html.escape(relation)}</label>'
        for relation in sorted(relations)
    )
    counts = " · ".join(f"{category}: {len(nodes.get(category, []))}" for category in COLUMNS)
    return f'''<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Amigo plugin architecture map</title>
<style>
:root {{ color-scheme: light dark; font-family: ui-sans-serif, system-ui, sans-serif; }}
body {{ margin: 0; }}
header {{ position: sticky; top: 0; z-index: 2; padding: 14px 18px; background: Canvas; border-bottom: 1px solid GrayText; }}
h1 {{ margin: 0 0 6px; font-size: 20px; }}
.summary {{ opacity: .75; font-size: 13px; }}
.controls {{ margin-top: 8px; display: flex; flex-wrap: wrap; gap: 10px; font-size: 12px; }}
main {{ overflow: auto; padding: 12px; }}
svg {{ background: Canvas; }}
.node rect {{ fill: Canvas; stroke: GrayText; stroke-width: 1; }}
.node text {{ fill: CanvasText; font-size: 11px; pointer-events: none; }}
.column-title {{ fill: CanvasText; font-weight: 700; font-size: 15px; }}
.edge {{ fill: none; stroke: GrayText; stroke-width: 1; opacity: .28; }}
.edge:hover {{ stroke-width: 3; opacity: 1; }}
.edge.hidden {{ display: none; }}
</style>
</head>
<body>
<header>
<h1>Amigo plugin architecture map</h1>
<div class="summary">{html.escape(counts)} · edges: {len(edges)}</div>
<div class="controls">{controls}</div>
</header>
<main>{svg}</main>
<script>
for (const input of document.querySelectorAll('[data-toggle]')) {{
  input.addEventListener('change', () => {{
    const relation = input.dataset.toggle;
    for (const edge of document.querySelectorAll(`[data-relation="${{relation}}"]`)) {{
      edge.classList.toggle('hidden', !input.checked);
    }}
  }});
}}
</script>
</body>
</html>'''


def main() -> int:
    args = parse_args()
    plugin_root = (ROOT / args.plugins).resolve()
    output = (ROOT / args.output).resolve()
    nodes, edges = collect_graph(plugin_root)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(document(nodes, edges), encoding="utf-8")
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
