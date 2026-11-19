# Component Boundaries & Module Ownership Specification

- **Document ID:** HIME-SPEC-04-01
- **Task ID:** TASK-04-01 (Component Boundaries & Module Ownership)
- **Phase:** Phase 04 — Engineering Planning & Implementation Specification
- **Authority:** System Architect & Technical Lead
- **Execution Role:** Senior Systems Engineer & Software Architect
- **Status:** **APPROVED IMPLEMENTATION SPECIFICATION**
- **Date:** 2026-11-10

---

## 1. Executive Summary & Architectural Paradigm

Dokumen ini mendefinisikan secara presisi batas-batas tanggung jawab (*bounded contexts*), kepemilikan modul, aturan ketergantungan (*dependency direction*), dan antarmuka publik untuk 6 target monorepo workspace Hime:
1. `apps/hime-desktop` (Tauri 2.0 Presentation Shell)
2. `crates/hime-core` (Modular Monolith Orchestration & Business Engine)
3. `crates/hime-ipc` (Binary Framing `HIME` & Transport IPC)
4. `crates/hime-pal` (Platform Abstraction Layer — Win32/Linux)
5. `crates/hime-storage` (SQLite WAL, FTS5 & Safe Filesystem Operations)
6. `crates/hime-security` (Secret Vault DPAPI, CSPRNG & Policy Engine)

### Prinsip Desain: Clean Architecture & Ports-and-Adapters (Hexagonal)
Arsitektur Hime menerapkan prinsip **Inversion of Control (IoC)** dan **Ports & Adapters**:
- `hime-core` mendefinisikan *domain logic* dan *ports* (traits) abstrak untuk penyimpanan, keamanan, platform, dan transportasi.
- Crate infrastruktur (`hime-storage`, `hime-security`, `hime-pal`, `hime-ipc`) mengimplementasikan *adapters* untuk trait-trait tersebut.
- `apps/hime-desktop` bertindak sebagai layer presentasi terluar yang mengonsumsi Hime Core melalui Tauri Commands dan IPC events.
- **Larangan Circular Dependencies:** Crate infrastruktur dilarang saling bergantung secara langsung; seluruh koordinasi lintas domain dilakukan melalui kontrak trait yang dikelola oleh `hime-core`.

---

## 2. Diagram Komponen & Matriks Ketergantungan (Dependency Matrix)

### A. Diagram Arsitektur Komponen (Mermaid)

```mermaid
graph TD
    subgraph Presentation_Layer ["Presentation Layer (apps/)"]
        Desktop["apps/hime-desktop<br>(Tauri 2.0 Rust Shell + React 19 UI)"]
    end

    subgraph Core_Engine ["Core Engine (crates/)"]
        Core["crates/hime-core<br>(Modular Monolith Orchestrator)"]
        CorePorts["hime-core::ports<br>(Traits: StoragePort, VaultPort, PalPort, IpcPort)"]
        Core --> CorePorts
    end

    subgraph Infrastructure_Adapters ["Infrastructure Adapters (crates/)"]
        IPC["crates/hime-ipc<br>(Binary Frame 'HIME' + JSON-RPC)"]
        PAL["crates/hime-pal<br>(Win32 DirectComposition / Linux Fallback)"]
        Storage["crates/hime-storage<br>(SQLite WAL, FTS5, 5D Safe Move)"]
        Security["crates/hime-security<br>(Windows DPAPI, Token CSPRNG)"]
    end

    subgraph External_Processes ["External Supervised Processes"]
        Hermes["Hermes Agent Subprocess<br>(Python / uv Virtualenv)"]
        Router["9Router AI Gateway<br>(Node.js 127.0.0.1:20128)"]
    end

    %% Dependency Arrows
    Desktop -->|Invokes Commands| Core
    Desktop -->|Imports Display Types| PAL

    CorePorts <|..- Storage : implements StoragePort
    CorePorts <|..- Security : implements VaultPort
    CorePorts <|..- PAL : implements PalPort
    CorePorts <|..- IPC : implements IpcPort

    Core -->|Supervises via JobObject| Hermes
    Core -->|Supervises & Proxies| Router
    Hermes -->|IPC tool_call.request| IPC
    IPC -->|Delivers to PEP| Core
```

