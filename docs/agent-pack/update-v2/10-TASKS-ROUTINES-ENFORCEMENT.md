# 10 — TUGAS, RUTINITAS, KOMITMEN & PENEGAKAN

Tiga hal yang dibangun di atas **satu model data di Core** (modul `tasks`, `routines`, `enforce`, `screentime`) dan ditampilkan di notch (ringkas), hub (papan penuh), dan Android.

## 1. TUGAS

### 1.1 Model
```
Task { id, title, category, due?, priority(rendah|biasa|penting), status(todo|doing|done),
       subtasks[], notes?, createdAt, targetDate?, completedAt?, source(manual|chat|shelf|import) }
Category { id, name, color(token), icon }       // default: Kuliah · Proyek · Organisasi · Pribadi · Lainnya (dapat diubah)
```
- **Kategori buatan pengguna** (nama, warna dari palet, ikon dari map). Jangan menanam nama kategori tertentu di kode.
- `targetDate` = **tenggat internal** (lihat aturan tenggat §1.5) — dapat lebih awal dari `due`.

### 1.2 Kartu Tugas di notch (panel **Tugas**)
```
┌ Tugas  · 3 hari ini                    (◔ cincin selesai/total) ┐
│ [Hari ini][Mendatang][Selesai]                                    │  ← Segmented
│ ☐ Laporan SKPL         ● Kuliah   Jum 19:00   (chip tenggat)     │  ← baris 44px
│ ☐ Poster lomba         ● Organisasi  Besok                       │
│ ☑ Baca modul 6         ● Kuliah   Selesai                        │
│ ─────────────────────────────────────────────────────────────── │
│ [ + Tambah tugas…              ] [kategori ▾] [tenggat ▾]       │  ← quick add
└───────────────────────────────────────────────────────────────────┘
```
- **Baris:** checkbox (animasi `task.complete`), judul 1 baris (≤ ~28 kar tampil, ellipsis), **titik/pill kategori** berwarna, **chip tenggat** berwarna menurut urgensi: *terlambat* red · *≤ 1 hari* amber · *≤ 3 hari* sky · lainnya netral. Prioritas "penting" = penanda kecil pink di kiri.
- **Header:** judul + hitungan + **cincin progres** (selesai/total hari ini). Slot kanan pill notch menampilkan **jumlah yang jatuh tempo** (angka kecil) saat panel tertutup.
- **Aksi baris** (hover/geser): selesai, ubah tenggat (+1 hari/besok/pilih), kategori, hapus (urungkan 8 dtk).
- **Tampilan ringkas:** 4–5 baris terlihat; "Lihat semua" → **hub** (papan penuh).
- **Papan penuh (hub):** daftar per kategori + tampilan **Kanban** (kolom Todo/Doing/Done, seret antar kolom), filter kategori/tenggat, subtugas, catatan. Satu bahasa desain; hemat: virtualisasi.
- **Kosong:** ikon + "Belum ada tugas." + tombol "Tambah".

### 1.3 Menambah tugas
1. **Manual (cepat):** kolom *quick add* di panel atau pintasan `Ctrl+Alt+N` (membuka panel Tugas dengan fokus di input). Saat mengetik, **parser bahasa alami** mengubah bagian teks menjadi **pill** hidup: kategori (cocok nama/alias), tenggat ("Jum 19:00"), prioritas ("penting"). Pengguna dapat mengklik pill untuk memperbaiki. `Enter` menyimpan.
2. **Lewat chat:** "ada tugas laporan SKPL kumpul Jumat jam 7 malam, kuliah" → Hime membuat tugas, menampilkan **kartu konfirmasi** kecil di thread (Judul · Kategori · Tenggat sebagai pill dapat diedit) dengan "Urungkan" 8 dtk. Bila yakin → **simpan otomatis**; bila ambigu (mis. tidak jelas Jumat yang mana) → satu pertanyaan singkat. Mendukung beberapa tugas dalam satu pesan.
3. **Dari Simpanan/teks:** seret teks ke kartu Tugas → tugas baru (judul = teks, tenggat kosong).
4. **Dari HP:** sama (16).

