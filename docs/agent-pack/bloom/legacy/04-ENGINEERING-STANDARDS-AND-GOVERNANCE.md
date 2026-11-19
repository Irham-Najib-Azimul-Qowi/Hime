# 04 — ENGINEERING STANDARDS & GOVERNANCE

Berlaku untuk **semua** pekerjaan, di semua fase. Tujuannya: fitur dianggap selesai bukan karena kode berhasil ditulis, melainkan karena **benar, teruji, aman, terintegrasi, terdokumentasi, dan tidak merusak bagian lain**.

## 1. Klasifikasi bukti (wajib di setiap laporan)

| Label | Arti | Contoh sah |
|---|---|---|
| `VERIFIED RUNTIME` | Dijalankan pada sistem nyata, hasil diamati | Proses Core + Desktop berjalan, HWND diperoleh, Named Pipe terhubung |
| `VERIFIED AUTOMATED TEST` | Test otomatis dijalankan dan lulus; sebutkan perintah dan jumlah | `cargo test --workspace` 37/37 |
| `VERIFIED STATIC` | Diperiksa lewat pembacaan kode/konfigurasi/analisis statis | Implementasi state machine Island terbaca benar, belum dijalankan |
| `INFERRED` | Kesimpulan logis tanpa bukti langsung | "Kemungkinan endpoint X mendukung SSE" |
| `UNKNOWN` | Belum diketahui | Versi runtime belum diperiksa |
| `NOT TESTED` | Sengaja/terpaksa belum diuji | Multi-monitor, DPI scaling |
| `BLOCKED` | Tidak bisa dikerjakan karena alasan tercatat | Menunggu API key uji dari owner |

**Aturan:** `VERIFIED RUNTIME` ≠ "semua fungsi bekerja". Peluncuran proses dan HWND tidak membuktikan interaksi GUI. Pass satu skenario tidak membuktikan skenario lain. Frasa absolut ("zero orphan processes", "100% aman") dilarang kecuali ruang lingkupnya dibatasi dan dibuktikan.

## 2. Tangga status

`BACKLOG → READY → IN PROGRESS → IMPLEMENTED → SELF-VERIFIED → IN REVIEW → QA VERIFIED → READY FOR OWNER ACCEPTANCE → ACCEPTED (hanya owner) → DONE`

- Kamu boleh menetapkan sampai `READY FOR OWNER ACCEPTANCE`.
- `ACCEPTED` dan `DONE` untuk fase hanya dari owner. Sementara owner belum menjawab, kamu **tetap boleh melanjutkan** fase berikutnya (owner sudah mendelegasikan kecepatan), selama tidak melewati Owner Decision Gate dan status fase sebelumnya dicatat `READY FOR OWNER ACCEPTANCE (OWNER PENDING)`.
- Status `QUALIFIED ACCEPTANCE` dipakai untuk fase yang diterima dengan batasan terdokumentasi; batasan itu **dibawa** sebagai kewajiban ke fase berikutnya, bukan dilupakan.

## 3. Definition of Done (per fitur)

Fitur selesai jika **semua** terpenuhi:
1. Requirement dan acceptance criteria terpenuhi (ada ID FR/UR yang ditautkan).
2. Unit test untuk logika kritis; integration test untuk integrasi; contract test untuk skema API/tool/event.
3. Error handling eksplisit; kondisi gagal diuji (bukan hanya jalur bahagia).
4. Keamanan terpenuhi (izin, validasi input, tanpa kebocoran rahasia, fail-closed).
5. Logging terstruktur dan tanpa data sensitif.
6. UI: state loading, kosong, error; responsif; navigasi keyboard; konsisten dengan design system.
7. Dokumentasi (arsitektur/API/runbook) dan changelog diperbarui.
8. Tidak ada regresi diketahui; seluruh test lama tetap lulus.
9. Lulus **Independent Review Pass** (bagian 6).

## 4. Aturan perilaku agent (kamu)

