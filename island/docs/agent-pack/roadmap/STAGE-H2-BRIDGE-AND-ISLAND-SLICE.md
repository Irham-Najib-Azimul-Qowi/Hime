# STAGE H2 — BRIDGE DAN IRISAN VERTIKAL ISLAND

**Tujuan:** satu irisan vertikal **nyata dan terlihat**: shell Bloom (Hime) memulai Hime Core, terhubung lewat IPC, menampilkan status Hime di Island dengan bahasa visual Bloom, dan menjalankan satu alur approval nyata melalui bubble. **Inilah ujian visual pertama** — kegagalan Fase 07 tidak boleh terulang.
**Entry:** H0 dan H1 selesai. **Rujukan:** berkas 04 (aturan + akseptansi visual), 06, 07.

## Task

### H2.1 Rebrand minimal (aman)
`productName`, `identifier` (berbeda dari Bloom), judul window; direktori data Hime via **satu konstanta** bila memungkinkan (bila tidak, pertahankan path Bloom dan dokumentasikan konflik bila Bloom asli terpasang). Ikon: pertahankan sementara; **jangan membuat logo asal-asalan** → tanyakan owner (OD) bila ingin aset Hime. Catat semua di `TOUCHPOINTS.md`.

### H2.2 Bridge
Implementasikan `src-tauri/src/hime/` (client, services, commands, mod) sesuai berkas 06: spawn/ensure Core, handshake, health, reconnect backoff, forwarder event → `app.emit("hime-…")`, command `hime_core_status` dan seperlunya. Mode offline anggun. Tidak menambah jalur exit; shutdown Core mengikuti jalur exit yang ada. Dokumentasikan `docs/api/SHELL-CORE-CONTRACT.md`. Tes Rust murni (`cargo test --locked`) untuk parsing/validasi/state machine koneksi.

### H2.3 Frontend dasar
`src/hime/` (api, events, state, types) mengikuti pola Bloom (hasil H0). Satu hook langganan event Hime. Registrasi minimal di `App.tsx` (satu titik).

### H2.4 Indikator Hime di Island
Tambahkan indikator ringkas Hime pada **pill/status strip** dengan gaya Bloom: siap / bekerja / menunggu approval / offline. Tidak mengubah layout Bloom yang ada kecuali yang diperlukan dan tercatat. Klik → panel Hime (H2.5).

### H2.5 Panel Assistant (kerangka jujur)
Panel Island bergaya Bloom dengan: status runtime ("AI belum terhubung" sampai 08R — **tampilkan apa adanya**), daftar percakapan kosong, input **dinonaktifkan** dengan alasan. Tidak ada respons palsu. Daftarkan sebagai mode panel Island (scroll/swipe) dengan memori panel Bloom.

### H2.6 Bubble approval (alur nyata)
Gunakan lifecycle approval Core yang sudah ada. Tambahkan perintah **hanya di build debug** (`hime_dev_trigger_approval`, ditandai jelas dan tidak masuk rilis) yang meminta Core membuat tiket approval uji. Bubble approval (bentuk rounded-square Bloom) muncul → klik → Island berekspansi menampilkan aksi, pratinjau, **Setujui/Tolak**, hitung mundur TTL → keputusan dikirim ke Core → bubble hilang dan hasil tampil singkat. Kasus: setuju, tolak, **TTL kedaluwarsa**, tiket dipakai ulang (ditolak Core), Core mati saat approval. Prioritas bubble sesuai berkas 07 §3.

### H2.7 Settings
Satu tab "Hime" di Settings (pola Bloom) berisi: status Core, toggle `hime-enabled`, versi, tautan dokumen. Kunci `hime-*` didokumentasikan di `SETTINGS.md`.

### H2.8 Verifikasi
1. Gerbang shell + Core (berkas 04 §7).
2. **Uji kegagalan:** Core tidak ada, Core dibunuh saat berjalan, pipe putus, token salah, event banjir, shell restart saat Core hidup. Shell tidak pernah hang/crash.
3. **Resource:** delta idle CPU/RAM terhadap baseline Bloom (tanpa polling idle).
4. **Visual:** screenshot semua state (idle, peek, terbuka, offline, bekerja, approval [4 kasus]) di 100%/125% DPI dan 1366×768, **berdampingan** dengan panel Bloom pembanding; checklist Bloom-native (berkas 04 §6.1) **seluruhnya** terisi.
5. Dokumen: `docs/hime/visual/h2/…`, `TOUCHPOINTS.md` final, laporan.

### H2.9 Gerbang owner — **VC-01**
Kirim `OWNER VISUAL CHECK` (templat 09 T-C) berisi screenshot dan cara menjalankan aman. **Lanjutkan pekerjaan non-UI (PHASE-08R backend: adapter, keamanan, IB-01/02)** sambil menunggu. Jangan mem-polish UI lebih jauh sebelum jawaban owner. Jika owner meminta revisi: perbaiki **di dalam bahasa visual Bloom**, bukan dengan desain baru.

## Exit gate
- Shell terhubung ke Core, mode offline anggun, shutdown bersih; taskbar selalu pulih (diuji).
- Alur approval nyata end-to-end (4 kasus) — `VERIFIED RUNTIME` dengan screenshot.
- Tidak ada polling idle; delta resource tercatat.
- Checklist Bloom-native lengkap; **VC-01 terkirim** (diterima/revisi dicatat).
- Commit lokal `feat(hime): ...`. Laporan `STAGE-H2-REPORT.md`.
