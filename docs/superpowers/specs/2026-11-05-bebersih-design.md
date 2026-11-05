# Design Document: Bebersih (Ubuntu Developer & System Cleaner)

## 1. Intent & Why
Pengguna Ubuntu, khususnya software engineer, sering kehabisan ruang disk akibat penumpukan file cache build tools (Cargo `target/`, Go build cache, `node_modules`), artifact Docker (dangling images, buildkit cache), package manager cache (APT, npm), dan file sementara/log sistem yang usang.

Tool pembersih umum yang ada (seperti BleachBit atau Stacer) umumnya hanya fokus pada log sistem/browser cache dan tidak mengerti struktur proyek developer. Bebersih dibuat sebagai aplikasi GUI desktop modern dan cepat khusus Ubuntu untuk mendeteksi serta membersihkan sampah sistem dan developer secara selektif dan aman.

## 2. Goals, Non-Goals, & Anti-Goals

### Goals
- Mendeteksi ukuran pemakaian disk untuk kategori:
  - **Dev Caches**: Rust `target/` folders, Go build/mod cache, Node.js `node_modules/` & npm cache.
  - **Docker Engine**: Dangling images, stopped containers, build cache, dan unused volumes (terpisah/opt-in).
  - **System Packages**: APT package cache (`/var/cache/apt/archives`), obsolete packages (`autoclean` / `autoremove`).
  - **Stale / Temp Files**: Thumbnail cache (`~/.cache/thumbnails`), crash reports (`/var/crash`), system logs/journal lama (`journalctl --vacuum-time=7d`), dan file usang di `~/.cache`.
- Menampilkan visual breakdown per kategori dengan indikator ukuran (MB/GB) dan status path.
- Menyediakan seleksi granular (checkbox tree / checklist) sehingga pengguna dapat memilih folder proyek mana yang boleh dihapus.
- Menjalankan proses scan secara paralel dan asinkron tanpa memblokir UI.
- Eksekusi aman:
  - Cache murni (APT, npm, docker cache) dihapus permanen.
  - Folder proyek (`node_modules`, `target`) mendukung pemindahan ke Linux Trash (`gio trash` / XDG Trash spec) secara default, dengan opsi hapus permanen.
- Eskalasi hak akses root secara aman via Polkit / `pkexec` dengan auth prompt tunggal (batch execution).

### Non-Goals
- Tidak menjadi task manager atau system monitor real-time seperti Htop/Stacer (fokus tunggal: disk cleaner).
- Tidak mengutak-atik file konfigurasi user (`~/.config`) atau data database aktif.
- Tidak mendukung multi-OS non-Linux pada fase awal (fokus spesifik Ubuntu / Debian-based GNOME).

### Anti-Goals
- **Zero data loss on active work**: Jangan menghapus file kode sumber `.rs`, `.go`, `.js`, git history `.git`, atau file tak terlacak tanpa izin eksplisit.
- **No silent background deletion**: Tidak ada auto-cleaner terjadwal otomatis di background tanpa konfirmasi manual pengguna.
- **No accidental volume wipe**: Docker volume tidak pernah dihapus secara otomatis tanpa checklist eksplisit tersendiri.

## 3. Constraints
- **Target OS**: Ubuntu 22.04 LTS dan 24.04 LTS (x86_64).
- **Desktop Environment**: GNOME / Wayland & X11.
- **Dependencies**:
  - Backend: Rust 1.75+ & Tauri v2.
  - Frontend: React 18+, TypeScript, Tailwind CSS.
  - System: `libwebkit2gtk-4.1`, Polkit (`pkexec`), `docker` CLI / engine socket.

## 4. Approaches Considered & Trade-offs

### Approach 1: Python + GTK4 / Libadwaita
- **Pros**: Integrasi visual native Ubuntu 100%, tanpa dependensi build webview.
- **Cons**: Traversal direktori murni Python lambat untuk folder proyek dalam (deep nested `node_modules` atau workspace mono-repo), ekosistem threading CPU-bound terbatas karena GIL.

### Approach 2: Go + Wails / Fyne
- **Pros**: Goroutine cepat untuk scan disk, single binary mudah didistribusikan.
- **Cons**: Fyne tidak mengikuti tema native Ubuntu; Wails memiliki ekosistem crate filesystem scanning yang lebih terbatas dibanding Rust.

