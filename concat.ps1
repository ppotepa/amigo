param(
    [string]$Root = ".",
    [string]$Output = "concat-output.txt",
    [switch]$NoZip,
    [switch]$Split,
    [ValidateSet("domain", "top-folder", "extension")]
    [string]$SplitMode = "domain",
    [string[]]$IncludeExtensions = @(
        ".ps1",
        ".rs",
        ".rhai",
        ".yml",
        ".yaml",
        ".toml",
        ".json",
        ".ts",
        ".tsx",
        ".js",
        ".jsx",
        ".css",
        ".scss",
        ".html",
        ".md"
    ),
    [string[]]$ExcludeExtensions = @(
        ".txt",
        ".zip"
    ),
    [string[]]$ExcludeDirectories = @(
        ".amigo",
        ".git",
        "target",
        "node_modules",
        "dist",
        "coverage",
        ".next",
        ".nuxt",
        ".svelte-kit",
        ".astro",
        ".turbo",
        ".parcel-cache",
        ".vite",
        ".idea",
        ".vscode",
        "bin",
        "obj"
    )
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$IncludeExtensions = $IncludeExtensions |
    ForEach-Object { $_.Trim().ToLowerInvariant() } |
    Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
    Sort-Object -Unique

$ExcludeDirectories = $ExcludeDirectories |
    ForEach-Object { $_.Trim() } |
    Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
    Sort-Object -Unique

$ExcludeExtensions = $ExcludeExtensions |
    ForEach-Object { $_.Trim().ToLowerInvariant() } |
    Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
    Sort-Object -Unique

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

    try {
        $entries = Get-ChildItem -LiteralPath $Directory -Force -ErrorAction Stop
    } catch [System.UnauthorizedAccessException] {
        Write-Warning "Skipping inaccessible directory: $Directory"
        return
    } catch {
        Write-Warning "Skipping unreadable directory: $Directory ($($_.Exception.Message))"
        return
    }

    foreach ($entry in $entries) {
        if ($entry.PSIsContainer) {
            if ($ExcludeDirectories -contains $entry.Name) {
                continue
            }

            Get-SourceFiles -Directory $entry.FullName
            continue
        }

        $extension = [System.IO.Path]::GetExtension($entry.Name).ToLowerInvariant()
        if (($IncludeExtensions -contains $extension) -and ($ExcludeExtensions -notcontains $extension)) {
            $entry
        }
    }
}

function Get-ConcatSplitKey {
    param(
        [Parameter(Mandatory = $true)]
        [string]$RelativePath,
        [Parameter(Mandatory = $true)]
        [System.IO.FileInfo]$File
    )

    if ($SplitMode -eq "extension") {
        $extension = [System.IO.Path]::GetExtension($File.Name).TrimStart(".").ToLowerInvariant()
        if ([string]::IsNullOrWhiteSpace($extension)) {
            return "no-extension"
        }
        return $extension
    }

    $path = $RelativePath.Replace("\", "/").ToLowerInvariant()

    if ($SplitMode -eq "top-folder") {
        $parts = $path.Split("/", [System.StringSplitOptions]::RemoveEmptyEntries)
        if ($parts.Length -eq 0) {
            return "root"
        }
        return $parts[0]
    }

    if ($path.StartsWith("crates/apps/amigo-editor/")) { return "editor" }
    if ($path.StartsWith("crates/apps/")) { return "apps" }
    if ($path.StartsWith("crates/tools/amigo-codemap/")) { return "codemap" }
    if ($path.StartsWith("crates/tools/")) { return "tools" }
    if ($path.StartsWith("crates/engine/")) { return "engine" }
    if ($path.StartsWith("crates/foundation/")) { return "foundation" }
    if ($path.StartsWith("crates/ui/")) { return "ui" }
    if ($path.StartsWith("crates/scripting/")) { return "scripting" }
    if ($path.StartsWith("crates/audio/")) { return "audio" }
    if ($path.StartsWith("crates/2d/")) { return "2d" }
    if ($path.StartsWith("crates/3d/")) { return "3d" }
    if ($path.StartsWith("crates/")) { return "crates-other" }
    if ($path.StartsWith("mods/")) { return "mods" }
    if ($path.StartsWith("config/")) { return "config" }
    if ($path.StartsWith("tools/")) { return "repo-tools" }
    if ($path.StartsWith("docs/")) { return "docs" }
    if ($path.EndsWith(".md")) { return "docs" }

    return "root"
}

function Write-ConcatFile {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [array]$Files,
        [Parameter(Mandatory = $true)]
        [string]$BasePath
    )

    $builder = [System.Text.StringBuilder]::new()

    foreach ($file in $Files) {
        $relativePath = Get-RelativePath -BasePath $BasePath -Path $file.FullName
        [void]$builder.AppendLine(("=" * 100))
        [void]$builder.AppendLine("FILE: $relativePath")
        [void]$builder.AppendLine(("=" * 100))
        [void]$builder.AppendLine((Get-Content -LiteralPath $file.FullName -Raw))
        [void]$builder.AppendLine()
    }

    $directory = Split-Path -Path $Path -Parent
    if (-not [string]::IsNullOrWhiteSpace($directory)) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    }

    $utf8NoBom = [System.Text.UTF8Encoding]::new($false)
    [System.IO.File]::WriteAllText($Path, $builder.ToString(), $utf8NoBom)
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