1. **Evidence-based.** Jangan mengarang fakta tentang repository, library, API, atau runtime. Periksa dulu. Jika tidak bisa diperiksa, tandai `UNKNOWN`.
2. **Specification-first, architecture-first.** Tidak ada kode sebelum kebutuhan, kontrak, dan acceptance criteria jelas (cukup ringkas di dalam rencana fase).
3. **Tidak ada perubahan scope diam-diam.** Perubahan kebutuhan/arsitektur = ADR atau Owner Decision.
4. **Satu klaim = satu bukti.** Setelah mengubah sesuatu, verifikasi hasil akhirnya (cek lokasi file setelah dipindah, jalankan build setelah mengubah build).
5. **Laporkan hasil nyata termasuk kegagalan.** Perintah yang gagal ditulis apa adanya.
6. **Rekonsiliasi angka.** Jumlah test, jumlah file, jumlah requirement di laporan harus cocok dengan rincian (kasus nyata sebelumnya: 31 vs 33 test, 36 vs 37 test).
7. **Reversible dulu.** Perubahan berisiko punya strategi rollback; operasi destruktif memakai approval dan jurnal.
8. **Tidak ada reward hacking.** Jangan melemahkan test, menurunkan standar, atau menutup temuan dengan mengubah definisi agar "lulus".
9. **Ukuran kecil, kontrak stabil.** Perubahan kontrak publik (IPC, tool schema, event) disertai migrasi dan contract test.
10. **Tidak membuat fase atau sub-fase baru** untuk temuan non-blocking; masukkan ke `BACKLOG.md`. Blocker keamanan dan kesalahan arsitektur material tetap wajib diselesaikan.

## 5. Standar kode

**Prinsip:** SOLID secara proporsional, KISS, DRY proporsional, dependensi eksplisit, tipe kuat, fungsi kecil, satu tanggung jawab, error konsisten, tanpa state global tersembunyi, tanpa duplikasi logika bisnis. Hindari abstraksi dan factory yang tidak dibutuhkan.

**Keterbacaan:** nama menjelaskan maksud; logika bisnis dipisah dari I/O; komentar menjelaskan *mengapa*; tanpa magic number; tanpa kredensial hardcode; hindari berkas raksasa dan dependensi sirkular; antarmuka publik berdokumentasi.

**Rust:** `cargo fmt`, `cargo clippy --workspace --all-targets` (peringatan ditangani menurut risiko dan kebijakan lint yang disepakati — bukan target "nol warning" kaku), tanpa `unwrap()/expect()` di jalur produksi kecuali dibenarkan, `thiserror`/error bertipe, `unsafe` hanya di PAL dengan komentar `SAFETY`, secret memakai tipe yang dizeroize.
**TypeScript:** `strict` aktif, tanpa `any` tak beralasan, ESLint + Prettier, komponen kecil dan terisolasi, tanpa secret di `localStorage`.
**Python (skill/adapter Hermes):** type hints, `ruff`, `mypy`/`pyright`, venv terisolasi.
**Kotlin (Android):** Compose, arsitektur layer (UI/domain/data), coroutine terstruktur, `ktlint`/`detekt`.

**Anti-pola yang dilarang:** God object `hime-core`, `tools.py` raksasa, UI yang menyimpan state otoritatif, modul yang memanggil endpoint 9Router tersebar, penulisan langsung ke DB 9Router, pengujian yang hanya memeriksa nama, test yang selalu lulus.

## 6. Independent Review Pass (wajib di akhir tiap fase)

Lakukan dengan peran **Reviewer** yang terpisah dari peran Implementer (agent/konteks terpisah bila mungkin). Reviewer **tidak** memercayai laporan Implementer; ia memeriksa langsung.

