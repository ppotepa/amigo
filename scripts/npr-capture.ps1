param(
    [string]$Preset = "toriyama_1989_black_mass",
    [string]$Model = "soldier",
    [string]$Output = "images/current.png",
    [string]$Entity = "playground-npr-model-1-soldier",
    [string]$Strategy = "",
    [string]$DebugMode = "",
    [string]$Scale = "",
    [string]$Translate = "",
    [string]$RotationDeg = "",
    [switch]$Trace,
    [switch]$ShowHud,
    [int]$Width = 1280,
    [int]$Height = 720,
    [int]$Warmup = 3,
    [int]$Settle = 1
)

$ErrorActionPreference = "Stop"

if ($Trace) {
    $env:AMIGO_NPR_GPU_TRACE = "1"
    $env:AMIGO_NPR_GPU_TRACE_CLEAR = "0"
    $env:AMIGO_NPR_GPU_TRACE_COLOR = "0"
} else {
    $env:AMIGO_NPR_GPU_TRACE = "0"
    Remove-Item Env:AMIGO_NPR_GPU_TRACE_CLEAR -ErrorAction SilentlyContinue
    Remove-Item Env:AMIGO_NPR_GPU_TRACE_COLOR -ErrorAction SilentlyContinue
}

$args = @(
    "run", "-p", "amigo-app", "--",
    "--preview-capture",
    "--mod=playground-npr",
    "--scene=comic-lines",
    "--entity=$Entity",
    "--model=$Model",
    "--preset=$Preset",
    "--output=$Output",
    "--width=$Width",
    "--height=$Height",
    "--warmup=$Warmup",
    "--settle=$Settle"
)

if ($Strategy -ne "") {
    $args += "--strategy=$Strategy"
}

if ($DebugMode -ne "") {
    $args += "--debug=$DebugMode"
}

if ($Scale -ne "") {
    $args += "--scale=$Scale"
}

if ($Translate -ne "") {
    $args += "--translate=$Translate"
}

if ($RotationDeg -ne "") {
    $args += "--rotation-deg=$RotationDeg"
}

if ($ShowHud) {
    $args += "--show-hud"
}

cargo @args
