# 02b — MONOREPO: SEMUA HIME DALAM SATU FOLDER

**Keputusan owner (U13):** seluruh sistem Hime — **termasuk Dynamic Island (Hime Island)** — berada di **satu folder `Hime`** (`X:\folder_ai\Hime`), satu repository Git, terkelola rapi. Tidak ada lagi folder terpisah seperti `Hime-Bloom`. **Berkas ini menang atas semua jalur direktori di paket sebelumnya** (lihat peta jalur §3).

## 1. Prinsip
1. **Satu repo, satu sumber kebenaran:** kode, desain, kontrak, skill, runtime (manifest), dokumen, skrip.
2. **Batas jelas antar bagian**, hubungan hanya lewat **kontrak** (`contracts/`) dan **token desain** (`design/`) — tidak ada impor silang kode.
3. **Satu pintu perintah** (`scripts/hime.ps1`) untuk setup, dev, build, test, check, ukur, rilis lokal.
4. **Setiap bagian mengikuti konvensinya sendiri:** `island/` mengikuti Bloom; `core/` mengikuti standar Rust Hime (legacy 04); `mobile/` mengikuti standar Kotlin/Compose.
5. **Mudah dicari dan dirawat:** nama folder pendek dan konsisten; setiap folder punya `README.md` satu paragraf (apa, siapa pemilik, cara jalan).

## 2. Struktur target

```
X:\folder_ai\Hime\
├── README.md  PROJECT_STATUS.md  BACKLOG.md  RISK_REGISTER.md  CHANGELOG.md
├── AGENTS.md            # panduan agent tingkat monorepo (menunjuk ke island/AGENTS.md untuk kode Island)
├── LICENSE  NOTICE      # lihat §10
├── VERSION              # satu versi produk
├── Cargo.toml           # root workspace Rust (keputusan §5) 
├── rust-toolchain.toml  .editorconfig  .gitignore  .gitattributes
│
├── island/              # HIME ISLAND — shell desktop (fork Bloom): Tauri v2 + React 19/TS + Bun
│   ├── src/             #   frontend (milik Bloom) + src/hime/ (framework Hime, UI kit, modul sisi shell)
│   ├── src-tauri/       #   backend shell (milik Bloom) + src/hime/ (bridge, modul sisi shell)
│   ├── public/ scripts/ docs/ ...  (sisa struktur Bloom)
│   ├── AGENTS.md CONTRIBUTING.md SETTINGS.md ...  (konvensi Bloom, dipertahankan & disesuaikan)
│   └── package.json bun.lock vite.config.ts tsconfig*.json .oxfmtrc.json
│
├── core/                # HIME CORE — proses terpisah (Rust)
│   ├── crates/          #   hime-core, hime-ipc, hime-pal, hime-security, hime-storage, hime-modsdk
│   ├── modules/<id>/    #   modul sisi Core (crate hime-mod-<id>)
│   ├── prompts/         #   persona Hime & templat prompt (versi, tes emas)
│   └── tests/           #   integrasi & kontrak
│
├── mobile/              # APLIKASI ANDROID (Kotlin + Compose)
│   └── app/ ...         #   modules/<id>/ per fitur (paket Kotlin)
│
├── runtimes/            # RUNTIME AI TERKELOLA (bukan source kita)
│   ├── runtimes.lock.json    # nama, versi terkunci, sumber, SHA-256, lisensi, perintah pasang
│   ├── hermes/  9router/     # hasil pasang (di-gitignore); hanya manifest/skrip yang di-track
│   └── README.md
│
├── modules/             # REGISTRI FITUR (manifest saja) — satu folder per modul
│   └── <id>/ manifest.json  README.md  tests/(lintas-bagian)
├── skills/              # skill pack (YAML) per domain — dapat ditambah tanpa mengubah kode
├── design/              # tokens.json, eyes.json, icon-map.md, brand/ (logo SVG), generator
├── contracts/           # JSON Schema: command, event, settings, manifest modul, protokol perangkat
│   └── generated/       #   hasil codegen (Rust/TS/Kotlin) — di-commit; diperiksa agar tidak drift
├── tools/               # generator, linter kustom (bun/PowerShell/Rust kecil)
├── scripts/             # hime.ps1 (pintu tunggal) + check-*.ps1, restore-taskbar.ps1, run-safe.ps1, ...
├── docs/                # INDEX.md, agent-pack/, architecture/, adr/, api/, hime/, product/, reference/, security/, legal/, history/, modules/(dihasilkan)
├── dist/                # artefak build/installer (di-gitignore)
├── .dev-data/           # data dev & uji (di-gitignore); jangan menyentuh %APPDATA%
├── _old/                # KARANTINA sementara selama migrasi (di-gitignore; dihapus di akhir H1)
└── .github/ .vscode/    # CI lokal & editor (gabungan)
```

