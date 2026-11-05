# Bebersih Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Membangun aplikasi desktop GUI Ubuntu (Tauri v2 + React) untuk mendeteksi dan membersihkan cache developer (Rust, Go, Node.js), Docker, package manager (APT), dan file sistem usang secara aman.

**Architecture:** Rust backend berbasis Tauri v2 menjalankan scanner paralel multithreaded (`jwalk` / `rayon`) dan cleaner engine yang memisahkan aksi trash (`gio trash`/XDG trash) untuk folder proyek dari direct purge untuk file cache murni. Eksekusi root digabung dalam batch script via single `pkexec`. Frontend React + Tailwind berkomunikasi dengan Rust melalui Tauri IPC commands dan events.

**Tech Stack:** Rust 1.75+, Tauri v2, React 18+, TypeScript, Tailwind CSS, Vite, `jwalk`, `serde`, `trash-rs`, `bollard`/Docker CLI.

**Spec:** `docs/superpowers/specs/2026-11-05-bebersih-design.md`

## Global Constraints
- Target platform: Ubuntu 22.04 LTS & 24.04 LTS (x86_64, GNOME Wayland/X11).
- Backend engine: Rust dengan Tauri v2.
- Zero data loss: Folder proyek (`node_modules`, `target`) wajib masuk Trash secara default.
- Docker safety: Docker volumes tidak boleh dibersihkan secara default tanpa seleksi checklist eksplisit.
- Root escalation: Seluruh task berhak akses root (APT, systemd journal, /var/crash) digabung dalam batch script via satu kali prompt `pkexec`.
- Path traversal protection: Path yang akan dihapus harus divalidasi tidak menyentuh root system critical (`/`, `/home`, `/usr`, `/etc`).

## Review Focus
1. Path traversal / accidental deletion of system directories: input target path `/` atau `/home` harus ditolak dengan error validasi.
2. Permission error saat scan: permission denied pada folder yang tidak terbaca tidak boleh membatalkan seluruh proses scanning.
3. Missing tools (Docker CLI tidak terpasang atau service daemon mati): scanner Docker harus mengembalikan graceful error/0 bytes tanpa panic.
4. Process cancellation: user membatalkan scan/clean di tengah jalan tidak boleh meninggalkan corrupted state.
5. Trash fallback failure: jika partition filesystem tidak mendukung FreeDesktop Trash, sistem harus memberikan peringatan eksplisit sebelum fallback ke permanent delete.

---

### Task 1: Project Scaffolding & Build Configuration

**Files:**
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `tailwind.config.js`
- Create: `postcss.config.js`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: Host dev tools (`cargo`, `npm`/`pnpm`, `gcc`, `libwebkit2gtk-4.1-dev`).
- Produces: Runnable baseline Tauri v2 application window.

- [ ] **Step 1: Inisialisasi package.json dan Vite React TypeScript**
  Buat `package.json` dengan scripts `dev`, `build`, dan dependensi: `@tauri-apps/api`, `@tauri-apps/plugin-shell`, `react`, `react-dom`, `lucide-react`, `tailwindcss`, `typescript`, `vite`.

- [ ] **Step 2: Konfigurasi Tailwind & Vite**
  Buat `vite.config.ts`, `tailwind.config.js`, dan `postcss.config.js` untuk Tauri frontend.

- [ ] **Step 3: Setup Tauri v2 backend scaffold**
  Buat `src-tauri/Cargo.toml` dengan dependensi `tauri = "2"`, `serde`, `serde_json`, `tokio`, `thiserror`. Buat `src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs`, dan `src-tauri/src/main.rs`.

- [ ] **Step 4: Verifikasi build project**
  Run: `cargo check --manifest-path src-tauri/Cargo.toml`
  Expected: PASS tanpa compile error.

- [ ] **Step 5: Commit**
  ```bash
  git add .
  git commit -m "chore: scaffold tauri v2 and react project structure"
  ```

---

### Task 2: Core Data Models & Error Handling

**Files:**
- Create: `src-tauri/src/models.rs`
- Create: `src-tauri/src/error.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/models_test.rs`

