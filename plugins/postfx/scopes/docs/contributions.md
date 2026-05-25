# Scopes Contributions

Scopes is a tooling target consumer for frame analysis.

## Consumes
- Final composite or scene analysis targets.
- Highlight and emissive targets when the selected scope needs source-channel
  inspection.

## Emits
- Diagnostics snapshot output containing scope data.

Scopes emits no render-source contributions and does not feed post-fx execution.
It reports measurements for already declared targets.
