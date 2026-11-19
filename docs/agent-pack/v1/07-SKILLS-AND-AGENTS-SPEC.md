# 07 — SKILLS & AGENTS SPECIFICATION

Dokumen ini punya **dua bagian yang berbeda — jangan tertukar**:

- **Bagian A — Skill untuk kamu (agent pembangun)** di workspace pengembangan.
- **Bagian B — Skill dan specialist di dalam produk Hime** (kemampuan yang dimiliki Hime untuk pengguna).

---

# BAGIAN A — SKILL WORKSPACE UNTUK AGENT PEMBANGUN

## A1. Kondisi saat ini
Pada Fase 00 workspace sudah disiapkan (governance, `AGENTS.md`, kurasi skill). **Periksa** apa yang sudah terpasang (`ENVIRONMENT_AUDIT.md`, folder skill workspace, `docs/governance/`). **Jangan memasang ulang atau menambah skill tanpa kebutuhan.** Identifikasi runtime agent yang benar-benar kamu pakai (format skill, mekanisme instruksi proyek, dukungan sub-agent/hook); jangan mengasumsikan.

## A2. Kandidat skill (hasil kurasi awal; verifikasi sebelum dipakai)

Kurasi ini berasal dari penelusuran saat perancangan dan **belum diverifikasi oleh penulis paket**. Perlakukan sebagai kandidat, bukan jaminan.

| Bidang | Kandidat | Gunanya untuk Hime | Prioritas |
|---|---|---|---|
| Disiplin workflow engineering | Superpowers (obra/superpowers) | brainstorming → rencana → TDD → debugging sistematis → review → delegasi sub-agent | Sangat tinggi |
| Kode minimal & mudah dirawat | Ponytail (DietrichGebert/ponytail) | cegah overengineering dan dependensi berlebih | Tinggi |
| Frontend / dokumen / pembuat skill | anthropics/skills (pilih per kebutuhan) | frontend design, webapp testing, dokumentasi | Tinggi |
| Frontend React | vercel-labs/agent-skills | praktik React, performa, pola komposisi | Tinggi |
| UI/UX | UI/UX Pro Max (nextlevelbuilder) | design system, tipografi, konsistensi — cegah UI generik | Sangat tinggi |
| Motion | emilkowalski/skills | animasi bermakna untuk Island (perhatikan performa UHD 620) | Sedang |
| Keamanan | trailofbits/skills (pilih yang cocok dengan Rust/TS/Python) | review keamanan, secure coding | Sangat tinggi |
| Git/CI/dokumentasi | github/awesome-copilot (pilih per kebutuhan) | code review, Git workflow, CI/CD | Sedang |

Tambahan per fase hanya bila stack memang memakainya: Rust & Tauri, React/TS, **Kotlin Android (bukan React Native — stack mobile sudah diputuskan native Kotlin)**, SQLite/migrasi, API/contract testing, Playwright, DevOps/packaging, AI agent/tool calling, technical writing.

## A3. Skill Vetting Pipeline (wajib sebelum memasang skill apa pun)

1. Temukan repository sumber asli; periksa maintainer dan aktivitas.
2. Periksa lisensi (kompatibel dengan proyek pribadi/rilis?).
3. Baca `SKILL.md` dan **seluruh** script, hooks, dan dependensi — skill bisa membawa executable.
4. Tolak skill yang meminta akses jaringan/berkas/hak yang tidak diperlukan.
5. Uji pada task terbatas.
6. Catat versi/commit yang dipasang di `docs/governance/SKILL-REGISTER.md`.
7. Beri status `approved | restricted | rejected`.
8. Jangan memasang puluhan skill sekaligus; instruksi yang tumpang tindih membuat agent bingung dan boros konteks. **Target: perpustakaan kecil dan berkualitas.**
9. Jangan menjalankan skrip instalasi dari internet tanpa membacanya; pemasangan yang butuh hak admin → Owner Gate #4.

## A4. Pembagian peran saat bekerja sendirian

Karena tidak ada Architect eksternal, pisahkan **peran** (konteks/giliran berbeda):

| Peran | Tugas | Larangan |
|---|---|---|
| Architect | Menetapkan rencana fase, kontrak, ADR | Tidak mengklaim implementasi benar |
| Implementer | Menulis kode, test, dokumentasi | Tidak menyatakan dirinya selesai tanpa review |
| QA | Test plan, jalankan test, uji kegagalan | Tidak mengubah kode produk (kecuali test) |
| Security Reviewer | Threat model, bypass, kanari rahasia | — |
| UI/UX Reviewer | Konsistensi design system, aksesibilitas, state | Tidak menerima UI hanya karena "terlihat modern" pada satu screenshot |
| Code Reviewer | Diff, duplikasi, dead code, dependensi, naming | — |
| Documentation | Sinkronkan dokumen dengan implementasi | — |

Jika Antigravity mendukung sub-agent/agent paralel, delegasikan QA/Security/Code Review ke agent terpisah. Jika tidak, lakukan sebagai pass berurutan dengan checklist kosong di `04-…` bagian 6.

---

# BAGIAN B — SKILL DAN SPECIALIST DI DALAM PRODUK HIME