### 1.4 Parser tanggal/waktu bahasa Indonesia (deterministik, di Core)
Modul Rust murni dengan **tes properti**: "hari ini, besok, lusa, nanti malam, pagi/siang/sore/malam, jam 8, jam 19.30, pukul 7 malam, Senin–Minggu, Jumat depan, minggu depan, akhir pekan, tgl 25, 25 Nov, 2 hari lagi, 3 jam lagi, sebelum Kamis". Zona waktu perangkat; aturan "hari yang sama sudah lewat → minggu depan" dijelaskan. **LLM hanya fallback** untuk kalimat kompleks; hasil LLM divalidasi parser. Ambigu → tanya.

### 1.5 Aturan tenggat ("harus cepat selesai")
`DeadlineRule { scope: semua|kategori, maxDaysAfterCreated N }` → tugas baru mendapat `targetDate = createdAt + N hari` (dibatasi oleh `due` bila ada). Contoh: "tugas kuliah selesai maksimal 2 hari setelah diberikan."
- Pengingat bertahap: 50% waktu · 80% · hari-H pagi · terlewat; intensitas mengikuti **tingkat penegakan** aturan itu (§3).
- Tugas melewati `targetDate` ditandai (chip amber/red) dan masuk ringkasan harian; **tidak** otomatis mengubah `due`.
- Dapat dibuat lewat chat ("buat aturan: tugas kuliah maksimal 2 hari") atau Settings.

### 1.6 Integrasi Bloom
Timer fokus/Pomodoro Bloom dipakai ulang untuk "Mulai" pada sebuah tugas; kalender Bloom menampilkan tenggat (jika aman menambah titik data tanpa memecah komponen).

## 2. RUTINITAS

### 2.1 Model
```
Routine { id, title, category, schedule{ days[], time, durationMin, windowMin? },
          enforcement(ingatkan|tegas|paksa), allowedApps[]?, distractionRules?, startGraceMin,
          escalation{...}, streak{count,best}, active }
```
Contoh: "Belajar 30 menit, tiap hari 19:00, **Tegas**, aplikasi belajar: [Word, Chrome-belajar, VS Code], pengalih: [media sosial, video, game]".

### 2.2 Alur sesi
1. **T−5 mnt (opsional):** bubble lembut "Belajar 19:00".
2. **Mulai:** bubble P1 + mata `fokus`: "Waktunya belajar 30 mnt" · aksi **[Mulai] [Tunda 10m] [Lewati]** ("Lewati" meminta alasan singkat dari daftar pill: *Sibuk, Sakit, Libur, Lainnya*).
3. **Selama sesi:** timer (cincin di slot kanan), pill menampilkan sisa waktu; pengawasan pengalih (§3) aktif; musik/lirik tetap boleh.
4. **Selesai:** mata `senang/bangga`, streak +1, ringkasan satu baris; bila diabaikan/terlewat: `sedih` lembut + "Mau jadwal ulang?" (tanpa menghakimi).
5. Sesi dapat **dimulai manual** kapan saja (`Ctrl+Alt+F`).

### 2.3 Membuat/mengatur rutinitas
UI di panel **Fokus** (daftar + tambah cepat) dan **Settings → Rutinitas**; atau lewat chat ("aku mau belajar 30 menit tiap hari jam 7 malam, tegas"). Parser sama dengan tugas. Edit/jeda/hapus jelas. Statistik minggu (jumlah sesi, menit, streak) sebagai satu baris + bar kecil.

## 3. KOMITMEN DAN PENEGAKAN

**Komitmen** = aturan yang **dibuat pengguna sendiri** agar Hime menagih: rutinitas, batas pengalih, aturan tenggat, jam tidur (opsional). Setiap komitmen punya **tingkat penegakan**.

