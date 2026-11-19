# PHASE 15 — HARDENING, RELEASE & LINUX
**Pengujian menyeluruh · Verifikasi GUI penuh · Keamanan · Performa · Reliabilitas · Installer/Update · Dukungan Linux · Pemeliharaan**

**Entry:** Fase 14 `READY FOR OWNER ACCEPTANCE`. Fitur v1.0 lengkap.
**Kewajiban dibawa (harus ditutup di sini bila belum):** G2 (GUI penuh), G3/G4 (IB-01/IB-02), G10 (Linux PAL), G11 (lisensi/aset), G12 (benchmark).
**Kebutuhan pengguna:** UR-17, UR-18, serta kriteria v1.0 (`02-…` bagian 7).
**Owner Gates:** push/rilis publik (Gate #3), menerima risiko HIGH/CRITICAL (Gate #8), penandatanganan kode dengan sertifikat milik owner, penerimaan formal fase.

## 1. Tujuan
Menjadikan Hime layak dipakai rutin: terbukti benar, aman, stabil, hemat resource, dapat dipasang/diperbarui/dipulihkan, dan (sebagai target resmi berikutnya) berjalan di Linux dengan batas yang jujur.

## 2. Task

### 15.1 Pengujian fungsional dan GUI (menutup G2)
- **Sesi GUI interaktif lengkap:** setiap layar dan interaksi utama: Island (idle/expanded/activity/approval/terjemahan/error), Omnibox, Conversation (streaming/stop/error), Activity, Task Board/Kanban, Notification Center, Settings, Storage, Devices, Documents, Browser/Dev panel, Floating Companion (drag, posisi tersimpan), tray, shortcut global, startup/shutdown.
- **Screenshot** setiap state (disimpan di `docs/qa/screenshots/`), **multi-monitor** (bila tak ada monitor kedua: simulasi dengan virtual display dan catat sebagai terbatas), **DPI scaling** 100/125/150/200%, layar kecil 1366×768, mode gelap/terang bila ada.
- E2E otomatis (Playwright untuk WebView Tauri atau alat setara) untuk alur kritis; uji regresi fitur Fase 06–14.
- **Alur lintas perangkat:** instruksi dari HP → Core → agent → approval di HP → hasil; notifikasi; reconnect.
- **Pemulihan kegagalan:** matikan Hermes/9Router/Core secara paksa, putus jaringan, disk penuh, DB terkunci.

### 15.2 Pengujian keamanan (`06-…` bagian 11)
Permission bypass; autentikasi/otorisasi (desktop IPC, 9Router, perangkat); injeksi (prompt, command, path, SQL); kebocoran data (kanari di semua jalur termasuk crash dump); eksposur layanan lokal (pindai port, uji dari proses lain dan dari LAN); penanganan kredensial; kriptografi pairing; dependensi (`cargo audit/deny`, `pnpm audit`, Gradle dependency check, pemindai Python); lisensi; pemindaian rahasia pada seluruh riwayat Git lokal; fuzzing parser IPC/protokol perangkat (frame cacat). Hasil: `docs/security/SECURITY-TEST-REPORT.md`. **Nol temuan CRITICAL/HIGH** atau penerimaan eksplisit owner.

### 15.3 Pengujian performa (menutup G12)
Ukur pada hardware target: startup (Core, Desktop, Island tampil), RAM idle/aktif/berat (per proses dan total), CPU, performa DB, throughput indeks file, penggunaan jaringan, beban latar, baterai Android. Bandingkan dengan anggaran (`03-…` bagian 7). Optimasi bila melampaui; bila tak bisa, catat penyimpangan + alasan + usulan. **Tidak ada angka tanpa pengukuran.**

### 15.4 Audit UI/UX
Konsistensi visual dan interaksi, aksesibilitas (keyboard, kontras, skala teks, pembaca layar dasar), responsivitas, animasi (tidak janky di UHD 620; hormati "reduce motion"), state error/kosong/loading, kegunaan (uji dengan skenario tugas nyata). Perbaiki temuan MAJOR; MINOR ke backlog.

### 15.5 Pengujian reliabilitas
Crash proses, kehilangan jaringan, perangkat offline, transfer terputus, pemulihan DB (korupsi simulasi, restore backup), kegagalan update, rollback, kehabisan disk/RAM, tidur/hibernasi sistem, perubahan jam/zona.

### 15.6 Persiapan rilis (Windows)
- **Installer:** Tauri bundler (NSIS atau MSI) — memasang Hime.exe, Core, runtime terkelola Hermes dan 9Router (versi terkunci), tool dokumen opsional (komponen terpisah), templat, skrip migrasi; **tanpa admin bila memungkinkan** (per-user install).
- **Compatibility manifest** terisi (Hermes, 9Router, skema DB, IPC, protokol perangkat).
- **Updater:** unduhan diverifikasi (hash/tanda tangan), backup sebelum migrasi DB, migrasi dapat diverifikasi, **rollback otomatis bila gagal**, komponen runtime dapat diperbarui terpisah selama kontrak kompatibel.
- **Penandatanganan kode:** siapkan alur; sertifikat milik owner (Gate). Tanpa itu: dokumentasikan peringatan SmartScreen.
- **Versioning dan catatan rilis** (SemVer), `CHANGELOG.md`, tag lokal `v1.0.0-rc.N`.
- **Dokumentasi pengguna:** panduan instalasi, penggunaan, pengaturan keamanan, pemecahan masalah, privasi (apa yang dikumpulkan), pemulihan.
- **Lisensi dan atribusi:** `docs/legal/THIRD-PARTY.md` lengkap; pastikan **tidak ada aset berhak cipta Nusa** (G11); tinjau AGPL/GPL (mis. Ghostscript) terkait distribusi.

### 15.7 Dukungan Linux (menutup G10) — "cross-platform by design"
**Pendekatan bertahap, klaim hanya setelah uji nyata:**
1. **PAL Linux:** IPC → Unix Domain Socket (izin 0600, direktori `$XDG_RUNTIME_DIR`); secret → Secret Service/libsecret (fallback terenkripsi dengan keyring pengguna, **bukan plaintext**); jalur data → XDG; notifikasi → freedesktop; supervisor → process group + `PR_SET_PDEATHSIG` + subreaper, **cgroups v2 hanya bila tersedia dan diizinkan** (tidak diasumsikan untuk non-root); pemantau jendela aktif (X11/Wayland sesuai kemampuan).
2. **Build:** CI lokal/WSL atau mesin Linux untuk `cargo build`/test target Linux; Tauri Linux (WebKitGTK); paket AppImage/deb/rpm sesuai kebutuhan.
3. **Island di Linux:** X11 relatif fleksibel; **Wayland membatasi** posisi/ukuran/always-on-top — rancang perilaku degradasi anggun (jendela biasa/panel), dokumentasikan perbedaan GNOME vs KDE. **Jangan menjanjikan perilaku identik.**
4. **Uji:** Ubuntu LTS dan Fedora (GNOME dan KDE Plasma; X11 dan Wayland). Fitur yang bergantung Windows (Job Object, DPAPI) harus memiliki padanan atau dinyatakan tidak tersedia.
5. **Status:** dinyatakan **"Beta Linux"** dengan daftar fitur didukung/tidak didukung bila uji penuh belum mungkin pada mesin owner; jangan mengklaim "didukung penuh".

### 15.8 Pemeliharaan
Triase bug, pembaruan keamanan dan dependensi (jadwal), pemantauan performa, regression suite terotomasi, pembaruan dokumentasi, `RUNBOOK` operasi dan pemulihan, proses pelaporan masalah.

## 3. Deliverables
Laporan QA (fungsional/GUI + screenshot), `SECURITY-TEST-REPORT.md`, `PERFORMANCE-REPORT.md`, `UIUX-AUDIT.md`, `RELIABILITY-REPORT.md`, installer Windows (artefak lokal), updater + manifest, dokumentasi pengguna dan pengembang, `docs/legal/THIRD-PARTY.md`, PAL Linux + build Linux (Beta), `docs/linux/LINUX-SUPPORT.md`, `CHANGELOG.md`, `project-management/RELEASE-CRITERIA-CHECKLIST.md`, `FINAL-HANDOVER-REPORT.md`.

## 4. Exit gate = Kriteria rilis v1.0
Seluruh 15 butir di `02-…` bagian 7 berstatus PASS (dengan bukti) atau risiko diterima eksplisit oleh owner; nol temuan keamanan CRITICAL/HIGH terbuka; IB-01 dan IB-02 RESOLVED (atau jalur kredensial sensitif dinonaktifkan dengan keputusan owner terdokumentasi); installer/updater/rollback berfungsi; G2/G10/G11/G12 ditutup atau dicatat sebagai keterbatasan rilis.

**Status akhir yang diperbolehkan untukmu:** `READY FOR OWNER ACCEPTANCE (v1.0 RC)`. Penerimaan formal, push, dan publikasi rilis adalah hak owner.
