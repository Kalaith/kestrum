<#
.SYNOPSIS
    Headless screenshot harness for the game template.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it through the env-var capture hook
    (GAME_TEMPLATE_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. Named scenes reset runtime state before each capture.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @("gameplay", "paused", "scrolled", "zoomed"),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$workspace = Split-Path -Parent $gameDir
if (-not (Test-Path (Join-Path $workspace "macroquad-toolkit"))) {
    $workspace = Split-Path -Parent $workspace
}
$shared = Join-Path $workspace "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Prefix "GAME_TEMPLATE" -Scenes $Scenes -Frames $Frames -OutputDir $OutputDir -SkipBuild:$SkipBuild
