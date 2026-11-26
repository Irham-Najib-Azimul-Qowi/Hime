# HIME × BLOOM — PAKET BRIEFING AI CODING AGENT

**Untuk:** Antigravity (AI coding agent)  **Project Owner:** pengguna  **Dibuat:** 14 November 2026
**Bahasa:** narasi Indonesia, istilah teknis dan nama berkas Inggris.

## 1. Apa yang terjadi (konteks singkat)

Hime versi sebelumnya (Tauri + React buatan sendiri, Fase 00–07) **gagal secara visual**: Dynamic Island buatan sendiri tampil seperti aplikasi desktop biasa dengan jendela transparan besar dan tombol minimize/maximize/close bawaan Windows, jauh dari Dynamic Island dan tidak mendekati Nusa. Pemilik proyek memutuskan **berhenti memakai UI buatan sendiri** dan **membangun ulang Hime di atas Bloom**, repo open source:

> `https://github.com/little-systems/bloom.git` (GPL-3.0; Tauri v2 + Rust + React 19/TypeScript; Windows-only; pengganti taskbar + notch/Dynamic Island + dock)

Bloom sudah punya Island/notch dengan fisika spring, dock, kontrol sistem, shelf, Wi-Fi/Bluetooth, pemutar musik, kalender+Pomodoro. Tugasmu: **menjadikan Bloom itu Hime** — memasukkan semua fitur Hime (AI agent, storage, tugas, dokumen, perangkat, dll.) ke dalam bahasa desain dan aturan kode Bloom, dengan **Hime Core (Rust, sudah dibangun di Fase 06–08)** sebagai otak di belakangnya.

## 2. Urutan baca (wajib, berurutan)

| # | Berkas | Isi |
|---|---|---|
| 1 | `01-MASTER-PROMPT.md` | Instruksi eksekusi utama (tempel ke Antigravity) |
| 2 | `02-STRATEGY-AND-DECISIONS.md` | Keputusan strategis, arsitektur tingkat tinggi, lisensi GPL, konsekuensi |
| 3 | `03-BLOOM-STUDY-PROTOCOL.md` | Cara mempelajari source Bloom + fakta Bloom yang sudah terverifikasi dari dokumen publiknya |
| 4 | `04-BLOOM-DESIGN-AND-CODE-RULES.md` | Aturan desain dan penulisan kode (mengikuti Bloom), larangan "AI slop", protokol akseptansi visual, protokol aman menjalankan shell pengganti taskbar |
| 5 | `05-HARVEST-AND-CLEANUP-MANIFEST.md` | Apa dari Hime lama yang dipakai ulang, apa yang dibuang |
| 6 | `06-INTEGRATION-ARCHITECTURE.md` | Cara Hime Core terhubung ke shell Bloom, struktur direktori, IPC, event, settings, window baru |
| 7 | `07-UX-SPEC-HIME-ON-BLOOM.md` | Setiap fitur Hime dipetakan ke permukaan Bloom (notch, bubble, dock, overlay, settings, window workspace) |
| 8 | `08-STATUS-AND-CARRY-FORWARD.md` | Status Fase lama, kewajiban yang dibawa, blocker yang masih berlaku |
| 9 | `roadmap/STAGE-H0 … H2` lalu `PHASE-08R … 15R` | Rencana kerja berurutan sampai selesai |
| 10 | `09-REPORT-TEMPLATES.md` | Templat laporan, termasuk laporan akseptansi visual |
| 11 | `legacy/` | Paket lama (spesifikasi backend, keamanan, standar kerja) — baca `legacy/README-LEGACY.md` dulu |

## 3. Prinsip utama (ringkas)

1. **Bloom adalah tuan rumah.** Hime tamu yang tahu diri: desain, struktur kode, pola settings, pola event, dan alur rilis mengikuti Bloom. Tambahan Hime bersifat **aditif dan terisolasi** (`src/hime/**`, `src-tauri/src/hime/**`) dengan titik sentuh minimal ke berkas Bloom, supaya perbaikan upstream tetap bisa diambil.
2. **Hime Core tetap proses terpisah** (control plane, PEP, vault, supervisor Hermes dan 9Router). Shell Bloom hanyalah klien UI-nya. Keamanan tidak boleh bergantung pada shell.
3. **Visual harus terbukti, bukan diklaim.** Kegagalan Fase 07 terjadi karena UI tidak pernah dilihat. Setiap perubahan UI wajib diverifikasi dengan screenshot nyata dan dibandingkan berdampingan dengan panel Bloom yang ada. Pemilik proyek adalah hakim terakhir untuk desain.
4. **Jangan merusak desktop pengguna.** Bloom menyembunyikan taskbar Windows dan bisa membuat layar "macet" jika salah. Ikuti protokol aman di berkas 04.
5. **Bukti di atas klaim**, klasifikasi bukti, review independen, dan gerbang keputusan owner tetap berlaku (lihat legacy 04).
6. **Hermes dan 9Router tidak dibuat ulang dan tidak diubah** tanpa persetujuan owner. IB-01 dan IB-02 tetap blocker.

## 4. Isi paket

```
hime-bloom-pack/
├── 00-README-START-HERE.md
├── 01-MASTER-PROMPT.md
├── 02-STRATEGY-AND-DECISIONS.md
├── 03-BLOOM-STUDY-PROTOCOL.md
├── 04-BLOOM-DESIGN-AND-CODE-RULES.md
├── 05-HARVEST-AND-CLEANUP-MANIFEST.md
├── 06-INTEGRATION-ARCHITECTURE.md
├── 07-UX-SPEC-HIME-ON-BLOOM.md
├── 08-STATUS-AND-CARRY-FORWARD.md
├── 09-REPORT-TEMPLATES.md
├── roadmap/
│   ├── STAGE-H0-STUDY-AND-IMPORT.md
│   ├── STAGE-H1-HARVEST-AND-RESTRUCTURE.md
│   ├── STAGE-H2-BRIDGE-AND-ISLAND-SLICE.md
│   ├── PHASE-08R-AI-RUNTIME.md
│   ├── PHASE-09R-INTELLIGENCE-CORE.md
│   ├── PHASE-10R-STORAGE-INTELLIGENCE.md
│   ├── PHASE-11R-MOBILE-AND-DEVICES.md
│   ├── PHASE-12R-PRODUCTIVITY-AND-DOCUMENTS.md
│   ├── PHASE-13R-BROWSER-INTEGRATIONS-DEV.md
│   ├── PHASE-14R-SPECIALISTS.md
│   └── PHASE-15R-HARDENING-RELEASE.md
└── legacy/  (paket lama sebagai referensi backend)
```

## 5. Penempatan di workspace

Letakkan paket di `docs/agent-pack/` pada repo baru (hasil klon Bloom). Jika `AGENTS.md` Bloom sudah ada (ada), **jangan timpa**; tambahkan bagian "Hime" di bagian bawahnya (lihat berkas 04).