**Interfaces:**
- Consumes: `serde::Serialize`, `serde::Deserialize`.
- Produces:
  - `pub enum CleanCategory`: `DevRust`, `DevGo`, `DevNode`, `DockerCache`, `DockerVolume`, `AptCache`, `SystemLogs`, `StaleCache`.
  - `pub struct CleanItem`: `id: String`, `category: CleanCategory`, `label: String`, `path: Option<String>`, `size_bytes: u64`, `requires_root: bool`, `supports_trash: bool`, `selected: bool`.
  - `pub struct ScanProgress`: `scanned_items: usize`, `current_path: String`, `found_bytes: u64`.
  - `pub struct CleanReport`: `deleted_bytes: u64`, `freed_items: Vec<String>`, `errors: Vec<String>`.

- [ ] **Step 1: Write failing test untuk serialisasi model**
  Buat `src-tauri/tests/models_test.rs` menguji serialisasi `CleanItem` ke JSON dan validasi kategori.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test models_test`
  Expected: FAIL (modules not found).

- [ ] **Step 3: Implementasi models.rs dan error.rs**
  Definisikan enum `CleanCategory`, struct `CleanItem`, `ScanProgress`, `CleanReport`, serta enum `AppError` dengan `thiserror`.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test models_test`
  Expected: PASS.

- [ ] **Step 5: Commit**
  ```bash
  git add src-tauri/src/models.rs src-tauri/src/error.rs src-tauri/src/lib.rs src-tauri/tests/models_test.rs
  git commit -m "feat(core): implement core clean models and error types"
  ```

---

### Task 3: Developer Cache Scanner (Cargo, Go, npm)

**Files:**
- Create: `src-tauri/src/scanner/dev.rs`
- Create: `src-tauri/src/scanner/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/dev_scanner_test.rs`

**Interfaces:**
- Consumes: `models::CleanItem`, `models::CleanCategory`, `jwalk::WalkDir`.
- Produces: `pub fn scan_dev_caches(roots: &[PathBuf]) -> Result<Vec<CleanItem>, AppError>`.

- [ ] **Step 1: Write failing test untuk deteksi folder build dev**
  Buat `src-tauri/tests/dev_scanner_test.rs` dengan direktori temporer:
  - Direktori dengan `Cargo.toml` dan `target/`.
  - Direktori dengan `package.json` dan `node_modules/`.
  - Verifikasi bahwa `scan_dev_caches` mendeteksi folder target/node_modules beserta ukuran byte-nya.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test dev_scanner_test`
  Expected: FAIL.

- [ ] **Step 3: Implementasi dev scanner**
  Implementasikan traversal `jwalk` di `src-tauri/src/scanner/dev.rs`. Deteksi:
  - `target/` yang bersebelahan dengan `Cargo.toml`.
  - `node_modules/` yang bersebelahan dengan `package.json`.
  - Path Go cache via `~/.cache/go-build` dan `~/go/pkg/mod/cache`.
  - Global Cargo cache `~/.cargo/registry/cache`.
  Hitung ukuran direktori secara rekursif.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test dev_scanner_test`
  Expected: PASS.

- [ ] **Step 5: Commit**
  ```bash
  git add src-tauri/src/scanner/ src-tauri/tests/dev_scanner_test.rs
  git commit -m "feat(scanner): implement developer cache scanning"
  ```

---

### Task 4: System & Stale Cache Scanner

**Files:**
- Create: `src-tauri/src/scanner/system.rs`
- Modify: `src-tauri/src/scanner/mod.rs`
- Test: `src-tauri/tests/system_scanner_test.rs`

**Interfaces:**
- Consumes: `models::CleanItem`, `models::CleanCategory`.
- Produces: `pub fn scan_system_caches(stale_days: u32) -> Result<Vec<CleanItem>, AppError>`.

