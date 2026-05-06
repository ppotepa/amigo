function Get-AmigoFile {
  param(
    [Parameter(Mandatory=$true)][string]$RepoRoot,
    [Parameter(Mandatory=$true)][string]$RelativePath
  )
  $path = Join-Path $RepoRoot $RelativePath
  if (!(Test-Path $path)) {
    throw "Missing file: $RelativePath"
  }
  return $path
}

function Read-AmigoText {
  param([Parameter(Mandatory=$true)][string]$Path)
  return [System.IO.File]::ReadAllText($Path, [System.Text.Encoding]::UTF8)
}

function Write-AmigoText {
  param(
    [Parameter(Mandatory=$true)][string]$Path,
    [Parameter(Mandatory=$true)][string]$Content
  )
  $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
  [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

function Ensure-Directory {
  param([Parameter(Mandatory=$true)][string]$Path)
  if (!(Test-Path $Path)) {
    New-Item -ItemType Directory -Path $Path | Out-Null
  }
}

function Write-NewFileIfChanged {
  param(
    [Parameter(Mandatory=$true)][string]$Path,
    [Parameter(Mandatory=$true)][string]$Content
  )
  $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
  if (Test-Path $Path) {
    $current = [System.IO.File]::ReadAllText($Path, [System.Text.Encoding]::UTF8)
    if ($current -eq $Content) {
      Write-Host "SKIP unchanged $Path"
      return
    }
  }
  Ensure-Directory (Split-Path -Parent $Path)
  [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
  Write-Host "OK wrote $Path"
}

function Replace-Once {
  param(
    [Parameter(Mandatory=$true)][string]$Text,
    [Parameter(Mandatory=$true)][string]$Pattern,
    [Parameter(Mandatory=$true)][string]$Replacement,
    [Parameter(Mandatory=$true)][string]$Label
  )
  $count = [regex]::Matches($Text, $Pattern, [System.Text.RegularExpressions.RegexOptions]::Singleline).Count
  if ($count -eq 0) {
    throw "Pattern not found for $Label"
  }
  if ($count -gt 1) {
    throw "Pattern matched $count times for $Label; refusing ambiguous replace."
  }
  return [regex]::Replace($Text, $Pattern, $Replacement, [System.Text.RegularExpressions.RegexOptions]::Singleline)
}

function Add-Before-Once {
  param(
    [Parameter(Mandatory=$true)][string]$Text,
    [Parameter(Mandatory=$true)][string]$Marker,
    [Parameter(Mandatory=$true)][string]$Insert,
    [Parameter(Mandatory=$true)][string]$Label
  )
  if (!$Text.Contains($Marker)) {
    throw "Marker not found for $Label"
  }
  return $Text.Replace($Marker, $Insert + $Marker)
}

function Add-After-Once {
  param(
    [Parameter(Mandatory=$true)][string]$Text,
    [Parameter(Mandatory=$true)][string]$Marker,
    [Parameter(Mandatory=$true)][string]$Insert,
    [Parameter(Mandatory=$true)][string]$Label
  )
  if (!$Text.Contains($Marker)) {
    throw "Marker not found for $Label"
  }
  return $Text.Replace($Marker, $Marker + $Insert)
}
