# REPORT-03: HERMES INTEGRATION CAPABILITY
**Project:** Hime Assistant — Reverse Engineering & Technical Analysis  
**Target:** Nous Research Hermes Agent (`hermes-agent`)  
**Components:** `gateway/control_socket.py`, `gateway/run_api.py`, `apps/desktop/electron/`  
**Author:** Senior Reverse Engineering Researcher & Technical Architect  
**Date:** November 2026  
**Status:** Completed  

---

## 1. Executive Summary

Laporan ini membedah kapabilitas integrasi antarmuka eksternal (*external integration capabilities*) dari **Hermes Agent**. Fokus analisis diletakkan pada protokol komunikasi antarkerja (*Inter-Process Communication / IPC*), antarmuka jaringan lokal berbasis WebSocket dan REST, skema autentikasi berbasis tiket, serta preseden integrasi desktop resmi yang diterapkan oleh tim Nous Research pada `apps/desktop`.

Hasil investigasi ini membuktikan bahwa Hermes Agent memiliki infrastruktur IPC kelas produksi yang sangat terstandarisasi di sistem operasi Windows, menjadikannya sangat siap untuk dihubungkan secara mulus dengan antarmuka desktop modern seperti **Hime Assistant**.

---

## 2. Inter-Process Communication (IPC) via Windows Named Pipe

Hermes Agent menghindari ketergantungan pada port TCP statis atau pemindaian proses yang rentan konflik (*PID-reuse heuristics*). Di lingkungan Windows, Gateway Daemon menyediakan permukaan koordinasi lokal (*local coordination surface*) melalui **Windows Named Pipe**.

### 2.1. Alamat & Spesifikasi Named Pipe
Didefinisikan dalam `gateway/control_socket.py:45-53`:
```python
def _home_hash(home: Path) -> str:
    return hashlib.sha256(
        os.path.normcase(str(Path(home).expanduser().resolve(strict=False))).encode("utf-8")
    ).hexdigest()[:16]

def windows_pipe_name(home: Path) -> str:
    """Per-HERMES_HOME named pipe path (Windows transport)."""
    return rf"\\.\pipe\hermes-gateway-{_home_hash(home)}"
```
- **Path Pipe:** `\\.\pipe\hermes-gateway-<hash_16_karakter>`
- **Prinsip Keamanan:** Menggunakan Access Control Lists (ACL) native Windows. Hanya proses yang berjalan di bawah konteks pengguna Windows (SID) yang sama yang diizinkan membuka koneksi ke Named Pipe ini.

### 2.2. Wire Contract & Verbs
Komunikasi Named Pipe menggunakan protokol *single-request / single-response*:
- **Format:** Satu baris JSON diakhiri dengan `\n` (newline), satu baris JSON balasan, kemudian koneksi ditutup oleh server.
- **Batas Ukuran:** Maksimum request 64 KB, maksimum response 512 KB.

#### Verb 1: `identify`
Digunakan untuk memverifikasi apakah gateway aktif dan mengambil informasi dasar runtime.
- **Request:**
  ```json
  {"protocol": 1, "verb": "identify", "id": "req-01"}
  ```
- **Response:**
  ```json
  {
    "ok": true,
    "protocol": 1,
    "id": "req-01",
    "result": {
      "kind": "gateway",
      "pid": 14280,
      "start_time": 1730801200.5,
      "hermes_home": "C:\\Users\\Satoshi\\.hermes",
      "profile": "default",
      "supervisor": "manual",
      "version": "0.8.2"
    }
  }
  ```

#### Verb 2: `status`
Mengambil metrik kesehatan live, beban CPU/memori, dan status runner.
- **Request:**
  ```json
  {"protocol": 1, "verb": "status", "id": "req-02"}
  ```

#### Verb 3: `session-ticket` (Kunci Integrasi Keamanan)
Menerbitkan tiket otorisasi sekali pakai (*one-time ticket*) berumur 30 detik yang diperlukan untuk membuka koneksi WebSocket atau REST API.
- **Request:**
  ```json
  {
    "protocol": 1,
    "verb": "session-ticket",
    "id": "req-03",
    "params": {
      "profile_id": "default",
      "instance_id": "<instance_id_dari_identify>",
      "purpose": "interactive_client"
    }
  }
  ```
