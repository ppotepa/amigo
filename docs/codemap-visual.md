# Visual plugin architecture map

Generate a standalone HTML/SVG architecture map from canonical `plugin.toml` manifests:

```sh
python3 scripts/codemap-html.py
```

The default output is `target/amigo-codemap.html`. It needs no web server or JavaScript dependencies; open the file directly in a browser. The map includes plugin ownership edges for capabilities, render targets, contributions, diagnostics, docs and tests. Relation checkboxes reduce visual noise while investigating a specific contract.

This visualization complements `amigo-codemap`: the Rust codemap remains the symbol/change-navigation tool, while the HTML map is a human overview of declarative plugin semantics.
