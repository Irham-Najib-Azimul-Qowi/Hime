# 15 — OPTIMASI RESOURCE (RAM/CPU/BATERAI)

**Keluhan owner:** total RAM yang dipakai sistem berjalan cukup besar. **Tujuan:** Hime Island seringan mungkin tanpa mengorbankan fitur: **bayar hanya untuk yang dipakai.** Target di bawah adalah **sasaran yang harus divalidasi dengan pengukuran**, bukan fakta.

## 1. Sumber biaya (peta)
| Komponen | Pendorong biaya | Tuas utama |
|---|---|---|
| **WebView2** (tiap window = instance webview; `main`/notch, `dock`, `overlay`, `settings`, `hub`) | proses renderer/GPU/utility, DOM, JS heap, cache | window on-demand, suspend/low-memory saat tersembunyi, bundel kecil |
| **Shell Rust** (Tauri) | thread, hook, cache gambar/thumbnail | event-driven, thread minimal, cache berbatas |
| **Hime Core** | thread runtime async, SQLite, modul | modul lazy/idle-stop, worker kecil |
| **Hermes (Python)** | interpreter + pustaka + konteks | **mulai on-demand, berhenti saat idle** |
| **9Router (Node.js)** | V8 heap | on-demand, batas heap; opsi pengganti (§6) |
| **Worker dokumen/OCR/STT** | LibreOffice, Tesseract, model suara | on-demand, keluar setelah selesai |
| **Android** | layanan latar, polling | event-driven, adaptif |

## 2. Anggaran (target; ukur lalu sahkan di ADR)
Pengukuran = **jumlah private working set** seluruh pohon proses milik Hime (`hime-island.exe`, `hime-core.exe`, `msedgewebview2.exe` milik app, `python`/`node` runtime, worker). Perangkat acuan: i5-8250U, UHD 620, Windows 11.

| Keadaan | Target RAM total | CPU rata-rata | Catatan |
|---|---|---|---|
| **Idle** (notch tertutup, semua modul ON tetapi tidak aktif, **runtime AI berhenti**) | **≤ 350 MB** | **< 0.5%** | Mode Hemat: **≤ 250 MB** |
| **Chat aktif** (Hermes + 9Router hidup) | **≤ 900 MB** | sesuai beban | termasuk transien streaming |
| **Puncak sementara** (dokumen/OCR/STT) | **+≤ 500 MB** | — | **dilepas ≤ 60 dtk** setelah selesai |
| **Android idle** | **≤ 80 MB** proses app | ~0 | layanan latar minimal |

**Aturan keras:** setiap modul mendeklarasikan `ramEstimateMB`; **hime perf** memverifikasi. Regresi > 10% atau melampaui anggaran tanpa ADR = **gagal gerbang**.

## 3. Prosedur wajib (berbasis bukti)
1. **Baseline Bloom** asli (H0): RAM per proses (shell + tiap WebView2), CPU idle, wakeup — `docs/hime/baseline/RESOURCES.md`.
2. `scripts/measure-resources.ps1` (dipanggil `hime perf`): sampling tiap 5 dtk selama N menit, menjumlahkan private working set pohon proses Hime, CPU (counter proses), jumlah proses, handle; keluaran CSV + ringkasan; skenario: idle, chat, translate, lirik, shelf, enforcement, dokumen.
3. **Satu perubahan → ukur → simpan/kembalikan** (catat di `docs/hime/perf/LOG.md`: tuas, sebelum/sesudah, keputusan). Jangan mengoptimasi tanpa angka.
4. Alat bantu: Task Manager WebView2/Edge (`CoreWebView2.OpenTaskManagerWindow()` atau Shift+Esc pada instance yang mendukung), Process Explorer, Performance Monitor/`typeperf`, WPR/ETW bila perlu, DevTools Performance/Memory untuk frontend.
5. Laporan resource di tiap laporan tahap (templat: sebelum/sesudah, delta terhadap baseline, kontributor terbesar).

## 4. Tuas optimasi (urut dampak yang diperkirakan; semua **diverifikasi**)

