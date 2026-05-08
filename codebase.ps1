param(
    [string]$Root = ".",
    [string]$Output = "codebase.zip",
    [switch]$Split,
    [ValidateSet("domain", "extension")]
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
    [string[]]$ExcludeDirectories = @(
        ".amigo",
        ".git",
        ".idea",
        ".vscode",
        ".vs",
        ".cache",
        ".next",
        ".nuxt",
        ".svelte-kit",
        ".astro",
        ".turbo",
        ".parcel-cache",
        ".vite",
        "bin",
        "build",
        "coverage",
        "dist",
        "node_modules",
        "obj",
        "out",
        "target",
        "tmp",
        "temp"
    ),
    [string[]]$ExcludeFileNames = @(
        ".DS_Store",
        ".git",
        "Thumbs.db",
        "desktop.ini"
    ),
    [string[]]$ExcludeExtensions = @(
        ".7z",
        ".a",
        ".br",
        ".cache",
        ".dll",
        ".dylib",
        ".exe",
        ".gz",
        ".lib",
        ".log",
        ".map",
        ".o",
        ".obj",
        ".pdb",
        ".rlib",
        ".rmeta",
        ".so",
        ".tar",
        ".txt",
        ".tmp",
        ".wasm",
        ".zip"
    )
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

$ExcludeDirectories = $ExcludeDirectories |
    ForEach-Object { $_.Trim() } |
    Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
    Sort-Object -Unique

$IncludeExtensions = $IncludeExtensions |
    ForEach-Object { $_.Trim().ToLowerInvariant() } |
    Where-Object { -not [string]::IsNullOrWhiteSpace($_) } |
    Sort-Object -Unique

$ExcludeFileNames = $ExcludeFileNames |
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

function Test-ExcludedFile {
    param(
        [Parameter(Mandatory = $true)]
        [System.IO.FileInfo]$File,
        [Parameter(Mandatory = $true)]
        [string]$ResolvedOutput
    )

    if ($File.FullName -eq $ResolvedOutput) {
        return $true
    }

    if ($ExcludeFileNames -contains $File.Name) {
        return $true
    }

    $extension = [System.IO.Path]::GetExtension($File.Name).ToLowerInvariant()
    if ($IncludeExtensions -notcontains $extension) {
        return $true
    }

    return $ExcludeExtensions -contains $extension
}

function Get-CodebaseFiles {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Directory,
        [Parameter(Mandatory = $true)]
        [string]$ResolvedOutput
    )

    foreach ($entry in Get-ChildItem -LiteralPath $Directory -Force) {
        if (($entry.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
            continue
        }

        if ($entry.PSIsContainer) {
            if ($ExcludeDirectories -contains $entry.Name) {
                continue
            }

            Get-CodebaseFiles -Directory $entry.FullName -ResolvedOutput $ResolvedOutput
            continue
        }

        if (-not (Test-ExcludedFile -File $entry -ResolvedOutput $ResolvedOutput)) {
            $entry
        }
    }
}

function Get-CodebaseSplitKey {
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

function New-CodebaseZip {
    param(
        [Parameter(Mandatory = $true)]
        [string]$ZipPath,
        [Parameter(Mandatory = $true)]
        [array]$Files,
        [Parameter(Mandatory = $true)]
        [string]$BasePath
    )

    $zipDirectory = Split-Path -Path $ZipPath -Parent
    if (-not [string]::IsNullOrWhiteSpace($zipDirectory)) {
        New-Item -ItemType Directory -Path $zipDirectory -Force | Out-Null
    }

    if (Test-Path -LiteralPath $ZipPath) {
        Remove-Item -LiteralPath $ZipPath -Force
    }

    $archiveStream = [System.IO.File]::Open(
        $ZipPath,
        [System.IO.FileMode]::CreateNew,
        [System.IO.FileAccess]::ReadWrite,
        [System.IO.FileShare]::None
    )

    try {
        $archive = [System.IO.Compression.ZipArchive]::new(
            $archiveStream,
            [System.IO.Compression.ZipArchiveMode]::Create,
            $false
        )

        try {
            foreach ($file in $Files) {
                $relativePath = Get-RelativePath -BasePath $BasePath -Path $file.FullName
                [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile(
                    $archive,
                    $file.FullName,
                    $relativePath,
                    [System.IO.Compression.CompressionLevel]::Optimal
                ) | Out-Null
            }
        } finally {
            $archive.Dispose()
        }
    } finally {
        $archiveStream.Dispose()
    }
}

if (-not (Test-Path -LiteralPath $Root)) {
    throw "Root path does not exist: $Root"
}

$resolvedRoot = (Resolve-Path -LiteralPath $Root).Path
$resolvedOutput = if ([System.IO.Path]::IsPathRooted($Output)) {
    [System.IO.Path]::GetFullPath($Output)
} else {
    [System.IO.Path]::GetFullPath((Join-Path -Path $resolvedRoot -ChildPath $Output))
}

$files = Get-CodebaseFiles -Directory $resolvedRoot -ResolvedOutput $resolvedOutput |
    Sort-Object {
        Get-RelativePath -BasePath $resolvedRoot -Path $_.FullName
    }

if (-not $Split) {
    if ((Test-Path -LiteralPath $resolvedOutput) -and (Get-Item -LiteralPath $resolvedOutput).PSIsContainer) {
        throw "Output path is a directory; choose a .zip file or use -Split: $resolvedOutput"
    }

    New-CodebaseZip -ZipPath $resolvedOutput -Files $files -BasePath $resolvedRoot

    $archiveInfo = Get-Item -LiteralPath $resolvedOutput

    Write-Host ("Wrote {0} file(s) to {1}" -f $files.Count, $resolvedOutput)
    Write-Host ("Archive size: {0:N2} MB" -f ($archiveInfo.Length / 1MB))
    Write-Host ("Included extensions: {0}" -f ($IncludeExtensions -join ", "))
    Write-Host ("Excluded directories: {0}" -f ($ExcludeDirectories -join ", "))
    Write-Host ("Excluded extensions: {0}" -f ($ExcludeExtensions -join ", "))
    return
}

$splitDirectory = $resolvedOutput
if ([System.IO.Path]::GetExtension($resolvedOutput).Equals(".zip", [System.StringComparison]::OrdinalIgnoreCase)) {
    $splitDirectory = Join-Path -Path (Split-Path -Path $resolvedOutput -Parent) -ChildPath ([System.IO.Path]::GetFileNameWithoutExtension($resolvedOutput))
}

if ((Test-Path -LiteralPath $splitDirectory) -and -not (Get-Item -LiteralPath $splitDirectory).PSIsContainer) {
    throw "Split output path is a file; choose a directory: $splitDirectory"
}

New-Item -ItemType Directory -Path $splitDirectory -Force | Out-Null

$groups = @{}
foreach ($file in $files) {
    $relativePath = Get-RelativePath -BasePath $resolvedRoot -Path $file.FullName
    $key = Get-CodebaseSplitKey -RelativePath $relativePath -File $file

    if (-not $groups.ContainsKey($key)) {
        $groups[$key] = [System.Collections.Generic.List[System.IO.FileInfo]]::new()
    }

    $groups[$key].Add($file)
}

$manifestArchives = @()
foreach ($key in ($groups.Keys | Sort-Object)) {
    $groupFiles = @($groups[$key] | Sort-Object {
        Get-RelativePath -BasePath $resolvedRoot -Path $_.FullName
    })
    $zipPath = Join-Path -Path $splitDirectory -ChildPath ("{0}.zip" -f $key)

    New-CodebaseZip -ZipPath $zipPath -Files $groupFiles -BasePath $resolvedRoot

    $archiveInfo = Get-Item -LiteralPath $zipPath
    $totalBytes = ($groupFiles | Measure-Object -Property Length -Sum).Sum
    if ($null -eq $totalBytes) {
        $totalBytes = 0
    }

    $manifestArchives += [ordered]@{
        key = $key
        archive = (Get-RelativePath -BasePath $splitDirectory -Path $zipPath)
        fileCount = $groupFiles.Count
        totalBytes = [int64]$totalBytes
        archiveBytes = [int64]$archiveInfo.Length
        archiveMb = [math]::Round($archiveInfo.Length / 1MB, 2)
    }
}

$allBytes = ($files | Measure-Object -Property Length -Sum).Sum
if ($null -eq $allBytes) {
    $allBytes = 0
}

$manifest = [ordered]@{
    generatedAt = (Get-Date).ToString("o")
    root = $resolvedRoot
    splitMode = $SplitMode
    fileCount = $files.Count
    totalBytes = [int64]$allBytes
    archives = $manifestArchives
    includeExtensions = $IncludeExtensions
    excludeDirectories = $ExcludeDirectories
    excludeExtensions = $ExcludeExtensions
}

$manifestPath = Join-Path -Path $splitDirectory -ChildPath "codebase.manifest.json"
$manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding UTF8

$totalArchiveBytes = 0
foreach ($entry in $manifestArchives) {
    $totalArchiveBytes += [int64]$entry.archiveBytes
}

Write-Host ("Wrote {0} split archive(s) to {1}" -f $manifestArchives.Count, $splitDirectory)
Write-Host ("Packed {0} file(s)" -f $files.Count)
Write-Host ("Archive size: {0:N2} MB" -f ($totalArchiveBytes / 1MB))
Write-Host ("Split mode: {0}" -f $SplitMode)
Write-Host ("Manifest: {0}" -f $manifestPath)
