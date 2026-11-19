# 14 — MODULARITAS & KUALITAS KODE

Tujuan: **mudah menambah, mematikan, dan merawat** setiap fitur, alat, dan skill — tanpa kode hantu, bau kode, celah keamanan, atau kerapuhan.

## 1. Model: semuanya modul

**Modul** = satuan fitur yang dapat dinyalakan/dimatikan, punya **manifest**, bagian di Core/shell/mobile (sesuai kebutuhan), pengaturan, pintasan, skill, string, tes, dan dokumentasi.

| Modul (contoh ID) | Bagian |
|---|---|
| `runs` (inti), `memory` (inti), `policy` (inti) | Core (tidak dapat dimatikan) |
| `chat`, `voice`, `attachments` | Core + shell + mobile |
| `tasks`, `routines`, `enforce`, `screentime`, `reminders` | Core + shell + mobile |
| `shelf` (Simpanan), `selection`, `translate` | Core + shell (+ mobile untuk shelf) |
| `lyrics`, `eyes`, `keybinds` | Core (lyrics) + shell |
| `storage`, `devices`, `documents`, `browser`, `github`, `drive`, `dev` | Core (+ shell/mobile) |
| `interact` (Hime proaktif) | Core + shell + mobile |

**Tiga jenis kemampuan** (semua dapat ditambah tanpa mengubah kode inti):
1. **Modul** — fitur utuh (manifest + kode).
2. **Skill** — kemampuan yang dipanggil `/` atau router; **deklaratif** (YAML) di `skills/<domain>/<id>/` (skema di `07-SKILLS…` legacy B2) + tes; hanya memakai **alat** yang diizinkan.
3. **Alat (tool)** — fungsi terdaftar di Tool Gateway (skema input/output, level risiko, idempotensi, timeout). Alat dimiliki satu modul.

## 2. Manifest modul (`modules/<id>/manifest.json`; skema `contracts/module-manifest.schema.json`)
```json
{
  "id": "translate", "version": "1.0.0",
  "title": "Terjemah", "desc": "Arti teks terpilih", "icon": "translate", "category": "terjemah",
  "core":   { "crate": "hime-mod-translate", "deps": ["policy","selection"], "lazy": true, "idleStopSec": 120 },
  "island": { "panels": [], "bubbles": ["translate-card"], "settings": "translate", "keybinds": ["translate.selection"] },
  "mobile": { "screens": [] },
  "permissions": ["network:provider", "selection:read"],
  "tools": ["translate.run"], "skills": ["translate.selection"],
  "resources": { "ramEstimateMB": 8, "timers": 0, "listeners": ["uia-selection"] },
  "privacy": { "collects": ["teks seleksi"], "sendsToCloud": true, "optIn": true },
  "default": { "enabled": true }
}
```
**Aturan:** teks (`title/desc`) tunduk batas copy-lint; semua izin dideklarasikan (ditegakkan Policy Engine); `resources` jujur dan **diverifikasi oleh pengukuran** (15).

## 3. Siklus hidup dan saklar ON/OFF (nol-biaya saat OFF)

**Core (`hime-modsdk`):**
```
trait Module { id(); version(); deps(); capabilities();
  fn start(&mut self, ctx) -> Result; fn stop(&mut self) -> Result; fn health(&self) -> Health;
  fn config_schema() -> Schema; }
Status: Disabled → Stopped → Starting → Running → Degraded → Failed
```
- **ModuleRegistry:** graf dependensi (tidak boleh siklik; startup topologis), enable/disable **saat runtime** (persisten), **lazy start** (mulai saat dibutuhkan) dan **idle stop** (berhenti saat idle), batas resource, supervisor dengan **backoff + circuit breaker**, `health()` berkala *tanpa polling berat*.
- **Isolasi:** tiap modul berjalan di task terpisah dengan `catch_unwind` di batas; panik satu modul **tidak** menjatuhkan Core (menjadi `Failed` + log + restart terbatas).
- **Komunikasi:** hanya lewat **bus peristiwa bertipe** dan **router permintaan** (kontrak); tidak ada pemanggilan langsung antar modul kecuali dependensi terdeklarasi.
- **Data:** tabel SQLite ber-namespace per modul + migrasi sendiri; "Hapus data modul" tersedia.

**Shell (`island/src/hime/`):** `ModuleRegistry` memuat modul **hanya bila ON** lewat `import()` dinamis (code-splitting) → modul OFF **tidak dimuat, tidak ada timer, tidak ada listener, tidak ada pintasan, tidak ada panel/bubble, tidak ada string aktif**. Mematikan modul saat berjalan: `dispose()` melepas semua (event, hotkey, observer, timer) — **diuji kebocoran**.
**Mobile:** modul sebagai paket Kotlin dengan `Feature` interface; OFF = tidak ada layanan/izin yang diminta.

