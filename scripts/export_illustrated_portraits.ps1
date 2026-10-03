param([switch]$ParityOnly, [switch]$ReviewOnly)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Text.Json
$projectRoot = Split-Path -Parent $PSScriptRoot
$references = Get-ChildItem -LiteralPath (Join-Path $PSHOME 'ref') -Filter '*.dll' |
    Select-Object -ExpandProperty FullName
$references += [System.Drawing.Bitmap].Assembly.Location
$references += Join-Path $PSHOME 'System.Private.Windows.Core.dll'
$references += Join-Path $PSHOME 'System.Private.Windows.GdiPlus.dll'
Add-Type -ReferencedAssemblies $references -Path (Join-Path $projectRoot 'assets/art-source/portraits/IllustratedPortraitExporter.cs')
Add-Type -ReferencedAssemblies $references -Path @(
    (Join-Path $projectRoot 'assets/art-source/portraits/PortraitArtSource.cs'),
    (Join-Path $projectRoot 'assets/art-source/portraits/PortraitArtReviewSheets.cs')
)
$catalogJson = Get-Content -Raw (Join-Path $projectRoot 'assets/data/portrait_catalog.json')
if ($ParityOnly) {
    [PortraitArtSource]::WriteParityFixtures($projectRoot, $catalogJson)
    return
}
if (-not $ReviewOnly) {
    $catalogRevision = ($catalogJson | ConvertFrom-Json).catalog_revision
    [IllustratedPortraitExporter]::Run($projectRoot, $catalogRevision)
}
[PortraitArtSource]::ReviewExistingExports($projectRoot, $catalogJson)
