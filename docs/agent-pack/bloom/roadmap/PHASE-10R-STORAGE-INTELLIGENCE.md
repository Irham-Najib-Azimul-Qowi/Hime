# PHASE 10R — STORAGE INTELLIGENCE

**Backend:** seluruh lingkup `legacy/roadmap/PHASE-10-STORAGE-INTELLIGENCE.md` (indeks, analyzer, duplikat/versi, organizer, **download intelligence**, **safe file operations** + Hime Recycle + jurnal, backup/pemulihan, tool+skill, pengujian sandbox). Aturan keselamatan: **uji hanya di sandbox**; operasi destruktif pada file pribadi nyata = Gate #5.

## UI

| Fitur | Permukaan Bloom |
|---|---|
| Pertanyaan unduhan duplikat/versi (UR-03) | **Bubble pertanyaan** ("Sudah ada. Ini versi terbaru?") → ekspansi Island dengan pilihan ringkas: ganti & arsipkan yang lama sebagai versi lama / simpan keduanya / abaikan / selalu lakukan untuk pola ini. Tidak mengganggu berulang (rate limit). Menunggu unduhan selesai sebelum bertanya |
| Proposal organizer ("simpan di mana?") (UR-01) | Bubble/ekspansi: Hime **bertanya** lokasi; jawaban menjadi aturan |
| Ringkasan storage laptop | **Panel Storage** ringkas di Island (ruang, 1–2 saran) |
| Dashboard analitik, duplikat/versi, antrean proposal, aturan, timeline jurnal, pemulihan | **Workspace → Storage** |
| Integrasi **Shelf Bloom** | Aksi "kirim ke Hime / arsipkan / rapikan" untuk berkas di shelf (pakai alur proposal+approval yang sama) |
| Retensi Recycle, root yang diizinkan | Tab Storage di Settings Hime |

Verifikasi visual + resource (indeks file tidak membuat idle shell naik; indeks berjalan di Core dengan prioritas rendah). **VC-04** (bubble unduhan + panel Storage).
