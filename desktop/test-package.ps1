param([string]$BinaryDirectory = (Join-Path $PSScriptRoot 'target/release'))
$ErrorActionPreference = 'Stop'
$exe = Join-Path $BinaryDirectory 'waid-desktop.exe'
& (Join-Path $PSScriptRoot 'test-version-info.ps1') -Executable $exe
$version = [Diagnostics.FileVersionInfo]::GetVersionInfo((Resolve-Path -LiteralPath $exe).Path).ProductVersion
$testRoot = Join-Path ([IO.Path]::GetTempPath()) ('waid-package-test-' + [Guid]::NewGuid().ToString('N'))
$packageScript = Join-Path $PSScriptRoot 'package.ps1'
try {
    $rejected = $false
    try { & $packageScript -Tag "v$version-mismatch" -BinaryDirectory $BinaryDirectory -OutputDirectory $testRoot }
    catch { $rejected = $_.Exception.Message -eq 'Release tag does not match the desktop executable version.' }
    if (-not $rejected -or (Test-Path -LiteralPath $testRoot)) { throw 'Mismatched release must fail before creating output.' }

    $packaged = & $packageScript -Tag "v$version" -BinaryDirectory $BinaryDirectory -OutputDirectory $testRoot
    $hash = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
    if ((Get-FileHash -LiteralPath $packaged -Algorithm SHA256).Hash.ToLowerInvariant() -ne $hash -or
        (Get-Content -LiteralPath (Join-Path $testRoot 'SHA256SUMS.txt')).Trim() -cne "$hash  waid-v$version-windows-x64.exe") {
        throw 'Packaged EXE or checksum differs from the build.'
    }
    $rejected = $false
    try { & $packageScript -Tag "v$version" -BinaryDirectory $BinaryDirectory -OutputDirectory $testRoot }
    catch { $rejected = $_.Exception.Message -eq 'Release output already exists. Use a fresh output directory.' }
    if (-not $rejected) { throw 'Existing release output must not be overwritten.' }
    Write-Output 'Package version, checksum and overwrite checks passed.'
} finally {
    if (Test-Path -LiteralPath $testRoot) {
        $resolved = (Resolve-Path -LiteralPath $testRoot).Path
        if ($resolved -cne [IO.Path]::GetFullPath($testRoot)) { throw 'Unexpected package test cleanup path.' }
        Remove-Item -LiteralPath $resolved -Recurse -Force
    }
}