### Approach 3: Rust + Tauri v2 + React (Dipilih)
- **Pros**:
  - Scanning paralel multithreaded sangat cepat memakai `jwalk` / `rayon`.
  - Tauri v2 mengkonsumsi memori jauh lebih kecil dibanding Electron.
  - React + Tailwind menyediakan fleksibilitas maksimal untuk visualisasi dashboard dan tree table checklist.
  - Keamanan memori Rust dan kemudahan integrasi Docker via `bollard` / subprocess.
- **Cons**: Memerlukan WebKitGTK di sistem host dan waktu build awal Rust lebih lama.

## 5. Architecture & How It Works

```
┌────────────────────────────────────────────────────────┐
│                   Frontend (React + TS)                │
│  - Dashboard & Disk Usage Meter                        │
│  - Scan Category Checklist (Dev, Docker, APT, Stale)   │
│  - Itemized Project List with Selection & Trash Toggle │
│  - Progress Bar & Log Output                           │
└───────────────────────────▲────────────────────────────┘
                            │ Tauri IPC (Commands & Events)
┌───────────────────────────▼────────────────────────────┐
│                    Backend (Rust Core)                 │
│                                                        │
│  ┌──────────────────┐  ┌──────────────────┐            │
│  │  Scanner Engine  │  │   Cleaner Engine │            │
│  │ (jwalk / rayon)  │  │ (Trash / Delete) │            │
│  └────────┬─────────┘  └────────┬─────────┘            │
│           │                     │                      │
│  ┌────────▼─────────────────────▼─────────┐            │
│  │               Providers                │            │
│  │  - DevProvider (Cargo, Go, npm)        │            │
│  │  - DockerProvider (Socket API / CLI)   │            │
│  │  - AptProvider (pkexec helper)         │            │
│  │  - StaleProvider (XDG cache, tmp)      │            │
│  └────────────────────────────────────────┘            │
└────────────────────────────────────────────────────────┘
```

### 5.1 Scanner Engine
- Menerima konfigurasi root search directory (default: `~` atau subfolder spesifik seperti `~/Projects`, `~/code`).
- Menggunakan traversal paralel (`jwalk`) dengan filter:
  - Mencari folder `target` yang berdampingan dengan `Cargo.toml`.
  - Mencari folder `node_modules` yang berdampingan dengan `package.json`.
  - Query Go cache path via `go env GOCACHE` dan `GOMODCACHE`.
  - Query Docker usage via Docker Engine API (`/var/run/docker.sock`) atau fallback command `docker system df --format json`.
  - Scan `/var/cache/apt/archives` untuk menghitung ukuran deb cache.
  - Scan `~/.cache` untuk mencari file dengan access/modification time > 30 hari.
  - Scan `/var/crash` untuk file `.crash` yang sudah berumur > 7 hari.
- Mengirim event progres scan (`scan:progress`) ke UI via Tauri Channel/Event.

### 5.2 Cleaner Engine & Safety Mechanisms
- **Trash Provider**: Menggunakan standard FreeDesktop Trash (`trash` crate atau `gio trash`) untuk artifact proyek user (`node_modules`, `target`).
- **Direct Purge Provider**:
  - Go cache: Menjalankan `go clean -cache -modcache`.
  - Cargo global cache: Menghapus `~/.cargo/registry/cache`.
  - Docker: Menjalankan `docker system prune -f` (hanya dangling images & build cache). Docker volumes dipisahkan dalam daftar tersendiri dan tidak dicentang secara default.
  - System Cleanup (Root Tasks):
    - Mengumpulkan seluruh tugas root (APT clean, `journalctl --vacuum-time=7d`, hapus `/var/crash/*`).
    - Dijalankan dalam satu script batch yang dipanggil via `pkexec` tunggal untuk menghindari prompt password berulang.
- **Pre-check**: Memastikan path yang akan dihapus tidak berada di luar target yang diizinkan (proteksi path traversal dan sanitasi path system-critical seperti `/`, `/home`, `/usr`).

## 6. Decisions & Builder Notes
1. **Docker Volumes**: Default tidak dicentang. Hanya dangling images dan build cache yang masuk rekomendasi default.
2. **Root Authentication**: Batch execution via single `pkexec bash -c ...` untuk meminimalisir auth dialog.
3. **Stale Threshold**: File cache sistem usang menggunakan ambang batas 30 hari; crash logs menggunakan 7 hari.
4. **Project Artifacts**: Menggunakan opsi Trash secara default agar user bisa me-restore folder proyek yang tidak sengaja terhapus.
