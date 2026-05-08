param(
    [string]$RepoRoot = "",
    [string]$CodeMap = "",
    [switch]$SkipVerify,
    [switch]$SkipRefresh
)

$ErrorActionPreference = "Stop"

$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$PlanPath = Join-Path $ScriptRoot "plan.yml"

if ([string]::IsNullOrWhiteSpace($RepoRoot)) {
    # Expected layout:
    #   D:\Git\amigo\
    #     step1\
    #       apply.ps1
    #       plan.yml
    #       updates\
    $RepoRoot = Split-Path -Parent $ScriptRoot
}

$RepoRoot = Resolve-Path $RepoRoot

if (-not (Test-Path $PlanPath)) {
    throw "Missing plan file: $PlanPath"
}

Push-Location $RepoRoot
try {
    if ([string]::IsNullOrWhiteSpace($CodeMap)) {
        $CandidateExe = Join-Path $RepoRoot "target\debug\amigo-codemap.exe"
        $CandidateNoExe = Join-Path $RepoRoot "target\debug\amigo-codemap"

        if (Test-Path $CandidateExe) {
            $CodeMap = $CandidateExe
        }
        elseif (Test-Path $CandidateNoExe) {
            $CodeMap = $CandidateNoExe
        }
        else {
            $CodeMap = "cargo run -p amigo-codemap --"
        }
    }

    function Invoke-CodeMap {
        param(
            [Parameter(Mandatory = $true)]
            [string[]]$Arguments
        )

        Write-Host ""
        Write-Host ">>> amigo-codemap $($Arguments -join ' ')" -ForegroundColor Cyan

        if ($CodeMap -like "cargo run*") {
            & cargo run -p amigo-codemap -- @Arguments
        }
        else {
            & $CodeMap @Arguments
        }

        if ($LASTEXITCODE -ne 0) {
            throw "amigo-codemap failed with exit code ${LASTEXITCODE}: $($Arguments -join ' ')"
        }
    }

    function Invoke-VerifyCommand {
        param(
            [Parameter(Mandatory = $true)]
            [string]$Command
        )

        Write-Host ""
        Write-Host ">>> verify: $Command" -ForegroundColor Cyan

        cmd.exe /c $Command

        if ($LASTEXITCODE -ne 0) {
            throw "Verify command failed with exit code ${LASTEXITCODE}: $Command"
        }
    }

    Write-Host "Repo root: $RepoRoot" -ForegroundColor DarkGray
    Write-Host "Plan:      $PlanPath" -ForegroundColor DarkGray
    Write-Host "CodeMap:   $CodeMap" -ForegroundColor DarkGray

    Invoke-CodeMap @("ops-preview", "--from", $PlanPath)
    Invoke-CodeMap @("ops-check", "--from", $PlanPath, "--strict")
    Invoke-CodeMap @("ops-apply", "--from", $PlanPath, "--write", "--backup", "--stop-on-error", "--strict")

    if (-not $SkipRefresh) {
        Invoke-CodeMap @("refresh")
    }

    if (-not $SkipVerify) {
        Invoke-VerifyCommand "cargo check -p amigo-editor-core"
        Invoke-VerifyCommand "cargo test -p amigo-editor-core"
        Invoke-VerifyCommand "npm run build --prefix crates/apps/amigo-editor"
    }

    Write-Host ""
    Write-Host "OK: step applied successfully." -ForegroundColor Green
}
finally {
    Pop-Location
}