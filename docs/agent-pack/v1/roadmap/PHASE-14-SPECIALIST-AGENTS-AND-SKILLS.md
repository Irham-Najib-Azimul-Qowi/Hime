# PHASE 14 — SPECIALIST AGENTS, ADVANCED SKILLS & EVALUATION
**Registri specialist · Delegasi terukur · Skill lanjutan (data/AI) · Memori lanjutan & bantuan proaktif · Evaluasi**

**Entry:** Fase 13 `READY FOR OWNER ACCEPTANCE`. Seluruh domain service memiliki tool dan skill dasar.
**Kebutuhan pengguna:** UR-13 (peran data engineer/analyst/scientist/AI engineer), UR-17.
**Prinsip:** *jangan memperbanyak agent demi jumlah.* Setiap specialist harus **terbukti berguna lewat evaluasi**.

## 1. Tujuan
Memperluas kemampuan Hime secara terukur: specialist yang terdefinisi, delegasi tertata, skill data/AI, bantuan proaktif yang tidak mengganggu, dan sistem evaluasi untuk mencegah regresi.

## 2. Task

### 14.1 Specialist registry
Entri per specialist: **ID, scope, tanggung jawab, skema input/output, allowed tools, level izin maksimum, anggaran resource (token/waktu/konkurensi), timeout, kriteria evaluasi, versi**. Disimpan sebagai data berversi di `skills/specialists/`. Penambahan butuh review (kualitas + keamanan).

### 14.2 Implementasi specialist (kandidat, aktifkan hanya yang lulus evaluasi)
Storage, Software Engineering, Research, Document, Productivity, Device, Security, Translation, **Data/AI** (data engineer–analyst–scientist–AI engineer sebagai satu specialist dengan mode).
**Bentuk implementasi (urutan preferensi):** (1) profil/prompt + tool group + skill di dalam satu Hermes runtime; (2) sesi Hermes terpisah bergiliran (bukan paralel) dengan konteks terisolasi; (3) worker paralel **hanya** jika evaluasi menunjukkan manfaat nyata dan anggaran RAM mencukupi (ukur).

### 14.3 Delegasi
Agent utama memilih specialist → mengirim task terdefinisi (skema) → specialist bekerja **dalam batas akses** (tool, folder, jaringan) → hasil terstruktur + bukti → agent utama **memvalidasi** (skema, bukti, konsistensi, kepatuhan izin) → menyajikan. Specialist tidak bisa meningkatkan izinnya sendiri; semua tool tetap lewat Gateway/Policy. Isolasi konteks antar-specialist.

### 14.4 Skill data dan AI (UR-13)
- **Data:** profil dataset, pembersihan, EDA, visualisasi (grafik berkualitas sesuai standar Fase 12), SQL, pipeline sederhana; reproduksibilitas (skrip + seed + lingkungan terkunci), keluaran notebook/laporan.
- **ML/AI engineering:** rancangan eksperimen, baseline, validasi tanpa kebocoran data, metrik sesuai masalah, pelaporan jujur (ketidakpastian, keterbatasan), model card sederhana; eksekusi di workspace terkendali (Fase 13) dengan batas resource (laptop i5 tanpa GPU: eksperimen kecil, atau rancangan untuk dijalankan di layanan cloud pengguna).
- Setiap skill memiliki: spesifikasi, prasyarat, langkah, penanganan gagal, keluaran, test case (normal/gagal/ambigu).

### 14.5 Memori lanjutan dan bantuan proaktif (P2/P3, hanya bila lulus kriteria)
- Retrieval lanjutan (re-ranking, ringkasan berkala, kedaluwarsa memori), tetap dengan provenance dan hapus.
- **Proaktif yang aman:** saran berdasarkan jadwal/tugas/pola unduhan; **opt-in**, rate-limited, dapat dibisukan; tidak pernah melakukan aksi berisiko tanpa approval; ukur apakah pengguna menerima atau menolak saran (metrik kegunaan).
- Alur adaptif: Hime mengusulkan menyimpan alur berulang sebagai skill — **disetujui pengguna** sebelum aktif.

### 14.6 Evaluasi
- Harness evaluasi (lanjutan Fase 09): kumpulan skenario per specialist/skill (normal, ambigu, adversarial, regresi); metrik: keberhasilan task, akurasi, keandalan tool, **penggunaan token, latensi, resource**, tingkat eskalasi approval, tingkat klaim tak berdasar.
- Jalankan sebelum dan sesudah perubahan (deteksi regresi); hasil disimpan di `docs/evaluation/`.
- **Gerbang manfaat:** specialist dipertahankan hanya jika mengalahkan baseline (agent utama + skill) pada metrik yang disepakati; selainnya digabung/dihapus.

## 3. Deliverables
Registri specialist, implementasi yang lulus evaluasi, skill data/AI, harness evaluasi + laporan, UI (daftar specialist/skill, status, pengaturan proaktif), dokumentasi `docs/architecture/SPECIALISTS.md`, `docs/evaluation/*`, laporan fase.

## 4. Exit gate
Setiap specialist aktif punya **kegunaan terbukti** (bukti evaluasi) dan batas akses terverifikasi; delegasi tidak melewati Policy; tidak ada regresi pada skenario fase 09–13; anggaran resource terpenuhi; proaktif opt-in dan tidak mengganggu.

## 5. Di luar cakupan
Banyak agent paralel tanpa bukti manfaat; inferensi model lokal besar; pembelajaran/fine-tuning otomatis dari data pengguna.
