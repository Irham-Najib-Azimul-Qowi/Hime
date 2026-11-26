# Cargo Workspace Virtual Manifest & Profile Architecture

- **Document ID:** HIME-ENV-05-02
- **Task ID:** TASK-05-04 (Cargo Workspace Virtual Manifest & Profile Architecture)
- **Phase:** Phase 05 — Development Environment & Implementation Readiness
- **Authority:** System Architect & Technical Lead
- **Execution Role:** Rust Build Engineer & Senior Software Architect
- **Status:** **APPROVED ARCHITECTURAL SPECIFICATION — BUILD VERIFICATION BLOCKED (TOOLCHAIN MISSING)**
- **Date:** 2026-11-10

---

## 1. Executive Summary & Monorepo Topology

Dokumen ini mendefinisikan secara presisi arsitektur **Cargo Workspace Virtual Manifest** untuk ekosistem Hime.

Sesuai rancangan Phase 04 ([`COMPONENT-BOUNDARIES-AND-OWNERSHIP.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/implementation/COMPONENT-BOUNDARIES-AND-OWNERSHIP.md)):
- Hime dibangun menggunakan arsitektur *Modular Monolith multi-crate* yang memisahkan presentation shell, kernel logika, protokol IPC, lapisan abstraksi platform, penyimpanan data, dan brankas keamanan.
- Root repositori menggunakan **Virtual Manifest** (`[workspace]` tanpa root package) dengan `resolver = "2"`.
- Seluruh versi dependensi eksternal dikonsolidasikan pada level workspace (`[workspace.dependencies]`) untuk menjamin keseragaman versi di seluruh crate dan mencegah *dependency duplication*.

---

## 2. Struktur Direktori Monorepo Target (Phase 06 Target)

```text
Hime/
├── Cargo.toml                       # Root Virtual Workspace Manifest
├── Cargo.lock                       # Lockfile terpin (dihasilkan saat Phase 06)
├── apps/
│   └── hime-desktop/
│       ├── package.json             # Frontend React 19 + TypeScript
│       ├── vite.config.ts           # Vite bundler configuration
│       └── src-tauri/
│           ├── Cargo.toml           # Tauri 2.0 Rust Application Manifest
│           └── tauri.conf.json      # Konfigurasi Window DirectComposition Tauri
└── crates/
    ├── hime-core/
    │   └── Cargo.toml               # Engine, Policy Engine, Process Supervisor
    ├── hime-ipc/
    │   └── Cargo.toml               # Framing 12-byte HIME, Named Pipe, UDS
    ├── hime-pal/
    │   └── Cargo.toml               # Platform Abstraction (Win32 / Wayland)
    ├── hime-storage/
    │   └── Cargo.toml               # SQLite WAL, FTS5 Trigram, 5D Safe Move
    └── hime-security/
        └── Cargo.toml               # DPAPI CryptProtectData, SecretString
```

---

## 3. Kompatibilitas Rust Edition, MSRV & Target Platform

| Parameter Teknis | Nilai Standar Terpilih | Justifikasi Teknis |
|:---|:---:|:---|
| **Rust Edition** | `2021` | Edisi paling stabil dengan dukungan 100% pada ekosistem crate Windows/Tauri. |
| **Minimum Supported Rust Version (MSRV)** | `1.80.0` | Mendukung fungsi `LazyLock` standar, pembatasan CFG kustom, dan kompatibilitas Tauri 2.0. |
| **Workspace Resolver** | `"2"` | Memisahkan resolusi fitur dependensi antara target build normal dan dev-dependencies/build-dependencies. |
| **Primary Target (Windows)** | `x86_64-pc-windows-msvc` | Target native Windows 11 64-bit menggunakan link-time MSVC toolchain v143. |
| **Secondary Target (Linux)** | `x86_64-unknown-linux-gnu` | Target cross-platform Linux menggunakan libc/UDS dan Wayland companion. |

---

## 4. Konsolidasi Dependensi Terpusat (`[workspace.dependencies]`)

Seluruh crate wajib mengimpor dependensi eksternal melalui tabel terpusat berikut:

| Crate Dependensi | Versi Terpin | Fitur Wajib yang Diaktifkan | Penggunaan di Hime |
|:---|:---:|:---|:---|
| **`tokio`** | `1.38.0` | `["full"]` | Asynchronous runtime untuk actor threads, timer, dan async I/O. |
| **`tauri`** | `2.0.0` | `["wry"]` | Desktop webview application framework. |
| **`tauri-build`**| `2.0.0` | Default | Build script generator untuk integrasi Tauri native shell. |
| **`serde`** | `1.0.203` | `["derive"]` | Framework serialisasi/deserialisasi terstruktur. |
| **`serde_json`**| `1.0.117` | Default | Parser payload JSON-RPC 2.0 dan JSONL log streaming. |
| **`rusqlite`** | `0.31.0` | `["bundled", "hooks"]` | Embedded SQLite 3.45+ engine terkompilasi (WAL + FTS5 trigram). |
| **`tracing`** | `0.1.40` | Default | Structured event emissions and span management. |
| **`tracing-subscriber`** | `0.3.18` | `["json", "env-filter"]`| Redaction layers and rolling file JSONL sink. |
| **`zeroize`** | `1.8.1` | `["zeroize_derive"]` | Memory wiping primitives untuk `SecretString`. |
| **`windows`** | `0.58.0` | Win32 foundation, security, system services | Native Win32 API binding (Job Objects, DPAPI, Named Pipes). |
| **`thiserror`** | `1.0.61` | Default | Deklarasi enum error deterministik bertingkat per crate. |
| **`anyhow`** | `1.0.86` | Default | Error handling kontekstual untuk aplikasi entrypoint. |

---

## 5. Profil Kompilasi Workspace (`[profile.dev]` & `[profile.release]`)

### Profil Development (`[profile.dev]`):
- `opt-level = 0`: Waktu kompilasi paling cepat untuk iterasi pengembangan harian.
- `debug = 2`: Simbol debug penuh untuk penelusuran stack trace via debugger LLDB/Visual Studio.
- `overflow-checks = true`: Memeriksa integer overflow secara runtime.
- `panic = "unwind"`: Mendukung unwinding stack untuk penanganan test failure yang rapi.

### Profil Production Release (`[profile.release]`):
- `opt-level = 3`: Optimasi kinerja maksimal compiler LLVM.
- `lto = "thin"`: Link-Time Optimization thin lintas crate untuk mengurangi ukuran binary tanpa membebani RAM kompilasi.
- `codegen-units = 1`: Menghilangkan fragmentasi unit kompilasi untuk optimalisasi inlining fungsi.
- `panic = "abort"`: Menghilangkan overhead landing pads exception unwinding, mengecilkan ukuran binary ~15%.
- `strip = true`: Membuang seluruh debug symbol dan nama fungsi privat dari executable final.

---

## 6. Status Kesiapan Manifes & Pemblokir Toolchain

### Status Validasi:
- Manifes virtual root `Cargo.toml` telah dirancang secara presisi.
- Namun, karena toolchain `cargo` dan `rustc` saat ini berstatus **MISSING** pada mesin host Windows 11, **VERIFIKASI KOMPILASI ATAU PARSING MANIFES BERSTATUS `BLOCKED BY MISSING TOOLCHAIN`**.
- Sesuai aturan mutlak tata kelola: **ZERO PHASE 06 IMPLEMENTATION CODE**. Tidak ada file kode `.rs` yang dibuat pada fase ini.