### B. Matriks Ketergantungan Antar-Crate (Strict Dependency Rules)

| Source Crate \ Target | `hime-core` | `hime-ipc` | `hime-pal` | `hime-storage` | `hime-security` | `apps/hime-desktop` |
|:---|:---:|:---:|:---:|:---:|:---:|:---:|
| **`apps/hime-desktop`** | **ALLOW** | **ALLOW** | **ALLOW** | FORBIDDEN | FORBIDDEN | - |
| **`crates/hime-core`** | - | **ALLOW** | **ALLOW** | **ALLOW** | **ALLOW** | FORBIDDEN |
| **`crates/hime-ipc`** | FORBIDDEN | - | FORBIDDEN | FORBIDDEN | FORBIDDEN | FORBIDDEN |
| **`crates/hime-pal`** | FORBIDDEN | FORBIDDEN | - | FORBIDDEN | FORBIDDEN | FORBIDDEN |
| **`crates/hime-storage`**| FORBIDDEN | FORBIDDEN | FORBIDDEN | - | FORBIDDEN | FORBIDDEN |
| **`crates/hime-security`**| FORBIDDEN | FORBIDDEN | FORBIDDEN | FORBIDDEN | - | FORBIDDEN |

*Aturan Baku:*
- Seluruh crate infrastruktur (`hime-ipc`, `hime-pal`, `hime-storage`, `hime-security`) berstatus **Leaf Crates** relatif terhadap satu sama lain dan terhadap `hime-core`. Tidak ada saling impor antar leaf crates.
- `apps/hime-desktop` dilarang menyentuh basis data (`hime-storage`) atau secret vault (`hime-security`) secara langsung. Seluruh akses data dan keamanan wajib melalui API/Command yang diekspos oleh `hime-core`.

---

## 3. Spesifikasi Rinci Komponen Workspace

---

### 3.1. `apps/hime-desktop` (Presentation Shell)

- **Tanggung Jawab (Responsibility):**
  1. Merender antarmuka pengguna Dynamic Island HUD, Omnibox, Activity Stream, Action Approval Modal, dan dashboard kartu widget.
  2. Mengelola siklus hidup jendela desktop menggunakan Tauri 2.0 (borderless, transparent window, floating topmost canvas).
  3. Menerjemahkan input mouse dan keyboard global (`Alt+Space`) menjadi pemanggilan command ke Hime Core.
  4. Menegakkan proteksi non-aktivasi fokus ketik (`WS_EX_NOACTIVATE`) pada notifikasi sekilas.
  5. Menampilkan animasi morphing 60 FPS menggunakan React 19, Tailwind CSS, dan Motion.
- **Bukan Tanggung Jawab (Explicit Non-Responsibility):**
  1. DILARANG mengeksekusi logika orkestrasi LLM atau penalaran agent secara mandiri.
  2. DILARANG membaca atau menulis langsung ke basis data SQLite atau filesystem di luar dialog file OS.
  3. DILARANG menyimpan atau mengenkripsi API keys provider secara langsung.
  4. DILARANG meluncurkan atau mengelola subproses sistem secara mandiri.
- **Antarmuka Publik (Tauri Commands):**
  ```rust
  #[tauri::command]
  pub async fn omnibox_submit(prompt: String, session_id: String) -> Result<TaskAck, CommandError>;

  #[tauri::command]
  pub async fn action_approval_respond(request_id: String, approved: bool) -> Result<(), CommandError>;

  #[tauri::command]
  pub async fn morph_hud_state(target_state: HudState) -> Result<(), CommandError>;
  ```
- **Modul Internal:**
  - `src-tauri/src/window/`: Subclassing Win32 HWND window procedure, hit-test filter (`HTTRANSPARENT`).
  - `src-tauri/src/bridge/`: Event emitter dari Hime Core async runtime ke WebView2 frontend.
  - `src/components/`: Komponen React 19 (HUD, Omnibox, ActivityStream, ApprovalModal, Widgets).
  - `src/motion/`: Kurva spring physics dan state transitions.
