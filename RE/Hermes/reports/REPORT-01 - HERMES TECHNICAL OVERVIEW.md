# REPORT-01: HERMES TECHNICAL OVERVIEW
**Project:** Hime Assistant — Reverse Engineering & Technical Analysis  
**Target:** Nous Research Hermes Agent (`hermes-agent`)  
**Commit Reference:** `c5f347eb` (Main Branch)  
**Author:** Senior Reverse Engineering Researcher & Technical Architect  
**Date:** November 2026  
**Status:** Completed  

---

## 1. Executive Overview

**Hermes Agent** adalah platform AI Agent otonom sumber terbuka (open-source) yang dikembangkan oleh **Nous Research** di bawah lisensi MIT. Berbeda dari wrapper LLM percakapan sederhana, Hermes dirancang sebagai *general-purpose autonomous reasoning and execution agent* yang dapat berjalan tanpa pengawasan (*unattended*) maupun secara interaktif (*interactive*).

Hermes dioptimalkan secara khusus untuk model keluarga **Hermes** (seperti Hermes-2, Hermes-3, dan turunan Nous Research), namun mempertahankan agnostisisme model melalui adaptor komprehensif untuk OpenAI, Anthropic, OpenRouter, DeepSeek, Google Gemini, Ollama, vLLM, dan llama.cpp.

### Karakteristik Inti Hermes Agent:
1. **Multi-Surface Accessibility:** Dapat diakses melalui Terminal CLI (`cli.py`), Webapp/Dashboard SPA (`hermes_cli/web_server.py`), Electron Desktop App (`apps/desktop`), Background Gateway Daemon (`gateway/run.py`), hingga bot platform pihak ketiga (Telegram, Discord, Slack).
2. **Deterministic & Resilient Loop:** Memiliki *conversation turn engine* bertingkat (`agent/conversation_loop.py`) dengan preflight validation, rate limit protection, retry loop, dan sanitasi pesan otomatis.
3. **Deep Tool System & Gating:** Dilengkapi registri *pluggable tools* (`tools/registry.py`) dengan tingkat persetujuan dinamis (*Smart Guardian LLM*, *YOLO mode*, *Manual approval*).
4. **Stateful Multi-Tier Memory:** Menggunakan basis data relasional berkinerja tinggi SQLite WAL (`hermes_state.py`) yang dilengkapi FTS5 full-text search, automatic trajectory compression, dan context pruning.

---

## 2. High-Level Architecture Diagram

Arsitektur Hermes Agent tersusun atas lapisan hierarkis yang memisahkan antara *Agent Core*, *Local Daemon/Gateway*, *Transport/API Layer*, dan *Client Surfaces*:

