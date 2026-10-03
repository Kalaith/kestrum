# Kestrum's deterministic title, atlas, and menu captures through the shared pool.
param(
    [string[]]$Scenes = @("title", "gameplay", "zoomed", "menu", "settings", "help", "confirm_new", "save_error"),
    [int]$Frames = 12,
    [int]$WindowWidth = 1920,
    [int]$WindowHeight = 1080,
    [switch]$Fullscreen,
    [switch]$SkipBuild,
    [switch]$ShowOutput
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"
& $shared -GameDir $gameDir -Prefix "KESTRUM" -Scenes $Scenes -Frames $Frames `
    -WindowWidth $WindowWidth -WindowHeight $WindowHeight -OutputDir "docs\verification" `
    -SkipBuild:$SkipBuild -Fullscreen:$Fullscreen -ShowOutput:$ShowOutput
if (-not $?) { exit 1 }
