# Install Harvester as the `harvester` command on Windows.
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $Root

function Ensure-Cargo {
    if (Get-Command cargo -ErrorAction SilentlyContinue) { return }
    Write-Host "cargo not found. Installing Rust via rustup-init…"
    $tmp = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile $tmp
    & $tmp -y
    $cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE ".cargo" }
    $env:Path = "$(Join-Path $cargoHome 'bin');$env:Path"
}

Ensure-Cargo
Write-Host "Building Harvester (release)…"
cargo build --release
$bin = Join-Path $Root "target\release\harvester.exe"
if (-not (Test-Path $bin)) { throw "build failed: $bin missing" }

$dest = if ($env:HARVESTER_PREFIX) { $env:HARVESTER_PREFIX } else { Join-Path $env:USERPROFILE ".local\bin" }
New-Item -ItemType Directory -Force -Path $dest | Out-Null
Copy-Item -Force $bin (Join-Path $dest "harvester.exe")

Write-Host ""
Write-Host "Installed $(Join-Path $dest 'harvester.exe')"
if (-not (Get-Command harvester -ErrorAction SilentlyContinue)) {
    Write-Host "Add this directory to PATH if needed: $dest"
}
Write-Host "Run:  harvester"
Write-Host ""
Write-Host "Development continues on Origin. This public GitHub clone is the playable tree."
