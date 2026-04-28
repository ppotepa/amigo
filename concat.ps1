param(
    [string]$Root = ".",
    [string]$Output = "concat-output.txt",
    [string[]]$IncludeExtensions = @(".ps1", ".rs", ".rhai", ".yml", ".yaml"),
    [string[]]$ExcludeDirectories = @(".git", "target")
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-RelativePath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$BasePath,
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $baseFullPath = [System.IO.Path]::GetFullPath($BasePath).TrimEnd('\', '/')
    $targetFullPath = [System.IO.Path]::GetFullPath($Path)

    if ($targetFullPath.StartsWith($baseFullPath, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $targetFullPath.Substring($baseFullPath.Length).TrimStart('\', '/').Replace("\", "/")
    }

    $baseUri = [System.Uri]($baseFullPath + [System.IO.Path]::DirectorySeparatorChar)
    $targetUri = [System.Uri]$targetFullPath
    return [System.Uri]::UnescapeDataString(
        $baseUri.MakeRelativeUri($targetUri).ToString()
    ).Replace("\", "/")
}

function Get-SourceFiles {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Directory
    )

    foreach ($entry in Get-ChildItem -LiteralPath $Directory -Force) {
        if ($entry.PSIsContainer) {
            if ($ExcludeDirectories -contains $entry.Name) {
                continue
            }

            Get-SourceFiles -Directory $entry.FullName
            continue
        }

        $extension = [System.IO.Path]::GetExtension($entry.Name).ToLowerInvariant()
        if ($IncludeExtensions -contains $extension) {
            $entry
        }
    }
}

if (-not (Test-Path -LiteralPath $Root)) {
    throw "Root path does not exist: $Root"
}

$resolvedRoot = (Resolve-Path -LiteralPath $Root).Path
$resolvedOutput = if ([System.IO.Path]::IsPathRooted($Output)) {
    $Output
} else {
    Join-Path -Path $resolvedRoot -ChildPath $Output
}

$outputDirectory = Split-Path -Path $resolvedOutput -Parent
if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
}

$files = Get-SourceFiles -Directory $resolvedRoot |
    Where-Object { $_.FullName -ne $resolvedOutput } |
    Sort-Object {
        Get-RelativePath -BasePath $resolvedRoot -Path $_.FullName
    }

$builder = [System.Text.StringBuilder]::new()

foreach ($file in $files) {
    $relativePath = Get-RelativePath -BasePath $resolvedRoot -Path $file.FullName
    [void]$builder.AppendLine(("=" * 100))
    [void]$builder.AppendLine("FILE: $relativePath")
    [void]$builder.AppendLine(("=" * 100))
    [void]$builder.AppendLine((Get-Content -LiteralPath $file.FullName -Raw))
    [void]$builder.AppendLine()
}

$utf8NoBom = [System.Text.UTF8Encoding]::new($false)
[System.IO.File]::WriteAllText($resolvedOutput, $builder.ToString(), $utf8NoBom)

Write-Host ("Wrote {0} file(s) to {1}" -f $files.Count, $resolvedOutput)