- **Ketergantungan yang Diperbolehkan:** `hime-core`, `hime-pal`, `hime-ipc`, `tauri`, `serde`, `serde_json`.
- **Ketergantungan yang Dilarang:** `rusqlite`, `ring`, `windows-sys::Win32::Security::Cryptography`.
- **Batas Data & Error:** Menerima DTO (*Data Transfer Objects*) via IPC Tauri; kesalahan dirender menjadi UI toast/error banner yang aman tanpa stack trace internal.

---

### 3.2. `crates/hime-core` (Modular Monolith Engine)

- **Tanggung Jawab (Responsibility):**
  1. Bertindak sebagai *Policy Decision Point (PDP)* & *Policy Enforcement Point (PEP)* terpusat bagi seluruh pemanggilan alat (*tool executions*).
  2. Mengelola siklus hidup subproses Hermes Agent dan 9Router melalui Process Supervisor (Job Object).
  3. Mengorkestrasi alur tugas pengguna, task queues FIFO, dan konkurensi (tepat 1 tugas aktif pada MVP).
  4. Mengelola Context & Memory Manager (FTS5 indexing dan context injection).
  5. Mengelola *Authenticating Reverse Proxy* lokal untuk mengamankan 9Router loopback port.
  6. Menegakkan jeda ergonomi 500 ms anti-slip pada Action Approval Modal sebelum persetujuan dieksekusi.
- **Bukan Tanggung Jawab (Explicit Non-Responsibility):**
  1. DILARANG memuat kode rendering visual UI (HTML, CSS, DirectX, WebView).
  2. DILARANG memodifikasi kode sumber internal Hermes atau 9Router.
  3. DILARANG menjalankan pemanggilan sistem berkas level rendah tanpa melalui `hime-storage`.
- **Antarmuka Publik (Traits & Engine API):**
  ```rust
  pub trait PolicyEngine: Send + Sync {
      fn evaluate_tool_request(&self, request: &ToolCallRequest) -> PolicyDecision;
  }

  pub trait AgentOrchestrator: Send + Sync {
      async fn dispatch_task(&self, task: AgentTask) -> Result<TaskReceipt, OrchestrationError>;
      async fn cancel_task(&self, task_id: &str) -> Result<(), OrchestrationError>;
  }

  pub struct HimeEngine { /* Modular domains coordinated via channels */ }
  ```
- **Modul Internal (Anti-God-Object Structure):**
  - `orchestrator/`: Logika antrean tugas, koordinasi reasoning loop Hermes.
  - `policy/`: Rule engine perizinan, verifikasi tanda tangan tool, mitigasi jailbreak.
  - `supervisor/`: Process manager pembungkus Win32 Job Object dan socket verifier.
  - `context/`: Token budget manager, perakit prompt context injection.
  - `proxy/`: Authenticating loopback reverse proxy ke port 20128.
- **Ketergantungan yang Diperbolehkan:** `hime-ipc`, `hime-pal`, `hime-storage`, `hime-security`, `tokio`, `tracing`, `async-trait`, `thiserror`.
- **Ketergantungan yang Dilarang:** `tauri`, `tauri-runtime`, `winit`, GUI crates.
- **Hubungan dengan Hermes & 9Router:** Membuka Named Pipe server untuk Hermes; menyuntikkan konfigurasi via environment block; memverifikasi kepemilikan PID socket TCP 20128 9Router; menjadi gerbang tunggal permintaan model.

---

### 3.3. `crates/hime-ipc` (Binary Framing & Transpor)

- **Tanggung Jawab (Responsibility):**
  1. Mengimplementasikan encoding dan decoding biner frame `HIME` (12-byte header).
  2. Melakukan validasi panjang payload sebelum alokasi heap (maksimum $16\text{ MB}$, tolak seketika jika melanggar).
  3. Mengelola transpor Windows Named Pipe (`\\.\pipe\hime-hermes`) dengan DACL eksklusif Current User SID.
  4. Mengelola transpor Linux Unix Domain Socket (`$XDG_RUNTIME_DIR/hime/hermes.sock`, mode `0600`).
  5. Menegakkan autentikasi handshake token CSPRNG 256-bit dan sequence nonce strictly monotonic ($N_{\text{msg}} = N_{\text{last}} + 1$).
  6. Menyediakan abstraksi stream framing untuk JSON-RPC 2.0 messages.
