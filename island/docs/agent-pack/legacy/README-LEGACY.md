# LEGACY PACK (paket Hime lama) — STATUS: REFERENSI, BUKAN PERINTAH

Folder ini adalah paket briefing sebelumnya (dibuat sebelum pivot ke Bloom). **Master prompt lama sudah tidak berlaku** (`01-MASTER-PROMPT.SUPERSEDED.md`).

## Apa yang masih berlaku (sisi backend dan keamanan)
| Berkas legacy | Status | Catatan |
|---|---|---|
| `02-PRODUCT-VISION-AND-REQUIREMENTS.md` | **Berlaku** untuk kebutuhan UR-01..18 dan kriteria v1.0 | Kriteria UI/desktop dibaca ulang lewat `../07-UX-SPEC-HIME-ON-BLOOM.md`. UR-18 (Linux) diubah: lihat `../02-STRATEGY-AND-DECISIONS.md` |
| `03-SYSTEM-ARCHITECTURE.md` | **Sebagian**: bagian Hime Core, protokol IPC, event, data, resource, observability, deployment tetap. **Tidak berlaku:** semua yang menyebut Hime Desktop = Tauri app sendiri / React UI sendiri / Island buatan sendiri | Arsitektur integrasi baru: `../06-INTEGRATION-ARCHITECTURE.md` |
| `04-ENGINEERING-STANDARDS-AND-GOVERNANCE.md` | **Berlaku** (klasifikasi bukti, DoD, review independen, protokol keputusan) | Standar kode TS/React/Rust UI **diganti** oleh aturan Bloom: `../04-BLOOM-DESIGN-AND-CODE-RULES.md`. Aturan Rust Core tetap |
| `05-STATUS-AUDIT-PHASE-00-08.md` | **Berlaku sebagai catatan sejarah** | Diperbarui di `../08-STATUS-AND-CARRY-FORWARD.md` |
| `06-SECURITY-SPECIFICATION.md` | **Berlaku penuh** (IB-01, IB-02, threat model, izin) | |
| `07-SKILLS-AND-AGENTS-SPEC.md` | **Berlaku** | |
| `08-REPORT-TEMPLATES.md` | **Berlaku** + tambahan di `../09-REPORT-TEMPLATES.md` | |
| `roadmap/PHASE-08..15` | **Berlaku untuk lingkup backend/domain**. Bagian UI (Tauri Desktop, Island, panel) **digantikan** oleh `../roadmap/PHASE-XXR-*.md` | Baca versi "R" lebih dulu; ia menunjuk ke bagian legacy yang masih dipakai |

## Aturan konflik
Jika legacy dan paket baru bertentangan → **paket baru menang**. Jika paket baru diam → legacy berlaku. Catat setiap konflik yang kamu temukan di `docs/hime/DECISIONS-LOG.md`.