### 4.1 WebView2 / window
1. **Window on-demand:** `settings` dan `hub` dibuat saat dibuka dan **dihancurkan** saat ditutup (bukan hanya disembunyikan). `overlay` dibuat saat dibutuhkan; bila HUD overlay Bloom dapat digabung ke window `main` tanpa merusak perilaku, evaluasi (perubahan besar → ADR).
2. **Saat tersembunyi/idle:** gunakan API WebView2 — `MemoryUsageTargetLevel = Low` (`ICoreWebView2_19`) untuk menurunkan memori ketika window tidak terlihat/tidak aktif; dan/atau `TrySuspend()` pada webview yang tersembunyi lama, `Resume()` saat tampil. **Dokumentasi Microsoft menyatakan keduanya ada; verifikasi** ketersediaan pada runtime WebView2 yang terpasang dan akses via Tauri/wry (`with_webview` + crate `webview2-com`), pilih satu strategi per window berdasar pengukuran (jangan menumpuk tanpa bukti).
3. **Satu environment/profil data bersama** untuk semua webview (default Tauri) agar proses browser dibagi; verifikasi jumlah proses `msedgewebview2.exe`.
4. **Argumen browser** (`additionalBrowserArgs`) hanya setelah diukur manfaatnya (mis. fitur yang dimatikan); jangan menyalin daftar bendera tanpa bukti.
5. Hindari efek GPU mahal pada UHD 620: `backdrop-filter`, blur besar, bayangan lebar, animasi `filter` (lihat 04/05); ukur dengan dan tanpa.

### 4.2 Frontend
- **Code-splitting per modul/panel** (`import()` dinamis); modul OFF tidak diunduh/dieksekusi.
- **Pustaka minimal:** tanpa pustaka besar (moment/lodash penuh/UI kit); gunakan `Intl`, utilitas kecil; ikon **per-impor**; font subset woff2 (Latin).
- **Render hemat:** memoization terarah, daftar **tervirtualisasi** (chat/riwayat/Simpanan/tugas), state terlokalisasi, hindari re-render global; `content-visibility` untuk daftar panjang.
- **Tidak ada loop idle:** satu `MotionController` hanya aktif saat ada animasi; timer dikoalesensi; `visibilitychange`/event window menghentikan pekerjaan saat tersembunyi.
- **Gambar:** thumbnail kecil on-demand, format hemat, cache berbatas byte (LRU).

### 4.3 Shell Rust
- **Event-driven penuh:** `SetWinEventHook` (foreground), `AddClipboardFormatListener`, `RegisterHotKey`, notifikasi media (GSMTC), WASAPI notifikasi — **tanpa polling**; timer dikoalesensi.
- **Thread minimal:** runtime async `current_thread` atau pool kecil bila memungkinkan; tidak ada busy-loop; kanal **berbatas**.
- **Profil rilis:** `lto`, `codegen-units=1`, `strip`, `opt-level` (ukur `s` vs `3`), `panic` sesuai kebutuhan isolasi; hapus dependensi tak terpakai (`cargo machete`); fitur crate dipangkas.
- **Cache berbatas** (thumbnail, ikon aplikasi dock) dengan batas byte dan pembersihan.

### 4.4 Hime Core
- **Modul lazy + idle-stop** (14 §3); modul OFF tidak dimulai.
- **Worker async kecil** (default 2) dapat diatur; hindari thread per tugas.
- **SQLite hemat:** WAL, `synchronous=NORMAL`, `cache_size` kecil (mis. ±2 MB), `mmap_size=0`, indeks hanya yang perlu; **indeks file di disk (FTS5)**, bukan di memori; operasi batch.
- **Alokator:** evaluasi `mimalloc` untuk Core (ukur fragmentasi/RSS) — hanya bila terbukti.
- **Antrean berbatas** + backpressure; cache LRU berbatas byte; tidak ada koleksi yang tumbuh tanpa batas (periksa dengan uji soak).

