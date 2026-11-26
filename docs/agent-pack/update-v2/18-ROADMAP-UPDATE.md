# 18 — ROADMAP (DIPERBARUI)

Menggantikan urutan di `hime-bloom-pack/roadmap/`. Isi backend fase lama (08R–15R, legacy) **tetap berlaku**; berkas ini mengatur **urutan**, **tahap baru**, dan **gerbang**. Semua jalur mengikuti monorepo (`02b`): `island/`, `core/`, `mobile/`, dll.

## 1. Urutan

```
H0   Studi & Impor Bloom (ke island/)         ← hime-bloom-pack/roadmap/STAGE-H0 (+ revisi di bawah)
H1   Monorepo + Panen (migrasi di tempat)      ← 02b §4 + 05 harvest
R0   Rebrand + Fondasi Desain + Fondasi Modul  ← BARU
H2   Bridge + irisan vertikal Island           ← STAGE-H2 (gaya baru: mata, pill, bubble)
U1   Notch Compose Manager + Mata + Animasi + Pintasan + Settings baru   ← BARU
08R  AI Runtime + Chat (07) + Persona (17)     ← 08R + 07 + 17
U2   Lirik (12)                                ← BARU (mandiri, cepat)
09R  Intelligence Core + Tugas (10 §1) + Terjemah (09) + Simpanan (08)    ← 09R + BARU
U3   Rutinitas, Komitmen, Penegakan Windows, Screen-time (10 §2–3)       ← BARU
10R  Storage Intelligence + Hime Proaktif (11) ← 10R + BARU
11R  Mobile & Devices + Paritas + Penegakan HP (16)                       ← 11R + BARU
12R  Dokumen & Produktivitas lanjutan          ← 12R (disesuaikan)
13R  Browser, Integrasi, Dev Workspace         ← 13R
U4   Optimasi Resource (15) — pass khusus       ← BARU (juga berjalan terus-menerus)
15R  Hardening & Rilis kandidat v1.0           ← 15R (tanpa Linux)
(14R Specialist agents → PASCA-v1.0)
```

**Alasan urutan:** fondasi merek/desain/modul dulu (agar semua fitur lahir benar); visual pertama diuji sedini mungkin (H2, U1); chat sebelum fitur yang memakai AI; lirik cepat dan murah; tugas/terjemah/simpanan sebelum penegakan; proaktif setelah ada peristiwa dari storage/perangkat; mobile setelah Core stabil; optimasi khusus sebelum rilis (tetapi anggaran dijaga sejak awal).

## 2. Tahap baru dan revisi

### Revisi H0
Tambahan: **klon/impor Bloom ke `island/`** (`02b §4` langkah 5) di dalam monorepo; baseline resource dan visual juga mencakup komponen notch yang akan diubah (jam, pill, panel musik, shelf), mesin spring, sistem ikon dan font. Jalankan gerbang Bloom di `island/` sebelum perubahan.

### Revisi H1
Gunakan **prosedur migrasi di tempat dengan `_old/`** (`02b §4`) menggantikan "repo baru + hapus folder lama". Hasil: satu folder `Hime`.

### R0 — Rebrand + Fondasi Desain + Fondasi Modul
**Entry:** H1 selesai; Bloom hijau di `island/`.
**Tugas:**
1. **Rebrand total** (03): inventaris, penggantian, `check-brand`, ikon/logo placeholder mata (SVG sumber, `tauri icon`), `NOTICE`, Tentang → Lisensi.
2. **Fondasi desain** (04): `design/tokens.json` → CSS/Compose (generator), `scripts/check-contrast`, pilih keluarga ikon (ADR), `design/icon-map.md`, **komponen dasar** di `island/src/hime/ui/` (Pill, Button, IconTile, Row, Toggle, Segmented, Progress, Ring, Keycap, Card, Input), registry string + `check-copy`. **Restyle komponen Bloom yang tampil** (pill, progress, tombol, ikon) ke token baru secara bertahap tanpa merusak perilaku (tercatat di `CHANGES.md`).
3. **Fondasi modul** (14): `contracts/` (skema manifest + generator), `hime-modsdk` (Core), `ModuleRegistry` (shell), `hime new module`, `check-modules`, `check-hygiene`, saklar nol-biaya terbukti pada **dua modul contoh** (mulai dengan `eyes` kerangka dan `keybinds`).
4. **Pintu tunggal:** `scripts/hime.ps1` (doctor/setup/dev/build/test/check/gen/perf/new module/restore-taskbar/bump), hook git, CI lokal.
5. **Resource:** `measure-resources.ps1`, baseline, anggaran draf (15).
**Exit:** `hime check` hijau; `check-brand` = 0; komponen dasar terdokumentasi dengan screenshot; **VC-R0** (logo/ikon/tray/installer + galeri komponen).

