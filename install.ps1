[CmdletBinding()]
param(
    [string]$InstallDir
)

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $MyInvocation.MyCommand.Path

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo is required; install Rust from https://rustup.rs"
}

if (-not $InstallDir) {
    if ($env:CARGO_HOME) {
        $InstallDir = Join-Path $env:CARGO_HOME "bin"
    } else {
        $InstallDir = Join-Path $HOME ".cargo\bin"
    }
}

& cargo build --release --locked --manifest-path (Join-Path $repo "Cargo.toml")
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$source = Join-Path $repo "target\release\gradle-checker.exe"
$destination = Join-Path $InstallDir "gradle-checker.exe"
$temporary = Join-Path $InstallDir (".gradle-checker.{0}.tmp" -f $PID)

try {
    Copy-Item -Force $source $temporary
    Move-Item -Force $temporary $destination
} finally {
    Remove-Item -Force -ErrorAction SilentlyContinue $temporary
}

Write-Host "Installed gradle-checker to $destination"
$pathEntries = $env:PATH -split [IO.Path]::PathSeparator
if ($InstallDir -notin $pathEntries) {
    Write-Warning "Add $InstallDir to PATH to run gradle-checker from any directory."
}
