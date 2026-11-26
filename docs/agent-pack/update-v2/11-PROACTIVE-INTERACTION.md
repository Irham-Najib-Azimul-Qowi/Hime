# 11 — HIME PROAKTIF: CARA BERINTERAKSI

Hime bukan alat pasif. Ia **menyapa, bertanya, mengingatkan, dan membantu merapikan** — di Windows dan HP — tanpa menjadi cerewet. Dokumen ini mendefinisikan kepribadian, aturan perhatian, dan katalog interaksi.

## 1. Persona (ringkas; lengkap di 17)
Teman belajar yang tenang dan cekatan. Bahasa Indonesia santai-sopan ("aku/kamu"), **singkat**, tidak berlebihan, tidak menjilat, tidak menghakimi. Memberi pilihan konkret (maks 3 tombol), mengingat jawaban, dan jujur tentang yang ia lakukan. Nada naik-turun sesuai konteks: ringan saat santai, tegas saat penegakan, hangat saat pengguna kesulitan.

## 2. Aturan perhatian (anggaran gangguan)
**Tingkat gangguan:**
| Tingkat | Bentuk | Contoh |
|---|---|---|
| **Bisik** | Perubahan halus (ekspresi mata, titik di pill) | mata `penasaran` saat ada file baru |
| **Colek** | Bubble ringkas tanpa aksi wajib, auto-hilang | "Disalin", saran ringan |
| **Tanya** | Bubble dengan ≤ 3 aksi | "File sudah ada. Ganti?" |
| **Penting** | Ekspansi + suara/getar pendek | approval, rutin mulai, tenggat hari-H |

**Anggaran (default "Seimbang"; Settings → Hime: `Pendiam` / `Seimbang` / `Cerewet`):**
| | Pendiam | Seimbang | Cerewet |
|---|---|---|---|
| Colek per jam | 1 | 3 | 6 |
| Tanya per jam | 1 | 3 | 5 |
| Penting | tak dibatasi tetapi hanya yang benar-benar perlu | sama | sama |
Melebihi anggaran → ditunda/digabung ke **ringkasan** (mis. "3 hal menunggu").

**Aturan emas:**
1. **Satu pertanyaan sekali.** Jangan menumpuk.
2. **Beri default yang masuk akal**; tombol pertama = pilihan paling mungkin.
3. **Ingat keputusan** ("Selalu begini" menjadi aturan yang dapat dilihat/diubah/dicabut).
4. **Belajar dari penolakan:** ditutup/diabaikan 3× → turunkan frekuensi jenis itu (backoff eksponensial, terlihat di Settings, dapat direset).
5. **Hormati konteks:** fokus, layar penuh, presentasi, rapat, jam tenang, mengetik, di luar jam aktif.
6. **Jangan mengulang** hal yang sudah ditolak hari ini.
7. **Jujur & dapat dibatalkan:** setiap tindakan Hime punya "Urungkan" bila mungkin; tidak ada kejutan destruktif.
8. **Kurangi, jangan tambah:** bila ragu antara bicara atau diam → diam, kecuali P0/P1.
9. **Jam tenang** (default 23:00–06:00, dapat diatur): hanya P0; mata boleh `ngantuk` tanpa teks.

## 3. Katalog interaksi (Windows ↔ HP)

Kolom: pemicu → tampilan Windows (via NCM, 06) → HP → ekspresi → teks contoh (≤ 40 kar) → pilihan. Semua teks tunduk copy-lint.

