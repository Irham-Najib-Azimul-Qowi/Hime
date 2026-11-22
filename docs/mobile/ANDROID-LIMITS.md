# Batasan & Arsitektur Penegakan Hime di Android

> Dokumen resmi batas teknis, izin, dan jaminan privasi penegakan fokus Hime Mobile.
> Rujukan: `16-MOBILE-PARITY.md §4`.

---

## 1. Batasan Jujur Sistem Operasi Android

Pada perangkat Android modern (Android 8.0 hingga Android 15+), sistem operasi membatasi kontrol proses antar-aplikasi demi keamanan pengguna. Tanpa akses *Root* atau izin *Device Owner / Enterprise Admin* (yang sengaja tidak digunakan oleh Hime untuk menjaga integritas keamanan perangkat), **tidak ada aplikasi pihak ketiga yang dapat menutup paksa (force-kill) aplikasi lain secara langsung**.

Oleh karena itu, penegakan fokus Hime di Android dirancang secara **jujur, elegan, dan bertingkat (Graduated Enforcement)**:

---

## 2. Tiga Tingkat Penegakan (Tier A / B / C)

| Tingkat | Nama | Mekanisme Teknis | Izin yang Diperlukan | Efek pada Pengguna |
|:---|:---|:---|:---|:---|
| **Tier A** | **Ingatkan** | Notifikasi interaktif & alarm tepat waktu | `POST_NOTIFICATIONS`, `SCHEDULE_EXACT_ALARM` | Pengingat rutin mulai, tenggat tugas dekat, saran kembali fokus |
| **Tier B** | **Tegas** | Deteksi aplikasi latar depan + Overlay layar penuh | `PACKAGE_USAGE_STATS`, `SYSTEM_ALERT_WINDOW` | Menampilkan overlay gelap dengan ekspresi maskot Hime, timer hitung mundur, dan opsi kembali belajar |
| **Tier C** | **Paksa** | Pengalihan otomatis ke Layar Beranda | `BIND_ACCESSIBILITY_SERVICE` (Layanan Aksesibilitas) | Memanggil `performGlobalAction(GLOBAL_ACTION_HOME)` untuk mengembalikan pengguna ke Beranda tanpa force-kill |

---

## 3. Jaminan Privasi Aksesibilitas

Layanan Aksesibilitas Hime (`HimeAccessibilityEnforcementService`) memegang komitmen privasi ketat:

1. **`canRetrieveWindowContent = false`**:
   Layanan dikonfigurasi secara eksplisit agar sistem operasi Android **tidak pernah** mengizinkan Hime membaca isi teks, formulir, sandi, atau konten layar apa pun.
2. **Hanya Memeriksa Nama Paket (`packageName`)**:
   Hime hanya memeriksa apakah aplikasi yang saat ini berada di latar depan merupakan aplikasi dalam daftar pengalih yang dibatasi pada jam belajar.
3. **Data Lokal**:
   Semua evaluasi penegakan berjalan 100% lokal di perangkat dan dapat bekerja secara offline tanpa koneksi ke server laptop.

---

## 4. Daftar Aplikasi Terlindungi (Protected Apps)

Aplikasi berikut **dikecualikan secara permanen** dari segala bentuk pemblokiran, overlay, maupun pengalihan ke Beranda:

- **Panggilan & Telepon**: `com.android.phone`, `com.android.dialer`, `com.google.android.dialer`
- **Panggilan Darurat & SOS**: `com.android.emergency`
- **Kontak**: `com.android.contacts`, `com.google.android.contacts`
- **Navigasi & Peta**: `com.google.android.apps.maps`
- **Antarmuka Sistem Android**: `com.android.systemui`
- **Aplikasi Hime**: `ai.hime.mobile`

---

## 5. Mekanisme Keamanan & Jeda Darurat

1. **Jeda Darurat (15 Menit)**:
   Pengguna dapat mengaktifkan Jeda Darurat kapan saja di tab Fokus. Semua penegakan ditunda selama 15 menit untuk urusan mendesak.
2. **Jeda Pelemahan Aturan (Rule Weakening)**:
   Perubahan batas waktu memerlukan jeda pendinginan (mis. 24 jam) untuk mencegah impuls saat sedang fokus.
3. **Panduan Baterai OEM**:
   Hime menyertakan panduan khusus untuk pabrikan dengan pembatas latar belakang agresif (Xiaomi MIUI/HyperOS, Oppo ColorOS, Vivo FuntouchOS, Samsung OneUI) agar alarm penegakan tidak dibunuh oleh OS.
