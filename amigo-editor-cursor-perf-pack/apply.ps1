$ErrorActionPreference = "Stop"

$repoRoot = (Get-Location).Path
if (!(Test-Path (Join-Path $repoRoot "Cargo.toml"))) {
  throw "Run this script from the Amigo repository root. Cargo.toml was not found in current directory: $repoRoot"
}
if (!(Test-Path (Join-Path $repoRoot "crates/apps/amigo-editor"))) {
  throw "Run this script from the Amigo repository root. crates/apps/amigo-editor was not found."
}

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$updatesDir = Join-Path $scriptRoot "updates"
$updates = Get-ChildItem $updatesDir -Filter "*.ps1" | Where-Object { $_.Name -ne "_common.ps1" } | Sort-Object Name

Write-Host "Amigo editor cursor/performance patch pack"
Write-Host "Repo root: $repoRoot"
Write-Host "Updates: $($updates.Count)"
Write-Host ""

foreach ($update in $updates) {
  Write-Host "==> $($update.Name)"
  & $update.FullName -RepoRoot $repoRoot
  Write-Host ""
}

Write-Host "Done."
Write-Host "Suggested checks:"
Write-Host "  cargo fmt -p amigo-editor"
Write-Host "  cargo check -p amigo-editor"
Write-Host "  cd crates/apps/amigo-editor; npm run build"
