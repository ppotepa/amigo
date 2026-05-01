param(
    [string[]] $Paths = @("crates"),
    [int] $WarningThreshold = 500,
    [int] $ErrorThreshold = 1000,
    [int] $LibWarningThreshold = 250,
    [int] $LibErrorThreshold = 500,
    [int] $Top = 40,
    [switch] $FailOnViolation
)

$errorPaths = @("target", ".git", ".github")
$files = Get-ChildItem -Path $Paths -Recurse -File -Filter *.rs -ErrorAction SilentlyContinue |
    Where-Object {
        foreach ($path in $errorPaths) {
            if ($_.FullName -like "*\$path\*") { return $false }
        }
        return $true
    }

$violations = [System.Collections.Generic.List[string]]::new()
$topEntries = @()

foreach ($file in $files) {
    $lineCount = (Get-Content -Path $file.FullName).Length
    if (-not $lineCount) { continue }

    $limit = $WarningThreshold
    $errorLimit = $ErrorThreshold
    $isLib = $file.Name -eq "lib.rs"
    if ($isLib) {
        $limit = $LibWarningThreshold
        $errorLimit = $LibErrorThreshold
    }

    $severity = ""
    if ($lineCount -gt $errorLimit) {
        $severity = "ERROR"
    } elseif ($lineCount -gt $limit) {
        $severity = "WARN"
    }

    if ($severity -ne "") {
        $violations.Add(
            ($severity + "`t" + $lineCount + "`t" +
                $file.FullName.Replace((Get-Location).Path + "\", ""))
        )
    }

    $topEntries += [PSCustomObject]@{
        File = $file.FullName.Replace((Get-Location).Path + "\", "")
        Lines = $lineCount
    }
}

$maxFileName = 80

if ($violations.Count -gt 0) {
    Write-Host "File size warnings/errors:"
    foreach ($item in $violations) {
        Write-Host "  $item"
    }
} else {
    Write-Host "No warning violations found for configured thresholds."
}

Write-Host ""
Write-Host "Top ${Top} files by size:"
$topEntries | Sort-Object Lines -Descending | Select-Object -First $Top | ForEach-Object {
    $name = $_.File.PadRight($maxFileName)
    Write-Host ("{0} {1}" -f $name, $_.Lines)
}

if ($FailOnViolation -and ($violations | Where-Object { $_ -like "ERROR*"}).Count -gt 0) {
    exit 1
}
