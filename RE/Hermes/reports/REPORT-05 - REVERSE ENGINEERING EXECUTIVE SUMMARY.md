# REPORT-05: REVERSE ENGINEERING EXECUTIVE SUMMARY
**Project:** Hime Assistant — System Architecture & Integration Blueprint  
**Target Audience:** ChatGPT, Lead System Architects, Technical Directors  
**Scope:** Complete Synthesis of Nusa Dynamic Island & Nous Research Hermes Agent  
**Author:** Senior Reverse Engineering Researcher & Technical Architect  
**Date:** November 2026  
**Status:** Master Document — Final  

---

## 1. Context & Purpose of this Document

Dokumen ini disusun sebagai ringkasan eksekutif komprehensif (*Master Executive Summary*) yang mengonsolidasikan seluruh temuan teknis dari investigasi mendalam (*reverse engineering*) terhadap dua sistem utama:
1. **Nusa Dynamic Island (`nusetup.exe` / `nusa.exe`):** Objek investigasi Tahap 2 — sebuah antarmuka floating desktop inovatif untuk Windows.
2. **Hermes Agent (`hermes-agent`):** Objek investigasi Tahap 3 — agen AI otonom sumber terbuka (*general-purpose autonomous reasoning engine*) yang dikembangkan oleh Nous Research.

Dokumen ini ditujukan sebagai **masukan primer bagi ChatGPT dan arsitek perangkat lunak** untuk merancang arsitektur sistem, memvalidasi keputusan teknis, dan memandu fase implementasi **Hime Assistant** tanpa perlu melakukan reverse engineering ulang.

---

## 2. Synthesis of Technical Discoveries

```
+===================================================================================================+
|                                    HIME ASSISTANT MASTER MATRIX                                   |
+===================================================================================================+
|  DIMENSI                  | NUSA DYNAMIC ISLAND              | HERMES AGENT                       |
+---------------------------+----------------------------------+------------------------------------+
|  Bahasa & Framework       | Rust + Tauri Framework           | Python 3.11-3.14 + Astral `uv`     |
|  Antarmuka Pengguna (UI)  | React / HTML5 / CSS (WebView2)   | Terminal CLI / Web SPA / Electron  |
|  Window Management        | WS_EX_LAYERED, HWND_TOPMOST      | Headless Gateway / Standard Window |
|  Model Eksekusi           | Event-driven, low resource       | Multi-phase State Machine Loop     |
|  Inter-Process Comms      | Internal Tauri IPC (WebView-Core)| Windows Named Pipe + WebSocket     |
|  Penyimpanan Status       | Local JSON / AppData Settings    | SQLite WAL + FTS5 Full Text Search |
|  Sistem Keamanan          | Windows UAC (Installer)          | Smart Guardian LLM + Manual Gates  |
+===================================================================================================+
```

### 2.1. Temuan Kunci Nusa Dynamic Island (Frontend/Shell Foundation)
1. **Pola Desain Jendela Melayang:** Nusa memanfaatkan kombinasi window styles Win32:
   - `WS_EX_TOPMOST`: Memastikan pulau selalu berada di atas semua jendela aplikasi lain.
   - `WS_EX_TOOLWINDOW`: Mencegah jendela muncul di taskbar Windows atau switcher `Alt+Tab`.
   - `WS_EX_LAYERED`: Mendukung transparansi alpha per-pixel untuk sudut melengkung halus (*rounded pill*) dan efek bayangan (*drop shadows*).
2. **Transisi Dimensi Adaptif:** Ukuran jendela diubah secara dinamis melalui event internal tanpa flickering, berpusat di bagian atas layar monitor utama (*top-center anchor*).
3. **Efisiensi Memori Luar Biasa:** Dengan basis Tauri/Rust, konsumsi RAM hanya berkisar 28–35 MB, jauh lebih hemat daripada Electron (150–250 MB).

### 2.2. Temuan Kunci Hermes Agent (Backend/Intelligence Foundation)
1. **Modular Mixin Core (`run_agent.py`):** Kelas inti `AIAgent` memecah logika ke dalam 14 mixin terisolasi. Siklus percakapan (`conversation_loop.py`) dijalankan sebagai mesin status 10-fase yang deterministik dengan kompresi trajectory otomatis.
2. **Windows Named Pipe Coordination (`gateway/control_socket.py`):** Gateway mengekspos Named Pipe native Windows di:
   $$\text{Pipe Path} = \texttt{\textbackslash\textbackslash.\textbackslash pipe\textbackslash hermes-gateway-} + \text{SHA256}(\text{HERMES\_HOME})[:16]$$
   Mendukung pertukaran JSON baris-tunggal untuk *handshake* (`identify`) dan pencetakan tiket autentikasi (`session-ticket`).
