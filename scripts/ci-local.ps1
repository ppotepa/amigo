$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')
& (Join-Path $PSScriptRoot 'validate.ps1') @args
exit $LASTEXITCODE
