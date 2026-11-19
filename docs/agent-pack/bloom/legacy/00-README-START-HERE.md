# HIME AGENT PACK — Mulai dari Sini

**Untuk:** AI coding agent (Antigravity) yang mengerjakan proyek Hime
**Project Owner:** pengguna (mahasiswa Teknologi Rekayasa Perangkat Lunak)
**Dibuat:** 13 November 2026, berdasarkan ekspor percakapan perancangan Hime (6–10 November 2026)
**Bahasa:** Indonesia untuk narasi, Inggris untuk istilah teknis dan nama berkas

---

## 1. Apa itu paket ini

Paket ini adalah **sumber kebenaran tunggal** untuk menyelesaikan Hime dari kondisi sekarang (Fase 00–07 diterima bersyarat, Fase 08 baru dikirim) sampai rilis v1.0.

Sebelumnya, perancangan dilakukan oleh "System Architect" (ChatGPT) yang mengirim prompt per fase kepada kamu. Mulai sekarang **kamu memegang dua peran sekaligus: Architect dan Executor**, dengan kontrol kualitas yang dijelaskan di `04-ENGINEERING-STANDARDS-AND-GOVERNANCE.md`. Project Owner hanya diminta untuk keputusan yang memang menjadi haknya (lihat bagian "Owner Decision Gates" di Master Prompt).

## 2. Urutan baca (wajib, berurutan)

| # | Berkas | Isi | Wajib sebelum |
|---|---|---|---|
| 1 | `01-MASTER-PROMPT.md` | Instruksi eksekusi utama: apa yang harus kamu lakukan, aturan keras, kondisi berhenti | Segalanya |
| 2 | `02-PRODUCT-VISION-AND-REQUIREMENTS.md` | Visi, kebutuhan pengguna, katalog fitur, prioritas, kriteria sukses v1.0 | Pekerjaan apa pun |
| 3 | `03-SYSTEM-ARCHITECTURE.md` | Arsitektur, keputusan teknologi, protokol, data, lintas platform | Menulis kode |
| 4 | `04-ENGINEERING-STANDARDS-AND-GOVERNANCE.md` | Alur kerja, Definition of Done, klasifikasi bukti, aturan agent, standar kode | Menulis kode |
| 5 | `05-STATUS-AUDIT-PHASE-00-08.md` | Apa yang dilaporkan selesai, apa yang **harus kamu verifikasi sendiri**, temuan tidak konsisten | Memulai Fase 08 |
| 6 | `06-SECURITY-SPECIFICATION.md` | Threat model, IB-01, IB-02, permission, credential, prompt injection | Fase 08 dan seterusnya |
| 7 | `07-SKILLS-AND-AGENTS-SPEC.md` | Skill workspace untuk kamu + skill dan specialist untuk produk Hime | Fase 09 dan 14 |
| 8 | `roadmap/PHASE-08 … PHASE-15` | Rencana rinci tiap fase sampai selesai | Fase yang bersangkutan |
| 9 | `08-REPORT-TEMPLATES.md` | Templat laporan fase, status, keputusan owner, ADR | Menutup fase |

## 3. Prinsip yang berlaku di semua berkas

1. **Bukti di atas klaim.** Status sebuah pekerjaan hanya sah jika ada bukti yang bisa diperiksa ulang (perintah, keluaran, berkas). Laporan lama, termasuk laporan agent sebelumnya, bukan bukti.
2. **Berkas di paket ini adalah rancangan, bukan fakta repository.** Kondisi repository aktual (`x:/folder_ai/Hime` pada mesin pengembangan menurut laporan) harus diperiksa langsung. Jika repository berbeda dari paket ini, **catat selisihnya**, jangan menebak.
3. **Jangan menulis ulang yang sudah diputuskan.** Keputusan di `03-…` dan ADR yang ada di repository (12 ADR) hanya boleh diubah lewat ADR baru yang berisi alasan teknis dan bukti.
4. **Hermes dan 9Router tidak dibuat ulang.** Keduanya dipakai sebagai runtime eksternal. Source upstream tidak diubah tanpa persetujuan owner.
5. **Keamanan bukan fitur tambahan.** IB-01 dan IB-02 adalah blocker untuk memakai kredensial provider sungguhan.
6. **Jujur tentang batas.** Gunakan klasifikasi bukti `VERIFIED RUNTIME`, `VERIFIED AUTOMATED TEST`, `VERIFIED STATIC`, `INFERRED`, `UNKNOWN`, `NOT TESTED`, `BLOCKED`. Jangan menulis PASS untuk sesuatu yang tidak dijalankan.

## 4. Peta isi singkat

```
hime-agent-pack/
├── 00-README-START-HERE.md
├── 01-MASTER-PROMPT.md                  <- tempel isi berkas ini ke Antigravity
├── 02-PRODUCT-VISION-AND-REQUIREMENTS.md
├── 03-SYSTEM-ARCHITECTURE.md
├── 04-ENGINEERING-STANDARDS-AND-GOVERNANCE.md
├── 05-STATUS-AUDIT-PHASE-00-08.md
├── 06-SECURITY-SPECIFICATION.md
├── 07-SKILLS-AND-AGENTS-SPEC.md
├── 08-REPORT-TEMPLATES.md
└── roadmap/
    ├── PHASE-08-AI-RUNTIME-INTEGRATION.md
    ├── PHASE-09-INTELLIGENCE-CORE.md
    ├── PHASE-10-STORAGE-INTELLIGENCE.md
    ├── PHASE-11-MOBILE-AND-DEVICE-LAYER.md
    ├── PHASE-12-PRODUCTIVITY-AND-DOCUMENTS.md
    ├── PHASE-13-BROWSER-AND-DEVELOPER-WORKSPACE.md
    ├── PHASE-14-SPECIALIST-AGENTS-AND-SKILLS.md
    └── PHASE-15-HARDENING-RELEASE-LINUX.md
```

## 5. Cara menyimpan paket ini di repository

Salin seluruh isi paket ke `docs/agent-pack/` di repository Hime (atau lokasi lain yang konsisten dengan `AGENTS.md`), lalu rujuk dari `AGENTS.md` dan `PROJECT_STATUS.md`. Jika `AGENTS.md` sudah ada, **gabungkan, jangan timpa**.
