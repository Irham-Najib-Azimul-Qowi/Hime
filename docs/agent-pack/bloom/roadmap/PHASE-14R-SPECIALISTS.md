# PHASE 14R — SPECIALIST AGENTS & EVALUASI

**Lingkup:** seluruh `legacy/roadmap/PHASE-14-SPECIALIST-AGENTS-AND-SKILLS.md` (registri specialist, bentuk implementasi dengan preferensi satu Hermes + profil/skill/tool group, delegasi tervalidasi, skill data/AI, memori lanjutan dan proaktif opt-in, harness evaluasi, **gerbang manfaat**). Prinsip tetap: tidak memperbanyak agent tanpa bukti manfaat dan anggaran RAM.

## UI (minimal; jangan membuat dashboard baru)
- Daftar specialist/skill aktif dan statusnya: **Workspace → AI Control Center** (bagian baru, gaya yang sama).
- Pengaturan proaktif (opt-in, bisukan, rate limit): tab Hime di Settings.
- Saran proaktif muncul sebagai **bubble** biasa dengan prioritas rendah, selalu dapat ditolak; **tidak pernah** melakukan aksi berisiko tanpa approval.
- Indikator specialist yang sedang bekerja: bagian dari bubble agent (nama singkat), bukan elemen baru.

Verifikasi: evaluasi sebelum/sesudah (regresi), anggaran resource, visual; **VC-09** hanya bila ada elemen UI baru.
