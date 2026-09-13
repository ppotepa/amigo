param(
  [string]$OutputName = "concat-written-files.txt",
  [string]$ZipName = "concat-written-files.zip",
  [switch]$NoZip
)

$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$outputPath = Join-Path $root $OutputName
$zipPath = Join-Path $root $ZipName

$projectFiles = @(
  "concat.ps1",
  "strokes.html",
  "package.json",
  "scripts/perf-report.mjs",
  "scripts/bake-walking-amc.mjs",
  "src/app/cameraControls.js",
  "src/main.js",
  "src/mesh/fbxAdapter.js",
  "src/mesh/fbxClipBake.js",
  "src/mesh/modelSources.js",
  "src/mesh/objParser.js",
  "src/mesh/meshRuntime.js",
  "src/mesh/objParseWorker.js",
  "src/paint/paintRegions.js",
  "src/scene/bounds.js",
  "src/scene/scenePartition.js",
  "src/render/projectionContext.js",
  "src/render/visibilitySelection.js",
  "src/render/detailPolicy.js",
  "src/render/renderSelection.js",
  "src/render/perfStats.js",
  "src/render/renderCache.js",
  "src/render/frameProtocol.js",
  "src/render/frameWorker.js",
  "src/state/controlSchema.js",
  "src/state/defaultState.js",
  "src/state/stylePresets.js"
)

$rustProjectFiles = @(
  "rust-impl/.gitignore",
  "rust-impl/Cargo.toml",
  "rust-impl/Cargo.lock",
  "rust-impl/README.md"
)

$rustSourceRoot = Join-Path $root "rust-impl/src"
if (Test-Path -LiteralPath $rustSourceRoot -PathType Container) {
  $rustProjectFiles += Get-ChildItem -LiteralPath $rustSourceRoot -Recurse -File |
    Where-Object {
      $_.FullName -notmatch "\\target\\" -and
      $_.Extension -in @(".rs", ".wgsl")
    } |
    Sort-Object FullName |
    ForEach-Object {
      $full = $_.FullName
      $rootFull = [System.IO.Path]::GetFullPath($root).TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
      $relative = $full.Substring($rootFull.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
      $relative -replace "\\", "/"
    }
}

$projectFiles += $rustProjectFiles

function Normalize-RelativePath([string]$path) {
  $rootFull = [System.IO.Path]::GetFullPath($root).TrimEnd([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
  $pathFull = [System.IO.Path]::GetFullPath($path)
  if ($pathFull.StartsWith($rootFull, [System.StringComparison]::OrdinalIgnoreCase)) {
    $relative = $pathFull.Substring($rootFull.Length).TrimStart([System.IO.Path]::DirectorySeparatorChar, [System.IO.Path]::AltDirectorySeparatorChar)
  } else {
    $relative = $pathFull
  }
  return $relative -replace "\\", "/"
}

function Count-Lines([string]$text) {
  if ($text.Length -eq 0) {
    return 0
  }

  $newlineCount = [System.Text.RegularExpressions.Regex]::Matches($text, "`n").Count
  if ($text.EndsWith("`n")) {
    return $newlineCount
  }

  return $newlineCount + 1
}

$files = foreach ($relative in $projectFiles) {
  $path = Join-Path $root ($relative -replace "/", [System.IO.Path]::DirectorySeparatorChar)
  if (Test-Path -LiteralPath $path -PathType Leaf) {
    Get-Item -LiteralPath $path
  }
}

$builder = [System.Text.StringBuilder]::new()
[void]$builder.AppendLine("# char-3d concat")
[void]$builder.AppendLine("# Scope: tools/char-3d mini-project files authored/changed during this work.")
[void]$builder.AppendLine("# Excludes dist, node_modules, logs, generated outputs, old demos, and large model assets.")
[void]$builder.AppendLine("# Generated: " + (Get-Date -Format "yyyy-MM-dd HH:mm:ss zzz"))
[void]$builder.AppendLine("# Root: " + $root)
[void]$builder.AppendLine("# File count: " + $files.Count)
[void]$builder.AppendLine("")

foreach ($file in $files) {
  $relative = Normalize-RelativePath $file.FullName
  $raw = Get-Content -LiteralPath $file.FullName -Raw
  $content = $raw -replace "`r`n", "`n"
  $lineCount = Count-Lines $content

  [void]$builder.AppendLine("================================================================================")
  [void]$builder.AppendLine("FILE: " + $file.Name)
  [void]$builder.AppendLine("PATH: " + $relative)
  [void]$builder.AppendLine("ABS_PATH: " + $file.FullName)
  [void]$builder.AppendLine("LINES: " + $lineCount)
  [void]$builder.AppendLine("CONTENT:")
  [void]$builder.AppendLine("--------------------------------------------------------------------------------")
  [void]$builder.Append($content)
  if (-not $content.EndsWith("`n")) {
    [void]$builder.AppendLine()
  }
  [void]$builder.AppendLine("")
}

[System.IO.File]::WriteAllText($outputPath, $builder.ToString(), [System.Text.UTF8Encoding]::new($false))

if (-not $NoZip) {
  if (Test-Path -LiteralPath $zipPath) {
    Remove-Item -LiteralPath $zipPath
  }
  Compress-Archive -LiteralPath $outputPath -DestinationPath $zipPath -Force
}

Write-Host "Wrote: $outputPath"
if (-not $NoZip) {
  Write-Host "Wrote: $zipPath"
}
Write-Host "Files: $($files.Count)"
