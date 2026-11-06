# REPORT-04: HIME-HERMES INTEGRATION FEASIBILITY
**Project:** Hime Assistant — Architectural Integration Feasibility Study  
**Objects of Study:** Nusa Dynamic Island (`nusa.exe`) + Hermes Agent (`hermes-agent`)  
**Author:** Senior Reverse Engineering Researcher & Technical Architect  
**Date:** November 2026  
**Status:** Completed  

---

## 1. Executive Summary & Feasibility Verdict

Studi kelayakan teknis ini menguji viabilitas penggabungan konsep antarmuka melayang (*floating dynamic island*) yang diadaptasi dari hasil reverse engineering **Nusa Dynamic Island** dengan mesin kecerdasan otonom **Hermes Agent**.

### Kesimpulan Kelayakan (Feasibility Verdict):
> **STATUS: SANGAT LAYAK (HIGHLY FEASIBLE — TINGKAT RISIKO RENDAH-SEDANG)**
> 
> Tidak ditemukan hambatan arsitektural maupun restriksi biner yang menghalangi integrasi. Hermes Agent telah memiliki lapisan *headless daemon* (`gateway/`) yang menyediakan Windows Named Pipe dan WebSocket terstruktur, sementara pola window management Nusa Dynamic Island berbasis **Tauri (Rust + Web)** terbukti sangat kompatibel untuk direkonstruksi dan disematkan antarmuka kontrol agen.

---

## 2. Cross-Synthesis: Nusa Architecture vs. Hermes Capabilities

Integrasi Hime Assistant didasarkan pada perpaduan dua aset teknis yang telah dibedah secara tuntas:

| Parameter Arsitektur | Nusa Dynamic Island (Hasil RE Tahap 2) | Hermes Agent (Hasil RE Tahap 3) | Target Hime Assistant |
| :--- | :--- | :--- | :--- |
| **Teknologi Utama** | Rust + WebView2 (Tauri Framework) | Python 3.11+ / Astral `uv` | **Tauri 2.0 (Rust) + Web Frontend** |
| **Konsumsi Memori** | ~32 MB RAM (sangat ringan) | ~120-250 MB RAM (Gateway Daemon) | **Total ~150-280 MB RAM** |
| **Window Management** | Frameless, Transparent, `WS_EX_LAYERED`, `HWND_TOPMOST` | Headless Daemon / CLI / Electron | Mengadopsi konfigurasi window Nusa pada Rust/Tauri. |
| **Interaksi Pengguna** | Widget ringkas di bagian atas monitor | Terminal TUI / Dashboard Web browser | **Dynamic Island melayang sebagai Command Center**. |
| **Metode Komunikasi** | Internal Tauri IPC (`invoke`, `listen`) | Windows Named Pipe + WebSocket `/api/ws` | **Rust IPC Bridge** menjembatani Tauri IPC ke Hermes WS. |
| **Event Handling** | Deteksi status baterai, audio, hardware | Streaming token, tool call, approval gate | **Visualisasi status Hermes secara dinamis di island**. |

---

## 3. Recommended Integration Architecture

Arsitektur yang direkomendasikan mengadopsi pola **Hybrid Decoupled Architecture**:

```
+---------------------------------------------------------------------------------------+
|                                HIME ASSISTANT DESKTOP                                 |
|                                                                                       |
|   +-------------------------------------------------------------------------------+   |
|   | UI Layer (Dynamic Island Frontend: React / Svelte / SolidJS + Tailwind CSS)   |   |
|   |  - Compact Pill (Idle State)                                                  |   |
|   |  - Expanding Pill (Thinking / Tool Execution Animation)                       |   |
|   |  - Approval Prompt Overlay ([Allow] / [Deny] Buttons)                         |   |
|   |  - Expandable Chat Drawer (Markdown, Code Blocks, Trajectory)                |   |
|   +---------------------------------------+---------------------------------------+   |
|                                           |                                           |
|                                           | Tauri IPC Commands & Events               |
|                                           v                                           |
|   +-------------------------------------------------------------------------------+   |
|   | Native Core Layer (Tauri 2.0 / Rust Backend)                                  |   |
|   |  - Window Manager: Win32 Subclassing, Acrylic/Mica Backdrop, HWND_TOPMOST    |   |
|   |  - Global Hotkey Manager: Register `Alt+Space` / `Win+Shift+H`                |   |
|   |  - Daemon Supervisor: Memeriksa dan menjalankan `hermes gateway` jika mati   |   |
|   |  - Named Pipe Client: Menghubungi `\\.\pipe\hermes-gateway-<hash>` (Tokio)    |   |
|   |  - WebSocket Transport: Menghubungkan `/api/ws` dengan event bus frontend     |   |
|   +---------------------------------------+---------------------------------------+   |
+-------------------------------------------|-------------------------------------------+
                                            |
                                            | Windows Local IPC (Named Pipe & WebSocket)
                                            v
+---------------------------------------------------------------------------------------+
|                               HERMES GATEWAY DAEMON                                   |
|   (Proses Latar Belakang Python Terpisah - `uv run hermes gateway start`)             |
|                                                                                       |
|   - Named Pipe Listener: \\.\pipe\hermes-gateway-<hash>                               |
|   - HTTP / WebSocket Server: 127.0.0.1:<port>                                         |
|   - AIAgent Loop Engine: Tool Execution, Trajectory Compression, SQLite WAL           |
+---------------------------------------------------------------------------------------+
```

