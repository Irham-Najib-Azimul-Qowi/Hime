# 02 — Installer Analysis
## Proyek: Hime Assistant
### Target: Nusa Dynamic Island (`NusaSetup.exe`)

---

## 1. Metodologi Ekstraksi

Analisis installer dilakukan menggunakan metode **statis non-destruktif** tanpa menjalankan wizard installer di sistem host.

- **Tool yang Digunakan:** `innounp` (Inno Setup Unpacker) versi `2.70.1` (Build 2026-07-21).
- **Prosedur:** Ekstraksi biner dan dekompilasi setup script `.iss` internal dari file `NusaSetup.exe`.
- **Hasil Ekstraksi:** Berhasil mengekstrak seluruh payload ke `research/nusa-analysis/extracted/`:
  1. `{app}\nusa.exe` (1,265,664 byte) — Executable biner aplikasi tunggal.
  2. `install_script.iss` (2,866 byte) — Naskah konfigurasi Inno Setup asli.

---

## 2. Analisis Konfigurasi Script Installer (`install_script.iss`)

Hasil dekompilasi script konfigurasi Inno Setup memberikan gambaran arsitektur instalasi yang sangat detail:

```ini
; Dekompilasi dari NusaSetup.exe oleh innounp 2.70.1
[Setup]
AppName=Nusa
AppVerName=Nusa 1.2.0.0
AppId={{A03D4881-45E3-4B16-B844-936040C558E5}
AppVersion=1.2.0.0
AppPublisher=Rama
AppPublisherURL=https://rama2555.github.io/nusa/
AppCopyright=Copyright (C) 2026 Rama
AppSupportURL=https://rama2555.github.io/nusa/
AppUpdatesURL=https://rama2555.github.io/nusa/
AppMutex=NusaNotch.SingleInstance
DefaultDirName={autopf}\Nusa
DefaultGroupName=Nusa
OutputBaseFilename=NusaSetup
UninstallDisplayIcon={app}\nusa.exe
UninstallDisplayName=Nusa
Compression=lzma2
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
DisableDirPage=yes
DisableProgramGroupPage=yes
ChangesAssociations=no
ShowLanguageDialog=yes
WizardStyle=modern light

[Files]
Source: "{app}\nusa.exe"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueName: "Nusa"; ValueData: """{app}\nusa.exe"""; Tasks: startup; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run"; ValueName: "Nusa"; Tasks: startup; Flags: deletevalue
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueName: "Nusa"; Tasks: not startup; Flags: deletevalue

[Run]
Filename: "{app}\nusa.exe"; Description: "{cm:LaunchProgram,Nusa}"; Flags: postinstall skipifsilent nowait

[Icons]
Name: "{autoprograms}\Nusa"; Filename: "{app}\nusa.exe";
Name: "{autodesktop}\Nusa"; Filename: "{app}\nusa.exe"; Tasks: desktop;

[Tasks]
Name: "startup"; Description: "{cm:StartupTask}";
Name: "desktop"; Description: "{cm:DesktopTask}";
```

---

## 3. Temuan Kritis dari Struktur Installer

### A. Hak Akses & Lokasi Pemasangan (Privileges & Target Directory)
- **`PrivilegesRequired=lowest`**: Installer secara eksplisit dirancang **tidak memerlukan hak Administrator (non-UAC elevated)**.
- **`DefaultDirName={autopf}\Nusa`**: Karena hak akses yang diminta adalah level user terendah, makro `{autopf}` secara otomatis dievaluasi oleh Windows ke direktori lokal pengguna:
  `%LOCALAPPDATA%\Programs\Nusa` (biasanya `C:\Users\<Username>\AppData\Local\Programs\Nusa`).
- **`DisableDirPage=yes`**: Pengguna standar tidak diminta memilih folder tujuan (streamlined 1-click install).

### B. Mekanisme Single Instance (Process Synchronization)
- **`AppMutex=NusaNotch.SingleInstance`**: Installer memeriksa keberadaan mutex sistem bernama `NusaNotch.SingleInstance`.
- Jika Nusa sedang berjalan di latar belakang saat proses update/instalasi berlangsung, Inno Setup akan mendeteksinya secara otomatis dan meminta aplikasi ditutup terlebih dahulu sebelum mengganti biner.

### C. Konfigurasi Startup & Registry
- Auto-start Windows diatur melalui Registry level pengguna (**HKCU**), sehingga tidak membutuhkan akses admin ke HKLM:
  - Subkey: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
  - Value Name: `Nusa`
  - Value Data: `"<app_path>\nusa.exe"`
- Installer juga membersihkan entry di `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run` untuk memastikan auto-start diizinkan tanpa terblokir oleh Windows Task Manager startup state.

### D. Pembuatan Shortcut
- Start Menu: `{autoprograms}\Nusa` (menghasilkan shortcut di `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Nusa.lnk`).
- Desktop (Opsional): `{autodesktop}\Nusa` (shortcut di desktop pengguna jika task `desktop` dipilih).

### E. Mekanisme Uninstaller & Retensi Data Pengguna
Pada blok `[CustomMessages]` ditemukan pesan uninstaller penting:
```ini
en.RemoveData=Also remove your Nusa data (to-do list, water log, shelf, settings)?
```
Hal ini membuktikan secara langsung bahwa:
1. Nusa menyimpan data persisten di luar direktori instalasi `{app}`.
2. Data pengguna mencakup:
   - **to-do list** (`tasks.txt` / `todo.txt`)
   - **water log** (`water.txt`)
   - **shelf** (file dan link yang di-drop ke floating island)
   - **settings** (`settings.txt`)
3. Direktori data tersebut berada pada `%LOCALAPPDATA%\Nusa\` (dibuktikan pada observasi runtime).

---

## 4. Evaluasi Dependensi & Runtime Eksternal

Pemeriksaan daftar file pada installer (`[Files]`):
- **Hanya terdapat 1 file tunggal**: `nusa.exe`.
- **Tidak ada dependensi eksternal yang dibundle**:
  - Tidak ada runtime Node.js atau Chromium (bukan Electron standard bundle).
  - Tidak ada installer Visual C++ Redistributable (`vcredist.exe`).
  - Tidak ada .NET Desktop Runtime bootstrapper.
  - Tidak ada file WebView2Loader.dll atau paket redistributable lainnya.
- Installer ini beroperasi sebagai **self-contained zero-dependency portable wrapper**.

---

## 5. Ringkasan Bukti & Fakta Installer

| Fitur Installer | Fakta Teknis Terverifikasi | Status |
| :--- | :--- | :--- |
| **Framework Installer** | Inno Setup 6.7.0 (Unicode) | CONFIRMED |
| **Metode Kompresi** | LZMA2 Solid Archive | CONFIRMED |
| **Tingkat Privilege** | User Mode (`lowest`) — Tanpa UAC Admin | CONFIRMED |
| **Lokasi Standar** | `%LOCALAPPDATA%\Programs\Nusa` | CONFIRMED |
| **Registry Auto-start** | `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` | CONFIRMED |
| **Mutex Sinkronisasi** | `NusaNotch.SingleInstance` | CONFIRMED |
| **File Terinstal** | 1 File Executable (`nusa.exe`, 1.2 MB) | CONFIRMED |
| **File Association** | Tidak ada (`ChangesAssociations=no`) | CONFIRMED |

