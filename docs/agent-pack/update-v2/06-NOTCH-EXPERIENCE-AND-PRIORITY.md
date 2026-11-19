# 06 — NOTCH EXPERIENCE & PRIORITY (MANAJEMEN KEMUNCULAN)

Notch itu kecil. Banyak modul ingin tampil. Tanpa pengatur, hasilnya berisik dan berebut. Spesifikasi ini mendefinisikan **satu arbiter** (Notch Compose Manager, NCM) yang memutuskan **apa tampil, di mana, kapan, berapa lama**, sehingga pengalaman tenang dan dapat diprediksi.

## 1. Anatomi

| Bagian | Isi | Catatan |
|---|---|---|
| **Pill** | Bentuk utama di tengah atas. **Slot kiri:** jam *atau* mata *atau* ikon status. **Tengah:** judul/info 1 baris (mis. lirik, "Menerjemahkan"). **Slot kanan:** hitungan/indikator (mis. 3 tugas, progres ring) | Satu bentuk yang bermorfosis (mekanik Bloom) |
| **Bubble** (maks 2) | Rounded-square di kiri/kanan pill; untuk hal yang berjalan atau butuh perhatian | Gaya Bloom; isi = ikon/mata + 1 baris |
| **Panel** | Kartu terbuka dengan klik; berganti dengan scroll/swipe/pintasan | Chat, Tugas, Simpanan, Musik, Fokus, Pusat Kontrol, Kalender, Status … |
| **Layar besar (large)** | Notch membesar untuk chat panjang atau kartu rinci | Maks ±60% lebar × ±55% tinggi layar |
| **Hub** | Window `hub` on-demand untuk papan tugas penuh, storage, perangkat | **Bukan chat**; dibuat saat dibutuhkan, dibebaskan setelah ditutup |

## 2. Daftar panel dan urutan default
Urutan ditetapkan agar yang paling sering dipakai paling dekat. Dapat diatur di Settings → Notch; maksimal **6 panel terlihat**, sisanya di "Lainnya".

| # | Panel | Ikon (map) | Deskripsi UI | Sumber |
|---|---|---|---|---|
| 1 | **Hime** (chat) | chat | "Tanya Hime" | modul chat |
| 2 | **Tugas** | tugas | "3 tenggat minggu ini" | modul tugas |
| 3 | **Simpanan** | simpanan | "Teks dan file sementara" | modul simpanan |
| 4 | **Musik** | musik | judul lagu | Bloom (+lirik) |
| 5 | **Fokus** | fokus | "Sesi belajar" | rutinitas + timer Bloom |
| 6 | **Kontrol** | kontrol | volume, kecerahan, Wi-Fi | Bloom |
| — | Kalender, Status, lainnya | | | Bloom |

Aturan: panel terakhir dibuka diingat (opsi Bloom). Panel modul yang **dimatikan** tidak muncul di daftar dan tidak memuat kode.

## 3. Penyajian (presentation) dan prioritas

Setiap modul tidak menampilkan apa pun sendiri. Ia **mengajukan presentation** ke NCM:
```
Presentation { id, module, kind, priority, ttl, interruptibility, userInitiated, coalesceKey, payload, actions[], expression? }
kind: ambient | bubble | alert | panel-hint | card
```

### 3.1 Tangga prioritas (angka kecil = lebih penting)
| Prio | Kelas | Contoh | Perilaku |
|---|---|---|---|
| **P0** | Kritis | **Approval** aksi berisiko; penegakan Paksa tahap akhir; keamanan | Menang atas semua; ekspansi; tidak tertutup tak sengaja; TTL → tolak otomatis (approval) |
| **P1** | Berbatas waktu | Rutinitas mulai; tenggat hari-H; timer fokus selesai; pengalih melewati batas | Bubble menonjol + ekspresi; bisa tunda |
| **P2** | Pertanyaan/aksi | Unduhan ganda ("Ganti?"), "simpan ke mana?", permintaan pairing, nudge tegas | Bubble berisi ≤ 3 aksi; timeout; "ingat pilihan" |
| **P3** | Progres | Agent bekerja, transfer file, dokumen dibuat | Bubble ringkas + progress; menyatu bila banyak |
| **P4** | Kontekstual | **Terjemahan**, salinan clipboard, saran Hime | Bubble/kartu dengan auto-tutup |
| **P5** | Ambient | Mata (ekspresi), baris lirik, jam | Hanya bila tidak ada yang lebih penting |

**Input pengguna menang:** `userInitiated=true` (klik pill, hotkey, seret ke notch) naik satu kelas di atas apa pun kecuali P0.