### Mengapa Tauri 2.0 (Rust) Lebih Unggul daripada Electron untuk Hime?
1. **Footprint Minimal:** Nusa membuktikan bahwa antarmuka floating desktop harus memiliki ukuran biner <20 MB dan memori <40 MB agar tidak memberatkan sistem saat pengguna membuka IDE berat. Electron memerlukan 150-200 MB RAM hanya untuk jendela kosong.
2. **Native Win32 Control:** Di Rust, memanggil API native Windows (`SetWindowPos`, `SetLayeredWindowAttributes`, `DwmSetWindowAttribute`) jauh lebih stabil dan tidak memerlukan native node addon (C++ NAN) yang rapuh.
3. **Kinerja Asinkron:** Pustaka `tokio` di Rust dapat menangani streaming WebSocket dari Hermes dan interaksi Win32 event loop secara paralel dengan latensi submili-detik.

---

## 4. Dynamic Island Interaction States & State Machine

Hime Assistant mengubah Dynamic Island menjadi antarmuka visual yang mencerminkan status kognitif Hermes Agent:

```
               [ User Menekan Hotkey / Klik ]
                             |
                             v
+---------------------------------------------------------------------------+
| STATE 1: IDLE COMPACT PILL (~160 x 36 px)                                 |
| [ * Hime Assistant | Hermes Idle ]                                        |
+---------------------------------------------------------------------------+
                             |
                             | Event: User membuka input text / suara
                             v
+---------------------------------------------------------------------------+
| STATE 2: INPUT EXPANDED PILL (~420 x 56 px)                               |
| [ > Ketik perintah untuk Hermes...                (Mic) (Enter) ]         |
+---------------------------------------------------------------------------+
                             |
                             | Action: Kirim pesan -> WS `/api/ws`
                             v
+---------------------------------------------------------------------------+
| STATE 3: THINKING & REASONING (~280 x 44 px)                              |
| [ (Wave Animation) Hermes sedang menganalisis...  [14 tok/s] ]            |
+---------------------------------------------------------------------------+
                             |
                             +----------------------------------------------+
                             |                                              |
      [Event: `tool:call_start`]                  [Event: `approval:pending`]|
                             v                                              v
+-----------------------------------------+   +-----------------------------+
| STATE 4: TOOL EXECUTION (~460 x 72 px)  |   | STATE 5: APPROVAL FLYOUT    |
| [ (Gear) Menjalankan:                   |   | [ (Alert) Perintah Bahaya   |
|   powershell git commit -m "update"     |   |   Hapus folder 'dist/'?     |
|   (Progress Bar Indeterminate) ]        |   |   [ TOLAK ]   [ IZINKAN ] ] |
+-----------------------------------------+   +-----------------------------+
                             |                                              |
                             +----------------------------------------------+
                             |
                             | Event: `turn:complete` / `agent:content_delta`
                             v
+---------------------------------------------------------------------------+
| STATE 6: EXPANDED CONVERSATION DRAWER (~520 x 480 px)                     |
| [ Hasil Eksekusi Hermes:                                                X ]|
| - Semua perubahan berhasil di-commit ke branch main.                       |
| - 4 file termodifikasi, 120 baris ditambahkan.                             |
| [Salin Hasil]  [Buka di Terminal]  [Lanjutkan Percakapan...]               |
+---------------------------------------------------------------------------+
```

