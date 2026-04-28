$ErrorActionPreference = "Stop"

$Repo = if ($env:DUST_REPO) { $env:DUST_REPO } else { "y3l1n4ung/dust" }
$Version = if ($env:DUST_VERSION) { $env:DUST_VERSION } else { "latest" }
$InstallDir = if ($env:DUST_INSTALL_DIR) { $env:DUST_INSTALL_DIR } else { Join-Path $HOME ".local\\bin" }
$InstallBin = Join-Path $InstallDir "dust.exe"
$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("dust-install-" + [System.Guid]::NewGuid().ToString("N"))

New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

try {
    $Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
    switch ($Arch) {
        "x64" { $Asset = "dust-x86_64-pc-windows-msvc.zip" }
        "arm64" { $Asset = "dust-aarch64-pc-windows-msvc.zip" }
        default { throw "unsupported architecture: $Arch" }
    }

    if ($Version -eq "latest") {
        $BaseUrl = "https://github.com/$Repo/releases/latest/download"
    } else {
        $BaseUrl = "https://github.com/$Repo/releases/download/$Version"
    }
    $Url = "$BaseUrl/$Asset"
    $ChecksumsUrl = "$BaseUrl/SHA256SUMS.txt"

    $Archive = Join-Path $TempDir $Asset
    $Checksums = Join-Path $TempDir "SHA256SUMS.txt"
    Write-Host "downloading $Url"
    Invoke-WebRequest -Uri $Url -OutFile $Archive
    Invoke-WebRequest -Uri $ChecksumsUrl -OutFile $Checksums

    $ChecksumLine = Get-Content $Checksums | Where-Object { $_ -match "\s+$([regex]::Escape($Asset))$" } | Select-Object -First 1
    if (-not $ChecksumLine) {
        throw "could not find checksum for $Asset"
    }

    $ExpectedHash = (($ChecksumLine -split "\s+") | Where-Object { $_ })[0].ToLowerInvariant()
    $ActualHash = (Get-FileHash -Algorithm SHA256 $Archive).Hash.ToLowerInvariant()
    if ($ActualHash -ne $ExpectedHash) {
        throw "checksum mismatch for $Asset; expected $ExpectedHash, got $ActualHash"
    }
    Write-Host "verified $Asset"

    Expand-Archive -Path $Archive -DestinationPath $TempDir -Force
    $TargetBin = Get-ChildItem -Path $TempDir -Recurse -Filter "dust.exe" | Select-Object -First 1
    if (-not $TargetBin) {
        throw "could not find dust.exe in release archive"
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item $TargetBin.FullName $InstallBin -Force

    Write-Host "installed dust -> $InstallBin"
} finally {
    Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
}
