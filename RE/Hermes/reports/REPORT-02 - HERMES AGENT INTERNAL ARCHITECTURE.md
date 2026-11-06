# REPORT-02: HERMES AGENT INTERNAL ARCHITECTURE
**Project:** Hime Assistant — Reverse Engineering & Technical Analysis  
**Target:** Nous Research Hermes Agent (`hermes-agent`)  
**Components:** `run_agent.py`, `agent/conversation_loop.py`, `tools/`, `hermes_state.py`  
**Author:** Senior Reverse Engineering Researcher & Technical Architect  
**Date:** November 2026  
**Status:** Completed  

---

## 1. Executive Summary

Laporan ini membedah arsitektur internal mesin **Hermes Agent**. Investigasi difokuskan pada bagaimana kelas inti `AIAgent` diorganisir, bagaimana siklus penalaran dan eksekusi (*conversation reasoning loop*) beroperasi sebagai mesin *state machine* deterministik, mekanisme registrasi dan perizinan *tools*, serta manajemen memori bertingkat (*multi-tier memory & compaction*).

Pemahaman mendalam tentang arsitektur internal ini sangat krusial agar Hime Assistant dapat menangkap setiap *event*, mengontrol status eksekusi secara presisi, dan menangani persetujuan eksekusi tool (*tool approvals*) langsung dari floating island.

---

## 2. Agent Core & Inheritance Hierarchy

Inti operasional Hermes berada pada kelas `AIAgent` yang didefinisikan dalam `run_agent.py:214`. Alih-alih menggunakan hierarki *deep inheritance* yang kaku, Hermes menerapkan pola arsitektur **Modular Mixin Composition** yang mewarisi 14 mixin fungsional:

```python
class AIAgent(
    TurnFacadeMixin,
    StreamDeliveryMixin,
    SessionPersistenceMixin,
    CompressionFacadeMixin,
    ToolsetFacadeMixin,
    PromptRefreshFacadeMixin,
    MessageMetadataMixin,
    ModelConfigMixin,
    FastModeMixin,
    VoiceRouteMixin,
    MoATurnMixin,
    TaskContextMixin,
    ObservabilityMixin,
    CredentialRefreshMixin
):
    """Facade orkestrator utama agen Hermes."""
```

### Karakteristik Desain Mixin:
1. **Separation of Concerns:** Setiap mixin mengisolasi domain logika tertentu (misalnya, `StreamDeliveryMixin` hanya menangani pengiriman chunk teks atau audio ke callback/WebSocket, sementara `SessionPersistenceMixin` mengelola commit ke basis data SQLite).
2. **Stateless Loop Separation:** Meskipun `AIAgent` menyimpan konfigurasi dan status jangka panjang (*long-lived state*), eksekusi percakapan aktual didelegasikan ke fungsi terpisah `agent/conversation_loop.py:run_conversation()`.
3. **Cross-Turn Caching:** Instance `AIAgent` di-cache di dalam Gateway Daemon untuk menghindari overhead parsing konfigurasi dan inisialisasi koneksi model pada giliran (*turn*) berikutnya. Variabel sementara per-turn secara eksplisit direset pada awal setiap iterasi untuk mencegah kebocoran status (*state leakage*).

---

## 3. The Conversation Reasoning Loop (`agent/conversation_loop.py`)

Siklus berpikir dan bertindak Hermes Agent berjalan melalui *state machine* tertutup di dalam `_run_conversation_turn` (`agent/conversation_loop.py:1640-1860`). Siklus ini berjalan dalam sebuah `while` loop yang dibatasi oleh `max_iterations` dan anggaran token/waktu (*iteration budget*).

### Tahapan Fase-Fase Loop (The 10 Iteration Phases):

