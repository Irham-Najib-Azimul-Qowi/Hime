# 02 — DECISIONS & PRECEDENCE

## 1. Keputusan baru (melanjutkan D1–D9 dari paket Bloom)

| # | Keputusan | Alasan | Mengubah |
|---|---|---|---|
| U1 | Produk bernama **Hime Island**; asisten bernama **Hime** | Permintaan owner | — |
| U2 | **Fork menjadi produk.** Boleh memodifikasi kode Bloom bila produk membutuhkannya; konvensi kode Bloom tetap standar; perubahan dicatat di `docs/hime/CHANGES.md`. Menarik pembaruan upstream = opsional (cherry-pick) | Fitur baru (mata di notch, chat, kartu tugas) memang harus masuk ke komponen notch; aturan "minimal touchpoint" menghambat | Melonggarkan D4 |
| U3 | **Satu thread chat global**; tidak ada workspace per percakapan/skill. Chat hidup di notch (bisa membesar). **Tidak ada chat di window lain.** Window `hub` (on-demand) hanya untuk tampilan data besar (papan tugas, storage, perangkat), bukan chat | Permintaan owner; hemat RAM dan kode | Mengganti "Workspace window" |
| U4 | **Arsitektur modul**: semua fitur Hime = modul (Core + shell) dengan manifest dan saklar ON/OFF nol-biaya | Maintainability, resource | Baru |
| U5 | **Maskot dua mata** adalah identitas visual utama (logo, ikon aplikasi, avatar chat, ambient notch) | Permintaan owner | Baru |
| U6 | **Penegakan** = komitmen buatan pengguna dengan pengaman (Jeda Darurat, jeda pelemahan aturan, log, tanpa kill paksa default) | Memenuhi permintaan tegas tanpa membahayakan | Baru |
| U7 | **Lirik** diambil lewat Core (LRCLIB + sumber lokal), bukan langsung dari shell | Shell tidak memanggil internet untuk fitur Hime | Baru |
| U8 | **Dipangkas dari v1.0:** specialist agents (Fase 14R → **pasca-v1.0**), Linux, Floating Companion, Notification Center terpisah, dashboard AI besar | Ringan, fokus, sesuai permintaan owner | Mengubah 14R/15R |
| U9 | **Resource-first**: modul berat lazy, runtime AI (Hermes/9Router) dimulai on-demand dan dihentikan saat idle; anggaran terukur | Keluhan owner tentang RAM | Baru |
| U10 | **Satu akar data:** `%APPDATA%/hime/` (`core/`, `shell/`, `shelf/`, `attachments/`, `cache/`, `logs/`) | Konsisten setelah rebrand | Mengubah §2 berkas 06 paket Bloom |
| U11 | **Window `hub`** menggantikan "Hime Workspace" | U3 | Rename |
| U13 | **Monorepo satu folder `Hime`**: Island (`island/`), Core (`core/`), mobile, runtimes (manifest), modul, skill, desain, kontrak, skrip, dokumen — satu repo, satu pintu perintah. Berkas **02b menang atas semua jalur** di paket sebelumnya | Permintaan owner: semuanya termanage dengan baik | Mengubah lokasi repo (Hime-Bloom → Hime) dan prosedur H1 |
| U12 | Pintasan global baru memakai `RegisterHotKey` (event-driven), bukan hook keyboard tambahan | Ringan dan tidak mengintersepsi input | Baru |

## 2. Dampak pada dokumen sebelumnya

| Dokumen sebelumnya | Status |
|---|---|
| `hime-bloom-pack/01-MASTER-PROMPT.md` | Digantikan oleh `01-MASTER-PROMPT-UPDATE.md` untuk konflik; sisanya tetap |
| `…/02-STRATEGY-AND-DECISIONS.md` | D1–D9 tetap kecuali D4 (dilonggarkan U2). D8 (Linux keluar) tetap |
| `…/03-BLOOM-STUDY-PROTOCOL.md` | **Tetap berlaku penuh** (Stage H0). Tambahan: pelajari juga komponen notch yang akan diubah (jam, pill, panel musik) dan sistem ikon/animasi |
| `…/04-BLOOM-DESIGN-AND-CODE-RULES.md` | Konvensi kode tetap. **Bagian visual digantikan oleh `04-DESIGN-SYSTEM.md` paket ini**: tampilan Hime Island *boleh dan harus* lebih berwarna dan modern daripada Bloom, tetapi tetap meminjam **mekanik** Bloom (spring, morph pill, klik-untuk-buka) |
| `…/05-HARVEST…` | Tetap |
| `…/06-INTEGRATION-ARCHITECTURE.md` | Tetap kecuali: struktur `src/hime/` menjadi `src/hime/modules/<id>/`; "Workspace" → "hub"; direktori data (U10) |
| `…/07-UX-SPEC-HIME-ON-BLOOM.md` | **Digantikan** oleh berkas 06–13 paket ini untuk semua yang tumpang tindih |
| `…/roadmap/*` | Diurutkan ulang oleh `18-ROADMAP-UPDATE.md`; isi backend tetap |
| `legacy/*` | Berlaku untuk keamanan, backend, standar kerja |

## 3. Kewajiban baru (tambahan carry-forward)

| ID | Kewajiban | Ditutup di |
|---|---|---|
| G18 | Rebrand total + gerbang `check-brand` | R0 |
| G19 | Design tokens + audit kontras + registry string + lint panjang | R0 |
| G20 | Registry modul (shell + Core) + scaffold + saklar nol-biaya terbukti | R0/U1 |
| G21 | Baseline resource Bloom + anggaran Hime + skrip ukur | H0, tiap tahap |
| G22 | Gerbang higienitas kode (`check-hygiene`) | R0 |
| G23 | Persona Hime di Hermes + uji emas identitas | 08R |
| G24 | Verifikasi visual tiap milestone (VC) | semua |

## 4. Aturan konflik
1. Paket ini > paket Bloom > legacy.
2. Jika source Bloom menyimpang dari anggapan paket mana pun → source menang; catat.
3. Jika dua cara sama baiknya → pilih yang lebih sederhana, lebih ringan, dan lebih mudah dimatikan.
4. Pertanyaan ke owner hanya untuk Owner Gate; selain itu putuskan dan catat sebagai ADR.
