[CmdletBinding()]
param(
    [string]$Version = $env:MOLDX_VERSION,
    [string]$Repository = $(if ($env:MOLDX_REPO) { $env:MOLDX_REPO } else { "LorenzoRottigni/moldx" }),
    [string]$InstallDirectory = $(if ($env:MOLDX_INSTALL_DIR) { $env:MOLDX_INSTALL_DIR } else { "$HOME\AppData\Local\Programs\moldx" })
)

$ErrorActionPreference = "Stop"
$binary = "moldx.exe"

if ([string]::IsNullOrWhiteSpace($Version)) {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repository/releases/latest"
    $Version = $release.tag_name
}

$architecture = switch ($env:PROCESSOR_ARCHITECTURE) {
    "AMD64" { "x86_64"; break }
    "ARM64" { "aarch64"; break }
    default { throw "Unsupported Windows architecture: $env:PROCESSOR_ARCHITECTURE" }
}

$assetName = "moldx-windows-$architecture"
$archiveName = "$assetName-$Version.zip"
$baseUrl = "https://github.com/$Repository/releases/download/$Version"
$tempDirectory = Join-Path ([System.IO.Path]::GetTempPath()) ("moldx-" + [Guid]::NewGuid())
$archivePath = Join-Path $tempDirectory $archiveName
$checksumsPath = Join-Path $tempDirectory "SHA256SUMS"

New-Item -ItemType Directory -Path $tempDirectory | Out-Null
try {
    Write-Host "Installing moldx $Version (windows/$architecture)"
    Invoke-WebRequest -Uri "$baseUrl/$archiveName" -OutFile $archivePath
    Invoke-WebRequest -Uri "$baseUrl/SHA256SUMS" -OutFile $checksumsPath

    $expected = Select-String -Path $checksumsPath -Pattern "  $([regex]::Escape($archiveName))$" |
        Select-Object -First 1
    if ($null -eq $expected) {
        throw "No checksum entry found for $archiveName"
    }

    $expectedHash = ($expected.Line -split "\s+")[0].ToLowerInvariant()
    $actualHash = (Get-FileHash -Path $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($expectedHash -ne $actualHash) {
        throw "Checksum verification failed for $archiveName"
    }

    Expand-Archive -Path $archivePath -DestinationPath $tempDirectory -Force
    New-Item -ItemType Directory -Path $InstallDirectory -Force | Out-Null
    Copy-Item (Join-Path $tempDirectory $binary) (Join-Path $InstallDirectory $binary) -Force

    Write-Host "Installed moldx at $(Join-Path $InstallDirectory $binary)"
    & (Join-Path $InstallDirectory $binary) --version
}
finally {
    Remove-Item $tempDirectory -Recurse -Force -ErrorAction SilentlyContinue
}
