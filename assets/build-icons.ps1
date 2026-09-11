# Convert the approved app tile to the PNG/ICO formats consumed by Windows.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
function Write-Asset([string]$path, [byte[]]$bytes) {
    $temporary = "$path.$([guid]::NewGuid()).tmp"
    try {
        [System.IO.File]::WriteAllBytes($temporary, $bytes)
        if (Test-Path -LiteralPath $path) { [System.IO.File]::Replace($temporary, $path, [NullString]::Value) }
        else { [System.IO.File]::Move($temporary, $path) }
    } finally {
        if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary }
    }
}
function Resize-Png([System.Drawing.Image]$source, [int]$size) {
    $bitmap = [System.Drawing.Bitmap]::new($size, $size)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $stream = [System.IO.MemoryStream]::new()
    try {
        $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $graphics.DrawImage($source, 0, 0, $size, $size)
        $bitmap.Save($stream, [System.Drawing.Imaging.ImageFormat]::Png)
        ,$stream.ToArray()
    } finally {
        $stream.Dispose()
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}
$source = [System.Drawing.Image]::FromFile((Join-Path $PSScriptRoot 'waid-app-icon.png'))
try {
    $sizes = @(16, 20, 24, 32, 40, 48, 64, 128, 256)
    $images = foreach ($size in $sizes) {
        ,(Resize-Png $source $size)
    }
    Write-Asset (Join-Path $PSScriptRoot '../desktop/assets/waid.png') $images[-1]
    $output = [System.IO.MemoryStream]::new()
    $writer = [System.IO.BinaryWriter]::new($output)
    try {
        $writer.Write([uint16]0)
        $writer.Write([uint16]1)
        $writer.Write([uint16]$sizes.Count)
        $offset = 6 + 16 * $sizes.Count
        for ($i = 0; $i -lt $sizes.Count; $i++) {
            $dimension = [byte]($sizes[$i] % 256)
            $writer.Write($dimension)
            $writer.Write($dimension)
            $writer.Write([uint16]0)
            $writer.Write([uint16]1)
            $writer.Write([uint16]32)
            $writer.Write([uint32]$images[$i].Length)
            $writer.Write([uint32]$offset)
            $offset += $images[$i].Length
        }
        foreach ($bytes in $images) { $writer.Write([byte[]]$bytes) }
        if ($output.Length -ne $offset) { throw 'ICO size mismatch.' }
        Write-Asset (Join-Path $PSScriptRoot 'waid.ico') $output.ToArray()
    } finally {
        $writer.Dispose()
        $output.Dispose()
    }
} finally { $source.Dispose() }
