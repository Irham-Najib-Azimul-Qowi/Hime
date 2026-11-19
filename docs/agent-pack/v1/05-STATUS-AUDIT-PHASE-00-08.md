# 05 — STATUS AUDIT: FASE 00–08

## 0. Cara membaca dokumen ini (penting)

Isi dokumen ini berasal dari **ekspor percakapan perancangan** (6–10 November 2026), yaitu dari prompt yang dikirim kepada agent, ringkasan laporan agent yang dikutip, dan **tinjauan System Architect (ChatGPT) atas laporan-laporan itu**. Artinya:

- Semua status di bawah adalah **klaim sekunder** (`INFERRED` dari sudut pandang paket ini). Penulis paket **tidak melihat repository, kode, maupun laporan aslinya**.
- Tinjauan Architect sendiri berkali-kali menyatakan bahwa ia menilai *laporan eksekusi*, bukan memeriksa berkas langsung.
- Tugasmu: **membuktikan atau membantah** setiap klaim dengan memeriksa repository aktual (Langkah 0 di Master Prompt). Hasil resminya ditulis ke `project-management/audits/PHASE-00-08-INDEPENDENT-AUDIT.md`.

Lokasi repository menurut laporan: `x:/folder_ai/Hime` (mesin pengembangan Windows). Jika lokasi/struktur berbeda, gunakan yang nyata.

## 1. Ringkasan status yang dilaporkan

| Fase | Nama (menurut repository) | Status dilaporkan | Bukti yang dilaporkan | Kekuatan bukti (menurut paket ini) |
|---|---|---|---|---|
| 00 | Project Governance & Foundation | ACCEPTED | Dokumen governance, `AGENTS.md`, `PROJECT_STATUS.md`, `BACKLOG.md`, git init | Sedang — administratif, mudah diverifikasi |
| 01 | Hardware, Environment & Security Discovery | ACCEPTED | `PHASE-01-ACCEPTANCE.md`, baseline hardware, `RISK_REGISTER.md`, `SECURITY-BASELINE.md` | Sedang |
| 02 | Nusa Reverse Engineering & Product Discovery | READY FOR OWNER ACCEPTANCE → diterima bersyarat | Task 02-01…02-03 (+kalibrasi), laporan RE Nusa | Lemah–sedang: UX dengan "documented limitations" |
| 03 | Product Requirements & System Architecture | READY FOR OWNER ACCEPTANCE → diterima bersyarat | PRD, UX architecture, System Architecture, 12 ADR, 33 FR, 18 NFR, 16 constraints, 2 blockers | Sedang |
| 04 | Engineering Planning | QUALIFIED ACCEPTANCE | 8 spesifikasi + master index; laporan validasi arsitektural | Sedang (spesifikasi, belum terbukti implementasi) |
| 05 | Development Readiness | QUALIFIED ACCEPTANCE | `check-env.ps1`, `build.ps1`, `test.ps1`, `dev.ps1`, security baseline | Sedang (toolchain belum ada saat itu) |
| 06 | Engineering Foundation | QUALIFIED ACCEPTANCE | Rust workspace, Tauri shell, IPC, PEP/PDP, DPAPI vault, supervisor; 33/33 test | Sedang |
| 07 | Hime Desktop Experience | QUALIFIED ACCEPTANCE; owner menjawab "ya" untuk lanjut | 37/37 test; build frontend; runtime sebagian | Sedang; **GUI belum terbukti** |
| **08** | **AI Runtime Integration** | **Prompt dikirim (10 Nov 2026 14:46). Tidak ada laporan di ekspor.** | **Tidak ada** | **Tidak diketahui** |

> Catatan penomoran: Rencana induk awal (HEMP) memiliki 16 fase dengan Fase 04 = Product/System Spec dan Fase 05 = Architecture/Security. Dalam eksekusi nyata, pekerjaan produk dan arsitektur diselesaikan di dalam Fase 02–03, sehingga **Fase 04 = Engineering Planning** dan **Fase 05 = Development Readiness**. Rekonsiliasi dilakukan pada 9–10 November (dokumen: `project-management/ROADMAP-RECONCILIATION-AND-EXECUTION-PLAN.md`). **Gunakan penomoran repository.** Fase 06 dan seterusnya selaras dengan HEMP.