| Folder | Isi | Bahasa/alat | Konvensi | Gerbang |
|---|---|---|---|---|
| `island/` | Shell desktop (Island/notch/dock/Settings/hub) | Rust (Tauri) + TS/React + Bun | **Bloom** (`island/AGENTS.md`) + `04-DESIGN-SYSTEM` | `bun run build`, `cargo check/clippy/test --locked`, brand/copy/contrast/hygiene |
| `core/` | Otak: kebijakan, vault, runs, modul, supervisor | Rust | legacy 04 + 14 | fmt, clippy, test, audit/deny, hygiene |
| `mobile/` | App Android | Kotlin/Compose, Gradle | Kotlin standar + 16 | ktlint/detekt, test, lint Android |
| `runtimes/` | Hermes, 9Router terkunci | Python/Node (eksternal) | tidak diubah | verifikasi hash + kompatibilitas |
| `modules/` | Manifest fitur | JSON | skema `contracts/module-manifest` | `check-modules` |
| `skills/` | Paket skill | YAML | skema skill + 14 §4 | validasi skema + tes |
| `design/` | Token, mata, ikon, brand | JSON/SVG | 04, 05 | kontras, kelengkapan ikon, parity |
| `contracts/` | Kontrak lintas bagian | JSON Schema | — | `gen --check` (tidak drift) |
| `docs/` | Dokumen | Markdown | front-matter status | tautan valid |

## 3. Peta jalur (paket sebelumnya → monorepo)
| Di paket sebelumnya | Sekarang |
|---|---|
| `src/…` (frontend Bloom) | `island/src/…` |
| `src/hime/…` | `island/src/hime/…` (framework) dan `island/src/hime/modules/<id>/` (modul) |
| `src-tauri/…` | `island/src-tauri/…` |
| `src-tauri/src/hime/…` | `island/src-tauri/src/hime/…` |
| `hime.html` (window hub) | `island/hub.html` (window `hub`) |
| `core/…` | `core/…` (tetap) |
| `apps/mobile/…` | `mobile/…` |
| `docs/hime/…`, `docs/adr/…` | tetap |
| `scripts/…` | tetap (root) |
| Perintah `bun …` | dijalankan di `island/` atau via `scripts/hime.ps1` |
| `AGENTS.md` Bloom | `island/AGENTS.md` (+ `AGENTS.md` root baru) |
| Direktori data runtime `%APPDATA%/hime/` | tetap (data pengguna, bukan source) |

## 4. Migrasi di tempat (menggantikan prosedur H1 sebelumnya)
Repo lama sudah ada di `X:\folder_ai\Hime` (kemungkinan **tanpa commit**). Ubah **di tempat** dengan karantina — aman dan hash-verified:

1. **Periksa:** `git status`, `git log`, ukuran folder; pastikan tidak ada proses Hime/Hermes/9Router yang memakai folder.
2. **Branch & baseline:** bila belum ada commit: `git commit --allow-empty -m "chore: init monorepo"` (lokal). Buat branch `hime/h1-monorepo`.
3. **Karantina:** buat `.gitignore` berisi `_old/`, `dist/`, `.dev-data/`, `target/`, `node_modules/`, heavy `runtimes/*/` dsb. **Pindahkan semua isi lama** (selain `.git` dan `docs/agent-pack/`) ke `_old/` (bukan hapus). Hitung SHA-256 inventaris → `docs/hime/harvest/OLD-INVENTORY.md`.
4. **Kerangka baru:** buat struktur §2 (folder + `README.md` per folder).
5. **Impor Bloom ke `island/`:** pilihan terbaik `git subtree add --prefix=island https://github.com/little-systems/bloom.git <tag-rilis-terbaru>` (menjaga riwayat & atribusi; tanpa `--squash` bila ukuran wajar). Jika `git subtree` tidak tersedia: klon ke folder sementara, salin isi (tanpa `.git`) ke `island/`, commit dengan pesan berisi **tag + SHA upstream**, dan simpan `docs/hime/UPSTREAM.md`. Pin versi, tanpa push, remote `upstream` hanya untuk fetch.
6. **Sesuaikan akibat pemindahan:** `.github/workflows` Bloom → `.github/` root dengan `working-directory: island`; `.vscode` digabung; `.gitignore` Bloom digabung; jalur skrip `bun run bump` dsb. tetap bekerja dari `island/`. **Jalankan gerbang Bloom di `island/` dan pastikan hijau sebelum mengubah apa pun (H0.6).**
7. **Panen** dari `_old/` ke tujuan baru sesuai `hime-bloom-pack/05-HARVEST…` dengan jalur yang dipetakan (§3): crate Core → `core/crates/`, dokumen → `docs/…`, skrip → `scripts/` (adaptasi). Hash sumber vs salinan dicocokkan; `HARVEST-MANIFEST.md`.
8. **Bangun & uji** `core/` mandiri dan `island/` (angka test nyata).
9. **Hapus `_old/`** (otorisasi owner: tanpa backup) **hanya** setelah checklist 05 §5.1 lulus; tulis `DELETION-LOG.md`. Bila ragu → jangan hapus, tulis OD.
10. **Commit lokal** per langkah bermakna (Conventional Commits: `chore(repo): …`, `feat(island): …`).

