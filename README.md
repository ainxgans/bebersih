# Bebersih

> Modern, fast, and safe disk cleaner for Ubuntu designed for software engineers and power users.

Bebersih is a desktop utility built with **Tauri v2**, **Rust**, and **React** that detects and safely purges accumulated build artifacts, package manager caches, Docker engine detritus, and stale system logs on Ubuntu systems.

---

## Features

- **Developer Caches**:
  - **Rust**: Project `target/` directories and Cargo package registry caches (`~/.cargo/registry/cache`).
  - **Node.js**: Deeply nested `node_modules/` and npm cache (`~/.npm`).
  - **Go**: Global build cache (`GOCACHE`) and module download cache (`GOMODCACHE`).
- **Docker Engine**:
  - Dangling images, buildkit cache, and stopped containers.
  - Unused local volumes detected separately with safety opt-in (unchecked by default).
- **Ubuntu System & APT**:
  - APT package cache archives (`/var/cache/apt/archives`).
  - System crash reports (`/var/crash` older than 7 days).
  - Obsolete system packages with single Polkit (`pkexec`) authentication batch prompt.
- **Stale Files & System Logs**:
  - Systemd journal vacuuming (`journalctl --vacuum-time=7d`).
  - Desktop thumbnail cache (`~/.cache/thumbnails`).
  - Stale XDG user caches (`~/.cache` files unaccessed for >30 days).
- **Safety First**:
  - **Linux Trash Default**: Developer project folders (`node_modules`, `target`) move to FreeDesktop Trash (`gio trash` / XDG Trash) by default, allowing easy restoration of accidental deletions.
  - **Path Protection**: System-critical directory blacklist prevents accidental deletion of `/`, `/home`, `/usr`, `/etc`, and other critical paths.
  - **Granular Selection**: Interactive UI table with per-item checkboxes and category toggles before any cleanup action is performed.
  - **Zero Silent Background Deletion**: All scans and deletions require explicit user confirmation.

---

## Prerequisites (Ubuntu 22.04 / 24.04 LTS)

### 1. System Packages

Install the required C/C++ build tools and WebKitGTK development libraries:

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  libwebkit2gtk-4.1-dev \
  curl \
  wget \
  file \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

### 2. Rust Toolchain

Install Rust (1.75+ required):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### 3. Node.js (v18+)

Install Node.js and npm via `nvm` or official NodeSource repository:

```bash
# Verify versions:
node --version # >= 18.0.0
npm --version
```

### 4. Optional: Docker

If you want to clean Docker build caches and dangling images, ensure the Docker daemon is running and your user has access to Docker:

```bash
sudo usermod -aG docker "$USER"
```

---

## Getting Started

### Installation & Setup

Clone the repository and install frontend dependencies:

```bash
git clone https://github.com/example/bebersih.git
cd bebersih
npm install
```

### Running Development Mode

Launch Bebersih in development mode with live reload:

```bash
npx tauri dev
```

This starts the Vite development server and launches the native Ubuntu GTK window running the Tauri backend.

### Running Frontend Standalone

If you only want to inspect or style UI components in the browser:

```bash
npm run dev
```

---

## Testing & Verification

### Run Rust Unit & Integration Tests

```bash
cargo test --all --manifest-path src-tauri/Cargo.toml
```

### Verify Frontend TypeScript & Production Build

```bash
npm run build
```

---

## Building Production Packages

To build production binaries and native Ubuntu `.deb` or `AppImage` packages:

```bash
npx tauri build
```

The output packages will be generated under:

- Debian package: `src-tauri/target/release/bundle/deb/`
- AppImage bundle: `src-tauri/target/release/bundle/appimage/`
- Native executable: `src-tauri/target/release/bebersih`

---

## Architecture

```
┌────────────────────────────────────────────────────────┐
│                   Frontend (React + TS)                │
│  - Dashboard & Disk Usage Meter                        │
│  - Category Checklist (Dev, Docker, APT, Stale)        │
│  - Itemized Selection Table & Trash/Purge Toggle       │
│  - Confirmation Modal & Cleanup Execution              │
└───────────────────────────▲────────────────────────────┘
                            │ Tauri IPC (Commands & Events)
┌───────────────────────────▼────────────────────────────┐
│                    Backend (Rust Core)                 │
│                                                        │
│  ┌──────────────────┐  ┌──────────────────┐            │
│  │  Scanner Engine  │  │   Cleaner Engine │            │
│  │ (jwalk / rayon)  │  │ (Trash / Purge)  │            │
│  └────────┬─────────┘  └────────┬─────────┘            │
│           │                     │                      │
│  ┌────────▼─────────────────────▼─────────┐            │
│  │               Providers                │            │
│  │  - DevProvider (Cargo, Go, npm)        │            │
│  │  - DockerProvider (CLI / Socket)       │            │
│  │  - SystemProvider (APT, Logs, Journal) │            │
│  │  - RootBatchProvider (Polkit pkexec)   │            │
│  └────────────────────────────────────────┘            │
└────────────────────────────────────────────────────────┘
```

- **Scanner Engine**: Multi-threaded filesystem traversal using `jwalk` for minimal overhead and non-blocking background scanning.
- **Cleaner Engine**: Safe removal engine that separates user-recoverable items (via XDG Trash) from permanent cache purges.
- **Root Tasks**: Bundles APT clean, crash log removal, and journal vacuuming into a single batch script executed via `pkexec` for a seamless single-prompt authentication experience.

---

## License

MIT
