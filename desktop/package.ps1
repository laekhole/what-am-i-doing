param(
    [Parameter(Mandatory)]
    [ValidatePattern('^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z]+([.-][0-9A-Za-z]+)*)?$')]
    [string]$Tag,
    [string]$BinaryDirectory = (Join-Path $PSScriptRoot 'target/release'),
    [string]$OutputDirectory = (Join-Path $PSScriptRoot 'target/release/bundle')
)
$ErrorActionPreference = 'Stop'
$files = @('waid-desktop.exe', 'waid.exe') | ForEach-Object {
    $file = Get-Item -LiteralPath (Join-Path $BinaryDirectory $_)
    if ($file.PSIsContainer -or $file.Length -eq 0) { throw "Missing binary: $_" }
    $file.FullName
}
$name = "waid-$Tag-windows-x64.zip"
$zip = Join-Path $OutputDirectory $name
$sums = Join-Path $OutputDirectory 'SHA256SUMS.txt'
if ((Test-Path -LiteralPath $zip) -or (Test-Path -LiteralPath $sums)) {
    throw 'Release output already exists. Use a fresh output directory.'
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
Compress-Archive -LiteralPath $files -DestinationPath $zip -CompressionLevel Optimal

# Verify both executable contents, not just whether a ZIP file was created.
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [System.IO.Compression.ZipFile]::OpenRead($zip)
try {
    if ($archive.Entries.Count -ne 2) { throw 'The portable ZIP must contain exactly two executables.' }
    foreach ($file in $files) {
        $entry = $archive.GetEntry([System.IO.Path]::GetFileName($file))
        if (-not $entry) { throw "ZIP entry missing: $file" }
        $stream = $entry.Open()
        try {
            if ((Get-FileHash -InputStream $stream -Algorithm SHA256).Hash -ne (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash) {
                throw "ZIP contents differ from the build: $file"
            }
        } finally { $stream.Dispose() }
    }
} finally { $archive.Dispose() }
$hash = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
Set-Content -LiteralPath $sums -Value "$hash  $name" -Encoding ascii
Write-Output $zip
