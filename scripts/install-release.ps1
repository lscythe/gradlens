param(
    [string]$InstallDir,
    [string]$Version = "latest",
    [string]$Repository = "lscythe/gradlens"
)

$ErrorActionPreference = "Stop"
if (-not $InstallDir) {
    $InstallDir = if ($env:CARGO_HOME) { Join-Path $env:CARGO_HOME "bin" } else { Join-Path $HOME ".cargo\bin" }
}
$arch = switch ([Runtime.InteropServices.RuntimeInformation]::OSArchitecture) {
    "X64" { "x86_64" }
    default { throw "Unsupported Windows architecture: $_" }
}
$target = "$arch-pc-windows-msvc"
if ($Version -eq "latest") {
    $base = "https://github.com/$Repository/releases/latest/download"
    $asset = "gradlens-$target.zip"
} else {
    $base = "https://github.com/$Repository/releases/download/$Version"
    $asset = "gradlens-$Version-$target.zip"
}
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("gradlens-" + [Guid]::NewGuid())
New-Item -ItemType Directory $tmp | Out-Null
try {
    Invoke-WebRequest "$base/$asset" -OutFile (Join-Path $tmp $asset)
    Invoke-WebRequest "$base/$asset.sha256" -OutFile (Join-Path $tmp "$asset.sha256")
    $expected = (Get-Content (Join-Path $tmp "$asset.sha256")).Split()[0]
    $actual = (Get-FileHash (Join-Path $tmp $asset) -Algorithm SHA256).Hash
    if ($actual -ne $expected) { throw "SHA-256 checksum mismatch" }
    Expand-Archive (Join-Path $tmp $asset) -DestinationPath $tmp
    $binary = Get-ChildItem $tmp -Recurse -Filter gradlens.exe | Select-Object -First 1
    if (-not $binary) { throw "Archive did not contain gradlens.exe" }
    New-Item -ItemType Directory -Force $InstallDir | Out-Null
    Copy-Item -Force $binary.FullName (Join-Path $InstallDir "gradlens.exe")
    Write-Host "Installed gradlens to $(Join-Path $InstallDir 'gradlens.exe')"
} finally {
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $tmp
}