### 4.5 Runtime AI (biaya terbesar)
1. **Mulai on-demand, berhenti saat idle** (default **10 menit**, pengaturan `hime-ai-idle-min`). Pemanasan cepat: saat pengguna membuka panel chat, mulai Hermes di latar ("pre-warm") bila pola pemakaian menunjukkan; jangan menjalankannya pada boot.
2. **Jalur cepat tanpa Hermes** untuk terjemah, parsing tugas, lirik, klasifikasi sederhana (Core → penyedia lewat adapter): **runtime berat tidak dinyalakan** untuk hal itu.
3. **Hermes:** profil minimal (nonaktifkan plugin/alat bawaan yang tak dipakai); lingkungan Python bersih (tanpa pustaka tak dipakai); `PYTHONDONTWRITEBYTECODE` sesuai, `PYTHONOPTIMIZE` bila aman; pemakaian satu proses; **ukur RAM nyata**.
4. **9Router (Node):** batas heap (`--max-old-space-size` rendah, mis. 128–192 MB), tanpa modul tak perlu, **ukur**.
5. **Opsi struktural (butuh Owner Gate #7 + ADR):** bila Node + Python melampaui anggaran, evaluasi **router provider native di Core (Rust)** menggantikan 9Router (juga menyelesaikan IB-01/IB-02 lebih bersih) atau runtime Hermes yang lebih ringan. **Jangan** dilakukan tanpa persetujuan.
6. Multi-run: batas paralel mengikuti pengukuran RAM (default 2).

### 4.6 Worker berat
LibreOffice headless, Tesseract, STT lokal: **spawn per pekerjaan, keluar setelah selesai**, batas waktu/memori (Job Object), satu pekerjaan berat sekaligus; model STT dimuat saat dipakai dan **dibongkar setelah 60 dtk idle**.

### 4.7 Animasi & mata
Kompositor saja (transform/opacity); tanpa rAF idle; ambient dihentikan saat layar penuh/hemat/tersembunyi; 30 fps adaptif bila frame lambat (05 §5.4).

### 4.8 Android
Layanan latar **minimal** (foreground service hanya bila perlu; WorkManager untuk periodik); **event-driven** bila Aksesibilitas aktif, **polling adaptif** (hanya saat aturan aktif) bila tidak; sinkron dibatch; patuh Doze; tanpa wakelock panjang; R8/shrink, baseline profile; ukuran APK terukur.

## 5. Mode Hemat
Pengaturan `hime-save-mode` (`off/auto/on`; **auto** aktif saat baterai-hemat sistem, RAM rendah, atau di baterai < 20%).
Efek: mata statis (tanpa ambient), tanpa blur/transisi dekoratif (cross-fade 120 ms), idle-stop runtime AI lebih cepat (3 mnt), modul opsional berjalan "malas" (indeks file dijeda, lirik tanpa karaoke), pekerja Core diturunkan, polling HP lebih jarang. **Fitur inti tetap bekerja.** Indikator kecil di Settings → Sumber Daya.

## 6. Halaman "Sumber Daya" (Settings)
Daftar proses Hime dengan RAM/CPU **langsung** (data dari Core), total, status modul + RAM tiap modul, tombol "Hentikan runtime AI sekarang", Mode Hemat, ambang peringatan (bubble bila total melampaui anggaran). Murah: pembaruan hanya saat halaman terlihat.

## 7. Gerbang dan regresi
- `hime perf` dalam `hime check` (versi singkat 3 menit) membandingkan dengan `docs/hime/perf/BASELINE.json`; gagal bila melampaui anggaran/regresi tanpa ADR.
- Uji **soak** (≥ 60 menit simulasi pemakaian): tidak ada kebocoran (RAM stabil), tidak ada pertumbuhan handle/thread.
- Uji **ON/OFF modul** (nol-biaya) dan **window on-demand** (RAM kembali ke baseline setelah ditutup, ≤ 60 dtk).

## 8. Akseptansi
- [ ] Baseline Bloom + anggaran disahkan (ADR) dengan angka nyata.
- [ ] Idle, chat, puncak, dan Mode Hemat memenuhi anggaran atau penyimpangan dicatat dengan alasan dan rencana.
- [ ] Runtime AI on-demand/idle-stop terbukti (RAM turun setelah idle); jalur cepat tidak menyalakannya.
- [ ] Window on-demand dan strategi suspend/low-memory terukur manfaatnya.
- [ ] Tanpa polling idle (audit panggilan timer/hook); CPU idle < 0.5%.
- [ ] Halaman Sumber Daya dan Mode Hemat bekerja; gerbang `perf` aktif.
- [ ] Laporan `docs/hime/perf/REPORT.md` (tuas, angka, keputusan).