- **Response:**
  ```json
  {
    "ok": true,
    "id": "req-03",
    "result": {
      "ticket": "tkt_9f81a7b2e4c3...",
      "expires_in_seconds": 30,
      "instance_id": "inst_12345",
      "profile_id": "default",
      "runtime_protocol": 1
    }
  }
  ```

---

## 3. Networking & Real-Time WebSocket Streaming

Setelah klien memperoleh `session-ticket` melalui Named Pipe, klien dapat membuka saluran komunikasi dua arah berkecepatan tinggi melalui WebSocket lokal.

### 3.1. Endpoint `/api/ws` (Interactive Event Stream)
- **URL Koneksi:** `ws://127.0.0.1:<port>/api/ws?ticket=<ticket>`
- **Protokol:** JSON-RPC 2.0 / Enveloped JSON Streaming.

### 3.2. Taksonomi Event Streaming Hermes
Saluran `/api/ws` mengirimkan event terstruktur yang mencerminkan fase-fase eksekusi agen:

```
[Klien Mengirim Prompt]
  --> {"action": "send_message", "session_id": "...", "content": "Tolong cek commit git terakhir"}

[Server Mengalirkan Respons Terstruktur]
  <-- {"event": "agent:thinking_delta", "delta": "Menganalisis repositori..."}
  <-- {"event": "agent:calling_model", "model": "hermes-3-llama-3.1-8b"}
  <-- {"event": "tool:call_start", "tool": "execute_command", "args": {"command": "git log -n 1"}}
  <-- {"event": "tool:output_chunk", "chunk": "commit c5f347eb..."}
  <-- {"event": "tool:call_end", "exit_code": 0}
  <-- {"event": "agent:content_delta", "delta": "Commit terakhir adalah c5f347eb dengan pesan..."}
  <-- {"event": "turn:complete", "usage": {"prompt_tokens": 420, "completion_tokens": 85}}
```

### 3.3. Interaktif Handshake untuk Persetujuan Alat (`approval.pending`)
Ketika Hermes hendak menjalankan perintah berisiko tinggi, alur persetujuan ditangani langsung melalui WebSocket:
1. **Server -> Klien:**
   ```json
   {
     "event": "approval:pending",
     "request_id": "appr_7712",
     "tool": "execute_command",
     "command": "powershell -Command \"Remove-Item -Recurse build/\"",
     "risk_level": "medium",
     "reason": "Destructive file deletion"
   }
   ```
2. **Klien -> Server (Tanggapan Pengguna dari Dynamic Island):**
   ```json
   {
     "action": "approval:respond",
     "request_id": "appr_7712",
     "verdict": "allow" // atau "deny", "allow_always"
   }
   ```

### 3.4. Endpoint Khusus Tambahan
1. **Interactive PTY Terminal (`/api/pty`):**  
   Menyediakan streaming raw ANSI/VT100 byte stream yang terhubung ke Windows ConPTY (`win_pty_bridge.py`). Memungkinkan integrasi emulator terminal penuh (`xterm.js`) di sisi klien.
2. **Audio Voice Streaming:**  
   - `/api/audio/speak-stream`: Aliran audio TTS (Text-to-Speech) untuk sintesis suara agen real-time.
   - `/api/audio/transcribe-stream`: Aliran audio input mikrofon untuk transkripsi Whisper lokal/remote.

---

## 4. Official Integration Precedent (`apps/desktop`)

Pemeriksaan mendalam terhadap direktori `apps/desktop/electron` mengungkap bagaimana tim pengembang Nous Research mengintegrasikan Hermes ke dalam aplikasi desktop Windows:

