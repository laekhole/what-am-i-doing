param(
    [Parameter(Mandatory)]
    [ValidatePattern('^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z]+([.-][0-9A-Za-z]+)*)?$')]
    [string]$Tag,
    [string]$BinaryDirectory = (Join-Path $PSScriptRoot 'target/release'),
    [string]$OutputDirectory = (Join-Path $PSScriptRoot 'target/release/bundle')
)
$ErrorActionPreference = 'Stop'
$file = Get-Item -LiteralPath (Join-Path $BinaryDirectory 'waid-desktop.exe')
if ($file.PSIsContainer -or $file.Length -eq 0) { throw 'Missing desktop executable.' }
$name = "waid-$Tag-windows-x64.exe"
$exe = Join-Path $OutputDirectory $name
$sums = Join-Path $OutputDirectory 'SHA256SUMS.txt'
if ((Test-Path -LiteralPath $exe) -or (Test-Path -LiteralPath $sums)) {
    throw 'Release output already exists. Use a fresh output directory.'
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
Copy-Item -LiteralPath $file.FullName -Destination $exe
$hash = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant()
if ($hash -ne (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant()) {
    throw 'Release executable differs from the build.'
}
Set-Content -LiteralPath $sums -Value "$hash  $name" -Encoding ascii
Write-Output $exe