- [ ] **Step 1: Write failing test untuk scanner sistem**
  Buat `src-tauri/tests/system_scanner_test.rs` menguji parsing direktori cache tiruan, filter umur file > N hari, dan deteksi crash log.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test system_scanner_test`
  Expected: FAIL.

- [ ] **Step 3: Implementasi scanner sistem**
  Di `src-tauri/src/scanner/system.rs`:
  - Scan `/var/cache/apt/archives` (deb packages).
  - Scan `~/.cache/thumbnails`.
  - Scan file di `~/.cache` dengan metadata modified time > `stale_days` hari.
  - Scan `/var/crash/*.crash` berumur > 7 hari.
  - Query journalctl size via `journalctl --disk-usage`.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test system_scanner_test`
  Expected: PASS.

- [ ] **Step 5: Commit**
  ```bash
  git add src-tauri/src/scanner/system.rs src-tauri/src/scanner/mod.rs src-tauri/tests/system_scanner_test.rs
  git commit -m "feat(scanner): implement system and stale cache scanner"
  ```

---

### Task 5: Docker Cache & Volume Scanner

**Files:**
- Create: `src-tauri/src/scanner/docker.rs`
- Modify: `src-tauri/src/scanner/mod.rs`
- Test: `src-tauri/tests/docker_scanner_test.rs`

**Interfaces:**
- Consumes: Docker Engine API via CLI fallback (`docker system df --format json` atau HTTP socket `/var/run/docker.sock`).
- Produces: `pub fn scan_docker_caches() -> Result<Vec<CleanItem>, AppError>`.

- [ ] **Step 1: Write failing test untuk Docker scanner parser**
  Buat `src-tauri/tests/docker_scanner_test.rs` yang menguji parsing output json `docker system df` atau payload Docker API.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test docker_scanner_test`
  Expected: FAIL.

- [ ] **Step 3: Implementasi Docker scanner**
  Di `src-tauri/src/scanner/docker.rs`:
  - Cek ketersediaan docker binary / socket. Jika tidak aktif, return `Ok(vec![])` tanpa crash.
  - Ambil info pemakaian: Build Cache, Dangling Images, Stopped Containers.
  - Ambil info Volumes yang tidak terpakai, tandai kategori `DockerVolume` dan beri flag `selected: false` default.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test docker_scanner_test`
  Expected: PASS.

- [ ] **Step 5: Commit**
  ```bash
  git add src-tauri/src/scanner/docker.rs src-tauri/tests/docker_scanner_test.rs
  git commit -m "feat(scanner): implement docker cache and volume scanner"
  ```

---

### Task 6: Cleaner Engine & Safety Guard

**Files:**
- Create: `src-tauri/src/cleaner/trash.rs`
- Create: `src-tauri/src/cleaner/purge.rs`
- Create: `src-tauri/src/cleaner/root.rs`
- Create: `src-tauri/src/cleaner/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/cleaner_test.rs`

**Interfaces:**
- Consumes: `Vec<CleanItem>`, `use_trash: bool`.
- Produces:
  - `pub fn clean_items(items: &[CleanItem], move_project_to_trash: bool) -> Result<CleanReport, AppError>`.
  - `pub fn validate_safe_path(path: &Path) -> Result<(), AppError>`.

- [ ] **Step 1: Write failing test untuk safety validator & cleaner**
  Buat `src-tauri/tests/cleaner_test.rs`:
  - Test `validate_safe_path` me-reject `/`, `/home`, `/usr`, `/etc`, path kosong.
  - Test penghapusan folder temporer dan test simulasi batch command root.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test cleaner_test`
  Expected: FAIL.

- [ ] **Step 3: Implementasi Cleaner Modules**
  - `src-tauri/src/cleaner/mod.rs`: `validate_safe_path` memeriksa blacklist path sistem.
  - `src-tauri/src/cleaner/trash.rs`: Memindahkan folder ke Linux Trash via `trash` crate atau `gio trash`.
  - `src-tauri/src/cleaner/purge.rs`: Menghapus permanen cache files/folders via standard `std::fs::remove_dir_all`.
  - `src-tauri/src/cleaner/root.rs`: Menghasilkan single shell script batch untuk task APT & journalctl, lalu memanggil `pkexec bash -c <script>` satu kali.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test cleaner_test`
  Expected: PASS.

- [ ] **Step 5: Commit**
  ```bash
  git add src-tauri/src/cleaner/ src-tauri/tests/cleaner_test.rs
  git commit -m "feat(cleaner): implement trash, purge, and batch root execution"
  ```

---

### Task 7: Tauri Commands & IPC State Management

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/tests/commands_test.rs`

**Interfaces:**
- Produces Tauri commands:
  - `#[tauri::command] async fn scan_all(custom_paths: Vec<String>) -> Result<Vec<CleanItem>, String>`
  - `#[tauri::command] async fn clean_selected(items: Vec<CleanItem>, move_project_to_trash: bool) -> Result<CleanReport, String>`
  - `#[tauri::command] async fn get_disk_free_space() -> Result<u64, String>`

- [ ] **Step 1: Write failing test untuk Tauri command handlers**
  Buat `src-tauri/tests/commands_test.rs` untuk memverifikasi eksekusi handler `scan_all` dan `clean_selected`.

- [ ] **Step 2: Run test to verify it fails**
  Run: `cargo test --test commands_test`
  Expected: FAIL.

- [ ] **Step 3: Implementasi commands.rs**
  Koneksikan scanner engine dan cleaner engine ke dalam fungsi Tauri command di `src-tauri/src/commands.rs`, daftarkan di `lib.rs` via `tauri::Builder::default().invoke_handler(...)`.

- [ ] **Step 4: Run test to verify it passes**
  Run: `cargo test --test commands_test`
  Expected: PASS.

- [ ] **Step 5: Commit**
  ```bash
  git add src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/tests/commands_test.rs
  git commit -m "feat(ipc): expose tauri commands for scan and clean operations"
  ```

---

### Task 8: Frontend UI (React + Tailwind)

**Files:**
- Create: `src/types.ts`
- Create: `src/components/Header.tsx`
- Create: `src/components/DiskMeter.tsx`
- Create: `src/components/CategoryCard.tsx`
- Create: `src/components/ItemTable.tsx`
- Create: `src/components/CleanConfirmModal.tsx`
- Create: `src/App.tsx`
- Create: `src/index.css`

**Interfaces:**
- Consumes: Tauri IPC `@tauri-apps/api/core` (`invoke`), `models.rs` JSON interfaces.
- Produces: Complete responsive desktop UI with category cards, items table, disk space metrics, and clean modal.

- [ ] **Step 1: Setup types & global CSS**
  Buat `src/types.ts` yang mencerminkan tipe dari Rust (`CleanItem`, `CleanCategory`, `CleanReport`). Setup `src/index.css` dengan tema dark/light modern khas GNOME.

- [ ] **Step 2: Buat komponen UI presentasional**
  - `Header.tsx`: Title bar, status scan, tombol Scan Now dan Clean Selected.
  - `DiskMeter.tsx`: Bar kapasitas pemakaian disk & total bytes sampah yang terdeteksi.
  - `CategoryCard.tsx`: Ringkasan ukuran sampah per kategori dengan toggle select-all.
  - `ItemTable.tsx`: Tabel daftar item, path, ukuran, toggle Trash vs Purge, dan checkbox pilihan.
  - `CleanConfirmModal.tsx`: Dialog konfirmasi sebelum eksekusi pembersihan.

- [ ] **Step 3: Implementasi App.tsx & integrasi state**
  Koneksikan state scan, pemilahan kategori, filter checklist, dan pemanggilan Tauri command `invoke("scan_all")` serta `invoke("clean_selected")`.

- [ ] **Step 4: Verifikasi build frontend**
  Run: `npm run build`
  Expected: PASS (Vite build dist output sukses tanpa TypeScript error).

- [ ] **Step 5: Commit**
  ```bash
  git add src/
  git commit -m "feat(ui): implement modern desktop UI in React and Tailwind"
  ```

---

### Task 9: System Integration & Manual Smoke Test

**Files:**
- Modify: `README.md`
- Test: Manual execution of `cargo tauri dev` or `cargo tauri build`

- [ ] **Step 1: Verifikasi build Tauri end-to-end**
  Run: `cargo test --all`
  Expected: Semua unit tests Rust PASS.

- [ ] **Step 2: Buat dokumentasi penggunaan di README.md**
  Jelaskan dependensi sistem Ubuntu yang dibutuhkan (`libwebkit2gtk-4.1-dev`, `build-essential`), cara menjalankan mode dev, dan panduan fitur.

- [ ] **Step 3: Commit**
  ```bash
  git add README.md
  git commit -m "docs: add setup and usage instructions in README"
  ```