```
+-------------------------------------------------------------------------------+
|                       CONVERSATION LOOP STATE MACHINE                         |
+-------------------------------------------------------------------------------+
                                        |
                                        v
                    +---------------------------------------+
                    | 1. begin_iteration                    |
                    |    Reset retry state, increment count |
                    +-------------------+-------------------+
                                        |
                                        v
                    +---------------------------------------+
                    | 2. prepare_iteration                  |
                    |    Check context limits & budget      |
                    +-------------------+-------------------+
                                        |
                                        v
                    +---------------------------------------+
                    | 3. assemble_api_request               |
                    |    Format messages, tools & system msg|
                    +-------------------+-------------------+
                                        |
                                        v
                    +---------------------------------------+
                    | 4. run_preflight_gate                 |
                    |    Compaction check, token estimation |
                    +-------------------+-------------------+
                                        |
                                        v
                    +---------------------------------------+
                    | 5. announce_api_call                  |
                    |    Emit `agent:calling_model` event   |
                    +-------------------+-------------------+
                                        |
                                        v
                    +---------------------------------------+
                    | 6. _run_api_retry_loop                |
                    |    Streaming inference, rate limits,  |
                    |    error recovery via TurnRetryState  |
                    +-------------------+-------------------+
                                        |
                                        v
                    +---------------------------------------+
                    | 7. normalize_model_response           |
                    |    Parse tool calls, repair JSON tags |
                    +-------------------+-------------------+
                                        |
                                        +-----------------------+
                                        |                       |
                     [Ada Tool Calls?]  |                       | [Teks Selesai]
                                        v                       v
                    +-----------------------+       +-----------------------+
                    | 8. run_tool_round     |       | 9. finish_text_resp   |
                    |    Approval gates,    |       |    Stream final delta,|
                    |    execute commands,  |       |    finalize usage     |
                    |    append tool result |       +-----------+-----------+
                    +-----------+-----------+                   |
                                |                               |
                                +-------> (Loop Kembali)        v
                                                    +-----------------------+
                                                    | 10. finalize_turn     |
                                                    |     Commit to SQLite, |
                                                    |     save trajectory   |
                                                    +-----------------------+
```

### Rincian Perilaku Fase Kritis:

- **Fase 4 (`run_preflight_gate`):** Sebelum memanggil model, Hermes menghitung estimasi jumlah token pesan dan tools. Jika ukuran melebihi batas model (*context window limit*), mesin secara otomatis memicu `trajectory_compressor` untuk merangkum percakapan lama sebelum permintaan dikirim.
- **Fase 6 (`_run_api_retry_loop`):** Mengelola kegagalan inferensi transien (HTTP 429 Rate Limit, HTTP 500, socket timeout). Menggunakan struktur data `TurnRetryState` untuk exponential backoff tanpa merusak riwayat pesan yang sudah valid.
- **Fase 7 (`normalize_model_response`):** Menangani model-model yang memiliki format tool call non-standar (seperti tag XML `<tool_call>` atau teks JSON mentah). Modul `message_sanitization.py` melakukan perbaikan sintaks secara transparan.
- **Fase 8 (`run_tool_round`):** Jika model meminta pemanggilan fungsi/alat, eksekusi dilakukan secara paralel atau serial tergantung tipe tool. Setiap tool diproses melalui lapisan keamanan `tools/approval.py`.

---

## 4. Tool Execution System & Security Gating

Sistem alat (*tool system*) Hermes dirancang modular, terisolasi, dan aman.

### 4.1. Tool Registration (`tools/registry.py`)
Setiap modul tool mendaftarkan fungsinya menggunakan dekorator atau panggilan langsung `registry.register()`:
```python
@registry.register(
    name="execute_command",
    description="Menjalankan shell command di lingkungan Windows/Linux",
    parameters={...},
    toolset="terminal",
    requires_approval=True
)
def execute_command(command: str, ...):
    ...
```
Hermes memanfaatkan Python `ContextVar` (`session_tool_scope`) untuk menyediakan alat-alat spesifik per sesi (*scoped dynamic tools*). Hal ini memungkinkan sesi tertentu memiliki tool khusus tanpa mengotori sesi pengguna lain.

### 4.2. Security & Approval Flow (`tools/approval.py`)
Hermes menerapkan 3 lapis kebijakan perizinan (*approval gates*):

1. **YOLO Mode (`HERMES_YOLO_MODE` / `/yolo`):**
   - Mode tanpa pengawasan di mana semua perintah dieksekusi tanpa konfirmasi pengguna.
   - Konstanta `_YOLO_MODE_FROZEN` di-cache saat modul diimpor untuk mencegah *prompt injection* yang mencoba memanipulasi variabel lingkungan saat runtime.
2. **Smart Approval (`_smart_verdict`):**
   - Menggunakan LLM sekunder berkategori ringan (*Guardian LLM*) untuk menganalisis risiko dari baris perintah shell sebelum dieksekusi.
   - Jika Guardian mendeteksi potensi destruktif, perintah dialihkan ke persetujuan manual atau dibatalkan.
