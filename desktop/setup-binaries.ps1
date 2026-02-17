$ErrorActionPreference = "Stop"

$TARGET_MSVC = "x86_64-pc-windows-msvc"
$TARGET_GNU = "x86_64-pc-windows-gnu"
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

Copy-Item $SRC "$DEST_DIR/furukawad-$TARGET_MSVC.exe"
Copy-Item $SRC "$DEST_DIR/furukawad-$TARGET_GNU.exe"
Write-Host "Copied $SRC to $DEST_DIR (both msvc and gnu)"