| # | Situasi | Windows | HP | Mata | Teks | Pilihan |
|---|---|---|---|---|---|---|
| 1 | **File ganda terdeteksi** (hash sama atau nama serupa di Unduhan/folder) | Bubble P2 → ekspansi | Notifikasi + aksi | penasaran | "File sudah ada. Ganti?" | **Ganti** (lama→v1) · **Simpan dua** · **Hapus baru** · "Selalu…" |
| 2 | **File baru di Unduhan** | Bubble P2 (setelah unduhan selesai) | — | penasaran | "Simpan ke mana?" | 3 folder saran · **Lainnya** · **Biarkan** |
| 3 | **Storage hampir penuh** (laptop/HP) | Bubble P2 | Notifikasi | khawatir | "C: tersisa 8%" | **Lihat** · **Nanti** |
| 4 | **Tenggat dekat** (H−1, hari-H pagi) | Bubble P1 | Notifikasi | netral/khawatir | "Laporan SKPL besok 19:00" | **Mulai** · **Tunda** · **Selesai** |
| 5 | **Rutinitas mulai** | Bubble P1 (+ panel Fokus) | Notifikasi + opsional overlay | fokus/semangat | "Waktunya belajar 30 mnt" | **Mulai** · **Tunda 10m** · **Lewati** |
| 6 | **Rutinitas terlewat** | Bubble P2 (lembut) | Notifikasi | sedih | "Belajar terlewat. Jadwal ulang?" | **Jam 21:00** · **Besok** · **Lewati** |
| 7 | **Pengalih melewati batas** | Bubble P1 → (Paksa) hitung mundur | Notifikasi/overlay | kesal/marah | "Sudah 45 mnt di luar belajar" | **Lanjut belajar** · **Tunda 5m** · **Jeda** |
| 8 | **Ringkasan pagi** (pemakaian pertama hari itu) | Panel Hime muncul 6 dtk | Notifikasi pagi | senang | "3 tugas hari ini. Belajar 19:00" | **Lihat** · **Tutup** |
| 9 | **Rekap malam** (opsional, 21:30) | Bubble P2 | Notifikasi | tenang | "2 selesai, 1 geser besok" | **Lihat** · **Tutup** |
| 10 | **Tugas selesai** | Bubble P4 singkat | — | senang/bangga | "Selesai. Mantap." | — |
| 11 | **Idle saat blok belajar** | Bubble P2 | Notifikasi | netral | "Masih belajar?" | **Ya** · **Istirahat** · **Selesai** |
| 12 | **Teks asing** (mode otomatis) | Bubble P4 | — | penasaran | "Terjemahkan?" | **Terjemah** · **Abaikan** |
| 13 | **Baterai rendah** (laptop/HP) | Bubble P2 | Notifikasi | khawatir | "Baterai 15%" | **Mode hemat** · **Tutup** |
| 14 | **HP terhubung/terputus; transfer selesai** | Bubble P3/P4 | — | netral | "HP terhubung" / "3 file diterima" | **Lihat** |
| 15 | **Agent selesai tugas panjang** | Bubble P3→P4 | Notifikasi | senang | "Ringkasan siap" | **Buka** |
| 16 | **Approval aksi berisiko** | Bubble P0 | Notifikasi + aksi | netral/waspada | "Pindahkan 12 file?" | **Setujui** · **Tolak** · **Detail** |
| 17 | **Runtime AI mati** | Pill offline + bubble P2 (sekali) | Notifikasi | khawatir | "Hime offline" | **Coba lagi** |
| 18 | **Larut malam** (ada rutinitas tidur atau pengguna masih aktif) | Mata `ngantuk` tanpa teks; bubble hanya jika rutinitas tidur ada | Notifikasi lembut | ngantuk | "Sudah larut. Tidur?" | **Sebentar lagi** · **Oke** |
| 19 | **Kembali setelah lama pergi** | Sapaan kecil | — | senang | "Selamat datang kembali." | — |
| 20 | **Aplikasi baru sering dipakai (belum dikategorikan)** | Bubble P2 | — | penasaran | "Kategori untuk Discord?" | **Belajar** · **Netral** · **Pengalih** |
| 21 | **Clipboard berisi tautan/teks panjang** (jika diminta) | Bubble P4 | — | netral | "Simpan sebagai tugas?" | **Ya** · **Abaikan** |
| 22 | **Pembaruan/kesalahan penting** | Bubble P2 | Notifikasi | pusing/khawatir | "Ada masalah. Lihat?" | **Lihat** |

Katalog dapat bertambah, tetapi **setiap interaksi baru wajib**: pemicu jelas, tingkat gangguan, prioritas NCM, teks lolos lint, batas frekuensi, opsi "ingat", cara mematikan (per jenis), dan tes keputusan.

## 4. Mekanisme (arsitektur)
- **Interaction Engine** (modul Core `interact`): menerima peristiwa dari modul lain (file, tugas, rutin, perangkat, runtime), memutuskan **apakah, kapan, di mana, dan dengan teks apa** (aturan 2), lalu mengirim `Presentation` ke NCM (06) dan/atau notifikasi ke Android.
- **Satu pintu keluar:** modul tidak berbicara ke pengguna sendiri; semua lewat Interaction Engine (konsisten, dapat dibatasi, mudah diuji).
- **Memori preferensi:** tabel `interaction_prefs` (jenis → keputusan/aturan, backoff, terakhir ditunjukkan); dapat dilihat dan diubah di Settings → "Hime bertanya".
- **Template teks** terpusat di registry string (lint); variasi kecil agar tidak monoton (2–3 varian) tetapi tetap singkat.
- **Tanggapan pengguna** (aksi, tutup, abaikan, tunda) dicatat untuk backoff dan statistik lokal.
- **Lintas perangkat:** Core menentukan **perangkat mana** yang diberi tahu (yang aktif/terakhir digunakan) untuk menghindari notifikasi ganda; kecuali P0/P1 penegakan yang berlaku di perangkat bersangkutan.

## 5. Akseptansi
- [ ] Setiap baris katalog 1–22 diimplementasikan atau **dinyatakan ditunda** dengan alasan; tiap yang ada punya tes keputusan (pemicu, anggaran, konteks).
- [ ] Anggaran per jam bekerja (uji dengan banjir peristiwa); ringkasan menggabungkan.
- [ ] Backoff setelah 3 penolakan; dapat dilihat dan direset; "Selalu…" menjadi aturan yang dapat dicabut.
- [ ] Jam tenang, fokus, layar penuh ditaati.
- [ ] Tidak ada notifikasi ganda lintas perangkat.
- [ ] Pengguna dapat mematikan tiap jenis interaksi; mode Pendiam/Seimbang/Cerewet terasa berbeda.
- [ ] **VC-U6** (rangkaian skenario 1, 2, 5, 7, 16 dengan screenshot Windows + tangkapan HP).
