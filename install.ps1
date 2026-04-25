$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$TargetBin = Join-Path $RootDir "target\\release\\dust.exe"
$InstallDir = if ($env:DUST_INSTALL_DIR) { $env:DUST_INSTALL_DIR } else { Join-Path $HOME ".local\\bin" }
$InstallBin = Join-Path $InstallDir "dust.exe"

if (-not (Test-Path $TargetBin)) {
    Write-Host "building dust..."
    cargo build --release -p dust_cli --manifest-path (Join-Path $RootDir "Cargo.toml")
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item $TargetBin $InstallBin -Force

Write-Host "installed dust -> $InstallBin"