### H2 (gaya baru)
Seperti STAGE-H2 tetapi: indikator Hime di pill = **mata mini**; bubble approval memakai komponen baru; dasar `Presentation` ke NCM (versi sederhana). **VC-01** menjadi bagian **VC-R0/H2**.

### U1 — Notch Compose Manager + Mata + Animasi + Pintasan + Settings
**Tugas:** NCM lengkap (06) dengan tes tabel keputusan; modul `eyes` (05): model parametrik, 22 ekspresi, Idle Director, morph jam↔mata, mata di header chat (kerangka); sistem animasi (preset, MotionController, katalog transisi); modul `keybinds` (13 A) + cheat sheet; **Settings baru** (13 B): schema registry, sidebar, bagian Modul dengan toggle, Sumber Daya awal, Tentang/Lisensi; ikon bot lama **dibuang**.
**Exit:** semua ekspresi terlihat di ukuran notch nyata; skenario konflik NCM lulus; Settings dari schema; pintasan teruji; resource tidak naik di atas anggaran; **VC-U1** (mata/notch/animasi), **VC-U8** (pintasan + Settings).

### 08R + Chat + Persona
Backend 08R (legacy) **plus**: modul `chat` (07): panel Hime, `/` skill dari registry, pill berwarna, lampiran (modul `attachments`), suara (modul `voice`, ADR engine), **Run Scheduler multi-run**, riwayat satu thread; **persona Hime** (17) + tes emas; AI **on-demand/idle-stop** (15 §4.5); jalur cepat Core disiapkan (dipakai terjemah/parsing).
**Exit:** semua akseptansi 07 dan 17; E2E nyata dengan kredensial uji (atau `BLOCKED`); IB-01/IB-02 status jujur; **VC-U2**.

### U2 — Lirik
Modul `lyrics` (12): Core (LRCLIB + tag + `.lrc` + cache + parser) + shell (panel Musik + pill). **Exit:** akseptansi 12; **VC-U7**.

### 09R + Tugas + Terjemah + Simpanan
Backend 09R (policy, gateway, task engine, event, memori, orkestrasi, resource) **plus**: modul `tasks` (10 §1: model, kategori, parser tanggal, aturan tenggat, kartu notch, quick-add, lewat chat; papan penuh di `hub` — window dibuat on-demand); modul `selection` + `translate` (09); modul `shelf` (08).
**Exit:** akseptansi 08, 09, 10 §1 + 09R; **VC-U3** (Simpanan, Terjemah), **VC-U4** (Tugas).

### U3 — Rutinitas, Komitmen, Penegakan, Screen-time
Modul `routines`, `enforce`, `screentime`, `reminders` (10 §2–3): jadwal, sesi, streak, tingkat penegakan, **penutupan anggun hanya pada aplikasi uji**, daftar pelindung, Jeda Darurat, jeda pelemahan, kill switch, riwayat; screen-time opt-in. **Exit:** akseptansi 10; **VC-U5**; **Owner Gate #10** sebelum penegakan pada aplikasi nyata.

### 10R + Hime Proaktif
Backend 10R (indeks, duplikat/versi, organizer, download intelligence, operasi aman, Recycle, backup) **plus** modul `interact` (11): Interaction Engine, anggaran perhatian, preferensi/backoff, katalog 1–22 yang relevan, ringkasan pagi/malam. **Exit:** akseptansi 10R + 11; **VC-U6**.

