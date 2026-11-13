# Runtime Discovery: Hermes Agent & 9Router Gateway
## Task 08.1 — Empirical Runtime Inspection & Compatibility Verification

- **Document ID:** HIME-RUNTIME-DISCOVERY-08-01
- **Task ID:** TASK-08.1
- **Classification:** `[VERIFIED STATIC]` & `[VERIFIED RUNTIME]`
- **Date:** 2026-11-13
- **Auditor:** Principal Software Engineer & AI Systems Lead

---

## 1. Identifikasi Instalasi Aktual Hermes Agent

Berdasarkan pemeriksaan fisik langsung terhadap repositori biner dan sumber di `RE/Hermes/00-source-materials/hermes-agent`:

| Atribut Runtime | Nilai Terdeteksi | Bukti / File Acuan | Status |
|:---|:---|:---|:---:|
| **Nama Paket** | `hermes-agent` | `pyproject.toml:6` | `[VERIFIED STATIC]` |
| **Versi Manifest** | v0.8.2 (git commit `c5f347eb`) | `pyproject.toml`, `gateway/control_socket.py` | `[VERIFIED STATIC]` |
| **Interpreter Python Target** | `>=3.11, <3.15` (Host: Python 3.13.16) | `pyproject.toml:15`, `scripts/check-env.ps1` | `[VERIFIED RUNTIME]` |
| **Package Manager** | Astral `uv` (Host: `uv 0.12.16`) | `uv.lock` (1.25 MB exact pins), `check-env.ps1` | `[VERIFIED RUNTIME]` |
| **Entry Point Daemon** | `uv run hermes gateway start` | `gateway/run.py`, `hermes_cli/main.py` | `[VERIFIED STATIC]` |
| **Entry Point CLI** | `python run_agent.py` / `uv run hermes -c` | Root scripts, `agent/runner.py` | `[VERIFIED STATIC]` |
| **IPC Transport** | Windows Named Pipe: `\\.\pipe\hermes-gateway-<hash>` | `gateway/control_socket.py:82` | `[VERIFIED STATIC]` |
| **Protokol Streaming** | Server-Sent Events (SSE) & WebSocket chunking | `agent/streaming.py`, `gateway/server.py` | `[VERIFIED STATIC]` |
| **Mekanisme Tool** | Function Calling (OpenAI format) via Tool Registry | `tools/registry.py`, `tools/environments/local.py` | `[VERIFIED STATIC]` |
| **Mekanisme Penghentian** | Signal `SIGINT`/`SIGTERM` & Named Pipe close | `gateway/control_socket.py:145` | `[VERIFIED STATIC]` |
| **Environment Variable Kunci** | `OPENAI_BASE_URL` (wajib: `http://127.0.0.1:20129/v1`) | `agent/config.py` | `[VERIFIED STATIC]` |

---

## 2. Identifikasi Instalasi Aktual 9Router Gateway

Berdasarkan pemeriksaan fisik langsung terhadap repositori sumber di `RE/9ROUTER/00-source-materials/9router`:

| Atribut Runtime | Nilai Terdeteksi | Bukti / File Acuan | Status |
|:---|:---|:---|:---:|
| **Nama Aplikasi** | `9router-app` | `package.json:2` | `[VERIFIED STATIC]` |
| **Versi Aplikasi** | `v0.5.112` (git commit `1fe586bb`) | `package.json:3`, `CHANGELOG.md` | `[VERIFIED STATIC]` |
| **Runtime Node.js** | Node.js v24 (Host: `v24.18.0`) | `check-env.ps1`, `package.json` | `[VERIFIED RUNTIME]` |
| **Framework Backend** | Next.js Standalone + Express/Node HTTP Server | `custom-server.js:12`, `next.config.mjs` | `[VERIFIED STATIC]` |
| **Port Default Listener** | TCP `20128` (dapat diubah via `--port 20128`) | `cli/cli.js:110`, `custom-server.js` | `[VERIFIED STATIC]` |
| **Host Default Listener** | `0.0.0.0` (Upstream) → **Wajib di-override `--host 127.0.0.1`** | `cli/cli.js:110` (RSK-12) | `[VERIFIED STATIC]` |
| **Endpoint Inferensi Publik** | `POST /v1/chat/completions` (OpenAI-compatible) | `open-sse/routes/completions.js` | `[VERIFIED STATIC]` |
| **Model Discovery** | `GET /v1/models` & `GET /api/models` | `src/app/api/models/route.js` | `[VERIFIED STATIC]` |
| **Penyimpanan Internal** | SQLite WAL (`data.sqlite` / `9router.db`) | `src/lib/db/schema.js`, `src/lib/localDb.js` | `[VERIFIED STATIC]` |
| **Autentikasi Internal** | Token Header `x-9r-cli-token` | `src/dashboardGuard.js:16` | `[VERIFIED STATIC]` |
| **Celah Keamanan IB-01** | Plaintext API keys disimpan di `data.sqlite` | `src/lib/db/repos/connectionsRepo.js` | `[VERIFIED STATIC]` |
| **Celah Keamanan IB-02** | Localhost request mem-bypass auth (`isLocalRequest`) | `src/dashboardGuard.js:161-165` | `[VERIFIED STATIC]` |

---

## 3. Matriks Kompatibilitas Runtime & Topologi Dual-Control Plane

```mermaid
flowchart TD
    subgraph UI ["Presentation Layer"]
        Desktop["Hime Desktop (Tauri 2 + React 18)"]
    end

    subgraph Core ["Central Control Plane (Hime Core)"]
        Supervisor["Process Supervisor (Win32 Job Object)"]
        Vault["DPAPI Credential Vault"]
        ProviderAdapter["Loopback Provider Adapter (:20129)"]
        RouterAdapter["Router Gateway Adapter"]
        HermesAdapter["Hermes Runtime Adapter"]
    end

    subgraph Subprocesses ["Managed Subprocesses (CREATE_SUSPENDED)"]
        HermesProc["Hermes Agent (Python 3.13 / uv)"]
        RouterProc["9Router Gateway (Node.js v24 / :20128)"]
    end

    subgraph Upstream ["Cloud AI Providers"]
        CloudAPI["OpenAI / Anthropic / Nous API"]
    end

    Desktop <-->|Named Pipe IPC (12-byte binary framing)| Core
    Supervisor -->|Win32 Job Object Kill-On-Close| HermesProc
    Supervisor -->|Win32 Job Object Kill-On-Close| RouterProc
    HermesProc -->|Inference HTTP| RouterProc
    RouterProc -->|Custom Node Route| ProviderAdapter
    ProviderAdapter -->|Decrypt on-the-fly via DPAPI| Vault
    ProviderAdapter -->|Outbound HTTPS| CloudAPI
    RouterAdapter -.->|Health Probes & Metrics| RouterProc
    HermesAdapter <-->|Turn Orchestration & SEC-09 CoT Redaction| HermesProc
```

---

## 4. Kesimpulan Task 08.1

1. Kedua runtime pihak ketiga (Hermes Agent v0.8.2 dan 9Router v0.5.112) telah teridentifikasi secara empiris, memiliki entry point deterministik, dan dapat dijalankan di bawah supervisi Hime Core tanpa modifikasi kode sumber upstream.
2. Batas integrasi telah dikunci melalui topologi Hime-Owned Loopback Provider Adapter pada port `20129` untuk meniadakan risiko penyimpanan plaintext kredensial (IB-01).
3. Manifest kompatibilitas resmi diterbitkan pada `config/compatibility-manifest.json`.
