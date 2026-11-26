# PHASE 09 — HIME INTELLIGENCE CORE
**Orchestrator · Policy & Permission Engine · Tool Gateway · Task Engine · Event Engine · Context & Memory · Agent Quality Controls · Island Intelligence (Terjemahan)**

**Entry:** Fase 08 `READY FOR OWNER ACCEPTANCE` (atau `BLOCKED` hanya pada bagian kredensial nyata). Percakapan dasar Hermes↔Desktop berjalan.
**Kewajiban dibawa:** G2 (smoke GUI), G6, G8, G13.

## 1. Tujuan
Ubah Hime dari "chat yang terhubung ke AI" menjadi sistem yang **merencanakan dan menjalankan pekerjaan multi-langkah** dengan batas izin yang teruji, observability, dan verifikasi hasil. Selesaikan juga fitur UX inti Island yang hanya butuh LLM: **terjemahan teks terpilih (UR-16)**.

## 2. Task

### 09.1 Policy & Permission Engine (PEP/PDP penuh)
- **Policy sebagai data** (TOML/YAML berversi) dievaluasi deterministik di Rust; **bukan** keputusan model.
- Permission registry: `resource × operation × scope × konteks → level (L0–L4) + keputusan (allow | require_approval | deny)`.
- Klasifikasi aksi dan tingkat risiko; deny rules (jalur sistem, folder sensitif, perintah terlarang); allowlist root file; kebijakan per mode (strict/balanced/santai berlaku untuk *nudge*, **bukan** untuk melonggarkan keamanan).
- Approval workflow: pratinjau perubahan, ticket 60 dtk sekali pakai (pertahankan), riwayat approval, "ingat keputusan" hanya untuk kelas aksi aman dan dapat dicabut.
- Audit trail tak-dapat-diubah-diam-diam (append-only + hash chain sederhana pada tabel `audit_logs`).
- Uji: tabel kasus izin (≥ kasus normal, batas, penolakan, injeksi path), uji properti pada kanonisasi path.

### 09.2 Tool Gateway
- **Registry tool** (id, versi, domain, deskripsi, skema input/output JSON Schema, level risiko, idempotensi, timeout, anggaran resource).
- Pipeline: validasi skema → cek izin → (approval) → eksekusi di domain service → validasi hasil → sanitasi output → audit.
- **Jembatan Hermes → Gateway:** mekanisme sesuai hasil discovery Fase 08 (preferensi: server tool lokal milik Hime Core yang kompatibel dengan mekanisme tool Hermes — mis. MCP bila didukung — dengan autentikasi sesi; **verifikasi** bukan asumsi). Pastikan **tidak ada jalur alternatif** bagi Hermes ke FS/shell (konfigurasi sandbox Hermes diverifikasi lewat uji: coba panggil tool bawaan Hermes yang berbahaya dan buktikan terblokir/nonaktif).
- Idempotency key per panggilan; hasil tool disimpan agar retry tidak mengulang efek.
- Tool awal (stub aman, diperluas fase berikutnya): `tasks.create/list/update`, `system.info` (read-only), `files.stat` (read-only, allowlist), `notify.send`.

### 09.3 Task Engine
- State machine: `queued → running → (waiting_approval | paused) → succeeded | failed | cancelled`, dengan retry policy (backoff, batas), prioritas, dependensi, pause/resume, scheduling (cron-like + sekali), progress reporting, persistensi (pulih setelah restart).
- Antrean dengan **backpressure** dan batas konkurensi dari Resource Manager.
- Task ID + Correlation ID di seluruh jalur; UI Task Board membaca dari Core (bukan state lokal).

### 09.4 Event Engine
- Definisi event bertipe + `schema_version`; routing ke UI/log/notifikasi; event latar, event perangkat (siapkan antarmuka untuk Fase 11), pemicu notifikasi.
- Delivery: at-least-once ke UI + dedup via ID (pertahankan pola Fase 07); buffer bergulir untuk UI offline.

