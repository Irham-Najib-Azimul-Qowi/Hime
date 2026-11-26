# PHASE 11R — MOBILE & DEVICES

**Lingkup:** seluruh `legacy/roadmap/PHASE-11-MOBILE-AND-DEVICE-LAYER.md` (aplikasi Android Kotlin/Compose, pairing QR + mutual TLS, CA lokal, protokol Device, storage HP via SAF/MediaStore, transfer resumable, approval lintas perangkat, postur keamanan, nudge/pengingat di HP, batas Android yang jujur). Aplikasi Android **bukan** bagian Bloom (klien terpisah di `apps/mobile/`); desainnya mengikuti identitas Hime/Bloom (gelap, ringkas, spring ringan) dan batasan Material/Compose.
**Gate:** perangkat Android nyata = Gate #6 (emulator dulu).

## UI sisi desktop (shell Bloom)
| Fitur | Permukaan |
|---|---|
| Pairing: tampil QR + konfirmasi kode | **Workspace → Devices** (QR besar butuh ruang); bubble "perangkat meminta pairing" di Island |
| Status HP (baterai, ruang), transfer berjalan | **Bubble transfer/perangkat** di Island; detail di Workspace |
| Storage HP | Workspace → Devices/Storage (analitik, operasi dengan approval) |
| Approval dari HP | Approval yang sama di desktop dan HP; hanya satu yang bisa menyelesaikan tiket (uji) |
| Revoke/rotasi | Workspace → Devices + tab Perangkat di Settings |

Verifikasi visual **VC-05** (Devices + bubble transfer). Uji keamanan perangkat sesuai legacy 06 §8 dan §11.