**Definisi "nol-biaya saat OFF" (diuji):** RAM, thread, timer, handle OS, hotkey, jaringan = sama seperti modul tidak ada (±toleransi terukur). `hime perf` membuktikannya per modul.

## 4. Skill dan alat yang dapat diperluas
- **Tambah skill:** buat folder `skills/<domain>/<id>/` berisi `skill.yaml` (id, nama pendek, deskripsi pendek, ikon, kategori+warna, alias, parameter, alat yang diizinkan, level izin maks, langkah, verifikasi, anggaran, `test_cases` minimal normal/gagal/ambigu) → **langsung muncul di menu `/`** (registry membaca manifest). Tanpa mengubah kode UI.
- **Skill pengguna** (opsional, P2): folder `%APPDATA%/hime/skills/` dengan YAML yang sama, **izin dibatasi dan ditampilkan**, aktif hanya setelah persetujuan eksplisit; tidak boleh melampaui izin modul yang diizinkan; tidak ada kode arbitrer (deklaratif).
- **Tambah alat:** implementasi di crate modul, daftarkan skema (`contracts`), level risiko, **tes kontrak**; Tool Gateway menegakkan. Alat tidak terdaftar = tidak ada.
- **Penonaktifan:** skill/alat ikut modul induk; juga dapat dimatikan individual di Settings → Modul.

## 5. Scaffold (`hime new module <id>`)
Menghasilkan, konsisten dan siap pakai:
- `modules/<id>/manifest.json`, `README.md` (satu paragraf + cara uji).
- `core/modules/<id>/` (crate `hime-mod-<id>` dengan trait terimplementasi, tes kerangka, migrasi kosong).
- `island/src/hime/modules/<id>/` (entri `register()`, `dispose()`, komponen panel/bubble bila diminta, string di registry, tes).
- `mobile/…/modules/<id>/` bila `mobile` dideklarasikan.
- Entri settings (schema), pintasan, skill contoh, dokumen otomatis `docs/modules/<id>.md`.
- Menambah ke daftar kontrak dan menjalankan `gen`. **Gagal bila ID tidak unik atau manifest tidak valid.**

## 6. Kualitas kode: tanpa kode hantu, tanpa bau

### 6.1 Definisi dan gerbang (`scripts/check-hygiene.ps1` + alat bahasa)
| Masalah | Aturan | Gerbang |
|---|---|---|
| **Kode mati/hantu** | Tidak ada fungsi/ekspor/berkas tak terpakai; tidak ada kode dikomentari; tidak ada fitur-flag usang | Rust: `#![deny(dead_code, unused)]` pada crate Hime, `cargo machete` (dependensi tak terpakai) bila tersedia; TS: `noUnused*` (Bloom) + deteksi ekspor yatim (alat dev, mis. `knip`/`ts-prune` — **verifikasi** dan pilih yang ringan); skrip grep kode dikomentari (heuristik) |
| **TODO/FIXME** | Hanya dengan **ID isu** (`TODO(H-123)`); tanpa ID = gagal | grep |
| **Placeholder/mock tampak nyata** | Dilarang di produk; mock hanya di tes dan **berlabel** | grep + review |
| **Duplikasi** | Tidak ada logika bisnis duplikat; ambang duplikat (mis. `jscpd` dev-only, verifikasi) | alat dev |
| **Kompleksitas** | Fungsi ≤ ~50 baris sebagai pedoman; `clippy::cognitive_complexity` warn; berkas ≤ ~400 baris (pedoman) | clippy + review |
| **Panic/unwrap produksi** | Tidak ada `unwrap()/expect()` di jalur produksi tanpa komentar `INVARIANT:`; `panic!` hanya pada bug yang pasti | clippy `unwrap_used` (Hime crates) |
| **`any` TS tak beralasan** | Dilarang tanpa komentar alasan | ESLint-lite/grep atau `tsc` strict |
| **Log sampah** | Tidak ada `console.log`/`println!` di produksi; logging terstruktur | grep |
| **Dependensi** | Tambah hanya dengan alasan; lisensi diperiksa; audit kerentanan | `cargo deny/audit`, `bun audit`/setara (verifikasi ketersediaan) |
| **Siklus dependensi** | Tidak ada | `cargo` (otomatis), `madge`/setara dev-only untuk TS (verifikasi) |
| **Nama/konvensi** | Mengikuti konvensi bagian (Bloom untuk `island/`) | fmt/oxfmt/clippy |
Gerbang **gagal keras** (bukan peringatan) untuk: TODO tanpa ID, kode dikomentari yang tertangkap, `unwrap` produksi di crate Hime, dependensi tak terpakai, siklus, rahasia.