```
+-----------------------------------------------------------------------------------+
|                              CLIENT SURFACES                                      |
|  +------------------+  +--------------------+  +-------------------------------+  |
|  | CLI (Terminal)   |  | Electron Desktop   |  | Future Hime Dynamic Island    |  |
|  | (cli.py / TUI)   |  | (apps/desktop)     |  | (Tauri 2.0 / Rust + Web)      |  |
|  +--------+---------+  +---------+----------+  +---------------+---------------+  |
+-----------|----------------------|-----------------------------|------------------+
            |                      |                             |
            | IPC / Stdio          | Windows Named Pipe / WS     | Windows Named Pipe / WS
            v                      v                             v
+-----------------------------------------------------------------------------------+
|                         HERMES GATEWAY DAEMON (`gateway/`)                        |
|                                                                                   |
|   +---------------------------------------------------------------------------+   |
|   | Local Control Socket (`gateway/control_socket.py`)                        |   |
|   |  - Windows Named Pipe: \\.\pipe\hermes-gateway-<home_hash>                |   |
|   |  - Verbs: `identify`, `status`, `session-ticket`                          |   |
|   +---------------------------------------------------------------------------+   |
|                                        |                                          |
|                                        v (Ticket Authentication)                  |
|   +---------------------------------------------------------------------------+   |
|   | Gateway HTTP & WebSocket API (`gateway/run_api.py`)                       |   |
|   |  - ASGI Server (Starlette / FastAPI / Uvicorn)                            |   |
|   |  - Endpoint `/api/ws` (Real-Time Bidirectional Event Streaming)           |   |
|   |  - Endpoint `/api/pty` (ConPTY / WinPtyBridge Interactive Terminal)      |   |
|   |  - Audio Stream Endpoints: `/api/audio/speak-stream`, `/api/audio/transcribe`|   |
|   +---------------------------------------------------------------------------+   |
+----------------------------------------|------------------------------------------+
                                         | In-Process / Runner Task
                                         v
+-----------------------------------------------------------------------------------+
|                            AGENT CORE ENGINE (`agent/`)                           |
|                                                                                   |
|   +---------------------------------------------------------------------------+   |
|   | AIAgent Facade (`run_agent.py:214`)                                       |   |
|   | (14 Mixins: Turn, Persistence, Compression, Stream, Toolset, etc.)        |   |
|   +---------------------------------------------------------------------------+   |
|                                        |                                          |
|                                        v                                          |
|   +---------------------------------------------------------------------------+   |
|   | Conversation Loop State Machine (`agent/conversation_loop.py`)            |   |
|   | [Prep] -> [Assemble] -> [Preflight] -> [LLM Call] -> [Tool Round/Finish]  |   |
|   +---------------------------------------------------------------------------+   |
|                      |                                       |                    |
|                      v                                       v                    |
|   +------------------------------------+   +----------------------------------+   |
|   | Tool System (`tools/`)             |   | Memory & State (`hermes_state.py`|   |
|   | - Registry (`registry.py`)         |   | - SQLite WAL Database            |   |
|   | - Approval Gate (`approval.py`)    |   | - Trajectory Compressor          |   |
|   | - Code Execution & Web Search      |   | - FTS5 Text Search Engine        |   |
|   +------------------------------------+   +----------------------------------+   |
|                      |                                       |                    |
|                      v                                       v                    |
|   +------------------------------------+   +----------------------------------+   |
|   | External LLM Providers             |   | Storage: ~/.hermes/              |   |
|   | (OpenAI, Anthropic, Ollama, etc.)  |   | state.db, skills/, logs/         |   |
|   +------------------------------------+   +----------------------------------+   |
+-----------------------------------------------------------------------------------+
```

---

## 3. Repository Structure & Module Breakdown

Hasil audit struktural terhadap repositori `hermes-agent` menghasilkan pemetaan modul fungsional sebagai berikut:

| Direktori / File | Ukuran / Komponen | Peran dan Tanggung Jawab Utama |
| :--- | :--- | :--- |
| `run_agent.py` | ~88 KB, 14 Mixins | Definisi kelas utama `AIAgent`. Pusat orkestrasi siklus hidup agen, konfigurasi konteks, dan integrasi mixin modul. |
| `agent/` | >40 file Python | Mesin eksekusi turn agen (`conversation_loop.py`), perakitan prompt (`prompt_builder.py`), strategi retry (`turn_retry_state.py`), dan kompresi trajectory. |
| `gateway/` | >50 file Python | Daemon lokal independen. Mengelola koneksi Named Pipe (`control_socket.py`), HTTP/WS server (`run_api.py`), platform adapters (Slack, Telegram, Discord), dan room multiplexing. |
| `hermes_cli/` | >60 file Python | Antarmuka pengguna CLI, parser argumen, bridging PTY terminal Windows/POSIX (`win_pty_bridge.py`), dan integrasi web server dashboard. |
| `tools/` | >70 file Python | Sistem registri tool internal (`registry.py`), modul keamanan (`approval.py`, `write_approval.py`), eksekusi perintah terminal, pencarian web, analisis visual, dan sintesis suara. |
| `apps/desktop/` | Proyek Electron/TS | Aplikasi GUI desktop resmi Nous Research. Menggunakan TypeScript, Vite, React, dan Tailwind CSS. Menjadi referensi utama integrasi desktop Windows. |
| `hermes_state.py` | ~95 KB + modular | Lapisan persistensi data berbasis SQLite WAL. Mengatur tabel `sessions`, `messages`, metadata biaya token, dan pencarian teks lengkap FTS5. |
| `providers/` | Multi-provider SDK | Abstraksi interaksi model bahasa: pemanggilan API streaming, normalisasi respons, token counting, dan penanganan rate limit. |
| `skills/` | Dynamic Skills | Kemampuan deklaratif berbasis markdown dan script python yang dapat dipelajari, disimpan, dan dieksekusi oleh agen secara otonom. |

