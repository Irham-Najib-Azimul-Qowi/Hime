# 01 — Initial Reconnaissance
## Proyek: Hime Assistant
### Target: Nusa Dynamic Island (`NusaSetup.exe`)

---

## 1. Ringkasan Eksekutif Reconnaissance

Tahap Initial Reconnaissance berfokus pada verifikasi integritas fisik, metadata, signature digital, dan format packaging dari installer Nusa Dynamic Island yang disediakan pada workspace proyek Hime Assistant.

Pemeriksaan membuktikan bahwa:
- Installer didistribusikan dalam dua format: arsip ZIP (`NusaSetup.zip`) dan executable mandiri (`NusaSetup.exe`).
- Isi `NusaSetup.zip` adalah executable `NusaSetup.exe` yang identik (byte-for-byte).
- Format installer executable adalah **Inno Setup versi 6.7.0 (Unicode)** dengan kompresi payload **LZMA2**.
- File installer **tidak ditandatangani secara digital** (Unsigned / NotSigned).
- Metadata internal mengidentifikasi pengembang aplikasi sebagai **Rama** (`https://rama2555.github.io/nusa/`).

---

## 2. Identifikasi & Verifikasi File Target

| Atribut | Nilai Pengamatan | Tingkat Keyakinan |
| :--- | :--- | :--- |
| **Nama File 1** | `NusaSetup.exe` | CONFIRMED |
| **Lokasi File 1** | `X:\folder_ai\Hime\NusaSetup.exe` | CONFIRMED |
| **Ukuran File 1** | 2,945,252 byte (2.81 MB / 2,877 KiB) | CONFIRMED |
| **SHA-256 Hash 1** | `BC4133305B28B929ECA42CC32833DC4FD34002CE35676D27761942419F4D4B02` | CONFIRMED |
| **Timestamp File 1** | 2026-11-01 20:29:00 (WIB) | CONFIRMED |
| **Nama File 2** | `NusaSetup.zip` | CONFIRMED |
| **Lokasi File 2** | `X:\folder_ai\Hime\NusaSetup.zip` | CONFIRMED |
| **Ukuran File 2** | 2,430,268 byte (2.32 MB) | CONFIRMED |
| **SHA-256 Hash 2** | `3590642776D0184423C4D2BDDF92D48D6FF03C9439406BA41FEC2DFD933CF115` | CONFIRMED |
| **Isi Arsip ZIP** | `NusaSetup.exe` (2,945,252 byte, SHA-256 identik) | CONFIRMED |

---

## 3. Metadata Portable Executable (PE) & VersionInfo

Pemeriksaan struktur header PE dan resource `RT_VERSION` pada `NusaSetup.exe` menghasilkan data berikut:

```text
FileVersionRaw     : 1.2.0.0
ProductVersionRaw  : 1.2.0.0
Comments           : This installation was built with Inno Setup.
CompanyName        : Rama
FileDescription    : Nusa Setup
FileVersion        : 1.2.0.0
InternalName       : [Empty]
IsDebug            : False
IsPatched          : False
IsPreRelease       : False
Language           : Language Neutral (0x0000)
LegalCopyright     : Copyright © 2026 Rama
OriginalFilename   : [Empty]
ProductName        : Nusa
ProductVersion     : 1.2.0.0
Subsystem          : Windows GUI (2)
Machine            : AMD64 / x86-64 (0x8664)
```

### Bukti Pendukung:
- Struktur string Inno Setup standar ditemukan pada resource PE:
  `Comments: This installation was built with Inno Setup.`
  `CompanyName: Rama`
  `ProductName: Nusa`

---

## 4. Digital Signature (Authenticode)

Pemeriksaan status tanda tangan digital dilakukan menggunakan modul PowerShell `Get-AuthenticodeSignature`:

```text
SignerCertificate      : None
TimeStamperCertificate : None
Status                 : NotSigned
StatusMessage          : The file NusaSetup.exe is not digitally signed.
SignatureType          : None
IsOSBinary             : False
```

### Kesimpulan Keamanan:
Installer tidak menggunakan sertifikat Code Signing komersial (misal Sectigo, DigiCert) maupun sertifikat internal Windows. Hal ini normal untuk proyek utilitas open-source atau independen, namun berarti SmartScreen / UAC Windows akan menampilkan prompt "Unknown Publisher" saat dieksekusi secara interaktif.

---

## 5. Identifikasi Teknologi Packaging Installer

Pemeriksaan pola biner dan signature bytes mengonfirmasi bahwa installer dibangun menggunakan **Inno Setup**:

1. **Signature Strings Ditemukan:**
   - Offset `0x000B530C`: `Inno Setup Setup Data (6.7.0)`
   - Offset `0x000B5330`: `Inno Setup Messages (6.5.0) (u)`
   - Manifest embedded XML: `<description>Inno Setup</description>`
   - Window class helper: `InnoSetupLdrWindow`

2. **Arsitektur Pembungkusan:**
   - Bagian awal file (offset `0x00000000` s/d `0x000B530B`, ~724 KB) merupakan Inno Setup standard self-extracting stub loader.
   - Bagian akhir file (offset `0x000B530C` s/d `0x002CEF04`, ~2.15 MB) merupakan payload terkompresi LZMA2 yang berisi file aplikasi utama.

---

## 6. Matrix Bukti & Tingkat Keyakinan

| Item Analisis | Hasil Observasi | Tingkat Keyakinan | Dasar Bukti |
| :--- | :--- | :--- | :--- |
| **Packaging Engine** | Inno Setup 6.7.0 (Unicode) | CONFIRMED | Byte signature `Inno Setup Setup Data (6.7.0)` pada offset `0xB530C`. |
| **Tipe Kompresi** | LZMA2 | CONFIRMED | Dekompilasi script `.iss` (`Compression=lzma2`). |
| **Arsitektur Target** | 64-bit (x64 compatible) | CONFIRMED | PE Header Machine `0x8664` dan direktif `.iss` `ArchitecturesAllowed=x64compatible`. |
| **Identitas Pengembang** | Rama | CONFIRMED | Metadata PE `CompanyName` dan `AppPublisherURL`. |
| **Status Signing** | Not Signed | CONFIRMED | `Get-AuthenticodeSignature` menghasilkan `Status: NotSigned`. |
| **Integritas Distribusi** | Utuh dan Valid | CONFIRMED | Hash SHA-256 terverifikasi terhadap file ZIP. |