### 4.1. Manajemen Proses Windows (`backend-child.ts`)
Node.js default `child_process.kill()` di Windows hanya membunuh proses induk, meninggalkan proses anak (seperti instance Python, PTY, atau MCP server) menggantung di memori.  
**Solusi Nous Research:**  
Menerapkan *Windows Tree-Kill Escalation*:
```typescript
export function stopBackendChild(child, deps) {
  if (isWindows && Number.isInteger(child.pid)) {
    // Mengeksekusi taskkill /T /F secara native
    deps.forceKillProcessTree(child.pid)
  } else {
    process.kill(-child.pid, 'SIGTERM')
  }
}
```
Pola ini wajib diadopsi oleh Hime Assistant jika Hime mengelola siklus hidup proses Hermes di latar belakang.

### 4.2. Kebijakan Transport & Socket Pool (`api-transport.ts`)
Pada sistem operasi Windows, membuka socket TCP baru untuk setiap request HTTP loopback menimbulkan fenomena *TCP Port Exhaustion* dan *ECONNRESET storm*.  
**Solusi Nous Research:**
- Memisahkan dua agen HTTP keep-alive:
  1. `HTTP_JSON_AGENT`: Kolam koneksi berkapasitas 50 socket paralel untuk panggilan JSON instan dan berlatensi rendah.
  2. `HTTP_DOWNLOAD_AGENT`: Kolam terisolasi berkapasitas 8 socket untuk streaming file besar, sehingga pengunduhan tidak memblokir panggilan interaktif.
- **Aturan Retry Idempoten:** Hanya mengulang *non-idempotent verbs* (seperti `POST /api/prompt`) jika dipastikan paket belum pernah keluar dari TCP buffer (`requestState.bodySent === false`), guna mencegah pengiriman prompt ganda (*double prompt execution*).

---

## 5. Architectural Comparison for Hime Assistant

| Kriteria Integrasi | Model 1: Subprocess Stdio (`cli.py`) | Model 2: Embedded Python SDK | Model 3: Daemon via Named Pipe + WS (Direkomendasikan) |
| :--- | :--- | :--- | :--- |
| **Waktu Inisialisasi (Latency)** | Lambat (2–5 detik setiap turn karena cold start Python VM & imports). | Cepat di memori, tetapi mengunci runtime Hime. | **Instan (<5 ms)** karena Gateway Daemon sudah aktif di background. |
| **Konsumsi Memori Hime** | Rendah di Hime, tetapi boros spike CPU setiap eksekusi. | Sangat tinggi (Hime harus memuat modul Python & torch/transformers). | **Sangat Ringan** (Hime hanya mengonsumsi ~30 MB RAM sebagai antarmuka UI murni). |
| **Dukungan Streaming Real-Time** | Parsial (harus mem-parsing stdout baris demi baris, rawan glitch ANSI). | Langsung via Python callback. | **Sempurna & Asinkron** via protokol WebSocket `/api/ws`. |
| **Interaktif Approval Prompt** | Sulit (harus menginjeksi stdin raw terminal). | Memerlukan sinkronisasi thread kompleks. | **Native via Event JSON** (`approval:pending` -> `approval:respond`). |
| **Kemandirian Proses (Resilience)** | Jika CLI crash, tugas hilang. | Jika Python crash, UI Hime ikut crash. | **Terisolasi:** Jika UI Hime ditutup, agen tetap dapat bekerja di background. |

---

## 6. Kesimpulan & Rekomendasi Integrasi

Arsitektur integrasi **Model 3 (Daemon via Named Pipe + WebSocket)** adalah pilihan mutlak untuk Hime Assistant.  

Dengan memanfaatkan:
1. **Windows Named Pipe** (`\\.\pipe\hermes-gateway-<hash>`) untuk handshake awal dan pengambilan tiket otorisasi.
2. **WebSocket** (`/api/ws`) untuk streaming respons teks, pemantauan alat, dan penanganan persetujuan keamanan langsung di pulau dinamis (*dynamic island*).

Hime Assistant dapat dibangun sebagai aplikasi desktop ultra-ringan (menggunakan framework seperti **Tauri 2.0 / Rust**), yang berinteraksi dengan Hermes layaknya kopilot AI native tanpa beban overhead runtime Python di antarmuka pengguna.

