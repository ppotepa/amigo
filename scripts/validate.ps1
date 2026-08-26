$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

function Invoke-Checked([scriptblock] $Command) {
    & $Command
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host '==> cargo fmt'
Invoke-Checked { cargo fmt --all -- --check }
Write-Host '==> plugin contracts'
Invoke-Checked { cargo run -p amigo-plugin-check -- validate --workspace --plugins plugins }
Write-Host '==> workspace check'
Invoke-Checked { cargo check --workspace --all-targets }
Write-Host '==> clippy critical crates'
Invoke-Checked { cargo clippy -p amigo-runtime -p amigo-plugin-api -p amigo-plugin-index -p amigo-plugin-loader -p amigo-render-api -p amigo-scripting-rhai --all-targets -- -D warnings }
Write-Host '==> contract tests'
Invoke-Checked { cargo test -p amigo-plugin-api -p amigo-plugin-index -p amigo-plugin-loader -p amigo-render-api -p amigo-scripting-rhai }
