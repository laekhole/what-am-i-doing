param([string]$NsisPath, [switch]$Portable, [string]$Tag)
$ErrorActionPreference = 'Stop'
if ($Portable -and $Tag -notmatch '^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z]+([.-][0-9A-Za-z]+)*)?$') {
    throw 'Portable releases require a version tag such as v1.0.0 or v1.0.0-rc.1.'
}

$cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
$cargoExe = if ($cargoCommand) { $cargoCommand.Source } else { Join-Path $env:USERPROFILE '.cargo/bin/cargo.exe' }
if (-not (Test-Path -LiteralPath $cargoExe)) { throw 'Rust MSVC toolchain (cargo) is required.' }
if (-not $Portable) {
    if (-not $NsisPath) {
        $nsisCommand = Get-Command makensis -ErrorAction SilentlyContinue
        $NsisPath = if ($nsisCommand) { $nsisCommand.Source } else { Join-Path $env:LOCALAPPDATA 'tauri/NSIS/makensis.exe' }
    }
    if (-not (Test-Path -LiteralPath $NsisPath)) { throw 'NSIS is required. Pass -NsisPath <makensis.exe>.' }
    $NsisPath = (Resolve-Path -LiteralPath $NsisPath).Path
}

Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    & $cargoExe test --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'Core tests failed.' }
    # The app embeds the Linux collector it installs on SSH servers (remote server settings).
    if (-not $env:WAID_LINUX_X64) {
        $rustupCommand = Get-Command rustup -ErrorAction SilentlyContinue
        $rustupExe = if ($rustupCommand) { $rustupCommand.Source } else { Join-Path $env:USERPROFILE '.cargo/bin/rustup.exe' }
        & $rustupExe target add x86_64-unknown-linux-musl
        if ($LASTEXITCODE -ne 0) { throw 'Could not add the Linux musl target.' }
        $env:CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER = 'rust-lld'
        & $cargoExe build --release --locked --target x86_64-unknown-linux-musl
        if ($LASTEXITCODE -ne 0) { throw 'Linux server collector build failed.' }
        $env:WAID_LINUX_X64 = (Resolve-Path -LiteralPath 'target/x86_64-unknown-linux-musl/release/waid').Path
    }
    & $cargoExe test --manifest-path desktop/Cargo.toml --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'Native shell tests failed.' }
    & $cargoExe build --manifest-path desktop/Cargo.toml --release --locked
    if ($LASTEXITCODE -ne 0) { throw 'Native shell build failed.' }
    & (Join-Path $PSScriptRoot 'test-package.ps1')
    if ($Portable) {
        & (Join-Path $PSScriptRoot 'package.ps1') -Tag $Tag
    } else {
        $metadata = & $cargoExe metadata --manifest-path desktop/Cargo.toml --no-deps --format-version 1 --locked | ConvertFrom-Json
        if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata failed.' }
        $version = ($metadata.packages | Where-Object name -eq 'waid-desktop').version
        $releaseDir = (Resolve-Path -LiteralPath 'desktop/target/release').Path
        New-Item -ItemType Directory -Path (Join-Path $releaseDir 'bundle') -Force | Out-Null
        & $NsisPath /V2 "/DWAID_VERSION=$version" "/DWAID_RELEASE_DIR=$releaseDir" (Join-Path $PSScriptRoot 'installer.nsi')
        if ($LASTEXITCODE -ne 0) { throw 'Installer build failed.' }
    }
} finally {
    Pop-Location
}
