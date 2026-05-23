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
  "scripts/perf-report.mjs",
  "src/app/cameraControls.js",
  "src/main.js",
  "src/mesh/fbxAdapter.js",
  "src/mesh/modelSources.js",
  "src/mesh/objParser.js",
  "src/mesh/meshRuntime.js",
  "src/mesh/objParseWorker.js",
  "src/paint/paintRegions.js",
  "src/render/perfStats.js",
  "src/render/renderCache.js",
  "src/render/frameProtocol.js",
  "src/render/frameWorker.js",
  "src/state/controlSchema.js",
  "src/state/defaultState.js"
)

function Normalize-RelativePath([string]$path) {
  $relative = [System.IO.Path]::GetRelativePath($root, $path)
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
[void]$builder.AppendLine("# Excludes dist, node_modules, logs, generated outputs, package metadata, old demos, and large model assets.")
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