3. **Real-Time Streaming WebSocket (`gateway/run_api.py`):** Endpoint `/api/ws` mengalirkan token teks secara real-time, event pemanggilan alat (*tool calling*), output eksekusi, serta pesan permintaan persetujuan keamanan interaktif (`approval.pending`).
4. **Persistensi SQLite WAL (`hermes_state.py`):** Seluruh sesi dan pesan dicatat dalam basis data SQLite berkinerja tinggi yang mendukung pembacaan simultan (*concurrent read*) tanpa memblokir proses inferensi agen.

---

## 3. Hime Assistant Target Architecture & System Specification

Hime Assistant dirancang sebagai **Desktop AI Command Center** yang memadukan keindahan antarmuka Nusa dengan kecerdasan eksekusi Hermes.

```
+---------------------------------------------------------------------------------------------------+
|                                      HIME ASSISTANT APPLICATION                                   |
+---------------------------------------------------------------------------------------------------+
|  [ PRESENTATION LAYER (Tauri WebView2 / React + Tailwind CSS + Lucide Icons) ]                    |
|    - Dynamic Island Shell (Compact Pill, Thinking Animation, Tool Banner, Expanded Drawer)       |
|    - Interactive Approval Dialogs ([Deny], [Allow], [Always Allow])                               |
|    - Voice Input Waveform & Audio Player Component                                                |
+---------------------------------------------------------------------------------------------------+
                                         |
                                         | Tauri Rust IPC (Commands & Events)
                                         v
+---------------------------------------------------------------------------------------------------+
|  [ DESKTOP CORE LAYER (Tauri 2.0 / Rust) ]                                                        |
|    - Window Manager: Win32 Subclassing, TopMost, NoActivate, DWM Rounded Corners                  |
|    - Global Shortcut Listener: `Alt + Space` / `Win + Shift + H`                                  |
|    - Daemon Supervisor: Managed Child Process Spawner & Taskkill /T /F Reaper                     |
|    - Hermes Pipe Client: Tokio Named Pipe IPC Client (`\\.\pipe\hermes-gateway-<hash>`)          |
|    - Hermes WebSocket Client: Tokio-Tungstenite Client (`ws://127.0.0.1:<port>/api/ws`)           |
+---------------------------------------------------------------------------------------------------+
                                         |
                                         | Windows Local IPC (Named Pipe & WebSocket)
                                         v
+---------------------------------------------------------------------------------------------------+
|  [ AUTONOMOUS REASONING ENGINE (Nous Research Hermes Agent) ]                                      |
|    - Gateway Daemon (`gateway/run.py` - Background Service via Astral `uv`)                       |
|    - Conversation Loop (`agent/conversation_loop.py` - 10-Phase State Machine)                    |
|    - Tool Registry & Approval Gate (`tools/registry.py`, `tools/approval.py`)                     |
|    - SQLite WAL Storage (`%LOCALAPPDATA%\hermes\state.db`)                                        |
+---------------------------------------------------------------------------------------------------+
```

---

## 4. Concrete Interface Protocols & Wire Contracts

### 4.1. Handshake & Authentication Sequence
1. **Langkah 1:** Hime Rust Core membuka Windows Named Pipe:
   `\\.\pipe\hermes-gateway-<hash>`
2. **Langkah 2:** Hime mengirimkan baris JSON verb `session-ticket`:
   ```json
   {"protocol": 1, "verb": "session-ticket", "id": "auth-1", "params": {"profile_id": "default", "instance_id": "<id>", "purpose": "interactive_client"}}
   ```
3. **Langkah 3:** Gateway merespons dengan tiket otentikasi sekali pakai:
   ```json
   {"ok": true, "id": "auth-1", "result": {"ticket": "tkt_81c7...", "expires_in_seconds": 30}}
   ```
4. **Langkah 4:** Hime membuka WebSocket ke `ws://127.0.0.1:<port>/api/ws?ticket=tkt_81c7...`

### 4.2. Streaming WebSocket Payload Specification
- **Kirim Prompt:**
  ```json
  {"action": "send_message", "session_id": "ses_default", "content": "Analisis error build di proyek ini"}
  ```
- **Terima Delta Berpikir (*Reasoning Stream*):**
  ```json
  {"event": "agent:thinking_delta", "delta": "Memeriksa log kegagalan kompilator..."}
  ```
- **Terima Pemanggilan Tool (*Tool Execution*):**
  ```json
  {"event": "tool:call_start", "tool": "execute_command", "args": {"command": "cargo check"}}
  ```
- **Terima Interupsi Persetujuan (*Approval Gating*):**
  ```json
  {"event": "approval:pending", "request_id": "appr_101", "tool": "execute_command", "command": "rmdir /s /q target"}
  ```
