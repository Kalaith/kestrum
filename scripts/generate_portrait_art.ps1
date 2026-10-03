param(
    [switch]$ValidateOnly
)

$ErrorActionPreference = 'Stop'
if (-not $ValidateOnly) {
    & (Join-Path $PSScriptRoot 'export_illustrated_portraits.ps1')
    return
}

Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Text.Json

$projectRoot = Split-Path -Parent $PSScriptRoot
$sourcePaths = @(
    (Join-Path $projectRoot 'assets/art-source/portraits/PortraitArtSource.cs'),
    (Join-Path $projectRoot 'assets/art-source/portraits/PortraitArtReviewSheets.cs')
)
$catalogPath = Join-Path $projectRoot 'assets/data/portrait_catalog.json'
$references = Get-ChildItem -LiteralPath (Join-Path $PSHOME 'ref') -Filter '*.dll' |
    Select-Object -ExpandProperty FullName
$references += [System.Drawing.Bitmap].Assembly.Location
$references += Join-Path $PSHOME 'System.Private.Windows.Core.dll'
$references += Join-Path $PSHOME 'System.Private.Windows.GdiPlus.dll'
Add-Type -ReferencedAssemblies $references -Path $sourcePaths

$catalogJson = Get-Content -Raw $catalogPath
[PortraitArtSource]::Run($projectRoot, $catalogJson, $ValidateOnly.IsPresent)