## 2. Rincian per fase: apa yang harus ada dan apa yang harus kamu periksa

### Fase 00 — Governance
**Seharusnya ada:** repo Git, `AGENTS.md` (aturan agent), `PROJECT_STATUS.md`, `BACKLOG.md`, struktur `project-management/`, konvensi penamaan, aturan version control.
**Periksa:** `git log`, `git status`; `AGENTS.md` ada dan konsisten dengan `04-…`; tidak ada berkas rahasia ter-track; `.gitignore` memadai.
**Kewajiban dibawa:** tentukan baseline commit (lihat bagian 4, butir G1).

### Fase 01 — Discovery & Environment
**Seharusnya ada:** baseline hardware (CPU i5-8250U, RAM ±20,8 GB, UHD 620, Windows 11 Pro, layar 1366×768), audit software, audit Nusa/Hermes/9Router awal, `RISK_REGISTER.md` (mis. RSK-10, RSK-12), `SECURITY-BASELINE.md`, `PHASE-01-ACCEPTANCE.md`.
**Periksa:** berkas ada; `RISK_REGISTER.md` memuat risiko keamanan 9Router (plaintext credential, bypass localhost, default password) dengan status terkini; baseline hardware sesuai kondisi mesin saat ini (`systeminfo`, `wmic`/`Get-CimInstance`).
**Catatan:** pengguna semula menyebut RAM 16 GB; audit melaporkan ±20,8 GB. Gunakan angka terukur, catat keduanya.

### Fase 02 — Nusa Reverse Engineering & Product Discovery
**Seharusnya ada:** `TASK-02-01` (discovery biner), `02-02` (arsitektur & protokol, + koreksi bukti 02A/02B), `02-03` (UX & perilaku, + kalibrasi 03B), `02-04` (PRD & Experience Architecture, + 04B/C/D), `02-05` (System Architecture, + 05A); `NUSA-BINARY-ARCHITECTURE.md`, `NUSA-COMMUNICATION-ANALYSIS`, feature parity matrix.
**Periksa:** setiap fitur Nusa punya status Preserve/Improve/Replace/Remove/New; tidak ada fitur hilang tanpa keputusan; catatan **documented limitations** pada UX terdaftar; aset Nusa tidak disalin ke repo (hak cipta).
**Kewajiban dibawa:** dokumen RE hanya referensi UX, bukan spesifikasi yang harus ditiru; reimplementasi independen.

### Fase 03 — Product Requirements & System Architecture
**Seharusnya ada:** PRD, UX Architecture, System Architecture, **12 ADR**, Implementation Roadmap, **16 Implementation Constraints**, **2 Implementation Blockers (IB-01, IB-02)**, **33 FR**, **18 NFR**; `PHASE-02-ACCEPTANCE.md`, `PHASE-03-ACCEPTANCE.md`, `PHASE-02-03-CONSOLIDATED-CLOSURE-REPORT.md`.
**Periksa:** jumlah ADR/FR/NFR/constraint cocok; tiap FR memiliki ID unik dan acceptance criteria; IB-01 dan IB-02 **tetap OPEN** di semua dokumen status; tidak ada ADR yang kontradiktif (terutama ADR-006 tentang kredensial).