### 3.1 Tingkat penegakan (tangga)
| Tingkat | Sifat | Tindakan Hime (laptop) | Tindakan Hime (HP, lihat 16) |
|---|---|---|---|
| **Ingatkan** | Lembut | Bubble P1/P2 + mata; sekali per kejadian; mudah ditunda | Notifikasi biasa |
| **Tegas** | Berulang | Pengingat berulang dengan interval menurun; **panel Island dibuka**; suara pendek opsional; "Tunda" butuh **alasan** (pill) dan dibatasi (mis. 3×); mata `kesal` | Notifikasi berulang + heads-up; **overlay layar penuh** (opt-in) |
| **Paksa** | Menutup pengganggu | Tahap: peringatan → hitung mundur → **tutup jendela pengganggu secara anggun** (§3.3) → ulangi bila dibuka lagi selama sesi/batas | Overlay + (opsional, opt-in) kembali ke Beranda via Aksesibilitas |

Tingkat dipilih per komitmen; ada **eskalasi otomatis** opsional (Ingatkan → Tegas bila diabaikan N kali), dapat dimatikan.

### 3.2 Yang dihitung "pengalih"
- **Kategori aplikasi** (pengguna menentukan): *Belajar* (allowlist), *Netral*, *Pengalih* (media sosial, video, game, dll.). Hime **bertanya sekali** saat melihat aplikasi baru yang sering dipakai ("Kategori untuk X?" → pill Belajar/Netral/Pengalih) dan mengingat.
- **Aturan berbasis waktu:** pengalih **selama blok rutinitas** (langsung ditegakkan), atau **kuota harian** ("pengalih maks 60 mnt/hari di luar belajar"), atau **berurutan** ("setelah belajar 30 mnt baru boleh").
- Di browser: klasifikasi berdasar **aplikasi/domain** yang dikonfigurasi; judul tab sensitif **tidak disimpan** (hanya kategori hasil kecocokan).

### 3.3 Menutup jendela (Windows) — rancangan aman
**Mekanisme:** hanya untuk aplikasi/jendela yang masuk **daftar pengalih** milik pengguna **dan** pada komitmen bertingkat **Paksa**.
1. **Deteksi:** event foreground (`SetWinEventHook` `EVENT_SYSTEM_FOREGROUND`) → cocokkan proses/domain dengan aturan; **tanpa polling**; deteksi idle input (≥ 30 dtk granularitas) agar waktu "menganggur" tidak dihitung.
2. **Peringatan:** bubble P1/P0 "Menutup **X** dalam 10" dengan **[Tunda 5m] (maks 2×)** dan **[Jeda darurat]**; mata `marah/kesal`.
3. **Penutupan anggun:** kirim **`WM_CLOSE`** (permintaan tutup) ke jendela tingkat atas. Aplikasi yang punya perubahan belum disimpan akan **menampilkan dialog simpan** — Hime **tidak memaksa**: menampilkan "Selesaikan dulu" dan menunggu.
4. **Tidak ada pembunuhan proses** (`TerminateProcess`) secara default. Opsi "**tutup paksa**" hanya per aplikasi, **dimatikan default**, memerlukan konfirmasi eksplisit saat membuat aturan, dan **Owner Gate #10** untuk pemakaian pertama.
5. **Browser:** menutup jendela browser menutup semua tab di jendela itu. Default untuk browser = **overlay + saran** (bukan tutup); "tutup jendela" hanya bila dipilih eksplisit dengan peringatan bahwa browser biasanya memulihkan sesi.
6. **Daftar pelindung (tidak pernah disentuh):** proses sistem Windows, shell, keamanan/antivirus, Hime sendiri, aplikasi dalam allowlist belajar, IDE/editor/terminal **kecuali** pengguna memasukkannya ke daftar pengalih secara eksplisit, aplikasi yang sedang merekam/presentasi/layar penuh yang ditandai Jangan Ganggu, dan jendela dengan dialog sistem aktif.
7. **Batas laju:** maks N penutupan per jam; jeda minimal antar tindakan; bila aplikasi dibuka kembali berulang → eskalasi ke overlay/ngobrol, bukan loop tak terbatas.
8. **Jangan** memodifikasi jaringan, menghapus data, atau memblokir akses darurat. Tidak mengunci layar (menagih, bukan memenjarakan).

