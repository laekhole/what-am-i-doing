param(
    [Parameter(Mandatory)]
    [ValidatePattern('^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$')]
    [string]$Tag,
    [string]$BinaryDirectory = (Join-Path $PSScriptRoot 'target/release'),
    [string]$OutputDirectory = (Join-Path $PSScriptRoot 'target/release/msix'),
    [string]$IdentityFile = (Join-Path $PSScriptRoot 'msix/identity.json'),
    # Signs with a throwaway self-signed certificate for local install tests.
    # Only its public .cer is kept; Store uploads stay unsigned and Microsoft signs them.
    [switch]$TestSign
)
$ErrorActionPreference = 'Stop'

function Find-SdkTool([string]$name) {
    $command = Get-Command $name -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }
    $kits = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10/bin'
    $found = Get-ChildItem -LiteralPath $kits -Recurse -Filter $name -ErrorAction SilentlyContinue |
        Where-Object { $_.Directory.Name -eq 'x64' } | Sort-Object FullName -Descending | Select-Object -First 1
    if (-not $found) { throw "Windows SDK tool $name is required." }
    $found.FullName
}

$file = Get-Item -LiteralPath (Join-Path $BinaryDirectory 'waid-desktop.exe')
if ($file.PSIsContainer -or $file.Length -eq 0) { throw 'Missing desktop executable.' }
$info = [Diagnostics.FileVersionInfo]::GetVersionInfo($file.FullName)
if ($info.ProductName -cne 'waid' -or $info.ProductVersion -cne $Tag.Substring(1)) {
    throw 'Release tag does not match the desktop executable version.'
}
$identity = Get-Content -LiteralPath $IdentityFile -Raw | ConvertFrom-Json
foreach ($key in 'name', 'publisher', 'publisherDisplayName', 'displayName') {
    if (-not $identity.$key) { throw "identity.json is missing $key." }
}

$name = "waid-$Tag-windows-x64"
$msix = Join-Path $OutputDirectory "$name.msix"
$cer = Join-Path $OutputDirectory "$name.cer"
if ((Test-Path -LiteralPath $msix) -or (Test-Path -LiteralPath $cer)) {
    throw 'MSIX output already exists. Use a fresh output directory.'
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null

$work = Join-Path ([IO.Path]::GetTempPath()) "waid-msix-$([guid]::NewGuid().ToString('N'))"
$stage = Join-Path $work 'stage'
New-Item -ItemType Directory -Path $stage -Force | Out-Null
try {
    Copy-Item -LiteralPath $file.FullName -Destination (Join-Path $stage 'waid-desktop.exe')
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'msix/Assets') -Destination (Join-Path $stage 'Assets') -Recurse

    # The MSIX version has four parts; Store requires the last one to be 0.
    $manifest = (Get-Content -LiteralPath (Join-Path $PSScriptRoot 'msix/AppxManifest.xml') -Raw -Encoding utf8).
        Replace('{{IDENTITY_NAME}}', [Security.SecurityElement]::Escape($identity.name)).
        Replace('{{PUBLISHER}}', [Security.SecurityElement]::Escape($identity.publisher)).
        Replace('{{PUBLISHER_DISPLAY_NAME}}', [Security.SecurityElement]::Escape($identity.publisherDisplayName)).
        Replace('{{DISPLAY_NAME}}', [Security.SecurityElement]::Escape($identity.displayName)).
        Replace('{{VERSION}}', "$($Tag.Substring(1)).0")
    if ($manifest -match '\{\{') { throw 'Unreplaced token in AppxManifest.xml.' }
    [IO.File]::WriteAllText((Join-Path $stage 'AppxManifest.xml'), $manifest, (New-Object Text.UTF8Encoding $false))

    # resources.pri lets Windows pick the scale/targetsize icon variants.
    $makepri = Find-SdkTool 'makepri.exe'
    $priconfig = Join-Path $work 'priconfig.xml'
    & $makepri createconfig /cf $priconfig /dq ko-KR_en-US /pv 10.0.0 /o | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'makepri createconfig failed.' }
    # One package, not a bundle: keep every scale in resources.pri instead of split resource packs.
    [xml]$config = Get-Content -LiteralPath $priconfig -Raw
    $packaging = $config.SelectSingleNode('/resources/packaging')
    if ($packaging) { [void]$packaging.ParentNode.RemoveChild($packaging) }
    $config.Save($priconfig)
    & $makepri new /pr $stage /cf $priconfig /mn (Join-Path $stage 'AppxManifest.xml') /of (Join-Path $stage 'resources.pri') /o | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'makepri new failed.' }

    & (Find-SdkTool 'makeappx.exe') pack /d $stage /p $msix | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'makeappx pack failed.' }

    if ($TestSign) {
        $cert = New-SelfSignedCertificate -Type Custom -Subject $identity.publisher -KeyUsage DigitalSignature `
            -FriendlyName 'waid MSIX local test' -CertStoreLocation 'Cert:\CurrentUser\My' -NotAfter (Get-Date).AddDays(30) `
            -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')
        try {
            & (Find-SdkTool 'signtool.exe') sign /fd SHA256 /sha1 $cert.Thumbprint /s My $msix | Out-Null
            if ($LASTEXITCODE -ne 0) { throw 'signtool sign failed.' }
            Export-Certificate -Cert $cert -FilePath $cer -Type CERT | Out-Null
        } finally {
            Remove-Item -LiteralPath "Cert:\CurrentUser\My\$($cert.Thumbprint)" -DeleteKey
        }
    }
} finally {
    Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
}
Write-Output $msix