### Fase 04 — Engineering Planning
**Seharusnya ada (8 spesifikasi + master index):**
1. Component Boundaries (6 crate, Ports & Adapters, bounded channels)
2. IPC Protocol (framing biner 12-byte, JSON-RPC 2.0, token + nonce/sequence)
3. Process Supervisor / `SUBPROCESS-LIFECYCLE-SPECIFICATION` (Job Object, `CREATE_SUSPENDED`)
4. `9ROUTER-CREDENTIAL-SECURITY-SPECIFICATION` (IB-01)
5. `9ROUTER-LOCALHOST-SECURITY-SPECIFICATION` (IB-02)
6. Storage/Filesystem Safety (staging, hashing, flush, rename)
7. Observability (redaksi 4 lapis)
8. QA & Definition of Done (traceability 33 FR / 18 NFR)
+ `PHASE-04-ARCHITECTURAL-VALIDATION-REPORT.md` (koreksi terarah).
**Temuan Architect yang HARUS tercermin di dokumen (periksa satu per satu):**
- Desain "In-Flight Credential Proxy dengan dummy key" **tidak terbukti** layak → arah yang dipilih: **Hime-Owned Provider Adapter** (9Router Custom Node → provider adapter milik Hime → secret vault → provider). Kompatibilitas Custom Node (streaming, discovery, error, retry, auth) **belum terbukti**.
- `X-Hime-Internal-Secret` **tidak memberi keamanan** jika 9Router tidak memvalidasinya; port ephemeral dan verifikasi PID **bukan autentikasi**.
- `dashboardGuard.js` punya bypass request localhost; `requireApiKey` saja belum cukup — harus dibuktikan autentikasi berjalan **sebelum** bypass lokal.
- Klaim "Zero Orphan Processes" harus dibatasi pada proses yang berhasil masuk Job Object dan tidak breakaway.
- Pemindahan sumber ke trash 30 hari **tidak otomatis aman** (butuh kebijakan persetujuan, pemulihan, penanganan gagal).
- `Zeroize` tidak menjamin semua salinan secret (HTTP client, buffer, runtime) terhapus.
- Target latensi proxy "<2 ms" tanpa benchmark → **bukan** acceptance requirement.
- Linux: `PR_SET_PDEATHSIG`, subreaper, cgroups v2 tidak boleh diasumsikan tersedia untuk non-root.

### Fase 05 — Development Readiness
**Seharusnya ada:** `scripts/check-env.ps1` (tidak memasang/mengubah apa pun; exit code deterministik), `build.ps1`, `test.ps1`, `dev.ps1`, matriks kompatibilitas dependensi, security baseline, konfigurasi dev lokal, manifest Cargo workspace.
**Keterbatasan yang dilaporkan:** saat itu Cargo belum tersedia sehingga workspace belum bisa diverifikasi; `dev.ps1` — pembersihan proses belum terbukti ketika proses benar-benar berjalan; `test.ps1` tidak boleh mengklaim lulus jika tidak menjalankan test.
**Periksa:** jalankan `scripts/check-env.ps1` dan catat exit code; semua toolchain terpasang (lihat Fase 06).

### Fase 06 — Engineering Foundation
**Toolchain (diremediasi 10 Nov):** Visual Studio Build Tools 2022 (MSVC v143, Windows SDK), Rust/Cargo (target `x86_64-pc-windows-msvc`), Node.js, pnpm, Python, uv — dilaporkan PASS. `cargo test` awal "0 passed" (belum ada test) lalu diganti test nyata.
**Dilaporkan terbangun:** Cargo workspace (6 crate), Hime Core, Tauri 2 Desktop Shell, IPC framing + JSON-RPC, Named Pipe transport, PEP/PDP dasar, Windows Process Supervisor (Job Objects), DPAPI vault + round-trip test, secret redaction, config & storage foundation, logging terstruktur. Vertical slice: Desktop → IPC → Core → response → graceful shutdown.
**Test dilaporkan:** **33/33** — Hime Core 2, Core Integration 7, Desktop Integration 3, Hime IPC 14, Hime PAL 1, Hime Security 5, Hime Storage 1. (Laporan awal menyebut 31 — sudah direkonsiliasi menjadi 33.)
**Tanda bintang dari Architect:** PASS untuk IPC, PEP/PDP, supervisor, vault, redaksi bersifat "berdasarkan laporan, bukan inspeksi independen".
**Periksa:** `cargo build --workspace`, `cargo test --workspace` (hitung ulang); baca `hime-security` (apakah PEP benar-benar memblokir?), `hime-pal` (satu test saja — memadai?), supervisor (uji Job Object sungguhan: matikan induk, apakah anak mati?), vault (round trip + pastikan tidak ada plaintext di disk/log).
**Kewajiban dibawa:** `hime-pal` dan `hime-storage` masih tipis (1 test masing-masing); pastikan jumlah crate/nama sesuai spesifikasi dan tidak ada God Object `hime-core`.

