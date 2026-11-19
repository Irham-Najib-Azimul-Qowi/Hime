# Local Development Configuration & Sandboxing Specification

- **Document ID:** HIME-ENV-05-01
- **Task ID:** TASK-05-03 (Local Development Configuration & Sandboxing Setup)
- **Phase:** Phase 05 — Development Environment & Implementation Readiness
- **Authority:** System Architect & Technical Lead
- **Execution Role:** Senior DevOps Engineer & Systems Engineer
- **Status:** **APPROVED SPECIFICATION**
- **Date:** 2026-11-10

---

## 1. Executive Summary & Sandboxing Philosophy

Untuk mencegah pencemaran data antara lingkungan pengembangan (*development*) dan lingkungan produksi (*production*), serta untuk menjamin bahwa kredensial atau basis data pengujian tidak pernah terekam di repositori Git:
1. Seluruh data runtime lokal selama proses rekayasa dipisahkan ke direktori khusus:
   - **Windows:** `%LOCALAPPDATA%\Hime\dev\`
   - **Linux:** `$XDG_DATA_HOME/hime/dev/` (atau `~/.local/share/hime/dev/`)
2. Penggunaan API key komersial pengguna **DILARANG KERAS** pada mode pengembangan. Pengujian model AI wajib menggunakan *Mock Providers* atau token dummy loopback (`hime-managed-key`).
3. Variabel lingkungan diatur melalui `.env.example` sebagai referensi deklaratif, dan seluruh file `.env` lokal dikecualikan secara mutlak dari Git tracking.

---

## 2. Struktur Hierarki Direktori `%LOCALAPPDATA%\Hime\dev\`

Struktur direktori sandboxing lokal diatur secara modular:

```text
%LOCALAPPDATA%\Hime\dev\
├── config/              # Konfigurasi aplikasi lokal dev (config.toml)
├── state/               # Runtime process state, PID files, active sockets
│   ├── hime-core.pid    # PID file Hime Core
│   ├── 9router.pid      # PID file 9Router subproses
│   └── hermes.pid       # PID file Hermes Agent
├── logs/                # Berkas log JSONL pengembang terstruktur
│   ├── app.log.jsonl    # Log terpadu Hime Core (tracing)
│   ├── 9router.log      # Log stdout/stderr 9Router
│   └── hermes.log       # Log stdout/stderr Hermes
├── tmp/                 # Direktori sementara & staging area 5D safe move
│   └── .hime_staging/   # Staging area untuk pengujian pemindahan berkas aman
├── secrets/             # Container pengujian DPAPI & master token lokal
│   └── hime_vault_dev.db# Database DPAPI vault terenkripsi khusus dev
└── fixtures/            # Berkas uji statis untuk validasi storage & FTS5
```

### Hak Akses NTFS (Discretionary Access Control):
Direktori `%LOCALAPPDATA%\Hime\dev\` dibuat secara eksklusif dengan izin NTFS hanya untuk **Current User SID** (`TokenUser`). Hak akses untuk `NETWORK`, `GUEST`, dan `ANONYMOUS` ditiadakan.

---

## 3. Matriks Pemisahan Parameter Dev vs Production

| Parameter Konfigurasi | Nilai Development (`dev`) | Nilai Production (`prod`) | Justifikasi Arsitektural |
|:---|:---|:---|:---|
| **Root Data Directory** | `%LOCALAPPDATA%\Hime\dev\` | `%LOCALAPPDATA%\Hime\data\` | Menjamin isolasi data total saat debug. |
| **Log Level & Format** | `DEBUG` / `TRACE` (JSONL + Stderr) | `INFO` / `WARN` (JSONL File Appender) | Memudahkan tracing developer tanpa overhead di prod. |
| **Named Pipe Transport**| `\\.\pipe\hime-hermes-dev` | `\\.\pipe\hime-hermes` | Mencegah konflik IPC jika versi dev dan prod berjalan. |
| **9Router Port** | Ephemeral / `20128` (Dev Loopback) | Dynamic Ephemeral (49152-65535) | Mempermudah penargetan debugging lokal di dev. |
| **Vault Database File** | `hime_vault_dev.db` | `hime_vault.db` | Mencegah kerusakan kunci produksi saat pengujian. |
| **API Key Credentials** | Dummy tokens (`hime-managed-key`) | DPAPI-encrypted live user keys | Kepatuhan mutlak terhadap **IB-01 & IB-02**. |
| **Redaksi CoT** | Di-filter pada UI, tercatat di dev log| Di-filter mutlak pada UI dan log | Menjaga privasi pemikiran agen pada produksi. |

---

## 4. Prosedur Inisialisasi Sandbox Otomatis

Skrip peluncur pengembangan `scripts/dev.ps1` secara otomatis memverifikasi dan membuat hierarki direktori ini pada saat dijalankan pertama kali tanpa memerlukan intervensi manual dari pengguna.

