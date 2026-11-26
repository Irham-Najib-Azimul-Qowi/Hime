# 05 — MOTION & EYES (MASKOT HIME)

## 1. Konsep maskot
Hime = **dua mata bot** (kiri dan kanan): kapsul tegak berwarna aksen di atas latar gelap. Tidak ada wajah lain, tidak ada mulut. Ekspresi dibentuk hanya oleh **bentuk mata, kelopak, kemiringan, arah pandang, dan gerak mikro**. Mata menggantikan ikon bot klasik di seluruh produk.

**Tempat tampil:**
| Tempat | Perilaku |
|---|---|
| Notch tertutup | Menampilkan **jam** secara default; **sesekali mata tampil menggantikan jam** (ambient), lalu kembali ke jam |
| Chat terbuka | Bagian atas yang biasanya jam menampilkan **mata terus-menerus**, bereaksi pada keadaan chat (mendengar, berpikir, menjawab, error) |
| Bubble/kartu | Ikon status = mata mini (ekspresi sesuai konteks) |
| Avatar chat, tray, ikon aplikasi, Android | Mark statis atau mata mini sesuai tempat |

## 2. Model parametrik mata (sumber tunggal `design/eyes.json`)

Setiap mata digambar dari satu kapsul + dua **kelopak** (atas, bawah). Satuan relatif terhadap ukuran dasar mata (`1.0`). Setiap ekspresi mendefinisikan parameter untuk mata kiri dan kanan (boleh berbeda, mis. kedip sebelah).

| Parameter | Arti | Rentang |
|---|---|---|
| `w`, `h` | skala lebar/tinggi | 0.6–1.5 |
| `r` | radius sudut (rasio) | 0.3–0.5 |
| `lidTop` | porsi mata tertutup dari atas | 0–1 |
| `tilt` | kemiringan kelopak atas (derajat; **+ = sisi dalam turun** → marah; **− = sisi luar turun** → sedih) | −25…+25 |
| `lidBot` | porsi tertutup dari bawah (melengkung ke atas → "^ ^" senang) | 0–0.6 |
| `dx`, `dy` | arah pandang (offset) | −0.5…0.5 |
| `rot` | rotasi pasangan mata (miring kepala) | −8…8 |
| `color` | token aksen | — |
| `opacity` | untuk redup/offline | 0.4–1 |

**Rendering:** SVG; mata = `<rect rx>`; kelopak = **SVG mask** (bukan penutup berwarna latar, supaya berfungsi di latar apa pun); transisi ekspresi = **spring pada parameter** (bukan morph path); gerak mikro = keyframe CSS (kompositor). Android: Compose Canvas dengan `animate*AsState` dari JSON yang sama.

Contoh skema (agar semua ekspresi seragam):
```json
{
  "netral":  { "l": {"w":1,"h":1,"r":0.5,"lidTop":0,"tilt":0,"lidBot":0,"dx":0,"dy":0}, "color":"cyan",  "blink":[3000,7000] },
  "senang":  { "l": {"w":1,"h":0.9,"r":0.5,"lidTop":0,"tilt":0,"lidBot":0.45,"dx":0,"dy":-0.05}, "color":"mint", "motion":"bounce-soft" },
  "marah":   { "l": {"w":1.05,"h":1,"r":0.4,"lidTop":0.35,"tilt":22,"lidBot":0,"dx":0,"dy":0}, "color":"red",  "motion":"jitter-2px" }
}
```
`r` (kanan) mengikuti `l` kecuali dinyatakan (mis. `wink`).

## 3. Katalog ekspresi (minimal 22; tambah bila perlu dengan skema yang sama)

Nama ekspresi = kunci JSON. "Prioritas" menentukan siapa menang bila dua pemicu bersamaan (angka kecil = lebih penting). Durasi = tampil minimal–maksimal. Semua ekspresi **kembali ke netral/jam** setelah selesai.

