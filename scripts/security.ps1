$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

if (-not (Get-Command cargo-audit -ErrorAction SilentlyContinue)) {
    throw 'cargo-audit is required: cargo install cargo-audit --locked'
}
if (-not (Get-Command cargo-deny -ErrorAction SilentlyContinue)) {
    throw 'cargo-deny is required: cargo install cargo-deny --locked'
}

cargo audit
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo deny check advisories licenses bans sources
exit $LASTEXITCODE
