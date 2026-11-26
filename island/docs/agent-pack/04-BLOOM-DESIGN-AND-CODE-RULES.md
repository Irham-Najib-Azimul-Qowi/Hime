# 04 — BLOOM DESIGN & CODE RULES

Aturan ini berlaku untuk **semua kode shell** (TypeScript/React di `src/`, Rust di `src-tauri/`). Kode **Hime Core** (workspace Rust terpisah) tetap mengikuti `legacy/04-…` dan `legacy/06-…`.

## 1. Hierarki otoritas aturan
1. `AGENTS.md`, `CONTRIBUTING.md`, `SETTINGS.md`, `TRAY_INTEGRATION.md`, `RELEASE_CHECKLIST.md` milik Bloom.
2. `docs/hime/BLOOM-CONVENTIONS.md` (hasil studimu dari source nyata, Stage H0).
3. Dokumen ini.
4. Preferensi pribadi agent — **tidak berlaku** bila bertentangan dengan 1–3.

> Prinsip: *kode Hime di dalam shell harus tidak bisa dibedakan dari kode Bloom oleh pembaca yang tidak tahu siapa penulisnya.*

## 2. Aturan Bloom yang sudah diketahui (konfirmasi di source)

**Frontend:** React 19 + TS strict (`noUnusedLocals/Parameters` menggagalkan build); Bun; **tidak ada `package-lock.json`**; Vite port 1420 strict; jangan jalankan Vite terpisah; `bun run build` = typecheck (`tsc && vite build`); format `bun run format` (oxfmt).
**Rust:** `cargo check --locked`, `cargo clippy --locked --all-targets` (gerbang CI); jalankan `cargo test --locked` sendiri; crate `windows` dipatok 0.61 + fitur eksplisit; semua `#[tauri::command]` didaftarkan di `generate_handler!` (`main.rs`); `commands.rs` = IPC, `services.rs` = pekerja Win32, `utils.rs` = settings cache/IO, `state.rs`/`types.rs` = bersama.
**Event:** backend→frontend lewat `app.emit`, frontend lewat `@tauri-apps/api/event`.
**Settings:** `%APPDATA%/bloom/settings.json`, objek datar, kunci berawalan `bloom-`, nilai string; tulis: `localStorage` → `invoke("save_setting")`; sinkron lewat `useSettingsSync` saja; dokumentasikan di `SETTINGS.md`.
**Jaringan:** CSP allow-list; origin baru ditambah eksplisit; plugin baru → `capabilities/default.json`.
**Exit:** setiap jalur exit/restart baru **wajib** memulihkan taskbar dan melepas AppBar.
**Commit:** Conventional Commits (`feat:`, `fix:`, `chore:`); satu perubahan terfokus per commit; pembersihan tak terkait dipisah.
**Dependensi:** tambahkan hanya dengan alasan kuat.
**Verifikasi Windows:** perubahan fokus/taskbar/tray/DPI harus diuji di Windows nyata; build saja tidak cukup.

## 3. Tambahan khusus Hime (bukan pengganti aturan Bloom)

| Topik | Aturan |
|---|---|
| **Isolasi** | Kode Hime baru di `src/hime/**` (frontend) dan `src-tauri/src/hime/**` (Rust). Sentuhan ke berkas Bloom hanya untuk **mendaftarkan** (panel, command, event, window, tab settings) dan dicatat di `docs/hime/TOUCHPOINTS.md` (berkas, baris, alasan, cara mem-revert) |
| **Penamaan** | Command Rust: `hime_*`. Event: `hime-*` (kebab). Kunci settings baru: `hime-*` (bukan `bloom-*`), string, terdokumentasi di `SETTINGS.md` (bagian "Hime"). Komponen React: nama yang mengikuti pola Bloom, diletakkan di `src/hime/` |
| **Rahasia** | **Tidak pernah** di `settings.json`, `localStorage`, event, atau log. Rahasia hanya di vault Core |
| **State** | State otoritatif tinggal di Core; UI hanya cache tampilan. Jangan menyimpan percakapan/task hanya di frontend |
| **Jaringan shell** | Shell **tidak** memanggil Hermes, 9Router, provider, atau internet untuk fitur Hime. Semua lewat Core. (Weather dsb. milik Bloom tetap) |
| **Idle** | Tanpa polling terjadwal di idle. Hanya event dari Core. Window baru dibuat *on-demand* dan ditutup saat tak dipakai |
| **Ketahanan** | Core mati/tidak ada → shell tetap berfungsi penuh sebagai Bloom; fitur Hime menampilkan state "offline" yang jujur. **Shell tidak pernah crash atau hang karena Core** |
| **Exit** | Bridge tidak boleh menambah jalur exit baru. Bila terpaksa, ikuti pola restorasi taskbar |
| **Tes** | Tes Rust murni untuk modul `hime/` (pola `cargo test --locked` Bloom). Tes frontend: ikuti yang ada; jika tidak ada infrastruktur, jangan menambah framework besar — gunakan tipe ketat + verifikasi visual terdokumentasi |
| **AGENTS.md** | Tambahkan bagian "Hime" di **akhir** `AGENTS.md` (jangan ubah bagian Bloom): perintah build Core, lokasi kode Hime, aturan isolasi, aturan settings Hime, link ke pack |

## 4. Larangan "AI slop" (penyebab kegagalan Fase 07)