> Folder `_old/` tidak pernah ikut commit dan tidak boleh berisi rahasia yang berpindah ke tempat lain. Setelah dihapus, **tidak ada Hime lama** yang tersisa selain yang dipanen.

## 5. Rust: workspace
**Keputusan awal (ADR di R0):** satu **root workspace** (`Cargo.toml`) dengan anggota `core/crates/*`, `core/modules/*`, `island/src-tauri` — satu `Cargo.lock`, satu `target/`, `[workspace.dependencies]` mengunci versi bersama (mis. `windows = 0.61` sesuai Bloom). **Manfaat:** versi konsisten, build lebih cepat, `path`-dependency `hime-ipc` mudah. **Syarat lulus uji:** `bun run tauri dev/build` dari `island/` tetap bekerja; `cargo check --locked` dan `clippy --locked --all-targets` hijau; hasil build installer benar. Profil rilis workspace-wide (catat: `panic` harus `unwind` bila Core memakai isolasi `catch_unwind`; atur per-profil secara sadar).
**Jika syarat gagal:** dua workspace terpisah (`core/` dan `island/src-tauri/`) dengan path-dependency; catat alasan di ADR. Jangan memaksa.

## 6. Pintu tunggal: `scripts/hime.ps1`
Pembungkus tipis (PowerShell kompatibel 5.1; `-WhatIf`/`-DryRun` untuk aksi mengubah; kode keluar deterministik; bantuan `-?`).

| Subperintah | Fungsi |
|---|---|
| `doctor` | cek lingkungan (Windows, Bun, Rust MSVC, Build Tools, WebView2, Git, JDK/Android SDK untuk mobile) tanpa mengubah apa pun |
| `setup` | pasang dependensi proyek (`bun install --frozen-lockfile`, `cargo fetch`, hook git, runtimes verify) |
| `dev [-Safe]` | build Core + jalankan Island lewat `run-safe` (taskbar dipulihkan otomatis) |
| `build [core|island|mobile|all]` | build per bagian; `island` → installer di `dist/` |
| `test [core|island|mobile|all]` | tes per bagian (`cargo test --locked`, dll.) |
| `check` | **semua gerbang**: fmt, lint, type, test, `check-brand`, `check-copy`, `check-contrast`, `check-hygiene`, `check-modules`, `gen --check`, audit lisensi/kerentanan, secret scan |
| `gen` | codegen: `design/` → CSS/Compose; `contracts/` → Rust/TS/Kotlin; dokumen modul; `SETTINGS.md` Hime |
| `new module <id>` | scaffold modul lengkap (14 §5) |
| `runtimes install|verify` | pasang/verifikasi Hermes & 9Router sesuai lock (update = butuh uji kompatibilitas + ADR) |
| `perf [-Minutes N]` | ukur resource (15) dan bandingkan baseline |
| `restore-taskbar` | pemulihan taskbar |
| `bump <x.y.z>` | naikkan versi di `VERSION`, island (skrip Bloom), core, mobile, manifest kompatibilitas |
| `clean` | bersihkan artefak (`dist/`, cache) tanpa menyentuh source/data |

