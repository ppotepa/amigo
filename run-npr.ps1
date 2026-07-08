param(
    [switch]$Release,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$ExtraArgs
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

Push-Location $repoRoot
try {
    $cargoArgs = @("run", "-p", "amigo-app")
    if ($Release) {
        $cargoArgs += "--release"
    }
    $cargoArgs += @("--", "--hosted", "--mod=playground-npr", "--scene=comic-lines")
    if (-not $Release) {
        $cargoArgs += "--dev"
    }
    $cargoArgs += $ExtraArgs

    cargo @cargoArgs
} finally {
    Pop-Location
}
