param([Parameter(Mandatory)][string]$Executable)
$ErrorActionPreference = 'Stop'
$metadata = cargo metadata --manifest-path (Join-Path $PSScriptRoot 'Cargo.toml') --no-deps --format-version 1 --locked --offline | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata failed.' }
$version = ($metadata.packages | Where-Object name -eq 'waid-desktop').version
$info = [Diagnostics.FileVersionInfo]::GetVersionInfo((Resolve-Path -LiteralPath $Executable).Path)
if ($info.ProductName -cne 'waid' -or $info.FileVersion -cne $version -or $info.ProductVersion -cne $version -or
    $info.FileDescription -cne 'waid - what am I doing?' -or $info.OriginalFilename -cne 'waid-desktop.exe') {
    throw 'Windows version information does not match the desktop package.'
}
$numericVersion = ($version -split '[-+]')[0] + '.0'
if ((@($info.FileMajorPart, $info.FileMinorPart, $info.FileBuildPart, $info.FilePrivatePart) -join '.') -cne $numericVersion -or
    (@($info.ProductMajorPart, $info.ProductMinorPart, $info.ProductBuildPart, $info.ProductPrivatePart) -join '.') -cne $numericVersion) {
    throw 'Windows numeric version does not match the desktop package.'
}
$info | Select-Object ProductName, ProductVersion, FileVersion, FileDescription, OriginalFilename
