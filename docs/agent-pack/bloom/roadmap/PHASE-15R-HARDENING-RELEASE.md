# PHASE 15R — HARDENING, RELEASE

**Lingkup:** `legacy/roadmap/PHASE-15-HARDENING-RELEASE-LINUX.md` **kecuali** §15.7 (Linux) yang **dibatalkan untuk v1.0** (keputusan D8; lihat `02-STRATEGY-AND-DECISIONS.md`). Tetap: pengujian fungsional, keamanan (`legacy/06` §11), performa (dibanding baseline Bloom), UI/UX audit, reliabilitas, persiapan rilis, pemeliharaan.

## Penyesuaian khusus Hime×Bloom

### 15R.1 Verifikasi GUI penuh (menutup G2/G17)
Sesi interaktif lengkap: seluruh panel/bubble/Workspace/Settings Hime **dan** fungsi inti Bloom (dock, notch, shelf, Wi-Fi/Bluetooth, media, Win+1..9, tray) — **regresi Bloom** harus nol (fitur Bloom tidak boleh rusak oleh Hime). Matriks: DPI 100/125/150/200%, 1366×768, multi-monitor, Fixed/Smart/Peek, mode gelap/terang bila ada, hibernasi/sleep, perubahan skala saat berjalan. Screenshot lengkap. **VC-10 (rilis kandidat)** ke owner.

### 15R.2 Keselamatan shell
Uji semua jalur exit/crash: kill shell, crash webview, Core mati, update gagal → **taskbar selalu pulih**, tidak ada window mati yang menutupi klik; `restore-taskbar.ps1` terbukti. Marker file/watchdog Bloom tetap berfungsi dengan tambahan Hime.

### 15R.3 Saluran pembaruan Hime
Updater Bloom tetap nonaktif sampai saluran pembaruan **Hime** ada: rancang (manifest bertanda tangan, hash, rollback, backup DB sebelum migrasi); implementasi bila owner menyediakan tempat hosting (Gate #3); jika tidak → pembaruan manual + dokumentasi.

### 15R.4 Installer dan manifest
`bun run tauri build` menyertakan Core (sidecar), Hermes dan 9Router terkelola (versi terkunci), templat, skrip migrasi; **manifest kompatibilitas** memuat tag/SHA Bloom dasar, versi Hermes, 9Router, skema DB, protokol IPC. Install per-user tanpa admin bila memungkinkan. Penandatanganan kode: sertifikat milik owner (Gate); tanpa itu, dokumentasikan peringatan SmartScreen/Defender.

### 15R.5 Lisensi dan atribusi
Kepatuhan GPL-3.0: `LICENSE`, pemberitahuan hak cipta Bloom, penandaan perubahan, instruksi mendapatkan source; `THIRD-PARTY.md` lengkap (Bloom, dependensi, Hermes, 9Router, tool dokumen, Ghostscript/AGPL jika relevan); tanpa aset Nusa.

### 15R.6 Kriteria rilis v1.0 (re-target)
Seluruh 15 butir `legacy/02-… §7` dengan penyesuaian: butir 1 dibaca sebagai "**Hime shell (fork Bloom)** — semua permukaan Hime dan fungsi Bloom terverifikasi pada sesi GUI interaktif" dan Linux dikecualikan. Status akhir yang boleh kamu tetapkan: `READY FOR OWNER ACCEPTANCE (v1.0 RC)`.