### 09.5 Context & Memory
- Lapisan: konteks sesi, konteks proyek/task, **preferensi yang disetujui pengguna**, memori jangka panjang.
- Setiap item memori: sumber, waktu, relevansi, aturan retensi, kapabilitas hapus; **provenance wajib**.
- Retrieval: SQLite FTS5 (+ re-ranking sederhana); **jangan** mengirim seluruh riwayat ke model — susun konteks dengan anggaran token.
- UI untuk melihat/menghapus memori. Opt-in untuk menyimpan memori otomatis.

### 09.6 Agent Orchestrator + Quality Controls
- Intent → rencana → pemilihan skill/tool group → eksekusi bertahap → penggabungan hasil.
- **Quality controls:** validasi rencana, verifikasi hasil tool, persyaratan bukti, verifikasi penyelesaian, deteksi loop, batas iterasi/waktu/token, pelaporan ketidakpastian, penolakan klaim tanpa dukungan tool-result.
- Mode konteks terisolasi per task (cegah kebocoran konteks antar-task).
- Harness evaluasi kecil: 15–20 skenario terskrip (normal, ambigu, adversarial) dengan penilaian otomatis sederhana; jalankan di CI lokal.

### 09.7 Resource Manager
- Mode **Efficiency / Balanced / Performance** (pengaturan UI); batas konkurensi; penundaan pekerjaan latar saat RAM rendah; indikator beban di AI Control Center. Semua angka berasal dari pengukuran (Fase 08, G12).

### 09.8 Island Intelligence: Terjemahan teks terpilih (UR-16)
- **Pengambilan teks terpilih (Windows):** utama = UI Automation `TextPattern`; fallback = hotkey global (mis. konfigurabel) yang menyalin seleksi via clipboard **dengan memulihkan isi clipboard asli**. Semua lewat PAL.
- **Terjemahan:** deteksi bahasa → terjemahkan lewat model (jalur Hermes/9Router atau panggilan ringan lewat Router untuk latensi rendah; putuskan lewat ADR + ukur); cache hasil; bahasa tujuan default Indonesia (konfigurabel).
- **Tampilan:** Island berekspansi menampilkan terjemahan, asli, tombol salin/tutup; tidak mencuri fokus; hilang otomatis.
- **Privasi:** fitur **opt-in**; indikator jelas bahwa teks dikirim ke penyedia cloud; jangan mengirim bila fokus pada kolom password/aplikasi terkecuali (daftar pengecualian).
- Uji: aplikasi berbeda (browser, PDF reader, editor), teks panjang, bahasa campuran, offline, runtime mati.

### 09.9 Verifikasi GUI interaktif (G2 — smoke)
Pada sesi interaktif: screenshot, klik Island, ekspansi, Omnibox, drag Floating Companion, always-on-top, preset jendela, **uji pada DPI berbeda dan, bila tersedia, monitor kedua**. Hasil dicatat dengan label bukti jujur. Jika tak ada sesi interaktif → tetap `NOT TESTED` dan masuk Fase 15.

## 3. Deliverables
Kode Core (policy, gateway, task, event, memory, orchestrator, resource); UI (Task Board terhubung, Approval, Activity, Memory, pengaturan Mode); skill `translate.selection`; skema tool/event/policy + contract test; harness evaluasi agent; `docs/architecture/INTELLIGENCE-CORE.md`, `docs/api/TOOL-CONTRACTS.md`, `docs/api/EVENT-CONTRACTS.md`, `docs/security/POLICY-REFERENCE.md`; laporan fase.

## 4. Acceptance / Exit gate
- Workflow multi-langkah contoh (mis. "buat 3 task dari pesan ini lalu jadwalkan pengingat") selesai dengan izin, audit, dan verifikasi hasil — `VERIFIED RUNTIME`.
- Bypass Tool Gateway **tidak terbukti** pada uji (termasuk tool bawaan Hermes).
- Task pulih setelah restart Core; retry tidak menggandakan efek (uji dengan fault injection).
- Test lama tetap lulus; traceability FR→test untuk modul fase ini (G8).
- Terjemahan Island bekerja di ≥3 aplikasi berbeda dan menghormati opt-in.
- Memori memiliki provenance dan dapat dihapus; konteks yang dikirim ke model dibatasi anggaran.

## 5. Di luar cakupan
Operasi file sungguhan (Fase 10), perangkat (11), dokumen (12), browser/dev (13), multi-specialist (14).