## B1. Keputusan arsitektur multi-agent

- **Awal: satu Hermes runtime.** Spesialisasi dicapai lewat **skills, tool groups, workflow templates, dan batas konteks** — bukan banyak agent.
- **Specialist worker terpisah** hanya dibuat jika kebutuhan paralel/isolasi **terbukti** (Fase 14) dan terukur manfaatnya.
- Alasan: hardware (i5-8250U), anggaran RAM, kompleksitas debugging, dan risiko konflik instruksi.

## B2. Model "Skill" Hime

Setiap skill Hime adalah paket deklaratif dengan skema berikut (disimpan di `skills/<domain>/<nama>/`):

```yaml
id: storage.organize-downloads
version: 1.0.0
domain: storage
purpose: ...
preconditions: [...]            # kondisi yang harus benar sebelum jalan
inputs:  { schema: ... }
outputs: { schema: ... }
allowed_tools: [storage.search_files, storage.propose_move, ...]
permission_level_max: L2        # tidak boleh lebih dari ini tanpa approval
steps: [...]                    # prosedur terstruktur
failure_handling: [...]
verification: [...]             # bagaimana hasil dibuktikan
quality_bar: [...]              # standar hasil (mis. format dokumen)
resource_budget: { tokens: ..., time_s: ..., concurrency: ... }
test_cases: [...]               # wajib: minimal 1 jalur normal, 1 gagal, 1 ambigu
```

Skill **tidak** memberi hak akses; hak tetap ditentukan Policy Engine. Skill baru wajib melewati review (kualitas + keamanan) sebelum aktif.

## B3. Skill inti yang harus tersedia di v1.0

| Domain | Skill (contoh ID) | Fase |
|---|---|---|
| storage | `storage.analyze-usage`, `storage.find-duplicates`, `storage.organize-files` (bertanya lokasi), `storage.handle-new-download` (UR-03), `storage.versioning`, `storage.backup-restore` | 10 |
| devices | `devices.phone-storage-overview`, `devices.transfer-files`, `devices.remote-status` | 11 |
| productivity | `tasks.capture-workload`, `tasks.daily-nudge`, `study.schedule`, `accountability.set-mode` | 12 |
| documents | `docs.create-report`, `docs.create-presentation`, `docs.create-spreadsheet`, `docs.convert`, `docs.compress`, `docs.merge-pdf`, `docs.ocr`, `docs.extract` | 12 |
| translation | `translate.selection` | 9 |
| browser | `browser.research`, `browser.download`, `browser.fill-form` (approval) | 13 |
| developer | `dev.analyze-requirements`, `dev.design-system`, `dev.implement-change`, `dev.run-tests`, `dev.code-review`, `dev.security-review`, `dev.write-docs` | 13 |
| data/AI | `data.profile-dataset`, `data.clean`, `data.analyze`, `ml.experiment`, `ml.evaluate` | 14 |
| integrations | `github.manage-repo`, `drive.manage-files` | 13 |

## B4. Specialist (Fase 14) — registri

Setiap specialist memiliki: **ID, scope, tanggung jawab, skema input/output, allowed tools, permission, anggaran resource, timeout, kriteria evaluasi**.

Kandidat: Storage, Software Engineering, Research, Document, Productivity, Device, Security, Translation. **Hanya yang terbukti berguna** (lihat evaluasi Fase 14) yang diaktifkan. Hasil specialist divalidasi oleh agent utama (struktur, bukti, konsistensi) sebelum disajikan.

## B5. Kontrol kualitas agent (runtime Hime)

Wajib ada pada orkestrasi: validasi rencana; verifikasi hasil tool; persyaratan bukti ("tampilkan lokasi akhir"); verifikasi penyelesaian; deteksi loop; batas iterasi, waktu, dan token; pelaporan ketidakpastian; deteksi halusinasi (klaim tanpa tool-result pendukung ditolak/ditandai).

Agent Hime **tidak boleh mengklaim**: file berhasil dipindah tanpa memeriksa lokasi akhir; build berhasil tanpa menjalankan build; API tersedia tanpa memverifikasi; test lulus tanpa menjalankan; fitur selesai hanya karena UI dibuat.

## B6. Standar hasil kerja Hime (output untuk pengguna)

- **Dokumen office:** struktur heading jelas, hierarki konsisten, gaya terpusat (style, bukan format manual), daftar isi untuk dokumen panjang, tabel/gambar bertitle, margin dan spasi standar, font terbaca, sitasi/referensi bila akademik, metadata terisi, **dibuka kembali dan diverifikasi** setelah dibuat (render ke PDF/gambar dan periksa).
- **Sistem software:** proses analisis → perancangan → rencana → implementasi → pengujian → review → dokumentasi; arsitektur berlapis, keamanan sejak awal, UI konsisten dengan design system, README dan dokumen teknis.
- **Laporan kuliah/kerja:** ikuti templat dan pedoman yang diberikan pengguna; bila tidak ada, gunakan struktur akademik baku (Pendahuluan, Landasan, Metode, Hasil, Pembahasan, Kesimpulan, Daftar Pustaka); jangan mengarang referensi/data.