- **Bukan Tanggung Jawab (Explicit Non-Responsibility):**
  1. DILARANG mengevaluasi izin atau kebijakan tool call (tanggung jawab `hime-core`).
  2. DILARANG mengakses basis data atau secret vault.
- **Antarmuka Publik:**
  ```rust
  pub struct IpcServer { /* Named Pipe / UDS Listener */ }
  pub struct IpcStream { /* Framed Async Read/Write */ }

  #[derive(Debug, PartialEq, Eq)]
  pub struct HimeFrame {
      pub header: FrameHeader,
      pub payload: Bytes,
  }

  impl IpcStream {
      pub async fn read_frame(&mut self) -> Result<HimeFrame, IpcError>;
      pub async fn write_frame(&mut self, frame: HimeFrame) -> Result<(), IpcError>;
  }
  ```
- **Ketergantungan yang Diperbolehkan:** `bytes`, `tokio`, `tokio-util`, `serde`, `serde_json`, `thiserror`, `windows-sys` (Win32 Pipes & Security).
- **Ketergantungan yang Dilarang:** Seluruh crate internal Hime lainnya (`hime-core`, `hime-storage`, dll.).

---

### 3.4. `crates/hime-pal` (Platform Abstraction Layer)

- **Tanggung Jawab (Responsibility):**
  1. Mengisolasi seluruh pemanggilan API spesifik OS (Windows Win32 / DirectComposition / DWM vs Linux X11 / Wayland).
  2. Mengimplementasikan hook `WM_NCHITTEST` untuk tembus-klik presisi (`HTTRANSPARENT`).
  3. Mendaftarkan shortcut keyboard global OS (`RegisterHotKey` Win32 `Alt+Space` / X11 grab key).
  4. Mengimplementasikan deteksi multi-monitor, koordinat primary display, dan safe display rects.
  5. Mengimplementasikan mekanisme fallback windowing untuk Linux Wayland (GNOME Floating Utility vs wlroots layer-shell).
- **Bukan Tanggung Jawab (Explicit Non-Responsibility):**
  1. DILARANG memuat state machine presentasi atau UI rendering.
  2. DILARANG mengelola persistensi berkas atau logika bisnis.
- **Antarmuka Publik:**
  ```rust
  pub trait PlatformWindowHook: Send + Sync {
      fn attach_hit_test_filter(&self, hwnd: isize) -> Result<(), PalError>;
      fn register_global_hotkey(&self, key: KeyCombo) -> Result<HotKeyHandle, PalError>;
      fn get_primary_screen_work_area(&self) -> Result<Rect, PalError>;
  }
  ```
- **Ketergantungan yang Diperbolehkan:** `windows-sys` (Win32 Foundation, UI, DWM, DirectComposition), `thiserror`.
- **Ketergantungan yang Dilarang:** `hime-core`, `hime-storage`, `hime-security`.

---

### 3.5. `crates/hime-storage` (Database & Filesystem Safety)

- **Tanggung Jawab (Responsibility):**
  1. Mengelola koneksi SQLite lokal dalam mode WAL (*Write-Ahead Logging*): single dedicated writer connection + pool connection pembacaan konkuren.
  2. Mengelola migrasi skema basis data dan tabel virtual FTS5 untuk pencarian semantik dokumen/percakapan.
  3. Mengimplementasikan mesin pemindahan berkas aman 5-Dimensi:
     - Partisi sama: atomic rename OS (`MoveFileExW(MOVEFILE_REPLACE_EXISTING)` / `renameat2`).
     - Lintas partisi: copy $\rightarrow$ SHA-256 verify $\rightarrow$ disk flush (`FlushFileBuffers`/`fsync`) $\rightarrow$ safe source delete.
  4. Mengelola jurnal pembatalan 30 hari (`storage_journal`) dan direktori safety trash.
  5. Mendeteksi berkas duplikat berdasarkan 5 kategori bukti.