**Aturan:** skrip tidak menyembunyikan perintah dasarnya (cetak apa yang dijalankan); tidak ada efek samping tak terdokumentasi; tidak memasang software butuh admin (Gate #4).

## 7. Data dev/uji dan lingkungan
- Variabel: `HIME_ENV=dev|test|prod`, `HIME_DATA_DIR` (override akar data). **Default dev/test → `.dev-data/`**, bukan `%APPDATA%/hime/`. Tes **tidak boleh** menyentuh data nyata owner.
- Produk terpasang memakai `%APPDATA%/hime/` (U10).
- Runtime AI saat dev: `runtimes/` (bila ada) atau lokasi yang ditunjuk `runtimes.lock.json`; produk terpasang membawa salinan di folder aplikasi.

## 8. Versi, commit, changelog
- Satu `VERSION` produk (SemVer). `scripts/hime.ps1 bump` menyinkronkan semua bagian.
- **Conventional Commits dengan scope wajib:** `feat(island): …`, `fix(core): …`, `feat(mobile): …`, `chore(repo): …`, `docs(adr): …`, `feat(mod/tasks): …`. Scope berasal dari daftar tetap (`island, core, mobile, design, contracts, runtimes, skills, modules, docs, repo, ci, mod/<id>`).
- `CHANGELOG.md` dihasilkan dari commit. Tag lokal `vX.Y.Z-rc.N`. **Tidak ada push.**

## 8b. Git hygiene
`.gitignore` lengkap; `.gitattributes` (LF untuk kode, CRLF hanya `*.ps1`/`*.bat` bila perlu; biner ditandai); hook pre-commit lokal (`scripts/hooks/`) menjalankan secret scan + `check-brand`/`check-copy` cepat; file besar (> 5 MB) ditolak kecuali LFS disetujui; tidak ada rahasia/ data pribadi.

## 9. CI lokal (`.github/workflows/ci.yml`)
Job dengan filter jalur: `core` (fmt/clippy/test/audit), `island` (bun typecheck + cargo check/clippy/test `--locked`), `mobile` (opsional), `checks` (brand/copy/contrast/hygiene/modules/gen-check). Workflow ditulis dan diuji lokal (mis. dengan menjalankan perintah yang sama via `hime.ps1 check`); **tanpa push**, jadi tidak ada eksekusi cloud tanpa izin.

## 10. Lisensi monorepo
- `island/` turunan Bloom → **GPL-3.0**; pertahankan `island/LICENSE` dan pemberitahuan hak cipta.
- **Default untuk seluruh repo: GPL-3.0** (`LICENSE` root) agar produk gabungan (installer berisi Island + Core) tidak ambigu. Owner dapat memilih lisensi lain untuk `core/`, `mobile/` bila produk tidak akan didistribusikan — **Owner Gate #7** untuk mengubah. Runtime pihak ketiga (Hermes, 9Router) mempertahankan lisensinya, dicatat di `docs/legal/THIRD-PARTY.md`.

## 11. Dokumentasi
`docs/INDEX.md` = peta navigasi; semua paket briefing di `docs/agent-pack/{legacy,bloom,update-v2}`; setiap dokumen punya front-matter (`status: draft|active|superseded`, `updated`); `docs/modules/<id>.md` dihasilkan dari manifest; ADR bernomor; `PROJECT_STATUS.md` selalu mutakhir di akar. `AGENTS.md` akar: peta folder, perintah `hime.ps1`, aturan isolasi per bagian, tautan paket.

## 12. Aturan batas (ditegakkan alat)
1. `island` tidak mengimpor kode dari `core`/`mobile`; berbicara hanya lewat IPC + tipe dari `contracts/generated`.
2. `mobile` tidak mengimpor `core`; hanya protokol perangkat (kontrak).
3. `core` tidak mengimpor `island`/`mobile`.
4. Token desain hanya dari `design/` (generate); tidak ada hex/ukuran ad-hoc di komponen.
5. Tidak ada dependensi siklik; modul hanya bergantung lewat deklarasi di manifest.
6. `check-modules` memverifikasi: setiap modul punya manifest valid, bagian yang dideklarasikan benar-benar ada, kunci settings/pintasan/skill terdaftar di schema, dan tidak ada modul yatim.

## 13. Akseptansi monorepo
- [ ] Satu folder `X:\folder_ai\Hime` berisi semuanya; tidak ada folder Hime/Bloom lain yang tersisa; `_old/` terhapus sesuai prosedur.
- [ ] `hime doctor/setup/build/test/check` bekerja dari akar (hasil nyata dicatat).
- [ ] Gerbang Bloom hijau di `island/`; Core hijau; workspace Rust sesuai ADR.
- [ ] Riwayat Bloom terjaga (subtree) atau `UPSTREAM.md` lengkap; remote `upstream` fetch-only; tanpa remote push.
- [ ] `docs/INDEX.md`, `AGENTS.md` akar, `README.md` per folder, `PROJECT_STATUS.md` ada.
- [ ] Tidak ada rahasia/artefak dalam git (secret scan + ukuran).