### 3.4 Pengaman terhadap diri sendiri dan terhadap bug
- **Jeda Darurat** selalu tersedia: tahan 3 dtk atau pintasan `Ctrl+Alt+Shift+P` → pilih alasan (pill) → jeda 15/30/60 mnt; tercatat; jumlah jeda per hari terlihat. Tidak ada cara membuat penegakan menolak jeda.
- **Jeda pelemahan aturan:** mengubah komitmen **Paksa** menjadi lebih longgar atau menghapusnya berlaku setelah **masa tunggu** (default 10 mnt; dapat diatur 0–24 jam *saat membuat aturan*) agar tidak impulsif; **penguatan berlaku seketika**.
- **Kill switch:** `/jeda`, tray "Jeda penegakan", dan pemulihan otomatis bila Core/shell crash (tidak ada keadaan "terkunci").
- **Transparansi:** halaman **Riwayat penegakan** (kapan, apa, tindakan, jeda); ringkasan mingguan.
- **Nada:** suportif, singkat, tidak menghakimi; frekuensi dibatasi (anggaran perhatian 11 §2); tidak ada teks mempermalukan.
- **Uji aman:** semua pengujian penutupan memakai **aplikasi uji buatan sendiri** (jendela uji) dan sandbox; pemakaian pada aplikasi nyata milik owner pertama kali = Gate #10.

### 3.5 Pemantauan penggunaan layar (modul `screentime`)
- **Opt-in** per perangkat; data lokal SQLite: sesi per (aplikasi, kategori, hari); retensi 90 hari (dapat diatur); **hapus semua** satu klik.
- **Tidak** merekam keystroke, layar, isi halaman, atau judul penuh; hanya aplikasi/kategori/durasi.
- Ringkasan ke Core lintas-perangkat (agregat harian) agar kuota gabungan (laptop+HP) dapat dihitung.
- Tampilan: panel/hub "Penggunaan" — bar per kategori (warna kategori), total belajar vs pengalih, tren 7 hari; **tanpa gamifikasi berlebihan** (streak sederhana saja).

## 4. Backend (Core)
Modul: `tasks` (CRUD, aturan tenggat, parser), `routines` (jadwal, sesi, streak), `enforce` (komitmen, eskalasi, Jeda, log), `screentime` (sesi, kuota), `reminders` (timer terkoalesensi — **satu** timer berikutnya, bukan banyak). Jadwal tahan restart; menangani tidur/hibernasi, perubahan jam/zona/DST; tidak ada duplikasi pengingat lintas perangkat (Core sumber kebenaran). Semua tindakan penegakan melewati **Policy Engine** sebagai aksi level tertentu dan tercatat di audit. Penutupan jendela dieksekusi lewat PAL (`hime-pal`), bukan langsung.

## 5. Akseptansi
- [ ] Tambah tugas manual dan lewat chat (kasus jelas/ambigu/multi); parser teruji (tes properti + tabel kasus bahasa Indonesia); kategori buatan pengguna; aturan tenggat bekerja dan memicu pengingat sesuai tingkat.
- [ ] Kartu Tugas di notch sesuai desain (04); papan penuh di hub; cincin/chip/animasi.
- [ ] Rutinitas: buat, jalan, tunda dengan alasan, lewati, selesai, streak; teruji dengan jam palsu (DST, hibernasi).
- [ ] Penegakan: Ingatkan/Tegas/Paksa teruji; **penutupan anggun hanya pada aplikasi uji**; `WM_CLOSE` dengan dialog simpan **tidak dipaksa**; daftar pelindung efektif; browser default aman; batas laju; Jeda Darurat dan jeda pelemahan bekerja; kill switch; log tampil.
- [ ] Screen-time opt-in/out terbukti; tidak ada data keluar; hapus semua bekerja.
- [ ] Tanpa polling idle (event hook + timer tunggal); CPU/RAM sesuai anggaran.
- [ ] Teks UI lolos copy-lint; **VC-U4** (Tugas), **VC-U5** (Rutinitas + nudge/penegakan).