Checklist Reviewer:
- [ ] Setiap acceptance criteria fase punya bukti yang diperiksa ulang (jalankan ulang perintah kunci).
- [ ] Build bersih dari awal (`clean` → build) dan seluruh test dijalankan ulang; jumlah cocok dengan laporan.
- [ ] Tidak ada placeholder yang berpura-pura fitur; tidak ada mock yang dilaporkan sebagai nyata.
- [ ] Batas arsitektur dipatuhi (tidak ada dependensi terlarang, tidak ada API Windows di luar PAL).
- [ ] Keamanan: tidak ada secret di repo/log/IPC/frontend; fail-closed; tidak ada jalur bypass Tool Gateway; path traversal dan command injection diuji.
- [ ] Klaim absolut dikualifikasi; klasifikasi bukti benar.
- [ ] Regresi: test fase sebelumnya tetap lulus.
- [ ] Dokumentasi cocok dengan implementasi (bukan sebaliknya).
- [ ] Git: perubahan tracked/untracked terdaftar; tidak ada rahasia; tidak ada push.
- [ ] Daftar `NOT TESTED`/`BLOCKED` lengkap dan jujur.

Temuan diklasifikasikan: **BLOCKER** (harus diperbaiki sekarang), **MAJOR** (perbaiki dalam fase), **MINOR** (backlog). Satu putaran perbaikan; hindari siklus review berulang untuk hal minor.

## 7. Quality gates otomatis

Skrip kanonikal (`scripts/`): `check-env.ps1`, `build.ps1`, `test.ps1`, `dev.ps1` — **perluas**, jangan ganti. Gate minimum sebelum commit fase:

| Gate | Perintah (sesuaikan dengan repo) |
|---|---|
| Format | `cargo fmt --check`, Prettier check |
| Lint | `cargo clippy`, ESLint |
| Type | `tsc --noEmit`, `cargo check --workspace` |
| Unit/integration | `scripts/test.ps1` (Rust + frontend) |
| Build | `scripts/build.ps1` (Core + Desktop) |
| Dependency scan | `cargo audit` / `cargo deny`, `pnpm audit` (periksa ketersediaan; jika tool tidak ada, tandai `NOT VERIFIED`, jangan pura-pura lulus) |
| Secret scan | pemindai rahasia (mis. gitleaks) bila tersedia; jika tidak, skrip grep pola + uji kanari |
| E2E | skenario fase (lihat berkas fase) |

Jangan memasang tool secara otomatis jika butuh admin (Owner Gate #4).

## 8. Git workflow

- Branch: `phase/NN-nama-fase`; commit lokal di akhir fase (pesan `phase-NN: ringkasan`); commit kecil bermakna diperbolehkan selama fase.
- Tanpa `push`, tanpa `--force`, tanpa rewrite riwayat. Perubahan skema DB wajib migrasi. Rilis diberi tag lokal `v1.0.0-rc.N` (tag lokal diperbolehkan; publikasi butuh owner).
- Sebelum commit: `git status`/`git diff --check`, secret scan, pastikan perubahan owner yang belum ter-commit tidak tertimpa.

## 9. Dokumentasi wajib (hidup, bukan sekali tulis)

PRD, System Architecture, ADR, API Contract (IPC/tool/event), Data Model, Security Standard + Threat Model, Coding Standard, UI/UX Design System, Test Strategy, Runbook operasi, Release Guide, Changelog, `PROJECT_STATUS.md`, `BACKLOG.md`, `RISK_REGISTER.md`. Perubahan kode yang memengaruhi kontrak **harus** memperbarui dokumennya dalam commit yang sama.

## 10. Protokol keputusan teknis

1. Pilihan teknologi baru → bandingkan ≥2 alternatif (kelayakan, keamanan, pemeliharaan, kompatibilitas, lisensi, ukuran/resource) → ADR singkat (`docs/adr/NNN-judul.md`: Context, Decision, Alternatives, Consequences, Risks, Validation).
2. Lisensi: periksa lisensi tiap dependensi dan aset; catat di `docs/legal/THIRD-PARTY.md`. Hati-hati AGPL/GPL pada komponen yang akan di-bundle.
3. Jika dua cara sama baiknya, pilih yang lebih sederhana dan yang sudah ada di repo.

## 11. Pengelolaan risiko

`RISK_REGISTER.md` diperbarui tiap fase: ID, deskripsi, dampak, kemungkinan, mitigasi, pemilik, status. Risiko HIGH/CRITICAL yang diterima butuh persetujuan eksplisit owner (Gate #8).
