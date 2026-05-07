param(
    [string]$Root = ".",
    [string]$Output = "codebase.zip",
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

if (-not (Test-Path -LiteralPath $Root)) {
    throw "Root path does not exist: $Root"
}

$resolvedRoot = (Resolve-Path -LiteralPath $Root).Path
$resolvedOutput = if ([System.IO.Path]::IsPathRooted($Output)) {
    [System.IO.Path]::GetFullPath($Output)
} else {
    [System.IO.Path]::GetFullPath((Join-Path -Path $resolvedRoot -ChildPath $Output))
}

$outputDirectory = Split-Path -Path $resolvedOutput -Parent
if (-not [string]::IsNullOrWhiteSpace($outputDirectory)) {
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
}

if (Test-Path -LiteralPath $resolvedOutput) {
    Remove-Item -LiteralPath $resolvedOutput -Force
}

$files = Get-CodebaseFiles -Directory $resolvedRoot -ResolvedOutput $resolvedOutput |
    Sort-Object {
        Get-RelativePath -BasePath $resolvedRoot -Path $_.FullName
    }

$archiveStream = [System.IO.File]::Open(
    $resolvedOutput,
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
        foreach ($file in $files) {
            $relativePath = Get-RelativePath -BasePath $resolvedRoot -Path $file.FullName
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

$archiveInfo = Get-Item -LiteralPath $resolvedOutput

Write-Host ("Wrote {0} file(s) to {1}" -f $files.Count, $resolvedOutput)
Write-Host ("Archive size: {0:N2} MB" -f ($archiveInfo.Length / 1MB))
Write-Host ("Included extensions: {0}" -f ($IncludeExtensions -join ", "))
Write-Host ("Excluded directories: {0}" -f ($ExcludeDirectories -join ", "))
Write-Host ("Excluded extensions: {0}" -f ($ExcludeExtensions -join ", "))
