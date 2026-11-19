# 06 — INTEGRATION ARCHITECTURE (Hime Core ⇄ Bloom Shell)

> Semua detail di sini adalah **rancangan awal**. Setelah Stage H0 (kamu membaca source Bloom), validasi dan perbaiki; catat perubahan sebagai ADR (`docs/adr/ADR-013-shell-fork-bloom.md` dan seterusnya).

## 1. Struktur repository target

```
Hime-Bloom/                         # hasil klon Bloom (pin tag), remote upstream dipertahankan
├── src/                            # frontend Bloom (React 19 + TS)  [JANGAN refactor]
│   ├── main.tsx App.tsx Dock.tsx Overlay.tsx Settings.tsx ...   (milik Bloom)
│   └── hime/                       # <— SEMUA frontend Hime di sini
│       ├── api/                    # pembungkus invoke bertipe (hime_*)
│       ├── events/                 # hook langganan event hime-*
│       ├── state/                  # state Hime (ikuti pola state Bloom)
│       ├── panels/                 # panel Island (Assistant, Tasks, Storage, ...)
│       ├── bubbles/                # live-activity bubble (agent, approval, translate, ...)
│       ├── settings/               # tab pengaturan Hime
│       ├── workspace/              # isi window Hime Workspace
│       └── types/                  # tipe DTO (di-port dari klien IPC lama)
├── src-tauri/                      # backend shell Bloom (Rust)  [JANGAN refactor]
│   └── src/
│       ├── main.rs commands.rs services.rs utils.rs state.rs types.rs  (milik Bloom)
│       └── hime/                   # <— SEMUA Rust shell-Hime di sini
│           ├── mod.rs              # init + registrasi
│           ├── client.rs           # klien IPC ke Core (Named Pipe)
│           ├── commands.rs         # #[tauri::command] hime_*  (didaftarkan di main.rs)
│           ├── services.rs         # supervisi Core, reconnect, forwarder event
│           ├── translate.rs        # tangkap teks terpilih (Fase 09R)
│           └── ...
├── core/                           # Hime Core (workspace Rust TERPISAH, hasil panen)
│   ├── Cargo.toml
│   └── crates/ hime-core hime-ipc hime-pal hime-security hime-storage ...
├── hime.html                       # entri window Workspace (baru, Fase 09R+)
├── docs/ hime/ product/ architecture/ adr/ reference/ security/ legal/ history/ agent-pack/
├── scripts/                        # skrip Bloom + skrip Hime (restore-taskbar, run-safe, build-core, check-env)
└── AGENTS.md                       # Bloom + bagian "Hime" di akhir
```

## 2. Hime Core: proses terpisah (kenapa dan bagaimana)

- **Kenapa:** batas keamanan (PEP, vault, Tool Gateway, supervisor Hermes/9Router) tidak boleh bergantung pada shell yang menghooking input, menyembunyikan taskbar, dan bisa di-restart oleh watchdog. Core boleh hidup lebih bersih dan lebih mudah diuji.
- **Peluncuran:** shell memastikan Core berjalan saat startup: (1) cek instance (named mutex/pipe), (2) bila belum ada → spawn sebagai **sidecar** (mekanisme Tauri `externalBin` atau eksekusi dari direktori aplikasi; putuskan setelah H0), (3) tunggu pipe siap dengan batas waktu, (4) handshake. Jika gagal → shell **lanjut normal** (mode Bloom murni) dengan state "Hime offline" yang jujur dan coba lagi dengan backoff (batas).
- **Umur Core:** default Core **berakhir bersama shell** (Job Object `kill-on-close`), shutdown anggun lewat IPC (batas ≤2 dtk) lalu paksa. Opsi `hime-core-background=true` (default `false`) bila nanti dibutuhkan pengingat tanpa shell.
- **Hak akses:** tanpa admin. Pipe `\\.\pipe\hime-ipc` dengan DACL user saat ini (sudah ada); token sesi dibuat Core, diteruskan ke shell lewat kanal peluncuran (stdin/handle terwarisi), **tidak** lewat argumen baris perintah/env yang tampak di Task Manager, **tidak** pernah ke webview.
- **Data:** `%APPDATA%/hime/` (DB SQLite WAL, blob vault DPAPI, log). Settings shell tetap pola Bloom.

## 3. Bridge di shell

