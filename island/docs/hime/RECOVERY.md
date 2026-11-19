# Panduan Pemulihan Desktop Hime (`RECOVERY.md`)

Dokumen ini adalah panduan keselamatan desktop untuk pengguna / owner jika tampilan layar atau taskbar Windows mengalami kendala saat menjalankan Hime / Bloom Shell.

---

## 1. Pemulihan Otomatis (Rekomendasi)

Jika taskbar Windows tampak hilang setelah menjalankan Hime, cukup jalankan skrip pemulihan yang sudah disediakan di PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\restore-taskbar.ps1
```

Skrip ini akan:
1. Menghentikan proses `bloom.exe` / `hime.exe` yang masih berjalan di latar belakang.
2. Melepaskan reservasi Windows AppBar pada tepi layar.
3. Mengembalikan visibilitas taskbar utama (`Shell_TrayWnd`) dan taskbar monitor sekunder (`Shell_SecondaryTrayWnd`).
4. Memulihkan pengaturan auto-hide ke kondisi normal.
5. Me-restart `explorer.exe` secara bersih jika handle taskbar belum terdeteksi.

---

## 2. Pemulihan Manual Cepat (Jika Tidak Bisa Buka PowerShell)

Jika layar tertutup atau taskbar tidak merespons:

1. Tekan kombinasi tombol **Ctrl + Shift + Esc** untuk membuka **Task Manager**.
2. Cari proses bernama **`bloom.exe`** atau **`hime.exe`** di daftar Processes.
3. Klik kanan lalu pilih **End task** (Akhiri tugas).
4. Di Task Manager, klik tombol **Run new task** (Jalankan tugas baru) di bagian atas.
5. Ketik `explorer.exe` lalu tekan Enter. Taskbar Windows akan langsung muncul kembali seperti semula.
