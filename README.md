# Rustker Desktop (formerly HATAKE Desktop)

Rustker Desktop is a lightweight, high-performance, and strictly compliant Docker Desktop alternative tailored for Windows and WSL2 environments. Built entirely from scratch using **Rust**, it integrates seamlessly with your local system while maintaining a strikingly low resource footprint.

## Why Rust? (The Rustker Advantage)

Building the core container engine (`rustkerd`) and the desktop application in Rust provides several distinct advantages over traditional Go-based or Electron-based solutions:

### 1. 🚀 Unrivaled Resource Efficiency & Launch Speed
Unlike Docker Desktop, which can consume gigabytes of RAM in the background even when idling, Rustker is compiled to a pure native binary with no Garbage Collector (GC). The background engine takes up only **tens of megabytes** of memory, ensuring your PC stays snappy while the engine is running. Launch speeds are nearly instantaneous.

### 2. 🛡️ Absolute Safety via Type-Safe FSMs
Container management involves complex state transitions (Created -> Running -> Stopped -> Deleted). In Rustker, these rules are enforced as **Compile-Time State Machines**. It is structurally impossible for the backend to accidentally start a "Deleted" container or delete a "Running" container. Rust’s strict compiler eliminates the nil-pointer dereferences and state-handling bugs common in complex Go orchestration tools.

### 3. 🔌 Low-Level Windows & WSL Integration
Rustker operates natively on Windows while deeply manipulating Linux processes running within WSL2. Rust’s supreme low-level capabilities allow it to communicate directly with Win32 APIs and Linux `chroot` environments simultaneously without the overhead of heavy abstractions. (e.g. Our custom layer unpacking cleanly handles Windows symlink restrictions automatically).

### 4. 🎨 Lightning Fast GUI (Tauri)
By pairing Rust with **Tauri** (instead of Electron), the Rustker Desktop GUI relies on the native OS Webview. This avoids bundling an entire Chromium instance, keeping the installer size small and rendering the frontend extremely responsive and lightweight.

### 5. ⚡ Fearless Concurrency
Containers are concurrent by nature (downloading layers, streaming UTF-16 logs cleanly from `wsl.exe`, managing networks). Rust’s ownership model provides fearless parallel execution. Concurrent tasks are safely resolved at compile time, eliminating data races.

## Features Built
- Content-addressable Image Store (pulls images directly from Docker Hub)
- Full Container Lifecycle Management
- Volume Mounting & Port Forwarding
- Isolated Custom Networks
- Live log streaming with native UTF-16 WSL support
- Docker Compose parsing natively using `rustker_compose`
- Beautiful, intuitive Dark Mode GUI (Tauri + React)

## Local Build & Installation

1. **Build the Backend Engine (`rustkerd`)**
   ```powershell
   cargo build --release --bin rustkerd
   ```
2. **Setup Sidecar Native Binaries**
   ```powershell
   cd desktop
   .\setup-binaries.ps1
   ```
3. **Build the Tauri Installer**
   ```powershell
   npm install
   npm run tauri build
   ```
   *The built `.msi` and `.exe` wizard installer will be located in `desktop/src-tauri/target/release/bundle/msi/`.*