- **Bukan Tanggung Jawab (Explicit Non-Responsibility):**
  1. DILARANG memutuskan kebijakan penghapusan berkas tanpa otorisasi dari `hime-core`.
  2. DILARANG mengekspos koneksi SQLite mentah ke layer presentasi.
- **Antarmuka Publik:**
  ```rust
  pub trait StorageService: Send + Sync {
      fn execute_transaction<F, T>(&self, f: F) -> Result<T, StorageError>
      where F: FnOnce(&rusqlite::Transaction) -> Result<T, StorageError>;

      async fn safe_move_file(&self, source: &Path, destination: &Path) -> Result<MoveReceipt, StorageError>;
      async fn rollback_undo(&self, journal_id: i64) -> Result<(), StorageError>;
  }
  ```
- **Ketergantungan yang Diperbolehkan:** `rusqlite` (bundled, fts5), `sha2`, `tokio`, `thiserror`, `tracing`, `windows-sys` (Storage Win32).
- **Ketergantungan yang Dilarang:** `hime-core`, `apps/hime-desktop`.

---

### 3.6. `crates/hime-security` (Secret Vault & Cryptography)

- **Tanggung Jawab (Responsibility):**
  1. Mengelola enkripsi dan dekripsi kredensial at-rest menggunakan Windows DPAPI (`CryptProtectData`/`CryptUnprotectData`) pada lingkup `CurrentUser`.
  2. Menyediakan abstraksi fallback enkripsi Linux (libsecret / Secret Service API).
  3. Menghasilkan token acak berkekuatan tinggi via CSPRNG (minimal 256-bit entropy) untuk inisialisasi password 9Router dan token IPC.
  4. Mengelola in-memory credential storage dengan zeroization (`zeroize` crate) saat buffer didrop untuk meminimalkan paparan memori.
- **Bukan Tanggung Jawab (Explicit Non-Responsibility):**
  1. DILARANG menyimpan master password atau enkripsi kunci secara hardcoded pada binary.
  2. DILARANG memodifikasi file `data.sqlite` milik 9Router secara langsung di disk.
- **Antarmuka Publik:**
  ```rust
  pub trait SecretVault: Send + Sync {
      fn store_secret(&self, key: &str, secret: &[u8]) -> Result<(), VaultError>;
      fn retrieve_secret(&self, key: &str) -> Result<Zeroizing<Vec<u8>>, VaultError>;
      fn generate_csprng_token(&self, byte_length: usize) -> Result<String, VaultError>;
  }
  ```
- **Ketergantungan yang Diperbolehkan:** `ring`, `zeroize`, `windows-sys` (Win32 Security Cryptography), `thiserror`.
- **Ketergantungan yang Dilarang:** `hime-core`, `rusqlite`, `tauri`.

---

## 4. Evaluasi Arsitektural: Mencegah God-Object Monolith pada Hime Core

Untuk menjamin `hime-core` tidak membengkak menjadi modul monolithic yang tidak terpelihara (*God-Object Anti-Pattern*), modularisasi ditegakkan melalui 3 aturan arsitektur:
1. **Sub-Domain Decoupling via Tokio Message Bus:**
   Setiap sub-modul internal (`Orchestrator`, `PolicyEngine`, `ProcessSupervisor`, `ContextManager`) berjalan sebagai task mandiri di dalam Tokio runtime dan berkomunikasi melalui message-passing (Tokio bounded `mpsc` channel), bukan memanggil mutasi state langsung pada struct bersama.
2. **Trait Decoupling (Inversion of Control):**
   Tidak ada komponen domain yang memanggil implementasi konkret infrastruktur secara langsung. Setiap interaksi eksternal diarahkan melalui trait port yang terinjeksi saat bootstrap (`Arc<dyn StoragePort>`, `Arc<dyn SecretVault>`).
3. **Immutability by Default:**
   State global dihindari mutlak. State operasional dikapsulasi di dalam aktor/state machine masing-masing domain (misal `SupervisorState`, `AgentExecutionState`).