---

## 4. Execution Environments & Runtime Dependencies

Hermes Agent dibangun di atas ekosistem modern yang sangat mengutamakan kecepatan inisialisasi dan keandalan runtime:

1. **Python Core Engine:**
   - **Versi yang Didukung:** Python 3.11 hingga 3.14.
   - **Package & Dependency Manager:** Menggunakan **Astral `uv`** (`uv.lock` berukuran ~1.2 MB). Penggunaan `uv` memungkinkan pembuatan *isolated virtual environment* dan resolusi dependensi dalam hitungan detik (10x-100x lebih cepat daripada standard pip).
   - **Pustaka Kunci:** `fastapi`, `starlette`, `uvicorn`, `pydantic`, `httpx`, `aiosqlite`, `prompt_toolkit`, `pywinpty` (khusus Windows ConPTY).

2. **Windows Platform Native Support:**
   - **Named Pipes:** Implementasi `\\.\pipe\hermes-gateway-<hash>` via asyncio Windows Proactor event loop (`IocpProactor`).
   - **Console Pseudo-Terminal (ConPTY):** Didukung penuh melalui `hermes_cli/win_pty_bridge.py` memanfaatkan pustaka `pywinpty`. Hal ini memungkinkan terminal interaktif dijalankan di background Windows tanpa menampilkan jendela popup command prompt hitam yang mengganggu.
   - **Process Tree Termination:** Menggunakan utilitas `taskkill /T /F` (`apps/desktop/electron/backend-child.ts`) untuk memastikan child process seperti bash, powershell, atau sub-tool tidak menjadi proses yatim piatu (*zombie processes*) saat agen dihentikan.

3. **Desktop & GUI Client Runtime:**
   - Aplikasi desktop resmi Nous Research berbasis Node.js v20+ dan Electron.
   - Menggunakan IPC internal berbasis HTTP keep-alive connection pool dan WebSocket untuk menghindari socket exhaust pada Windows loopback.

---

## 5. Primary Entry Points

Hermes Agent memiliki 4 titik masuk (*entry points*) utama yang melayani berbagai mode operasional:

1. **CLI Mode (`cli.py`):**
   - Perintah: `hermes chat`, `hermes run "prompt"`, `hermes gateway start`.
   - Digunakan untuk interaksi langsung terminal, debugging, dan administrasi profil agen.
2. **Gateway Daemon (`gateway/run.py`):**
   - Berjalan sebagai layanan latar belakang (*background service*).
   - Membuka control socket lokal dan port HTTP/WS loopback.
   - Merupakan arsitektur yang direkomendasikan untuk integrasi aplikasi pihak ketiga (seperti Hime Assistant) karena agen tetap hidup di memori tanpa overhead cold-start setiap kali pengguna mengirim prompt.
3. **Core Library API (`run_agent.py`):**
   - Dapat diimpor langsung dalam skrip Python: `from run_agent import AIAgent; agent = AIAgent(...)`.
   - Menjalankan iterasi percakapan via `agent.run_conversation(...)`.
4. **Desktop Application (`apps/desktop`):**
   - Titik masuk: `apps/desktop/electron/main.ts`.
   - Secara otomatis mendeteksi keberadaan gateway atau memunculkan backend Python managed child di background.

---

## 6. Summary for Hime Assistant Architecture

Dari tinjauan teknis ini, dapat disimpulkan bahwa **Hermes Agent bukanlah monolit tertutup**, melainkan sistem modular dengan pemisahan yang sangat tegas antara mesin inferensi (*Agent Core*) dan antarmuka (*Surfaces*).

Keberadaan **Gateway Daemon** dengan transport **Windows Named Pipe** dan **Streaming WebSocket** menjadikan Hermes mitra sempurna untuk **Hime Assistant**. Hime tidak perlu menjalankan ulang Python runtime pada setiap interaksi, melainkan cukup bertindak sebagai *client surface* ramping yang berkomunikasi dengan Hermes Gateway melalui IPC Windows berkecepatan tinggi.

