[CmdletBinding()]
param(
    [string]$SourceRoot = "extras/platformer-kit",
    [string]$TargetRoot = "mods/playground-sidescroller"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

function New-ResolvedPath([string]$Path) {
    return [System.IO.Path]::GetFullPath((Join-Path (Get-Location) $Path))
}

function New-Canvas([int]$Width, [int]$Height) {
    return [System.Drawing.Bitmap]::new(
        $Width,
        $Height,
        [System.Drawing.Imaging.PixelFormat]::Format32bppArgb
    )
}

function New-Graphics([System.Drawing.Bitmap]$Bitmap) {
    $graphics = [System.Drawing.Graphics]::FromImage($Bitmap)
    $graphics.Clear([System.Drawing.Color]::Transparent)
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::NearestNeighbor
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::Half
    $graphics.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceOver
    $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighSpeed
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::None
    return $graphics
}

function Save-Png([System.Drawing.Bitmap]$Bitmap, [string]$Path) {
    $directory = Split-Path -Parent $Path
    if (-not (Test-Path $directory)) {
        New-Item -ItemType Directory -Path $directory | Out-Null
    }
    $Bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
}

function Use-Image([string]$Path, [scriptblock]$Block) {
    $image = [System.Drawing.Image]::FromFile($Path)
    try {
        & $Block $image
    }
    finally {
        $image.Dispose()
    }
}

function Build-SpriteSheet([string[]]$SourceFiles, [string]$TargetFile, [int]$FrameWidth, [int]$FrameHeight) {
    $sheet = New-Canvas ($FrameWidth * $SourceFiles.Length) $FrameHeight
    $graphics = New-Graphics $sheet
    try {
        for ($index = 0; $index -lt $SourceFiles.Length; $index++) {
            $sourceFile = $SourceFiles[$index]
            Use-Image $sourceFile {
                param($image)
                $graphics.DrawImage(
                    $image,
                    [System.Drawing.Rectangle]::new($index * $FrameWidth, 0, $FrameWidth, $FrameHeight),
                    [System.Drawing.Rectangle]::new(0, 0, $image.Width, $image.Height),
                    [System.Drawing.GraphicsUnit]::Pixel
                )
            }
        }
        Save-Png $sheet $TargetFile
    }
    finally {
        $graphics.Dispose()
        $sheet.Dispose()
    }
}

function Copy-Image([string]$SourceFile, [string]$TargetFile) {
    $directory = Split-Path -Parent $TargetFile
    if (-not (Test-Path $directory)) {
        New-Item -ItemType Directory -Path $directory | Out-Null
    }
    Copy-Item -LiteralPath $SourceFile -Destination $TargetFile -Force
}

function Build-TiledBackground([string]$SourceFile, [string]$TargetFile, [int]$Width, [int]$Height) {
    Use-Image $SourceFile {
        param($image)
        $canvas = New-Canvas $Width $Height
        $graphics = New-Graphics $canvas
        try {
            for ($y = 0; $y -lt $Height; $y += $image.Height) {
                for ($x = 0; $x -lt $Width; $x += $image.Width) {
                    $graphics.DrawImage(
                        $image,
                        [System.Drawing.Rectangle]::new($x, $y, $image.Width, $image.Height),
                        [System.Drawing.Rectangle]::new(0, 0, $image.Width, $image.Height),
                        [System.Drawing.GraphicsUnit]::Pixel
                    )
                }
            }
            Save-Png $canvas $TargetFile
        }
        finally {
            $graphics.Dispose()
            $canvas.Dispose()
        }
    }
}

function Read-TextureAtlas([string]$XmlPath) {
    [xml]$document = Get-Content -LiteralPath $XmlPath
    $atlasDirectory = Split-Path -Parent $XmlPath
    $imagePath = Join-Path $atlasDirectory $document.TextureAtlas.imagePath
    $subTextures = @{}
    foreach ($node in $document.TextureAtlas.SubTexture) {
        $subTextures[$node.name] = @{
            X = [int]$node.x
            Y = [int]$node.y
            Width = [int]$node.width
            Height = [int]$node.height
        }
    }
    return @{
        ImagePath = $imagePath
        SubTextures = $subTextures
    }
}

function Copy-SubTexture(
    [System.Drawing.Bitmap]$AtlasBitmap,
    [hashtable]$Region,
    [bool]$TransparentWhite = $false
) {
    $bitmap = New-Canvas $Region.Width $Region.Height
    $graphics = New-Graphics $bitmap
    try {
        $graphics.DrawImage(
            $AtlasBitmap,
            [System.Drawing.Rectangle]::new(0, 0, $Region.Width, $Region.Height),
            [System.Drawing.Rectangle]::new($Region.X, $Region.Y, $Region.Width, $Region.Height),
            [System.Drawing.GraphicsUnit]::Pixel
        )
    }
    finally {
        $graphics.Dispose()
    }

    if ($TransparentWhite) {
        for ($y = 0; $y -lt $bitmap.Height; $y++) {
            for ($x = 0; $x -lt $bitmap.Width; $x++) {
                $pixel = $bitmap.GetPixel($x, $y)
                if ($pixel.A -gt 0 -and $pixel.R -ge 250 -and $pixel.G -ge 250 -and $pixel.B -ge 250) {
                    $bitmap.SetPixel($x, $y, [System.Drawing.Color]::FromArgb(0, $pixel.R, $pixel.G, $pixel.B))
                }
            }
        }
    }

    return $bitmap
}

function Build-TiledAtlasBackground(
    [string]$AtlasXmlPath,
    [string]$BaseLayerName,
    [string[]]$OverlayLayerNames,
    [string]$TargetFile,
    [int]$Width,
    [int]$Height
) {
    $atlas = Read-TextureAtlas $AtlasXmlPath
    $atlasBitmap = [System.Drawing.Bitmap]::FromFile($atlas.ImagePath)
    try {
        $baseTile = $null
        if (-not [string]::IsNullOrWhiteSpace($BaseLayerName)) {
            $baseRegion = $atlas.SubTextures[$BaseLayerName]
            if ($null -eq $baseRegion) {
                throw "Missing background subtexture `$BaseLayerName in $AtlasXmlPath"
            }
            $baseTile = Copy-SubTexture $atlasBitmap $baseRegion
        }
        $overlayTiles = @()
        try {
            foreach ($layerName in $OverlayLayerNames) {
                $overlayRegion = $atlas.SubTextures[$layerName]
                if ($null -eq $overlayRegion) {
                    throw "Missing background subtexture `$layerName in $AtlasXmlPath"
                }
                $overlayTiles += ,(Copy-SubTexture $atlasBitmap $overlayRegion $true)
            }

            $canvas = New-Canvas $Width $Height
            $graphics = New-Graphics $canvas
            try {
                if ($null -ne $baseTile) {
                    for ($y = 0; $y -lt $Height; $y += $baseTile.Height) {
                        for ($x = 0; $x -lt $Width; $x += $baseTile.Width) {
                            $graphics.DrawImage(
                                $baseTile,
                                [System.Drawing.Rectangle]::new($x, $y, $baseTile.Width, $baseTile.Height),
                                [System.Drawing.Rectangle]::new(0, 0, $baseTile.Width, $baseTile.Height),
                                [System.Drawing.GraphicsUnit]::Pixel
                            )
                        }
                    }
                }

                foreach ($overlayTile in $overlayTiles) {
                    for ($y = 0; $y -lt $Height; $y += $overlayTile.Height) {
                        for ($x = 0; $x -lt $Width; $x += $overlayTile.Width) {
                            $graphics.DrawImage(
                                $overlayTile,
                                [System.Drawing.Rectangle]::new($x, $y, $overlayTile.Width, $overlayTile.Height),
                                [System.Drawing.Rectangle]::new(0, 0, $overlayTile.Width, $overlayTile.Height),
                                [System.Drawing.GraphicsUnit]::Pixel
                            )
                        }
                    }
                }

                Save-Png $canvas $TargetFile
            }
            finally {
                $graphics.Dispose()
                $canvas.Dispose()
            }
        }
        finally {
            foreach ($overlayTile in $overlayTiles) {
                $overlayTile.Dispose()
            }
            if ($null -ne $baseTile) {
                $baseTile.Dispose()
            }
        }
    }
    finally {
        $atlasBitmap.Dispose()
    }
}

function Build-TileAtlas([hashtable[]]$Tiles, [string]$TargetFile, [int]$TileSize) {
    $atlas = New-Canvas ($TileSize * $Tiles.Length) $TileSize
    $graphics = New-Graphics $atlas
    try {
        for ($index = 0; $index -lt $Tiles.Length; $index++) {
            $tile = $Tiles[$index]
            Use-Image $tile.Path {
                param($image)
                $graphics.DrawImage(
                    $image,
                    [System.Drawing.Rectangle]::new($index * $TileSize, 0, $TileSize, $TileSize),
                    [System.Drawing.Rectangle]::new(0, 0, $image.Width, $image.Height),
                    [System.Drawing.GraphicsUnit]::Pixel
                )
            }
        }
        Save-Png $atlas $TargetFile
    }
    finally {
        $graphics.Dispose()
        $atlas.Dispose()
    }
}

$sourceRoot = New-ResolvedPath $SourceRoot
$targetRoot = New-ResolvedPath $TargetRoot

$playerFrames = @(
    (Join-Path $sourceRoot "Sprites/Characters/Default/character_beige_idle.png"),
    (Join-Path $sourceRoot "Sprites/Characters/Default/character_beige_walk_a.png"),
    (Join-Path $sourceRoot "Sprites/Characters/Default/character_beige_walk_b.png"),
    (Join-Path $sourceRoot "Sprites/Characters/Default/character_beige_jump.png")
)
$coinFrames = @(
    (Join-Path $sourceRoot "Sprites/Tiles/Default/coin_gold.png"),
    (Join-Path $sourceRoot "Sprites/Tiles/Default/coin_gold_side.png"),
    (Join-Path $sourceRoot "Sprites/Tiles/Default/coin_gold.png"),
    (Join-Path $sourceRoot "Sprites/Tiles/Default/coin_gold_side.png")
)
$finishSource = Join-Path $sourceRoot "Sprites/Tiles/Default/flag_green_b.png"
$tileSources = @(
    @{ Name = "ground_single"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block.png") },
    @{ Name = "ground_left_cap"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_horizontal_left.png") },
    @{ Name = "ground_middle"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_horizontal_middle.png") },
    @{ Name = "ground_right_cap"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_horizontal_right.png") },
    @{ Name = "ground_side_left"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_left.png") },
    @{ Name = "ground_side_right"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_right.png") },
    @{ Name = "ground_center"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_center.png") },
    @{ Name = "ground_top_cap"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_top.png") },
    @{ Name = "ground_vertical_middle"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_vertical_middle.png") },
    @{ Name = "ground_bottom_cap"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_bottom.png") },
    @{ Name = "ground_outer_corner_top_left"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_top_left.png") },
    @{ Name = "ground_outer_corner_top_right"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_top_right.png") },
    @{ Name = "ground_outer_corner_bottom_left"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_bottom_left.png") },
    @{ Name = "ground_outer_corner_bottom_right"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_bottom_right.png") },
    @{ Name = "ground_inner_corner_top_left"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_center.png") },
    @{ Name = "ground_inner_corner_top_right"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_center.png") },
    @{ Name = "ground_inner_corner_bottom_left"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_center.png") },
    @{ Name = "ground_inner_corner_bottom_right"; Path = (Join-Path $sourceRoot "Sprites/Tiles/Default/terrain_grass_block_center.png") }
)

Build-SpriteSheet $playerFrames (Join-Path $targetRoot "textures/player.png") 128 128
Build-SpriteSheet $coinFrames (Join-Path $targetRoot "textures/coin.png") 64 64
Copy-Image $finishSource (Join-Path $targetRoot "textures/finish.png")
Build-TileAtlas $tileSources (Join-Path $targetRoot "tilesets/platformer.png") 64

Write-Output "Generated playground-sidescroller gameplay assets from $sourceRoot"
Write-Output "Background layers are authored separately and are not regenerated by this script."
