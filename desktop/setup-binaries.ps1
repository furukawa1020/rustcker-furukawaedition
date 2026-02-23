$ErrorActionPreference = "Stop"

$TARGET_MSVC = "x86_64-pc-windows-msvc"
$TARGET_GNU = "x86_64-pc-windows-gnu"
$SRC = "../target/release/rustkerd.exe"
$DEST_DIR = "src-tauri/binaries"
$DEST = "$DEST_DIR/rustkerd-$TARGET.exe"

Write-Host "Setting up binaries for build..."

if (!(Test-Path $DEST_DIR)) {
    New-Item -ItemType Directory -Path $DEST_DIR | Out-Null
}

# Ensure release build exists
if (!(Test-Path $SRC)) {
    Write-Host "Building rustkerd (release)..."
    cargo build --release -p rustkerd
}

Copy-Item $SRC "$DEST_DIR/rustkerd-$TARGET_MSVC.exe"
Copy-Item $SRC "$DEST_DIR/rustkerd-$TARGET_GNU.exe"
Write-Host "Copied $SRC to $DEST_DIR (both msvc and gnu)"
