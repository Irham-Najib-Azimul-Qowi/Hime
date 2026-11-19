# PHASE 13 — BROWSER, INTEGRATIONS & DEVELOPER WORKSPACE
**Browser agent · GitHub · Google Drive · Workspace software engineering · Keselamatan eksekusi**

**Entry:** Fase 12 `READY FOR OWNER ACCEPTANCE` (Fase 09–10 minimal; Documents siap untuk keluaran SE).
**Kebutuhan pengguna:** UR-05, UR-08, UR-13, UR-14 (sisi software), UR-07.
**Owner Gates:** mengakses akun **GitHub/Google Drive sungguhan** dan token nyata (Gate #6). Bangun dengan akun/repo uji dan mock yang diberi label; sediakan panduan koneksi untuk owner.

# BAGIAN A — BROWSER (UR-05)

## A1. Pemilihan teknologi (putuskan lewat ADR dengan bukti)
Kriteria: keamanan (isolasi profil), kontrol (Hime sebagai PEP), keandalan, resource (RAM 300–800 MB tambahan; 1 sesi), kemudahan dipasang tanpa admin, lisensi, dukungan streaming screenshot/DOM.
| Kandidat | Catatan |
|---|---|
| **Playwright** (Chromium) — **default yang disarankan** | kontrol granular, profil terisolasi, tracing, unduhan, screenshot, stabil |
| Alat agen browser open source (mis. browser-use dan sejenisnya) | berguna untuk perencanaan aksi, tetapi lapisan kontrol kita tetap Hime; evaluasi manfaat vs risiko |
| Chrome DevTools Protocol langsung / MCP Chrome | alternatif bila perlu menyetir browser pengguna (lebih berisiko) |
Evaluasi singkat ≥2 kandidat pada 3 tugas nyata; catat hasil; pilih; jangan mengunci tanpa data.

## A2. Browser Service
- **Profil terpisah milik Hime** (tanpa cookie/sesi pribadi pengguna secara default). Login ke situs = tindakan eksplisit pengguna (approval) dan tersimpan hanya di profil itu.
- Kemampuan: sesi, navigasi, ekstraksi halaman (teks/struktur/tabel), interaksi formulir, unduhan (→ karantina → alur Fase 10), screenshot, tangkap jaringan terbatas. Satu browser sekaligus.
- **Kebijakan otomasi:** allowlist/denylist domain; blok SSRF (loopback, RFC1918, link-local, metadata cloud); batas waktu dan jumlah langkah; **isian sensitif, pembayaran, kirim formulir, unduh dan jalankan berkas = approval**; tidak mengeksekusi unduhan.
- **Konten halaman = data tak tepercaya.** Teks halaman dibungkus penanda khusus saat diberikan ke model; instruksi di dalam halaman tidak boleh memicu aksi tanpa melewati Policy; uji dengan korpus prompt-injection.
- Tool: `browser.open_page`, `browser.extract`, `browser.click`, `browser.type`, `browser.download`, `browser.screenshot`, `browser.close`. Skill: `browser.research` (pencarian + sintesis + **sitasi sumber nyata**), `browser.download`, `browser.fill-form`.

# BAGIAN B — INTEGRASI (UR-08)

## B1. GitHub
- Autentikasi: **OAuth device flow** atau fine-grained PAT, scope minimal per repo; token **di vault** (bukan DB/log/UI).
- Kemampuan: lihat repo/isu/PR/aksi CI, buat isu, buat branch/PR, baca diff, komentar. **Push/merge/hapus/rilis = approval** (dan Owner Gate #3 untuk repo Hime sendiri selama fase pembangunan ini).
- Adapter tunggal `integrations/github/` dengan antarmuka domain (bukan panggilan HTTP tersebar); penanganan rate limit; contract test dengan server tiruan berlabel.
- Tool: `github.inspect_repository`, `github.list_issues`, `github.create_issue`, `github.create_branch`, `github.open_pr`, `github.get_diff`.

## B2. Google Drive
- OAuth **installed-app (loopback + PKCE)**; scope default **`drive.file`** (hanya berkas yang dibuat/dibuka Hime), tingkat lebih luas hanya dengan persetujuan eksplisit; token di vault.
- Kemampuan: daftar, unggah, unduh, buat folder, pindah/rename, bagikan (**approval**), sinkron folder tertentu (satu-arah, konflik → versi, bukan timpa). Integrasi tujuan backup Fase 10.
- Adapter tunggal; kuota/rate limit; tangani token kedaluwarsa/dicabut.
- Tool: `drive.list`, `drive.upload`, `drive.download`, `drive.move`, `drive.share`.

# BAGIAN C — DEVELOPER WORKSPACE (UR-13, UR-14)

## C1. Workspace terkendali
- Direktori **workspace proyek** terdaftar (allowlist); setiap proyek punya kebijakan: perintah yang diizinkan, jaringan, variabel lingkungan yang boleh diwariskan (**tanpa secret Hime**).
- Eksekusi perintah: **tanpa string shell**; argumen terstruktur; **allowlist** (git, cargo, pnpm/npm, python/uv, pytest, dotnet/gradle bila perlu, formatter/linter); perintah di luar allowlist = approval; di bawah **Job Object** dengan batas CPU/RAM/waktu; stdout/stderr ditangkap dan **direduksi** (secret, path pribadi).
- Operasi destruktif (hapus, `git reset --hard`, `rm -rf`, `git push --force`) = approval eksplisit atau dilarang; semua perubahan lewat **diff yang ditinjau** sebelum dianggap selesai; cabang kerja otomatis (`hime/…`) agar mudah dibuang.
- Rahasia di proyek pengguna: pemindai rahasia sebelum commit; larangan membaca `.env` tanpa izin.

## C2. Kemampuan
Inspeksi repository, pencarian kode, edit berkas (patch), build, test, debug (membaca log/stack trace), operasi Git lokal, pembuatan komit dengan pesan baik. Tool: `dev.open_workspace`, `dev.search_code`, `dev.read_file`, `dev.apply_patch`, `dev.run_command` (allowlist), `dev.run_tests`, `dev.git_*`.

## C3. Alur software engineering untuk kuliah dan proyek (UR-13)
Skill terstruktur dengan keluaran dokumen berkualitas (memakai Document Service dan standar Fase 12):
1. **Analisis kebutuhan** → SKPL/SRS (fungsional/non-fungsional, aktor, use case, user story, batasan).
2. **Perancangan** → arsitektur, ERD/skema, UML (PlantUML/Mermaid dirender ke gambar), desain API, desain UI (design system konsisten), model ancaman ringkas.
3. **Rencana implementasi** → pemecahan task, estimasi, risiko.
4. **Implementasi** → kode bersih, test (TDD bila logika kritis), keamanan sejak awal (validasi input, authz, secret, dependensi).
5. **Pengujian** → rencana uji, kasus uji, laporan hasil.
6. **Review** → code review dan security review (checklist OWASP-relevan).
7. **Dokumentasi** → README, dokumen teknis, DPPL/laporan.
Mendukung web, mobile, dan sistem lain; templat proyek yang aman dan terstruktur (mis. web fullstack, API, aplikasi mobile) disimpan di `resources/project-templates/` (bukan scaffolding acak).

## C4. Safety rekayasa
Isolasi workspace; allowlist; proteksi secret; approval aksi destruktif; review diff; verifikasi build/test aktual sebelum klaim selesai; Hime **tidak** menjalankan kode tak tepercaya di luar Job Object.

# PENGUJIAN
Korpus prompt-injection untuk browser dan tool repo; uji SSRF; uji tool allowlist (percobaan bypass: argumen injeksi, `;`, `&&`, path traversal, variabel lingkungan); uji batas resource; uji OAuth (token kedaluwarsa, dicabut, scope berlebih ditolak); uji kegagalan jaringan; uji rate limit; contract test adapter (server tiruan berlabel); skenario nyata ber-sandbox: "riset topik X dengan sitasi", "buat repo uji, perbaiki bug sintetis, jalankan test, buat PR ke repo uji".

# DELIVERABLES
Browser Service + kebijakan; adapter GitHub dan Drive; Developer Workspace service; skill SE; templat proyek; UI (Browser panel/Activity, Developer, Integrations di Settings); dokumentasi `docs/architecture/BROWSER.md`, `docs/architecture/DEV-WORKSPACE.md`, `docs/integrations/*`; ADR (browser, OAuth, allowlist); panduan koneksi akun untuk owner; laporan fase.

# EXIT GATE
Hime dapat meneliti web dengan sitasi nyata dan unduhan terkarantina; mengelola repo uji dan Drive uji dengan izin minimal dan approval; mengubah software di workspace terkendali dengan bukti build/test dan diff; seluruh uji bypass/injection lulus; token hanya di vault.

# DI LUAR CAKUPAN
Mengendalikan browser pribadi pengguna dengan sesi login nyata (kecuali opsi eksplisit di masa depan), CI/CD multi-tenant, penerbitan paket.