| Ekspresi | Bentuk (ringkas) | Warna | Pemicu utama | Prio | Durasi |
|---|---|---|---|---|---|
| `netral` | kapsul penuh, kedip acak 3–7 dtk, saccade kecil | cyan | default | 9 | — |
| `senang` | kelopak bawah melengkung "^ ^", memantul lembut | mint | tugas/rutin selesai, ucapan terima kasih, kembali setelah istirahat | 5 | 2–4 dtk |
| `semangat` | lebih tinggi, dua denyut | mint | awal sesi fokus, streak naik | 6 | 1.5–3 |
| `bangga` | "^ ^" tenang + sedikit naik | lime | target harian tercapai, tugas besar selesai | 5 | 3–5 |
| `fokus` | menyipit datar, stabil, jarang berkedip | lime | sesi fokus/rutin berjalan | 4 | selama sesi |
| `berpikir` | menatap kiri-atas, mata menyempit sedikit, pandangan berayun | violet | agent bekerja | 3 | selama proses |
| `dengar` | sedikit lebih tinggi + **cincin denyut** mengikuti level suara | mint | mikrofon aktif | 2 | selama rekam |
| `penasaran` | mata kiri lebih besar, kepala miring, menatap ke arah objek | cyan | file baru di Unduhan, perangkat baru, teks asing terdeteksi | 6 | 3–5 |
| `kaget` | melebar tinggi, "pop" cepat | cyan terang | peristiwa tak terduga, error besar | 4 | 1–2 |
| `khawatir` | alis (kelopak) turun di sisi luar sedikit, mata agak besar | amber | baterai/storage rendah, runtime AI error | 3 | 4–6 |
| `bosan` | kelopak setengah turun datar, melirik pelan kiri-kanan | cyan redup | idle siang > 20 mnt, tak ada aktivitas | 8 | 4–7 |
| `ngantuk` | kelopak atas turun ±50%, kedip lambat, kepala turun perlahan | cyan 70% | 23:00–04:59 saat ambient/idle | 7 | 5–8 |
| `menguap` | menyipit → menutup rapat → membuka | cyan 70% | malam, sekali tiap 10–15 mnt | 7 | 1.6 |
| `tidur` | garis tipis, napas pelan | cyan 50% | 01:00–04:59 + idle > 10 mnt (bila notch tampil) | 8 | selama idle |
| `kesal` | kelopak atas datar, melirik ke samping | coral | pengalih terdeteksi (tingkat Tegas), pengingat diabaikan | 4 | 3–5 |
| `marah` | kelopak miring ke dalam 22°, getar 2 px | red | penegakan Paksa aktif, tunda 3×, tenggat lewat jauh | 2 | 3–5 |
| `sedih` | kelopak miring ke luar, mata turun sedikit | sky | rutin terlewat/streak putus (lembut, dengan teks suportif) | 5 | 3–5 |
| `curiga` | menyipit satu sisi, melirik | amber | membuka pengalih saat jadwal belajar (peringatan ringan) | 5 | 2–4 |
| `nakal` | kelopak miring 10° + sedikit "^", melirik | pink | peringatan bercanda tingkat Ingatkan; langka | 7 | 2–3 |
| `kedip-sebelah` | satu mata menutup cepat | mint | respons terima kasih, ambient langka (≤ 1/hari) | 8 | 0.6 |
| `pusing` | mata mengorbit kecil berlawanan | violet | tidak paham/gagal ringan | 6 | 1.2 |
| `offline` | garis redup rata, opacity 0.4 | abu (`text-3`) | Core mati/putus | 1 | selama offline |

**Catatan nada:** ekspresi negatif (`marah`, `sedih`, `kesal`) **tidak menghakimi**; selalu dipasangkan teks pendek yang suportif/netral (lihat 11). Frekuensi dibatasi (bagian 4).

## 4. Idle Director (mesin kemunculan mata)

Modul `eyes-director` memutuskan **kapan** dan **ekspresi apa**. Event-driven dan hemat.

**Aturan:**
1. **Default = jam.** Mata muncul sebagai *ambient peek* 4–7 detik pada interval acak sesuai pengaturan **Mata di notch**: `Mati` · `Jarang` (≈ 20–40 mnt) · `Sedang` (≈ 8–20 mnt) · `Sering` (≈ 3–8 mnt). Default: `Sedang`.
2. **Pilih ekspresi** dari kolam berbobot: konteks waktu (malam → `ngantuk/menguap`), aktivitas terakhir (selesai tugas → `senang`), keadaan (baterai rendah → `khawatir`), dan sedikit `netral/bosan/kedip-sebelah`. Tidak mengulang ekspresi yang sama dua kali berturut-turut.
3. **Pemicu kontekstual** (tabel §3) mengambil alih sesuai prioritas dan lewat **Notch Compose Manager** (06), tidak langsung.
4. **Ditahan (suppress)** ketika: aplikasi layar penuh/presentasi (`SHQueryUserNotificationState`), mode Jangan Ganggu, sesi fokus (hanya `fokus`), Mode Hemat (mata statis, tanpa ambient), baterai-hemat sistem, pengguna sedang mengetik di chat.
5. **Chat terbuka:** mata permanen di header; ekspresi mengikuti keadaan: menunggu = `netral`; merekam = `dengar`; proses = `berpikir`; selesai = `senang` singkat; gagal = `khawatir`/`pusing`; runtime mati = `offline`.
6. **Anggaran gangguan:** ekspresi non-darurat dihitung ke anggaran perhatian (11 §2).
7. **Pandangan mengikuti kursor:** hanya bila mata tampil **dan** kursor dalam radius notch; diperbarui ≤ 30 Hz; dapat dimatikan.
8. **Gerak mikro:** kedip acak, saccade kecil, napas (skala 1→1.02, 4 dtk). Hanya saat mata tampil.
9. **Morph jam ↔ mata:** angka jam memudar naik, garis tipis muncul di tempat mata, **membuka** dengan spring (≈ 220 ms); sebaliknya saat kembali. Bentuk pill tidak berubah ukuran mendadak.
10. **Uji:** aturan waktu/anggaran diuji unit dengan jam palsu; tiap ekspresi punya screenshot (`docs/hime/visual/eyes/`).

## 5. Sistem animasi

