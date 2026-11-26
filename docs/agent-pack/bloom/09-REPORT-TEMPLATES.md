# 09 — REPORT TEMPLATES

Gunakan juga templat di `legacy/08-REPORT-TEMPLATES.md` (laporan fase, status, keputusan owner, ADR, review, audit, handover). Berikut tambahan untuk Hime×Bloom.

## T-A. `docs/hime/TOUCHPOINTS.md` (registry sentuhan ke berkas Bloom)

```markdown
| # | Berkas Bloom | Baris/area | Perubahan (ringkas) | Alasan | Cara revert | Commit |
```
Aturan: setiap perubahan pada berkas milik Bloom **harus** tercatat. Perubahan tak tercatat = BLOCKER review.

## T-B. `docs/hime/BLOOM-CONVENTIONS.md` (kerangka)

```markdown
# Bloom Conventions (dari source, versi <tag/SHA>)
## 1. Frontend: struktur, komponen, hooks, state, pola invoke/listen (dengan path:baris)
## 2. Animasi: mesin spring, konfigurasi, tabel ukuran pill/panel/bubble
## 3. Styling: pendekatan, token (warna, radius, bayangan, tipografi), tema, ikon
## 4. Panel Island: cara mendaftar, urutan, scroll/swipe, memori panel, panel arrows, peek
## 5. Live-activity bubble: definisi, prioritas, ukuran, klik
## 6. Settings: pola tab/kontrol, kunci, SETTINGS.md, sinkron
## 7. Backend: commands, services, events, state, utils, window/AppBar/multi-monitor/DPI
## 8. Pemulihan taskbar dan jalur exit
## 9. Jaringan keluar (updater, announcement, cuaca, lainnya), CSP, capabilities
## 10. Pengujian, CI, rilis, skrip
## 11. Gaya kode (penamaan, komentar, error handling, format)
## 12. Hal yang tidak boleh diubah / rapuh
```

## T-C. `OWNER VISUAL CHECK` (permintaan persetujuan visual)

```markdown
### VC-NN — <fitur/milestone>
- **Apa yang dilihat:** (1–2 kalimat)
- **Cara menjalankan (owner):** `scripts\run-safe.ps1 -Seconds 60` lalu <langkah singkat>   (taskbar otomatis pulih)
- **Screenshot terlampir:** docs/hime/visual/<fitur>/... (idle, peek, terbuka, loading, error, offline; sebelum/sesudah; berdampingan dengan panel Bloom pembanding)
- **Checklist Bloom-native:** (salin dari berkas 04 §6.1 dengan centang dan bukti)
- **Perbedaan sengaja terhadap Bloom + alasan:**
- **Resource:** baseline Bloom vs sekarang (CPU idle, RAM)
- **Yang BELUM diuji (jujur):** (mis. multi-monitor)
- **Pertanyaan untuk owner:** diterima / revisi (apa yang harus diubah)
- **Status:** OPEN | DITERIMA (tanggal) | REVISI (catatan)
```
Jawaban owner cukup: `VC-NN: diterima` atau `VC-NN: revisi, <catatan>`.

## T-D. Laporan akseptansi visual (bagian dari laporan fase)

```markdown
## Visual
| Permukaan/state | Screenshot | DPI | Monitor | Hasil Bloom-native | Catatan |
Metode screenshot: <alat> | Sesi interaktif: ya/tidak | Bila tidak → NOT TESTED + permintaan VC
```

## T-E. `docs/hime/harvest/HARVEST-MANIFEST.md` dan `DELETION-LOG.md`

```markdown
| Sumber (lama) | Tujuan (baru) | Kelas | SHA-256 sumber | SHA-256 salinan | Cocok |
```
```markdown
| Jalur dihapus | Kelas | Ukuran | Alasan | Tanggal |
```

## T-F. `docs/hime/NUSA-PARITY-ON-BLOOM.md`

```markdown
| Fitur Nusa | Status parity (Preserve/Improve/Replace/Remove/New) | Dimiliki Bloom? | Permukaan Bloom untuk Hime | Fase | Catatan |
```

## T-G. `docs/hime/RECOVERY.md` (untuk owner)
Langkah manual jika layar macet: (1) Ctrl+Shift+Esc → Task Manager → akhiri `hime`/`bloom`; (2) jalankan `scripts\restore-taskbar.ps1`; (3) jika taskbar belum muncul: restart `explorer.exe`; (4) kontak/rujukan log. Tulis dengan bahasa awam.
