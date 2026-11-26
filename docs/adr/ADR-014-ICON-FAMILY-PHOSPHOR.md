# ADR-014: Standardisasi Keluarga Ikon Phosphor Icons

## Status
Diterima (Accepted)

## Konteks
Hime Island membutuhkan satu keluarga ikon yang konsisten di seluruh antarmuka desktop (Windows) dan aplikasi pendamping (Android). Kode warisan Bloom dan prototipe awal menggunakan campuran Lucide, SF Symbols, dan SVG kustom. Ketidakkonsistenan ini menimbulkan variasi ketebalan garis, distorsi pada resolusi rendah (16–20 px), dan ketiadaan gaya *duotone* yang kohesif dengan sistem aksen warna Hime.

## Keputusan
Memilih dan menstandarkan **Phosphor Icons** (lisensi MIT) sebagai satu-satunya keluarga ikon di seluruh ekosistem Hime Island:
1. **Bobot Duotone:** Digunakan pada `IconTile` berwarna di panel daftar, dialog, dan empty states. Latar belakang 16% aksen dipadukan dengan foreground aksen solid.
2. **Bobot Bold:** Digunakan untuk tombol aksi mini (16–20 px) agar tetap tajam dan terbaca di layar non-HiDPI.
3. **Bobot Regular:** Digunakan untuk teks inline dan navigasi standar.
4. **Impor Per-Ikon:** Tidak ada barrel export besar (`import { Chat } from "@phosphor-icons/react"` yang memuat semua ikon). Hanya impor ikon yang terdaftar di `design/icon-map.md`.
5. **Konversi ke Android:** Seluruh ikon diekspor ke SVG dan dikonversi langsung ke Jetpack Compose `ImageVector` di modul `mobile/`.

## Konsekuensi
- **Positif:** Identitas visual bersatu; konsistensi pixel-perfect pada ukuran 16 px, 20 px, dan 24 px; ukuran bundel tetap minimal melalui tree-shaking per-ikon.
- **Negatif:** Ikon Lucide yang lama akan dimigrasikan secara bertahap tanpa merusak build saat ini.