3. **Interactive Manual Gate (`_await_gateway_decision`):**
   - Ketika perintah berbahaya terdeteksi (seperti memodifikasi registry Windows, menghapus direktori, atau menjalankan skrip yang diunduh dari internet), eksekusi diblokir sementara.
   - Gateway mengirimkan event `approval.pending` ke antarmuka klien (CLI, Desktop, atau Hime Island).
   - Klien mengirim balik respons `approval.respond` dengan nilai `allow`, `deny`, atau `allow_always`.
4. **Hardline Blocking:**
   - Perintah-perintah ekstrim tertentu (seperti manipulasi `sudo` via stdin atau format disk) diblokir secara permanen di tingkat filter regex dasar (`approval_floors.py`) tanpa opsi persetujuan.

---

## 5. Memory, Context & State Management

Hermes Agent mengimplementasikan sistem memori multi-lapis (*multi-tier state store*) untuk menyeimbangkan kecepatan akses, persistensi, dan batas konteks model:

```
+-------------------------------------------------------------------------------+
|                       HERMES MULTI-TIER MEMORY MODEL                          |
+-------------------------------------------------------------------------------+

[ LAPIS 1: WORKING CONTEXT (In-Memory Ring Buffer) ]
  - Berisi pesan-pesan dari turn saat ini.
  - Sangat cepat, berada dalam memori proses Python.
  - Mengalami micro-compaction otomatis di akhir setiap turn.
         |
         v (Sinkronisasi Asinkron / WAL Write)
[ LAPIS 2: PERSISTENT STORAGE (SQLite WAL DB - `state.db`) ]
  - Berlokasi di `%LOCALAPPDATA%\hermes\state.db` (Windows).
  - Skema Relasional: `sessions`, `messages`, `tags`, `tokens_usage`.
  - Menggunakan Journal Mode WAL (Write-Ahead Logging): 
    Mendukung pembaca paralel (GUI / CLI) bersamaan dengan satu penulis (Agent).
  - Dilengkapi Mesin Pencari Teks Penuh (FTS5) untuk pencarian historis lintas sesi.
         |
         v (Saat Context Window Penuh)
[ LAPIS 3: TRAJECTORY COMPRESSION & PRUNING ]
  - Modul: `trajectory_compressor.py`.
  - Merangkum 20-50 giliran pesan sebelumnya menjadi ringkasan faktual terpadat.
  - Membuka fork sesi baru dengan atribut `parent_session_id`, sehingga riwayat
    asli tetap utuh dan dapat diaudit kembali.
         |
         v (Evolusi Otonom)
[ LAPIS 4: DYNAMIC AGENTIC SKILLS (`~/.hermes/skills/`) ]
  - Agen mampu merangkum prosedur kerja yang berhasil menjadi file skill markdown
    dan Python script yang dapat dipanggil kembali pada tugas-tugas masa depan.
```

### Skema Database SQLite WAL (`hermes_state.py`):
Basis data `state.db` memuat tabel-tabel utama:
- `sessions`: `id`, `name`, `created_at`, `parent_session_id`, `model`, `system_prompt`.
- `messages`: `id`, `session_id`, `role`, `content`, `tool_calls`, `created_at`, `token_count`.
- `fts_messages`: Tabel indeks virtual SQLite FTS5 yang memungkinkan pencarian kata kunci di masa lalu dalam waktu <5 milidetik.

---

## 6. Arsitektur Implikasi untuk Hime Assistant

1. **Event Interception Tanpa Polling:**
   Karena `AIAgent` menerbitkan status pada setiap transisi fase melalui gateway (seperti `calling_model`, `tool_starting`, `tool_output`), Hime Assistant dapat menampilkan animasi dinamis (misalnya island berdenyut saat berpikir, melebar saat menjalankan terminal, atau berkedip saat meminta approval) secara real-time.
2. **Approval Handling:**
   Hime tidak perlu membangun engine parser perintah sendiri. Hime cukup menangani payload JSON `approval.pending` dari Hermes dan menyajikan tombol `[Izinkan]` / `[Tolak]` pada antarmuka Dynamic Island.
3. **Session Continuity:**
   Dengan membaca basis data `state.db` secara langsung dalam mode *read-only* (atau melalui REST API Hermes), Hime Assistant dapat menampilkan daftar riwayat sesi dan melanjutkan percakapan yang sebelumnya dibuat di terminal.

