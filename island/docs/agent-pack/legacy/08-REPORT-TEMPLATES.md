# 08 — REPORT TEMPLATES

Gunakan templat ini apa adanya (boleh diperluas, jangan dikurangi bagian jujurnya).

---

## T1. `PHASE-NN-COMPLETION-REPORT.md`

```markdown
# PHASE NN — <NAMA FASE> — COMPLETION REPORT

**Tanggal:** YYYY-MM-DD   **Branch:** phase/NN-...   **Commit lokal:** <hash>
**Status akhir:** READY FOR OWNER ACCEPTANCE | BLOCKED - OWNER DECISION REQUIRED
**Rekomendasi:** ACCEPT | QUALIFIED ACCEPTANCE | REWORK (dengan alasan)

## 1. Ringkasan (maks. 10 baris)
## 2. Ruang lingkup yang dikerjakan vs rencana (tabel: Task | Status | Catatan)
## 3. Struktur perubahan (berkas/crate/modul utama yang dibuat atau diubah)
## 4. Bukti
### 4.1 VERIFIED RUNTIME   (perintah + ringkasan keluaran nyata)
### 4.2 VERIFIED AUTOMATED TEST   (perintah; total passed/failed/ignored; rincian per suite; rekonsiliasi jumlah)
### 4.3 VERIFIED STATIC
### 4.4 INFERRED / UNKNOWN
### 4.5 NOT TESTED / BLOCKED   (alasan + apa yang dibutuhkan untuk menguji)
## 5. Acceptance criteria fase (tabel: Kriteria | PASS/FAIL/NOT TESTED | Bukti)
## 6. Keamanan (status IB-01, IB-02, temuan baru, residual risk)
## 7. Resource (pengukuran nyata: RAM/CPU/latensi; bandingkan anggaran)
## 8. Hasil Independent Review Pass (temuan BLOCKER/MAJOR/MINOR dan tindak lanjut)
## 9. Kewajiban yang ditutup / dibawa (rujuk carry-forward register)
## 10. Technical debt dan backlog baru
## 11. Keputusan arsitektur (ADR baru) dan permintaan owner (rujuk OWNER-DECISIONS-NEEDED.md)
## 12. Git: tracked-modified, untracked, HEAD, jumlah commit lokal, "zero push" dikonfirmasi
## 13. Rekomendasi untuk fase berikutnya
```

## T2. Format `PROJECT_STATUS.md` (bagian wajib di bagian atas)

```markdown
# PROJECT STATUS — HIME
**Diperbarui:** YYYY-MM-DD HH:MM
## POSISI SAAT INI
- Fase aktif: NN — <nama>
- Tugas berjalan: <task id> — <deskripsi>
- Langkah berikutnya (tepat): <perintah/berkas pertama yang dibuka>
- Cara melanjutkan setelah sesi terputus: <ringkas>
## STATUS FASE
| Fase | Status | Laporan | Catatan |
## BLOCKER AKTIF
| ID | Deskripsi | Status | Bukti |   (IB-01, IB-02, ...)
## KEWAJIBAN DIBAWA (G1..Gn)
## KEPUTUSAN OWNER MENUNGGU
## METRIK TERAKHIR (test, build, resource)
```

## T3. `OWNER-DECISIONS-NEEDED.md` (satu entri per keputusan)

```markdown
### OD-NN — <judul singkat>
- **Gate:** #<nomor> (<nama gate>)
- **Fase/Task terdampak:** ...
- **Mengapa dibutuhkan:** (1–3 kalimat, bahasa awam)
- **Pilihan:** A) ... B) ... C) ...
- **Rekomendasi saya:** ... **Alasan:** ...
- **Risiko bila ditunda / bila disetujui:** ...
- **Pekerjaan yang tetap berjalan sambil menunggu:** ...
- **Cara memberi jawaban:** balas dengan "OD-NN: A" (atau pilihan lain)
- **Status:** OPEN | ANSWERED (tanggal, jawaban)
```
Urutkan: dampak terbesar di atas. Gunakan bahasa yang bisa dipahami non-teknis; jangan mengharuskan owner membaca log.

## T4. ADR (`docs/adr/NNN-judul.md`)

```markdown
# ADR-NNN: <Judul>
**Status:** Proposed | Accepted | Superseded by ADR-XXX    **Tanggal:** ...
## Context
## Decision
## Alternatives considered (min. 2, dengan alasan ditolak)
## Consequences (positif dan negatif)
## Risks
## Validation plan / bukti (hasil uji yang mendukung keputusan)
```

## T5. Independent Review Report (ringkas, bagian 8 laporan fase)

```markdown
| ID | Tingkat (BLOCKER/MAJOR/MINOR) | Temuan | Bukti | Tindakan | Status |
```

## T6. Audit independen Fase 00–08 (`PHASE-00-08-INDEPENDENT-AUDIT.md`)

```markdown
| Klaim (sumber) | Hasil pemeriksaan | Label bukti | Selisih? | Tingkat | Tindakan |
```
Ditambah: posisi nyata Fase 08, hasil build/test dari bersih (angka nyata), daftar selisih terhadap `05-…`, dan baseline Git yang ditetapkan.

## T7. `FINAL-HANDOVER-REPORT.md`
Status tiap fase + bukti; 15 kriteria v1.0 (PASS/FAIL/NOT TESTED); risiko tersisa; keputusan owner menunggu; cara membangun/menjalankan/menguji Hime (Windows dan Linux Beta); cara memasang Android; daftar commit lokal per fase; cara melanjutkan pengembangan; lokasi seluruh dokumen.