### 6.2 Desain
- **Kontrak dulu:** skema di `contracts/` sebelum kode; tipe dihasilkan ke Rust/TS/Kotlin (`gen`); `gen --check` memastikan tidak drift; **tes kontrak** lintas bahasa.
- **Batas modul:** tanggung jawab tunggal; API publik kecil; implementasi tersembunyi; tanpa state global tersembunyi; injeksi dependensi eksplisit.
- **Tanpa abstraksi spekulatif:** tidak ada factory/plugin-system yang belum dibutuhkan; tambah saat ada dua penggunaan nyata.
- **Konfigurasi bertipe** (schema), bukan string longgar.
- **Naming:** konsisten dengan bagian; istilah domain Indonesia hanya di UI/registry, kode berbahasa Inggris.

### 6.3 Keamanan (per modul dan global)
- **Model ancaman ringkas per modul** di `docs/modules/<id>.md` (aset, ancaman, kontrol).
- **Izin dideklarasikan dan ditegakkan** Policy Engine; modul tanpa izin tidak dapat mengakses jaringan/FS/proses.
- **Validasi di batas** (IPC/command/tool): tipe, panjang, rentang, path kanonik, allowlist.
- **Rahasia** hanya di vault; tidak di log, event, settings, localStorage, crash dump (uji kanari).
- **Privasi bawaan:** modul yang mengumpulkan data (`screentime`, `shelf`, `translate`, `voice`) opt-in, lokal, dapat dihapus, dideklarasikan di `privacy` manifest dan ditampilkan di Settings → Privasi & Data.
- **Webview:** hanya bundle lokal, CSP ketat, sanitasi markdown, capabilities minimal.
- **Dependensi & supply chain:** versi dikunci (`--locked`, lockfile), audit, lisensi, runtime terverifikasi hash (`runtimes.lock.json`), tidak menjalankan skrip unduhan tak terbaca.
- **Penegakan & penutupan jendela** lewat PAL dengan batas laju dan daftar pelindung (10).

### 6.4 Keandalan
- **Isolasi kegagalan** (catch_unwind, supervisor, circuit breaker); **degradasi anggun** (modul gagal → fitur itu saja tidak tersedia dengan pesan jujur).
- **Idempotensi & at-least-once** untuk run, tugas, event; antrean persisten (SQLite); dedup via ID.
- **Timeout & batas** di semua I/O; **backpressure** (kanal terbatas); tidak ada antrean tak terbatas.
- **Migrasi** DB eksplisit, berversi, dengan backup sebelum migrasi berisiko; uji naik versi dari data contoh.
- **Pemulihan:** startup memeriksa keadaan setengah jadi (operasi file, run) dan menyelesaikan/membatalkan dengan aman.
- **Watchdog:** shell memantau Core (reconnect), Core memantau runtime AI dan modul.
- **Uji kekacauan ringan:** matikan Core/runtime/modul di tengah operasi; putus pipe; disk penuh; jam berubah — perilaku jujur, tidak ada kehilangan data.
- **Observability:** log JSON terstruktur dengan correlation ID; halaman **Diagnostik** (Tentang): status modul, antrean, versi, 200 log terakhir (redaksi), tombol salin laporan.

### 6.5 Pengujian
Piramida: unit (logika murni: parser tanggal/LRC, NCM, anggaran perhatian, kebijakan), integrasi (Core+modul, bridge), **kontrak** (skema, lintas bahasa), E2E terbatas (skenario kunci), **visual** (screenshot per fitur, 04), **resource** (15), **keamanan** (06 legacy). Tes properti untuk parser. Cakupan diukur pada logika kritis (target yang masuk akal, bukan angka kosong); tes yang selalu lulus dilarang.

## 7. Definition of Done modul (tambahan pada legacy 04 §3)
- [ ] Manifest valid; `check-modules` hijau; scaffold dipakai.
- [ ] ON/OFF bekerja; **OFF = nol-biaya (terukur)**; ON→OFF→ON tanpa kebocoran.
- [ ] Izin dideklarasikan dan ditegakkan; model ancaman ringkas ada; privasi dideklarasikan.
- [ ] Settings via schema; pintasan via manifest; string via registry (lint lolos).
- [ ] Tes: unit + kontrak + skenario gagal; tidak ada kode mati/TODO tanpa ID.
- [ ] Resource diukur dan sesuai `ramEstimateMB` (toleransi) atau estimasi diperbarui dengan bukti.
- [ ] Dokumen `docs/modules/<id>.md` dihasilkan dan akurat.
- [ ] Review independen lulus (peran Reviewer; checklist legacy 04 §6 + hygiene).

## 8. Akseptansi kerangka modul (di Stage R0/U1)
- [ ] `hime-modsdk` + `ModuleRegistry` (Core, shell) + scaffold + `check-modules` + schema manifest bekerja dengan **dua modul contoh nyata** (mis. `eyes` dan `keybinds`).
- [ ] Bukti nol-biaya saat OFF (angka sebelum/sesudah).
- [ ] Gerbang higienitas hijau dan menolak pelanggaran uji (sisipkan pelanggaran sengaja di cabang uji lalu pastikan gagal).