### 3.1 Klien IPC
- Opsi A (disukai): `src-tauri` memakai crate `hime-ipc` sebagai **path-dependency** (`../core/crates/hime-ipc`) agar framing/protokol tidak digandakan. **Syarat:** tidak menarik dependensi yang mengubah versi `windows` 0.61 atau memperbesar build secara tidak wajar; `cargo check --locked` tetap lulus.
- Opsi B: klien minimal sendiri di `src-tauri/src/hime/client.rs` yang mengimplementasikan framing + JSON-RPC dari spesifikasi (dengan test vector bersama). Pilih A kecuali terbukti bermasalah; catat di ADR.
- Perilaku: koneksi tunggal multipleks, ID request unik, timeout per request, reconnect backoff, deteksi putus, **tidak ada panggilan sinkron yang memblok thread UI**.

### 3.2 Command (shell → Core) — awal
Nama `hime_*`, didaftarkan di `generate_handler!` (`main.rs`), logika di `src-tauri/src/hime/commands.rs`. Semua memvalidasi input di sisi bridge sebelum meneruskan ke Core. Contoh (final saat H2/08R):
`hime_core_status`, `hime_conversation_list`, `hime_conversation_open`, `hime_message_send`, `hime_message_cancel`, `hime_approval_list`, `hime_approval_resolve`, `hime_task_list/create/update`, `hime_activity_list`, `hime_models_list`, `hime_runtime_health`, `hime_settings_get/set` (hanya pengaturan **non-rahasia** milik Core).

### 3.3 Event (Core → shell → frontend) — awal
Bridge menerima event Core (event model legacy 03 §4.2) dan **mem-forward** sebagai `app.emit("hime-…")`:

| Event shell | Sumber Core | Dipakai untuk |
|---|---|---|
| `hime-core-status` | koneksi/handshake/health | indikator Island ("offline"/"siap") |
| `hime-agent-status` | `agent.status` | bubble aktivitas agent |
| `hime-message-chunk` / `hime-message-done` | `agent.message.*` | streaming di panel Assistant |
| `hime-tool-event` | `tool.*` | Activity stream |
| `hime-approval-requested` / `hime-approval-resolved` | `approval.*` | bubble approval |
| `hime-task-changed` | `task.*` | panel Tasks, bubble progres |
| `hime-runtime-health` | `runtime.health` | AI Control (settings) |
| `hime-file-event` | `file.*` | prompt unduhan (Fase 10R) |
| `hime-device-event` | `device.*` | perangkat (Fase 11R) |
| `hime-nudge` | produktivitas | nudge/check-in (Fase 12R) |
| `hime-translation` | translate | bubble terjemahan |
| `hime-error` | `error` | toast/bubble error (tanpa membocorkan rahasia) |

Event membawa `correlation_id`, `seq`, `schema_version`; frontend mendedup berdasar ID; **kapasitas buffer terbatas**; event berukuran besar tidak melewati emit (ambil via command).

### 3.4 Supervisi Core (`hime/services.rs`)
Pola mengikuti `services.rs` Bloom: pekerja berumur panjang, tidak aktif saat tak dibutuhkan, berhenti saat shutdown dimulai ("stop touching windows/taskbar once shutdown begins" ala Bloom). Tidak menambah jalur exit.

## 4. Titik sentuh yang diperkirakan (validasi di H0; semua dicatat di `docs/hime/TOUCHPOINTS.md`)

| Berkas Bloom | Perubahan minimal |
|---|---|
| `src-tauri/src/main.rs` | `mod hime;` + `hime::init(...)` di setup + tambah command `hime_*` ke `generate_handler!` + hook shutdown Core pada jalur exit yang sudah ada |
| `src-tauri/tauri.conf.json` | window `hime` (Workspace; on-demand), bundle (sidecar Core), **updater dinonaktifkan/diarahkan**, productName/identifier (rebrand, H2), CSP (**tidak menambah origin**) |
| `src-tauri/capabilities/default.json` | izin untuk window `hime` dan plugin baru (bila ada) |
| `src-tauri/Cargo.toml` | path-dependency `hime-ipc` (bila Opsi A), fitur `windows` hanya jika benar-benar dibutuhkan |
| `src/App.tsx` | daftarkan panel Island Hime + bubble Hime (satu baris/pemanggilan pendaftaran; logika di `src/hime/`) |
| `src/Settings.tsx` | daftarkan tab "Hime" (isi di `src/hime/settings/`) |
| `vite.config.ts` | input `hime.html` |
| `hime.html` + `src/hime/workspace/main.tsx` | entri window Workspace |
| `SETTINGS.md`, `AGENTS.md` | dokumentasi kunci `hime-*` dan bagian "Hime" |
| `.github/workflows/*` | job tambahan untuk `core/` (cargo check/clippy/test) — lokal saja, **tanpa push** |