Dilarang keras di semua permukaan:
1. Jendela dengan **chrome bawaan Windows** (title bar, tombol minimize/maximize/close) sebagai Island/notch. Island/notch/dock/overlay adalah window **borderless/transparan** seperti Bloom.
2. **Persegi translucent besar** berisi pill kecil. Island adalah *satu bentuk* (pill) yang **berubah bentuk** (lebar, tinggi, radius, posisi) dengan spring, bukan jendela dengan elemen di dalamnya.
3. Estetika generik: gradien ungu/biru, glow neon, emoji sebagai ikon (✨ dsb.), card bertumpuk tak perlu, "dashboard AI" template.
4. Sistem desain baru, palet baru, font baru, set ikon baru, pustaka UI baru (shadcn, MUI, dll.) — **pakai yang Bloom pakai**.
5. Teks UI berlebihan, copywriting marketing, status palsu, tombol yang tidak berfungsi.
6. Animasi yang tidak memakai mesin spring Bloom, atau yang membuat CPU idle naik.
7. Menyatakan UI "selesai" tanpa screenshot nyata pada Windows.

## 5. Protokol aman menjalankan shell pengganti taskbar (WAJIB)

Bloom menyembunyikan taskbar Windows dan mendaftarkan AppBar saat berjalan; bug dapat membuat layar owner macet.

1. Buat **`scripts/restore-taskbar.ps1`** (idempoten, tanpa admin): matikan proses shell Hime/Bloom, hapus AppBar, pulihkan taskbar dan pengaturan auto-hide, restart `explorer.exe` bila perlu. Uji skrip ini **sebelum** menjalankan Bloom pertama kali. Dokumentasikan di `docs/hime/RECOVERY.md` juga cara manual untuk owner (Ctrl+Shift+Esc → akhiri proses; jalankan skrip).
2. Buat **`scripts/run-safe.ps1 -Seconds N [-Screenshot path]`**: jalankan `bun run tauri dev` (atau build rilis lokal) **dengan batas waktu**, ambil screenshot di momen yang ditentukan, lalu **selalu** hentikan proses (`try/finally`) dan panggil `restore-taskbar.ps1`.
3. **Jangan pernah meninggalkan shell berjalan tanpa pengawasan** saat sesi kerja berakhir/menunggu. Pada akhir setiap sesi: pastikan taskbar pulih.
4. Jangan menguji jalur crash/kill di mesin owner kecuali melalui `run-safe.ps1`.
5. Bila terminal Antigravity tidak punya sesi desktop interaktif (tidak bisa screenshot/menampilkan window): jangan berpura-pura; tandai `NOT TESTED` dan buat **permintaan uji visual untuk owner** (templat di berkas 09) berisi langkah persis + daftar screenshot yang diminta.

## 6. Protokol akseptansi visual

Setiap perubahan yang tampak oleh pengguna melewati ini sebelum disebut selesai:

**6.1 Bloom-native checklist** (semua harus ya, dengan bukti screenshot):
- [ ] Wadah, padding, radius, warna, tipografi, bayangan, ukuran ikon **identik dengan panel Bloom yang sudah ada** (nilai diambil dari kode, bukan ditebak).
- [ ] Transisi memakai mesin spring Bloom dengan konfigurasi yang sama (atau konfigurasi dari tabel Bloom yang paling dekat); lebar/tinggi/radius/posisi beranimasi mandiri.
- [ ] Perilaku interaksi sama: klik untuk membuka, hover hanya mengintip, scroll/swipe berganti panel, panel arrows, memori panel terakhir.
- [ ] Set ikon yang sama; tidak ada emoji.
- [ ] Tidak ada chrome window; tidak ada persegi translucent besar.
- [ ] Padat dan ringkas (bukan dashboard); teks minimal; bahasa Indonesia konsisten.
- [ ] Mode visibilitas Fixed/Smart/Peek, multi-monitor, DPI 100%/125%, layar 1366×768 diuji.
- [ ] State: idle, hover/peek, terbuka, loading, kosong, error, offline (Core mati).
- [ ] Keyboard dan fokus tidak merusak (tidak mencuri fokus dari aplikasi pengguna kecuali interaksi eksplisit).
- [ ] CPU/RAM idle dibandingkan baseline Bloom; delta dicatat.

**6.2 Bukti:** `docs/hime/visual/<fitur>/` berisi screenshot sebelum/sesudah, **berdampingan** dengan panel Bloom pembanding, + catatan perbedaan yang disengaja dan alasannya.

**6.3 Gerbang owner:** milestone visual berikut **wajib** dikirim ke owner untuk persetujuan sebelum dianggap selesai (lihat `OWNER VISUAL CHECK` di berkas 09): (a) irisan vertikal pertama di Stage H2; (b) panel Assistant + bubble aktivitas/approval; (c) Translation di Island; (d) Workspace window; (e) tiap rilis kandidat. Sambil menunggu, **lanjutkan pekerjaan non-UI**; jangan mem-polish UI lebih jauh dan jangan menyatakan visual diterima.

## 7. Gerbang kualitas shell (sebelum commit lokal)
```
bun install --frozen-lockfile      # (tanpa membuat package-lock.json)
bun run build                      # typecheck frontend (strict)
bun run format                     # lalu pastikan diff format bersih
cd src-tauri
cargo check --locked
cargo clippy --locked --all-targets
cargo test --locked                # CI tidak menjalankan ini; kamu wajib
```
Tambahan: gerbang Core (`legacy/04` bagian 7), secret scan, dan verifikasi visual + resource untuk perubahan UI.
