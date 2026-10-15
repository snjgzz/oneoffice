# Build an unsigned NSIS installer around target\release\oneoffice.exe.
# Usage: packaging\windows\package.ps1 v0.1.0
$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

$root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$label = $args[0]
if (-not $label) {
    throw "usage: packaging\windows\package.ps1 v0.1.0"
}
$version = $label -replace '^v', ''
if ($version -notmatch '^[0-9A-Za-z._-]+$') {
    throw "refusing to use version: $version"
}
$vi = "0.0.0.0"
if ($version -match '^\d+\.\d+\.\d+$') {
    $vi = "$version.0"
}

$exe = Join-Path $root "target\release\oneoffice.exe"
if (-not (Test-Path $exe)) {
    throw "missing $exe (build with: cargo build --release -p oneoffice)"
}

$dist = Join-Path $root "dist"
New-Item -ItemType Directory -Force -Path $dist | Out-Null
$out = Join-Path $dist "OneOffice-$version-windows-x86_64-setup.exe"

$makensis = $null
$cmd = Get-Command makensis -ErrorAction SilentlyContinue
if ($cmd) {
    $makensis = $cmd.Source
} else {
    $candidate = Join-Path ${env:ProgramFiles(x86)} "NSIS\makensis.exe"
    if (Test-Path $candidate) {
        $makensis = $candidate
    }
}
if (-not $makensis) {
    throw "makensis not found. Install NSIS, then rerun this script."
}

& $makensis "/DVERSION=$version" "/DVI_VERSION=$vi" "/DOUTFILE=$out" "/DSRC=$exe" (Join-Path $PSScriptRoot "oneoffice.nsi")
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}
Write-Host "wrote $out"