### Fase 07 — Hime Desktop Experience
**Dilaporkan terbangun:** Dynamic Island (state machine: Idle, Expanded, Agent Activity), Omnibox (navigasi keyboard Arrow/Enter/Escape), Conversation Interface (belum terhubung ke Hermes — ditandai rencana Fase 08), Activity Stream (event audit, dedup berdasarkan ID), Task Board, Notification Center, Settings, Floating Companion, preset jendela (Compact HUD 460×88 vs Full Dashboard 980×700), always-on-top lewat Tauri command, design tokens obsidian dark + skala animasi spring; Permission Approval Lifecycle (correlation ID, TTL 60 dtk, tiket sekali pakai, anti-replay, fail-closed).
**Test dilaporkan:** **37/37** (6 crate Rust; sempat disebut 36 pada laporan awal — direkonsiliasi) + build frontend sukses.
**Klasifikasi bukti terkoreksi (kondisi per 10 Nov 14:43):**
- `VERIFIED RUNTIME`: Core diluncurkan; Desktop diluncurkan; HWND diperoleh; Named Pipe `\\.\pipe\hime-ipc` tersedia; pembersihan proses (`PROCESS CLEANUP VERIFIED` untuk PID yang dipantau).
- `VERIFIED AUTOMATED TEST`: lifecycle approval, anti-replay, TTL, framing IPC, error saat Core offline, task management + audit ke SQLite WAL, 37 test.
- `VERIFIED STATIC`: navigasi Omnibox, state machine Island, preset jendela, always-on-top, dedup event, token desain.
- **`NOT TESTED`:** interaksi GUI Island (klik/pointer), input Omnibox, drag Floating Companion, perilaku animasi pada GPU/display, **screenshot aktual** (sesi non-interaktif tidak punya GDI), **multi-monitor dan DPI scaling** (host tunggal 1366×768).
**Kewajiban dibawa (ditutup paling lambat Fase 15, idealnya Fase 09):** verifikasi GUI interaktif lengkap (lihat `roadmap/PHASE-15-…`, Task 15.1), termasuk screenshot, multi-monitor, DPI.
**Status git yang dilaporkan:** ada perubahan tracked-modified dan untracked, tanpa commit dan tanpa push.

### Fase 08 — AI Runtime Integration
**Status:** prompt dikirim tanpa hasil di ekspor. **Anggap belum dimulai sampai audit membuktikan sebaliknya.**
**Periksa awal:** `git status`/`git log`; apakah `docs/runtime/*.md` atau `PHASE-08-*` sudah ada; `PROJECT_STATUS.md`; apakah ada adapter Hermes/9Router di kode; apakah Hermes dan 9Router terpasang dan dapat dijalankan di mesin (versi, entry point, konfigurasi).
**Rencana kerja Fase 08:** `roadmap/PHASE-08-AI-RUNTIME-INTEGRATION.md`.

## 3. Prosedur audit independen (Langkah 0.3)

Jalankan dan catat keluaran nyata (ringkas). Semua pemeriksaan non-destruktif.

```powershell
# Kondisi repository
git -C X:\folder_ai\Hime status --short --branch
git -C X:\folder_ai\Hime log --oneline -n 30
git -C X:\folder_ai\Hime diff --stat
git -C X:\folder_ai\Hime ls-files --others --exclude-standard   # untracked

# Lingkungan
.\scripts\check-env.ps1 ; "exit=$LASTEXITCODE"

# Build dan test dari bersih
cargo clean
.\scripts\build.ps1 ; "exit=$LASTEXITCODE"
.\scripts\test.ps1  ; "exit=$LASTEXITCODE"
cargo test --workspace 2>&1 | Select-String "test result"   # hitung total passed/failed/ignored

# Frontend
pnpm -C <path-desktop> install --frozen-lockfile
pnpm -C <path-desktop> build
pnpm -C <path-desktop> test        # bila ada
```

