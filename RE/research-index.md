# Indeks Riset & Target Investigasi Teknis (Research Index)
## Proyek Hime Assistant

Dokumen ini memetakan seluruh aktivitas investigasi, target riset, status penyelesaian, dan lokasi artefak laporan teknis.

---

## Matriks Status Target Investigasi

| ID Target | Objek Riset | Kategori / Fokus | Sumber / Repository | Status | Lokasi Dokumen & Laporan |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **TRG-01** | **Nusa Dynamic Island** (`NusaSetup.exe` v1.2.0.0) | Desktop Window Management, DirectComposition, UI/UX, Local Storage | Biner Installer Lokal (`rama2555.github.io/nusa`) | **COMPLETED** | [`RE/Nusa/reports/FINAL-REVERSE-ENGINEERING-REPORT.md`](file:///x:/folder_ai/Hime/RE/Nusa/reports/FINAL-REVERSE-ENGINEERING-REPORT.md) |
| **TRG-02** | **Hermes Agent** | AI Agent Core, Reasoning Loop, Tool Calling, Session/Memory, Integration APIs | `https://github.com/NousResearch/hermes-agent` | **COMPLETED** | [`RE/Hermes/reports/REPORT-01 - HERMES TECHNICAL OVERVIEW.md`](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-01%20-%20HERMES%20TECHNICAL%20OVERVIEW.md) |
| **TRG-03** | **Hime-Hermes Integration** | Arsitektur Komunikasi, IPC/WebSocket, Event Streaming, UI Control Center | Konseptual / Bridge | **COMPLETED** | [`RE/Hermes/reports/REPORT-04 - HIME-HERMES INTEGRATION FEASIBILITY.md`](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-04%20-%20HIME-HERMES%20INTEGRATION%20FEASIBILITY.md) |
| **TRG-04** | **9Router AI Multi-Provider Gateway** | Multi-Provider Routing, Model Combos, Account Fallback, Translators, SQLite | `https://github.com/decolua/9router` | **COMPLETED** | [`RE/9ROUTER/14-FINAL-REVERSE-ENGINEERING-REPORT.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/14-FINAL-REVERSE-ENGINEERING-REPORT.md) |

---

## Daftar Lengkap Laporan Riset Teknis

### 1. Investigasi Nusa Dynamic Island (`RE/Nusa/reports/`)
- [`FINAL-REVERSE-ENGINEERING-REPORT.md`](file:///x:/folder_ai/Hime/RE/Nusa/reports/FINAL-REVERSE-ENGINEERING-REPORT.md): Laporan komprehensif struktur biner Inno Setup, PE header, Win32 direct window styling, DirectComposition rendering GPU, dan analisis fitur Nusa.

### 2. Investigasi Hermes Agent & Integrasi Hime (`RE/Hermes/reports/`)
- [`REPORT-01 - HERMES TECHNICAL OVERVIEW.md`](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-01%20-%20HERMES%20TECHNICAL%20OVERVIEW.md): Overview sistem, modul repositori, dependensi runtime (Python, Astral `uv`, Windows ConPTY), dan entry points.
- [`REPORT-02 - HERMES AGENT INTERNAL ARCHITECTURE.md`](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-02%20-%20HERMES%20AGENT%20INTERNAL%20ARCHITECTURE.md): Arsitektur `AIAgent`, 14 mixins, 10 fase conversation loop state machine, sistem perizinan tool (YOLO, Smart Guardian LLM, Manual Gate), dan memori bertingkat SQLite WAL.
- [`REPORT-03 - HERMES INTEGRATION CAPABILITY.md`](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-03%20-%20HERMES%20INTEGRATION%20CAPABILITY.md): Protokol Windows Named Pipe (`\\.\pipe\hermes-gateway-<hash>`), streaming WebSocket `/api/ws`, autentikasi tiket, event streaming envelope, dan preseden Electron desktop app.
- [`REPORT-04 - HIME-HERMES INTEGRATION FEASIBILITY.md`](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-04%20-%20HIME-HERMES%20INTEGRATION%20FEASIBILITY.md): Sintesis Nusa + Hermes, viabilitas integrasi, arsitektur hybrid Tauri 2.0 (Rust) + Web, 6 status morfologi dynamic island, solusi teknis DPI/focus/click-through.
- [`REPORT-05 - REVERSE ENGINEERING EXECUTIVE SUMMARY.md`](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-05%20-%20REVERSE%20ENGINEERING%20EXECUTIVE%20SUMMARY.md): Master blueprint untuk evaluasi ChatGPT dan System Architect, roadmap implementasi Fase 4.

### 3. Investigasi 9Router Multi-Provider Gateway (`RE/9ROUTER/`)
- [`01-INITIAL-RECONNAISSANCE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/01-INITIAL-RECONNAISSANCE.md): Metadata repositori, monorepo Next.js + CLI, env contract, pragma SQLite WAL.
- [`02-SOURCE-CODE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/02-SOURCE-CODE-ARCHITECTURE.md): Arsitektur heksagonal, diagram komponen, batas antarmuka `src/` vs `open-sse/`.
- [`03-CORE-ROUTING-ENGINE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/03-CORE-ROUTING-ENGINE.md): Alur siklus hidup request 12-fase, intermediate format pivot, dan seleksi akun.
- [`04-PROVIDER-ADAPTER-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/04-PROVIDER-ADAPTER-ARCHITECTURE.md): `BaseExecutor`, `DefaultExecutor`, adapter 140+ provider (OpenAI, Anthropic, Gemini, Nous, dll.).
- [`05-MODEL-DISCOVERY-AND-MANAGEMENT.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/05-MODEL-DISCOVERY-AND-MANAGEMENT.md): Endpoint `/v1/models`, live resolvers, basis data kapabilitas `capabilities.js`.
- [`06-SMART-ROUTING-COMBO-AND-FALLBACK.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/06-SMART-ROUTING-COMBO-AND-FALLBACK.md): Two-tier fallback, rotasi multi-akun, empty stream detection (`emptyFailover.js`).
- [`07-API-GATEWAY-AND-PROTOCOL.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/07-API-GATEWAY-AND-PROTOCOL.md): Spesifikasi endpoint chat, messages, responses, audio TTS/STT, search, paritas OpenAI.
- [`08-DASHBOARD-AND-UI-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/08-DASHBOARD-AND-UI-ARCHITECTURE.md): Arsitektur Next.js 16 / React 19 UI, Zustand, Recharts, API REST internal `/api/*`.
- [`09-AUTHENTICATION-AND-SECURITY.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/09-AUTHENTICATION-AND-SECURITY.md): Audit keamanan, spoofing countermeasure, kerentanan penyimpanan plaintext token.
- [`10-DATA-STORAGE-AND-STATE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/10-DATA-STORAGE-AND-STATE.md): ERD database SQLite, schema tables, pragma WAL, mutasi status in-memory vs disk.
- [`11-PERFORMANCE-AND-RELIABILITY.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/11-PERFORMANCE-AND-RELIABILITY.md): Connection pooling `undici`, streaming backpressure, tuning timeout TTFT 200s.
- [`12-HERMES-INTEGRATION-ANALYSIS.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/12-HERMES-INTEGRATION-ANALYSIS.md): Pengujian kompatibilitas Hermes ↔ 9Router, streaming tool calls, thinking deltas.
- [`13-HIME-INTEGRATION-FEASIBILITY.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/13-HIME-INTEGRATION-FEASIBILITY.md): Evaluasi topologi (Dual-Control Plane), visualisasi failover di Dynamic Island.
- [`14-FINAL-REVERSE-ENGINEERING-REPORT.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/14-FINAL-REVERSE-ENGINEERING-REPORT.md): Jawaban atas 12 pertanyaan fundamental, sintesis akhir, dan rekomendasi implementasi.