Jika sebuah integrasi mengharuskan menyentuh **banyak** berkas Bloom → hentikan, cari titik ekstensi yang lebih tipis, atau buat ADR yang menjelaskan.

## 5. Window baru "Hime Workspace"
Island dan dock untuk informasi ringkas dan aksi cepat (aturan Bloom). UI besar (Kanban, riwayat Activity, dashboard Storage, Documents, Devices, AI Control Center) membutuhkan ruang → **satu window baru `hime`** berpola sama dengan `settings` (window biasa tetapi bergaya Bloom, **bukan** Island): dibuka dari bubble/panel Island atau shortcut, dibuat **on-demand** dan ditutup/dibebaskan saat tidak dipakai (hemat RAM WebView2). Ikuti desain window Settings Bloom (chrome, bayangan, tipografi) — cari tahu di H0 seperti apa itu.

## 6. Pengaturan Hime
- Kunci `hime-*` di `%APPDATA%/…/settings.json` (pola Bloom: datar, string, `localStorage` → `save_setting`). **Pengaturan non-rahasia saja** (toggle fitur, bahasa target terjemahan, mode nudge, ambang waktu layar, dll.).
- Rahasia (API key, token OAuth) → vault Core via flow terpisah (input di UI → command → Core vault; **tidak** tersimpan di `localStorage`/settings.json; field dikosongkan setelah kirim; tidak muncul di log/event).
- Kebijakan keamanan (izin, root file yang diizinkan, domain browser) → disimpan Core, bukan settings shell.

## 7. Keamanan sisi shell
- Webview hanya memuat **bundle lokal**; CSP ketat; **tanpa konten remote**; teks dari model/web/dokumen dirender sebagai **teks/markdown tersanitasi** (tanpa `dangerouslySetInnerHTML` pada data tak tepercaya).
- Capabilities Tauri minimal; command `hime_*` memvalidasi tipe/panjang/rentang; command berbahaya tidak diekspos ke window selain yang perlu.
- **Keputusan approval berasal dari aksi pengguna yang nyata di UI** (klik/tombol) dan tetap memerlukan tiket Core (TTL, sekali pakai, anti-replay). Webview tidak punya jalur untuk menyetujui otomatis tanpa tiket.
- Shell tidak menyimpan/melihat plaintext kredensial setelah dikirim ke Core.
- Pastikan tidak ada telemetri/pengiriman data Bloom/Hime ke luar tanpa sepengetahuan owner (audit H0).

## 8. Build, versi, CI
- **Core:** `scripts/build-core.ps1` (cargo build --release di `core/`, salin exe ke lokasi sidecar). Gerbang Core: legacy 04 §7.
- **Shell:** gerbang Bloom (berkas 04 §7). `bun run tauri build` menghasilkan installer yang menyertakan Core (sidecar) dan manifest kompatibilitas (versi Hermes, 9Router, skema DB, protokol IPC, **tag/SHA Bloom dasar**).
- **Versi:** gunakan `bun run bump` Bloom (empat berkas). **Jangan** menjalankan `bun run release` (melakukan push).
- **Updater:** nonaktif sampai ada saluran pembaruan Hime sendiri (Fase 15R); jangan pernah mengarah ke rilis Bloom.

## 9. Kontrak dan pengujian
- Kontrak command/event didokumentasikan di `docs/api/SHELL-CORE-CONTRACT.md` dan diuji: (a) test integrasi `core/` yang menjalankan Core dan klien bridge (pengganti "Desktop Integration"), (b) `cargo test --locked` untuk modul `hime/` di shell (parsing, validasi, state machine koneksi), (c) uji visual (berkas 04 §6).
- Uji kegagalan wajib: Core tidak ada, Core crash saat request, pipe putus, event banjir, token salah, versi skema tak cocok, shell restart saat Core hidup.