if (-not $Split) {
    Write-ConcatFile -Path $resolvedOutput -Files $files -BasePath $resolvedRoot

    $resolvedZip = [System.IO.Path]::ChangeExtension($resolvedOutput, ".zip")
    if (-not $NoZip) {
        if (Test-Path -LiteralPath $resolvedZip) {
            Remove-Item -LiteralPath $resolvedZip -Force
        }

        Compress-Archive -LiteralPath $resolvedOutput -DestinationPath $resolvedZip
    }

    Write-Host ("Wrote {0} file(s) to {1}" -f $files.Count, $resolvedOutput)
    if (-not $NoZip) {
        Write-Host ("Wrote ZIP archive to {0}" -f $resolvedZip)
    }
    Write-Host ("Included extensions: {0}" -f ($IncludeExtensions -join ", "))
    Write-Host ("Excluded extensions: {0}" -f ($ExcludeExtensions -join ", "))
    Write-Host ("Excluded directories: {0}" -f ($ExcludeDirectories -join ", "))
    return
}

$splitDirectory = [System.IO.Path]::Combine(
    (Split-Path -Path $resolvedOutput -Parent),
    [System.IO.Path]::GetFileNameWithoutExtension($resolvedOutput)
)

if ((Test-Path -LiteralPath $splitDirectory) -and -not (Get-Item -LiteralPath $splitDirectory).PSIsContainer) {
    throw "Split output path is a file; choose another output: $splitDirectory"
}

New-Item -ItemType Directory -Path $splitDirectory -Force | Out-Null

$groups = @{}
foreach ($file in $files) {
    $relativePath = Get-RelativePath -BasePath $resolvedRoot -Path $file.FullName
    $key = Get-ConcatSplitKey -RelativePath $relativePath -File $file

    if (-not $groups.ContainsKey($key)) {
        $groups[$key] = [System.Collections.Generic.List[System.IO.FileInfo]]::new()
    }

    $groups[$key].Add($file)
}

$manifestFiles = @()
foreach ($key in ($groups.Keys | Sort-Object)) {
    $groupFiles = @($groups[$key] | Sort-Object {
        Get-RelativePath -BasePath $resolvedRoot -Path $_.FullName
    })
    $splitFile = Join-Path -Path (Join-Path -Path $splitDirectory -ChildPath $key) -ChildPath "concat.txt"

    Write-ConcatFile -Path $splitFile -Files $groupFiles -BasePath $resolvedRoot

    $splitInfo = Get-Item -LiteralPath $splitFile
    $manifestFiles += [ordered]@{
        key = $key
        file = (Get-RelativePath -BasePath $splitDirectory -Path $splitFile)
        sourceFileCount = $groupFiles.Count
        bytes = [int64]$splitInfo.Length
    }
}

$manifest = [ordered]@{
    generatedAt = (Get-Date).ToString("o")
    root = $resolvedRoot
    splitMode = $SplitMode
    sourceFileCount = $files.Count
    files = $manifestFiles
    includeExtensions = $IncludeExtensions
    excludeExtensions = $ExcludeExtensions
    excludeDirectories = $ExcludeDirectories
}

$manifestPath = Join-Path -Path $splitDirectory -ChildPath "concat.manifest.json"
$manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding UTF8

$resolvedZip = [System.IO.Path]::ChangeExtension($resolvedOutput, ".zip")
if (-not $NoZip) {
    if (Test-Path -LiteralPath $resolvedZip) {
        Remove-Item -LiteralPath $resolvedZip -Force
    }

    Compress-Archive -Path (Join-Path -Path $splitDirectory -ChildPath "*") -DestinationPath $resolvedZip
}

Write-Host ("Wrote {0} split concat file(s) to {1}" -f $manifestFiles.Count, $splitDirectory)
Write-Host ("Packed {0} source file(s)" -f $files.Count)
Write-Host ("Split mode: {0}" -f $SplitMode)
Write-Host ("Manifest: {0}" -f $manifestPath)
if (-not $NoZip) {
    Write-Host ("Wrote ZIP archive to {0}" -f $resolvedZip)
}
Write-Host ("Included extensions: {0}" -f ($IncludeExtensions -join ", "))
Write-Host ("Excluded extensions: {0}" -f ($ExcludeExtensions -join ", "))
Write-Host ("Excluded directories: {0}" -f ($ExcludeDirectories -join ", "))