Pemeriksaan tambahan (statik dan fungsional):
1. **Dokumen vs klaim:** hitung 12 ADR, 33 FR, 18 NFR, 16 constraints; konfirmasi IB-01/IB-02 `OPEN`.
2. **Crate:** daftar crate, grafik dependensi (`cargo tree`), cari dependensi sirkular, cari pemakaian API Windows di luar `hime-pal`.
3. **Keamanan:** cari secret (pola API key, token), cari `unwrap()` di jalur produksi, periksa PEP (apakah ada jalur eksekusi tool tanpa evaluasi kebijakan), periksa validasi panjang frame sebelum alokasi, periksa TTL/anti-replay approval.
4. **Supervisor:** uji nyata: jalankan proses uji di bawah supervisor, bunuh proses induk Hime secara paksa, pastikan anak ikut berhenti (dan uji kasus breakaway).
5. **UI:** jalankan Desktop pada sesi interaktif; ambil screenshot; uji drag, Omnibox, ekspansi Island, always-on-top. Jika sesi Anda non-interaktif, tandai `NOT TESTED` dan masukkan ke kewajiban Fase 15.
6. **Test quality:** pilih 5 test acak dan baca — apakah menguji perilaku atau hanya nama? Tandai test yang "selalu lulus".
7. **Hermes dan 9Router:** apakah terpasang? Versi? Lokasi? Dapat dijalankan manual? Catat apa adanya (tanpa mengubah).

**Keluaran:** `PHASE-00-08-INDEPENDENT-AUDIT.md` dengan tabel `Klaim | Hasil periksa | Label bukti | Tindakan`. Semua selisih diklasifikasikan BLOCKER/MAJOR/MINOR. Perbaiki BLOCKER sebelum Fase 08.

## 4. Kewajiban yang dibawa ke depan (carry-forward register)

| ID | Kewajiban | Asal | Ditutup di |
|---|---|---|---|
| G1 | Tetapkan baseline Git: commit lokal baseline pada branch (setelah secret scan) agar perubahan Fase 00–07 tidak hanya berada di working tree | Fase 07 (zero commit) | Awal Fase 08 |
| G2 | Verifikasi GUI interaktif penuh + screenshot + multi-monitor + DPI | Fase 07 | Fase 09 (smoke) dan 15 (penuh) |
| G3 | IB-01 RESOLVED atau tetap diblokir | Fase 03/04 | Fase 08 |
| G4 | IB-02 RESOLVED atau tetap diblokir | Fase 03/04 | Fase 08 |
| G5 | Uji Job Object: breakaway, nested job, kegagalan assignment; ganti klaim "zero orphan" dengan klaim terbatas | Fase 04/06 | Fase 08 |
| G6 | Perluas test vector IPC: fragmentasi, konkurensi, reconnect, frame cacat, replay | Fase 04 | Fase 08–09 |
| G7 | Kebijakan lint disepakati (bukan "zero warning" kaku); tetapkan versi Rust minimum berdasarkan kebutuhan Tauri/dependensi | Fase 05/06 | Fase 08 |
| G8 | Traceability FR/NFR → test yang benar-benar berjalan | Fase 04 | Fase 09 dst. |
| G9 | Kebijakan trash/pemulihan operasi file (persetujuan, recovery, kegagalan) | Fase 04 | Fase 10 |
| G10 | Dukungan Linux di PAL (UDS, Secret Service, supervisor) tanpa asumsi cgroups | Fase 04 | Fase 15 |
| G11 | Lisensi dan aset Nusa: tidak ada aset berhak cipta tersalin | Fase 02 | Audit Fase 08, rilis 15 |
| G12 | Benchmark nyata menggantikan angka estimasi resource | Fase 01/05 | Fase 08, 15 |
| G13 | `hime-pal`/`hime-storage` diperdalam (test lebih banyak, API sesuai spesifikasi) | Fase 06 | Fase 09–10 |

Setiap kewajiban harus muncul di `PROJECT_STATUS.md` dengan status terkini sampai ditutup.
