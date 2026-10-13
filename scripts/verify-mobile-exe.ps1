param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$Version)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'release-pipeline-common.ps1')
$file = Get-Item -LiteralPath $Path
$bytes = [IO.File]::ReadAllBytes($file.FullName)
$pe = [BitConverter]::ToInt32($bytes, 0x3c)
if ([BitConverter]::ToUInt32($bytes, $pe) -ne 0x4550 -or
    [BitConverter]::ToUInt16($bytes, $pe + 4) -ne 0x8664 -or
    [BitConverter]::ToUInt16($bytes, $pe + 24) -ne 0x20b -or
    [BitConverter]::ToUInt16($bytes, $pe + 24 + 68) -ne 2) { throw 'Expected an x64 Windows GUI executable.' }
if ($file.VersionInfo.ProductVersion -ne $Version -or
    $file.VersionInfo.ProductName -ne 'Win-CodexBar + Mobile Connect') { throw 'EXE product name or version does not match the release.' }
$imports = @(Get-PeImportedDllNames -Path $file.FullName)
if (@(Get-DynamicCrtImports $imports).Count -gt 0 -or $imports -contains 'WebView2Loader.dll') {
    throw 'Standalone EXE must statically include the CRT and WebView2 loader.'
}
Write-Host "Verified $($file.Name): x64 GUI, $Version, static CRT and WebView2 loader."
