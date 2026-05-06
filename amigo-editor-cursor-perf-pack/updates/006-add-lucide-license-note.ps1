param([Parameter(Mandatory=$true)][string]$RepoRoot)
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot "_common.ps1")

$target = Join-Path $RepoRoot "crates/apps/amigo-editor/THIRD_PARTY_NOTICES.editor-cursors.md"

$content = @'
# Third-party notices: editor cursors

## Lucide Icons

The editor viewport cursor overlay uses icons from the `lucide-react` package already declared by `amigo-editor`.

- Project: Lucide Icons
- Site: https://lucide.dev/
- License: ISC License
- Usage: React SVG icon components rendered only inside the editor viewport cursor overlay.

Lucide license text should remain available through the installed npm package and project dependency metadata. Keep this notice if these cursor icons remain part of the editor UI.
'@

Write-NewFileIfChanged -Path $target -Content $content