- **Kirim Respon Persetujuan dari Island:**
  ```json
  {"action": "approval:respond", "request_id": "appr_101", "verdict": "allow"}
  ```

---

## 5. Recommended Technology Stack

| Komponen | Pilihan Teknologi | Alasan Pemilihan & Keunggulan |
| :--- | :--- | :--- |
| **Desktop Framework** | **Tauri 2.0 (Rust)** | Jejak memori minimal (~30 MB vs 200 MB Electron), kontrol native Win32 window yang presisi, performa asinkron Tokio tinggi. |
| **Frontend UI** | **React + Vite + TypeScript** | Ekosistem komponen melimpah, dukungan Tailwind CSS untuk styling dinamis, Framer Motion untuk animasi morfologi island yang mulus. |
| **Iconography & Styling** | **Tailwind CSS + Lucide Icons** | Mudah menghasilkan desain visual glassmorphic modern (acrylic/mica, rounded-full pill). |
| **Backend Agent** | **Nous Research Hermes Agent** | Kemampuan otonom kelas atas, registri tool komprehensif, multi-model adapter, arsitektur daemon teruji. |
| **Python Environment** | **Astral `uv` (Python 3.11+)** | Instalasi dan bootstrapping dependensi Hermes ultra-cepat dalam hitungan detik. |
| **Penyimpanan Lokal** | **SQLite WAL** | Konsistensi data tinggi, pencarian instan FTS5, bebas konflik pembacaan. |

---

## 6. Phase 4 Implementation Roadmap

Berikut adalah rencana kerja bertahap untuk pengembangan proyek Hime Assistant:

```
[ MILESTONE 1: TAURI 2.0 SHELL & WINDOW ENGINE ]
  - Inisialisasi proyek Tauri 2.0 di folder `Projects/Hime-Assistant/`.
  - Implementasi Win32 Layered Window Subclassing (HWND_TOPMOST, WS_EX_TOOLWINDOW).
  - Implementasi penempatan jendela Top-Center dan penanganan DPI Scaling.
  - Implementasi hit-testing click-through pada area transparan.

[ MILESTONE 2: HERMES DAEMON SUPERVISOR & IPC CLIENT ]
  - Rust client untuk mendeteksi / meluncurkan Hermes Gateway Daemon di background.
  - Rust implementation untuk Windows Named Pipe handshake & ticket retrieval.
  - Rust WebSocket connection manager dengan auto-reconnect & buffering.

[ MILESTONE 3: DYNAMIC ISLAND MORPHOLOGY & ANIMATION ]
  - Pembuatan komponen UI Dynamic Island (React + Framer Motion).
  - 5 Status Visual: Idle Pill, Active Input, Thinking Pulse, Tool Banner, Approval Card.
  - Register Global Shortcut Windows (`Alt + Space` / `Win + Shift + H`).

[ MILESTONE 4: FULL EVENT INTEGRATION & DRAWER VIEW ]
  - Integrasi streaming text delta ke conversation view di expanded drawer.
  - Penanganan tombol approval langsung di island (Allow / Deny).
  - Syntax highlighted code blocks dan copy-to-clipboard.

[ MILESTONE 5: TESTING, PACKAGING & POLISH ]
  - Pengujian stabilitas multi-monitor dan resource leak (RAM/CPU).
  - Packaging installer mandiri berbasis Inno Setup atau MSI/NSIS.
```

---

## 7. Strategic Architectural Decisions for ChatGPT / Stakeholders

Sebagai bahan pertimbangan dalam diskusi arsitektur selanjutnya bersama ChatGPT:
1. **Model Peluncuran Gateway:**  
   *Opsi A:* Hime Assistant mengasumsikan gateway sudah berjalan terpisah (user menjalankan `hermes gateway start`).  
   *Opsi B (Direkomendasikan):* Hime Assistant bertindak sebagai supervisor mandiri; jika pipe tidak ditemukan, Hime memanggil `uv run hermes gateway start` secara tersembunyi (*headless*).
2. **Framework Frontend:**  
   React direkomendasikan karena ketersediaan pustaka animasi seperti Framer Motion yang sangat krusial untuk animasi morfologi island (dari bentuk pill kecil ke kartu lebar).
3. **Tingkat Persetujuan Default:**  
   Direkomendasikan mengonfigurasi Hermes dalam mode `smart approval`, sehingga perintah rutin berjalan otomatis, sementara perintah destruktif memicu pop-up konfirmasi di island.

---

*Laporan ini secara resmi mengakhiri Tahap Investigasi dan Reverse Engineering. Seluruh materi, kode biner, dan artefak analisis tersimpan secara terstruktur di dalam direktori `x:\folder_ai\Hime\RE\`.*

