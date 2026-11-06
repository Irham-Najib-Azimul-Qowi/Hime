# Direktori Riset Teknis: Hermes Agent
## Proyek Hime Assistant

Direktori ini memuat seluruh materi sumber, hasil investigasi kode, dan laporan teknis mengenai **Hermes Agent** (Nous Research).

---

## Struktur Direktori

```text
RE/Hermes/
│
├── README.md                          <- Dokumen ini
├── 00-source-materials/               <- Repositori resmi hermes-agent (commit c5f347eb)
├── 01-repository-analysis/            <- Catatan struktur direktori & dependencies
├── 02-codebase-architecture/          <- Analisis arsitektur kelas AIAgent & mixins
├── 03-agent-lifecycle/                <- Catatan siklus hidup giliran percakapan
├── 04-agent-core/                     <- Analisis conversation_loop.py & prompt builder
├── 05-tool-system/                    <- Analisis registry.py & approval gating
├── 06-memory-and-context/             <- Analisis SQLite WAL & trajectory compression
├── 07-session-and-conversation/       <- Analisis manajemen sesi & forks
├── 08-model-and-provider-system/      <- Analisis adapter provider LLM
├── 09-api-and-integration/            <- Analisis Named Pipe, WebSocket, REST API
├── 10-execution-and-runtime/          <- Analisis ConPTY, subprocess tree kill
├── 11-security-and-permissions/       <- Analisis YOLO, Smart Guardian, manual gate
├── 12-extension-and-customization/    <- Analisis dynamic skills & MCP integration
├── findings/                          <- Catatan teknis & snippets penting
└── reports/                           <- 5 Laporan Investigasi Formal (Deliverables)
    ├── REPORT-01 - HERMES TECHNICAL OVERVIEW.md
    ├── REPORT-02 - HERMES AGENT INTERNAL ARCHITECTURE.md
    ├── REPORT-03 - HERMES INTEGRATION CAPABILITY.md
    ├── REPORT-04 - HIME-HERMES INTEGRATION FEASIBILITY.md
    └── REPORT-05 - REVERSE ENGINEERING EXECUTIVE SUMMARY.md
```

---

## Ringkasan Laporan Teknis Utama (Deliverables)

1. **[REPORT-01 - HERMES TECHNICAL OVERVIEW.md](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-01%20-%20HERMES%20TECHNICAL%20OVERVIEW.md)**  
   Gambaran umum sistem, modul kode, dependensi runtime (Python 3.11+, Astral `uv`, Windows ConPTY), dan titik masuk operasional (CLI, Daemon, Library, Desktop).
2. **[REPORT-02 - HERMES AGENT INTERNAL ARCHITECTURE.md](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-02%20-%20HERMES%20AGENT%20INTERNAL%20ARCHITECTURE.md)**  
   Struktur `AIAgent` (14 mixins), mesin status 10-fase `conversation_loop.py`, sistem perizinan tool multi-level, dan penyimpanan memori bertingkat SQLite WAL.
3. **[REPORT-03 - HERMES INTEGRATION CAPABILITY.md](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-03%20-%20HERMES%20INTEGRATION%20CAPABILITY.md)**  
   Protokol IPC Windows Named Pipe (`\\.\pipe\hermes-gateway-<hash>`), otentikasi tiket 30 detik, event streaming WebSocket `/api/ws`, dan preseden integrasi desktop resmi.
4. **[REPORT-04 - HIME-HERMES INTEGRATION FEASIBILITY.md](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-04%20-%20HIME-HERMES%20INTEGRATION%20FEASIBILITY.md)**  
   Kajian kelayakan integrasi antarmuka Nusa Dynamic Island dengan Hermes Agent, arsitektur hybrid Tauri 2.0 (Rust) + Web, 6 status morfologi island, dan mitigasi DPI/focus.
5. **[REPORT-05 - REVERSE ENGINEERING EXECUTIVE SUMMARY.md](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-05%20-%20REVERSE%20ENGINEERING%20EXECUTIVE%20SUMMARY.md)**  
   Master blueprint teknis untuk evaluasi ChatGPT dan arsitek sistem, roadmap implementasi 5 milestone.