### 11R + Paritas Mobile
Backend/protokol 11R **plus** paritas UI dan fitur (16), penegakan Tier A/B/C, keandalan OEM, offline-first, Bagikan ke Hime. **Exit:** akseptansi 16 + 11R; **VC-U9**; Gate #6 untuk perangkat nyata.

### 12R / 13R
Dokumen & produktivitas lanjutan; browser/integrasi/dev workspace — isi sesuai berkas fase, UI memakai komponen baru dan modul manifest; kategori warna skill konsisten. Tidak ada specialist agent.

### U4 — Optimasi Resource (pass khusus)
Jalankan prosedur 15 secara menyeluruh: profil, tuas WebView2 (suspend/low-memory/window on-demand), code-splitting, runtime AI idle-stop, SQLite/alokator, worker on-demand, Mode Hemat, halaman Sumber Daya, soak test, uji nol-biaya OFF per modul. **Exit:** anggaran 15 §2 terpenuhi atau ADR penyimpangan; laporan `perf/REPORT.md`.

### 15R — Hardening & Rilis
Sesuai 15R **tanpa Linux**; tambahan: regresi **fitur Bloom** (dock, notch, shelf, Wi-Fi/Bluetooth, media, tray) nol; keselamatan taskbar di semua jalur; installer Hime Island (per-user, ikon baru, tanpa jejak Bloom di UI), manifest kompatibilitas; audit GPL/atribusi; uji ulang penegakan, privasi (opt-in/out), kanari rahasia; **VC-U10** (rilis kandidat). Status akhir: `READY FOR OWNER ACCEPTANCE (v1.0 RC)`.

## 3. Gerbang visual (VC) — ringkasan
| VC | Isi | Setelah |
|---|---|---|
| VC-R0 | Logo/ikon/tray/installer + galeri komponen | R0 |
| VC-U1 | Mata (22 ekspresi), jam↔mata, notch, animasi, skenario konflik | U1 |
| VC-U8 | Pintasan + cheat sheet + Settings | U1 |
| VC-U2 | Chat (`/` pill, lampiran, suara, multi-run) | 08R |
| VC-U7 | Musik + lirik | U2 |
| VC-U3 | Simpanan (seret), Terjemah | 09R |
| VC-U4 | Kartu Tugas + papan | 09R |
| VC-U5 | Rutinitas + nudge/penegakan | U3 |
| VC-U6 | Interaksi proaktif (file ganda dll.) | 10R |
| VC-U9 | Android | 11R |
| VC-U10 | Rilis kandidat | 15R |
Aturan: tidak menyatakan diterima tanpa jawaban owner; pekerjaan non-UI berlanjut.

## 4. Anggaran dan kualitas di setiap tahap (tidak opsional)
Setiap tahap memuat: `hime check` hijau (fmt, lint, type, test, brand, copy, contrast, hygiene, modules, gen-check), **laporan resource** (delta vs baseline), **screenshot nyata** (04 §9), **review independen**, pembaruan `PROJECT_STATUS.md`, `docs/hime/CHANGES.md`, commit lokal ber-scope.

## 5. Yang sengaja ditunda (pasca-v1.0) dan dihapus
**Ditunda:** specialist agents (14R), skill pengguna kustom (P2), relay cloud, sinkron dua arah penuh folder, widget Android, kamus pribadi terjemah.
**Dihapus dari rencana:** Linux, Floating Companion lama, Notification Center terpisah, dashboard AI besar, chat di window lain, workspace per percakapan/skill.
**Alasan:** ringan, fokus, sesuai permintaan owner. Owner dapat meminta penambahan kembali (Gate #7).

## 6. Templat laporan tambahan
Gunakan templat legacy 08 + paket Bloom 09 + tambahan: **Laporan Resource** (tabel: skenario · RAM sebelum/sesudah · CPU · proses · tuas · keputusan) dan **Laporan Visual** (tabel: permukaan/state · screenshot · DPI · hasil checklist 04 §9 · catatan).