### 5.1 Prinsip
Animasi adalah informasi. Selalu **dapat diinterupsi** (ganti target tanpa lompatan). Cepat (≤ 320 ms untuk sebagian besar). **Satu mesin spring** (milik Bloom; verifikasi) dipakai untuk notch dan Hime; CSS/WAAPI untuk gerak sederhana.

### 5.2 Preset (titik awal; ganti dengan preset Bloom bila ada)
| Preset | Stiffness / Damping / Mass | Kesan | Pemakaian |
|---|---|---|---|
| `snappy` | 520 / 38 / 1 | cepat, tegas | toggle, pill, tombol |
| `smooth` | 320 / 30 / 1 | halus | morph notch, panel, kartu |
| `soft` | 200 / 26 / 1 | lembut | ekspresi mata, progress |
| `bouncy` | 380 / 18 / 1 | memantul (jarang) | kemunculan bubble, selesai tugas |
Durasi acuan: mikro 120–180 ms · standar 220–320 ms · besar 380–500 ms (**tidak lebih**, kecuali ambient mata). Stagger 20–30 ms per item (maks 6 item).

### 5.3 Katalog transisi
| # | Nama | Perilaku |
|---|---|---|
| 1 | `notch.morph` | tertutup ↔ terbuka ↔ besar: lebar, tinggi, radius, posisi masing-masing spring; isi cross-fade setelah ukuran ±60%; elemen bersama (mata/jam) bergerak, bukan hilang-muncul |
| 2 | `panel.switch` | geser 24 px + fade menurut arah indeks; ikon/label stagger; dapat diinterupsi |
| 3 | `bubble.in/out` | tumbuh dari tepi pill (skala 0.6→1, overshoot ≤ 4%); keluar mengecil ke pill; bubble sejenis **menyatu** (coalesce) |
| 4 | `approval.attention` | tepi bubble "bernapas" 1.6 dtk, pelan; **tanpa goyang** |
| 5 | `task.complete` | centang tergambar 160 ms → baris menutup (spring) → progress naik → mata `senang` |
| 6 | `pill.select` | pop 0.94→1 (`snappy`) |
| 7 | `chat.message` | fade + naik 8 px; teks streaming tanpa lompat layout (ruang dipesan) |
| 8 | `run.progress` | bar spring; selesai → kartu hasil menggantikan (morph) |
| 9 | `shelf.drop` | zona jatuh membesar; item terbang ke tempat (FLIP); saat diseret: terangkat skala 1.04 |
| 10 | `translate.expand` | bubble membesar menjadi kartu; terjemahan streaming |
| 11 | `list.reflow` | tambah/hapus/urut dengan FLIP spring, tanpa layout thrash |
| 12 | `skill.pill.insert` | pill muncul di caret (skala), menu menutup (fade 120 ms) |
| 13 | `mic.pulse` | cincin denyut dari level audio, ≤ 30 Hz, hanya saat merekam |
| 14 | `lyric.scroll` | baris aktif membesar/terang, lainnya redup; scroll vertikal spring |
| 15 | `state.offline` | mata meredup dan merata; elemen Hime memudar ke abu |

### 5.4 Aturan performa animasi (UHD 620)
- Animasikan **`transform`, `opacity`**, ukuran notch (sesuai Bloom). **Jangan** animasikan `filter`, `backdrop-filter`, `box-shadow`, atau properti yang memicu layout pada banyak elemen.
- **Satu `MotionController`** dengan satu loop `requestAnimationFrame` yang **hanya berjalan saat ada animasi aktif**; berhenti saat diam. Tidak ada rAF di idle.
- Frame budget: bila waktu frame > 24 ms selama 1 dtk → turunkan ke 30 fps/hentikan dekorasi (ambient) otomatis.
- `will-change` hanya selama animasi. Jeda semua animasi saat window tersembunyi (`visibilitychange` + event window Tauri).
- **Reduce motion / Mode Hemat:** ganti semua transisi dengan cross-fade 120 ms; mata statis.
- Android: `Animatable`/`animate*AsState`, hindari recomposition berlebih, uji pada perangkat kelas menengah.

### 5.5 Pengujian
Screenshot sebelum/sesudah tiap transisi (frame kunci); uji interupsi (ganti target di tengah animasi); ukur CPU saat animasi dan idle (harus kembali ke baseline); uji `reduce motion`; uji DPI 100/125%.

## 6. Akseptansi
- [ ] 22 ekspresi ada, sesuai katalog, terlihat jelas dan **berbeda** pada ukuran notch sebenarnya.
- [ ] Jam ↔ mata mulus; mata menggantikan jam saat chat terbuka.
- [ ] Idle Director: aturan frekuensi, penahanan, dan waktu teruji; tidak mengganggu fullscreen/fokus.
- [ ] Semua transisi 5.3 ada, dapat diinterupsi, memakai preset; reduce-motion bekerja.
- [ ] Mata/animasi tidak menaikkan CPU/RAM idle di atas anggaran (15).
- [ ] Ikon bot klasik tidak tersisa. **VC-U1** (screenshot semua ekspresi + rekaman/urutan frame transisi).
