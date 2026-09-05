param([string]$NsisPath)
$ErrorActionPreference = 'Stop'

$cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
$cargoExe = if ($cargoCommand) { $cargoCommand.Source } else { Join-Path $env:USERPROFILE '.cargo/bin/cargo.exe' }
if (-not (Test-Path -LiteralPath $cargoExe)) { throw 'Rust MSVC toolchain (cargo) is required.' }
if (-not $NsisPath) {
    $nsisCommand = Get-Command makensis -ErrorAction SilentlyContinue
    $NsisPath = if ($nsisCommand) { $nsisCommand.Source } else { Join-Path $env:LOCALAPPDATA 'tauri/NSIS/makensis.exe' }
}
if (-not (Test-Path -LiteralPath $NsisPath)) { throw 'NSIS is required. Pass -NsisPath <makensis.exe>.' }
$NsisPath = (Resolve-Path -LiteralPath $NsisPath).Path

Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    & $cargoExe test --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'Core tests failed.' }
    & $cargoExe build --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'Core build failed.' }
    & $cargoExe test --manifest-path desktop/Cargo.toml --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'Native shell tests failed.' }
    & $cargoExe build --manifest-path desktop/Cargo.toml --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'Native shell build failed.' }
    New-Item -ItemType Directory -Path 'desktop/target/release/bundle/nsis' -Force | Out-Null
    & $NsisPath /V2 (Join-Path $PSScriptRoot 'installer.nsi')
    if ($LASTEXITCODE -ne 0) { throw 'Installer build failed.' }
} finally {
    Pop-Location
}