### 3.2 Aturan arbiter
1. **Maksimal 2 bubble** sekaligus (kiri/kanan) + pill. Yang lebih rendah antre atau menyatu.
2. **Coalesce:** presentation dengan `coalesceKey` sama digabung ("3 file baru", bukan 3 bubble).
3. **Preempt:** yang lebih penting menggeser yang lebih rendah; yang tergeser **kembali** setelahnya bila masih relevan (TTL belum habis).
4. **Tidak menutupi pekerjaan pengguna:** bila pengguna sedang mengetik di chat atau menyeret sesuatu, hanya P0 yang boleh mengambil alih (dan tidak merebut fokus keyboard).
5. **Mode senyap/fokus/Jangan Ganggu:** hanya P0 dan P1 yang lolos (dan P1 ditandai "boleh di fokus" per modul); sisanya ditahan di antrean atau dibuang bila kedaluwarsa.
6. **Layar penuh/presentasi:** hanya P0 (tanpa suara).
7. **Anggaran perhatian** (11 §2) membatasi P2–P4 per jam; melebihi → digabung ke ringkasan.
8. **TTL & auto-tutup:** P3 mengikuti proses; P4 6–10 dtk (hover menahan); P2 sesuai timeout aksi; P1 sampai ditanggapi atau ditunda; P5 4–7 dtk.
9. **Gerakan penolakan:** geser/klik ✕ = tutup; `Esc`; tutup berulang oleh pengguna → pembelajaran (backoff) di modul asal (11).
10. **Satu sumber kebenaran:** semua tampilan berasal dari NCM; modul tidak boleh membuka bubble sendiri (dicegah oleh API: tidak ada jalur lain).
11. **Multi-monitor:** tampil di monitor tempat notch aktif (ikuti Bloom).
12. **Keluaran suara/getar (HP):** hanya P0–P1 dan P2 yang dipilih pengguna.

### 3.3 Konflik umum (jawaban pasti)
| Situasi | Hasil |
|---|---|
| Lirik tampil + terjemahan diminta | Terjemahan (P4) menggeser lirik (P5) sementara; lirik kembali setelah ditutup |
| Agent bekerja + approval muncul | Approval (P0) ekspansi; bubble agent tetap di sisi lain |
| 2 unduhan ganda berurutan | Satu bubble "2 file ganda" → ekspansi daftar |
| Rutinitas mulai saat fokus lain berjalan | P1 bubble; menawarkan "Gabung/Ganti" |
| Chat terbuka + nudge belajar | Nudge `Colek` ditunda sampai chat ditutup, kecuali tenggat < 1 jam (P1) |
| Mic aktif | Mata `dengar`; P3–P5 ditahan |

## 4. Slot dan tata letak
- **Slot kiri** (hanya satu): prioritas isi → mata (ekspresi bermakna) > ikon status > jam. Jam **selalu kembali** bila tidak ada yang lebih penting.
- **Slot tengah:** baris pendek bila perlu (lirik/judul/aksi); disembunyikan jika tak ada.
- **Slot kanan:** hitungan (tugas, antrean), progress ring, indikator perangkat. Maks satu indikator utama + titik kecil untuk yang lain.
- Lebar pill menyesuaikan isi dengan spring; tidak melompat.

## 5. Gestur dan input
Klik pill = buka panel terakhir; klik bubble = ekspansi aksi; scroll/swipe = ganti panel (mekanik Bloom); drag file/teks ke pill = **Simpanan** (zona jatuh), ke area chat = lampiran; klik kanan pill = menu cepat ("Fokus", "Jeda", "Pengaturan"); `Esc` menutup; hover = peek saja. Pintasan: lihat 13.

## 6. State notch (diagram)
```
TERTUTUP (jam/mata)
  ├─ peek (hover)  ──klik──► TERBUKA (panel)
  ├─ bubble(s)     ──klik──► TERBUKA (ekspansi aksi)
  └─ hotkey        ───────► TERBUKA / BESAR (chat)
TERBUKA ──perlu ruang──► BESAR ──Esc/klik luar──► TERTUTUP
Presentation P0 dapat memaksa TERBUKA dari state mana pun.
```
Tutup otomatis: klik di luar, Esc, atau (opsional) tak ada interaksi N detik (kecuali chat berjalan/menunggu input).

## 7. Implementasi (arah)
- **Core** menyimpan antrean dan kebijakan (aturan 3.2 yang bergantung konteks: fokus, anggaran, fullscreen); **shell** merender dan menangani animasi/input. Presentation melintas IPC sebagai event `hime-present` (bertipe, berversi).
- NCM di shell = state machine murni (mudah diuji): input = daftar presentation + state; output = tata letak. **Wajib tes unit tabel keputusan** (semua baris 3.3 + prioritas + coalesce + TTL).
- Modul mengajukan lewat `ctx.present(...)` (SDK modul, 14). Tidak ada akses langsung ke komponen notch.
- Log keputusan NCM (mode debug) untuk menjelaskan "kenapa ini tampil".

## 8. Akseptansi
- [ ] Semua presentation lewat NCM; tidak ada jalur lain (diperiksa statis).
- [ ] Tabel keputusan lulus; skenario 3.3 teruji end-to-end.
- [ ] Maks 2 bubble; coalesce bekerja; preempt dan kembali bekerja.
- [ ] Mode fokus/senyap/layar penuh menahan sesuai aturan.
- [ ] Jam selalu kembali; tidak ada elemen "nyangkut".
- [ ] Panel modul nonaktif tidak muncul dan tidak memuat kode.
- [ ] **VC-U1** mencakup rangkaian skenario konflik (screenshot/urutan frame).