---

## 5. Technical Challenges & Concrete Solutions

### Tantangan 1: Multi-Monitor & DPI Scaling pada Windows
- **Masalah:** Jendela melayang sering kali mengalami pergeseran posisi atau blur saat berpindah antar monitor dengan scaling DPI berbeda (misalnya 100% vs 125% atau 150%).
- **Solusi:**  
  Tauri Rust backend mendaftarkan hook `WM_DPICHANGED`. Menggunakan fungsi Win32 `GetDpiForWindow` dan `MonitorFromWindow` untuk menghitung koordinat horizontal tengah:
  $$\text{X} = \text{Monitor.Left} + \frac{\text{Monitor.Width} - \text{Window.Width}}{2}$$
  $$\text{Y} = \text{Monitor.Top} + \text{MarginAtas}$$

### Tantangan 2: Focus Management & Non-Intrusive Overlay
- **Masalah:** Saat Dynamic Island memperbarui animasi atau menampilkan status Hermes, jendela tidak boleh mencuri fokus (*steal keyboard focus*) dari jendela aktif pengguna (seperti VS Code atau browser).
- **Solusi:**  
  Terapkan extended style Win32 `WS_EX_NOACTIVATE` saat island hanya menampilkan status thinking atau tool execution. Gaya aktivasi keyboard hanya diaktifkan saat pengguna secara sadar mengklik island atau menekan global shortcut pemanggilan prompt.

### Tantangan 3: Click-Through pada Area Transparan
- **Masalah:** Island memiliki sudut membulat (*rounded pill*) dan area bayangan transparan. Jika pengguna mengklik di luar area pill tetapi masih dalam bounding box jendela, klik tidak boleh tertelan.
- **Solusi:**  
  Menangani pesan Windows `WM_NCHITTEST`. Jika koordinat kursor berada di luar radius pill, kembalikan `HTTRANSPARENT` sehingga klik mouse tembus ke aplikasi di bawahnya (*click-through*).

### Tantangan 4: Otomatisasi Siklus Hidup Gateway Hermes
- **Masalah:** Pengguna desktop tidak ingin membuka terminal secara manual dan mengetik `hermes gateway start` sebelum menggunakan Hime.
- **Solusi:**  
  1. Saat Hime diluncurkan, Rust backend memeriksa keberadaan Named Pipe `\\.\pipe\hermes-gateway-<hash>`.
  2. Jika pipe tidak merespons, Hime secara transparan memunculkan proses latar belakang:
     `uv run hermes gateway start --headless`
     tanpa membuka jendela konsol hitam (`CREATE_NO_WINDOW = 0x08000000`).
  3. Saat Hime ditutup, Hime dapat memilih untuk membiarkan gateway tetap hidup (opsi standby) atau melakukan *tree-kill* aman via `taskkill /T /F`.

---

## 6. Strategic Risk Assessment

| Potensi Risiko | Tingkat Dampak | Probabilitas | Mitigasi yang Disiapkan |
| :--- | :--- | :--- | :--- |
| **Pembaruan Protokol Hermes (Breaking Changes)** | Sedang | Rendah | Menggunakan protokol versi eksplisit (`CONTROL_PROTOCOL_VERSION = 1`) yang diverifikasi pada handshake `identify`. |
| **Koneksi Socket Drop saat Heavy Load** | Rendah | Sedang | Menerapkan reconnection loop otomatis dengan eksponensial backoff di layer Rust client. |
| **Prompt Injection pada Tool Execution** | Tinggi | Rendah | Bergantung pada lapisan keamanan internal Hermes (`tools/approval.py` Smart Guardian + Approval Flyout manual di Hime). |

---

## 7. Kesimpulan Akhir

Studi kelayakan teknis menyimpulkan dengan tegas bahwa proyek **Hime Assistant memiliki jalur implementasi yang jelas, aman, dan sangat dapat dicapai**. 

Dengan mereplikasi sistem visual dan arsitektur desktop Nusa Dynamic Island dalam **Tauri 2.0 (Rust)** dan menghubungkannya ke **Hermes Agent Gateway** melalui **Windows Named Pipe dan WebSocket**, Hime Assistant akan menjadi aplikasi Dynamic Island AI pertama di Windows yang memadukan estetika UI kelas dunia dengan kekuatan penalaran agen otonom tingkat tinggi.

