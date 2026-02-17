$ErrorActionPreference = "Stop"

$TARGET = "x86_64-pc-windows-msvc"
$SRC = "../target/release/furukawad.exe"
$DEST_DIR = "src-tauri/binaries"
$DEST = "$DEST_DIR/furukawad-$TARGET.exe"

Write-Host "Setting up binaries for build..."

if (!(Test-Path $DEST_DIR)) {
    New-Item -ItemType Directory -Path $DEST_DIR | Out-Null
}

# Ensure release build exists
if (!(Test-Path $SRC)) {
    Write-Host "Building furukawad (release)..."
    cargo build --release -p furukawad
}

Copy-Item $SRC $DEST
Write-Host "Copied $SRC to $DEST"
