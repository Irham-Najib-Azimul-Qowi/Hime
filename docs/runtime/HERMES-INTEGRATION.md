# Hermes Agent Integration Specification

- **Document ID:** HIME-RUNTIME-08-02
- **Phase Covered:** Phase 08 — AI Runtime Integration
- **Classification:** `[VERIFIED STATIC]` & `[VERIFIED AUTOMATED TEST]`
- **Status:** APPROVED TECHNICAL SPECIFICATION
- **Date:** 2026-11-10

---

## 1. Identifikasi Runtime Hermes

Berdasarkan pemeriksaan langsung terhadap repositori Hermes Agent (`RE/Hermes/00-source-materials/hermes-agent`):
- **Versi Runtime:** Hermes Agent v0.8.2 (`pyproject.toml`, `gateway/control_socket.py`).
- **Paket Manajer:** Astral `uv` (`uv.lock`).
- **Python Target:** Python 3.11+.
- **Entry Points Utama:**
  * Headless Gateway Daemon: `uv run hermes gateway start`
  * CLI & Direct Agent Runner: `python run_agent.py` / `uv run hermes -c`
  * Control Socket: Windows Named Pipe `\\.\pipe\hermes-gateway-<home_hash>`

---

## 2. Pengawasan Proses Native (Win32 Job Object)

Hermes Agent dijalankan di bawah naungan `JobSupervisor` milik Hime Core:
1. **Pembuatan Terkendali:** Proses diluncurkan menggunakan flag Win32 `CREATE_SUSPENDED`.
2. **Asosiasi Job Object:** Sebelum thread utama diizinkan berjalan (`ResumeThread`), handle proses langsung diasosiasikan ke Win32 Job Object dengan limit `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.
3. **Pencegahan Proses Yatim (*Orphan Prevention*):** Jika Hime Core ditutup atau mengalami terminasi mendadak, kernel Windows secara otomatis mematikan proses Python Hermes dan seluruh proses anaknya.
4. **Isolasi Environment:** Direktori kerja diarahkan ke direktori terisolasi Hime dan environment variable `OPENAI_BASE_URL` disetel secara eksplisit menunjuk ke Loopback Provider Adapter Hime Core (`http://127.0.0.1:20129/v1`).

---

## 3. Wire Contract & Sesi Agen (`HermesRuntimeAdapter`)

Interaksi turn percakapan dikendalikan oleh adapter Rust `HermesRuntimeAdapter`:
- **Model Default:** `hermes-3-llama-3.1-8b` (Nous Research).
- **Aliran Turn:**
  1. `execute_turn(conversation_id, user_prompt, event_tx)` diinisialisasi dengan correlation ID unik.
  2. Adapter memancarkan status `thinking` ke event bus (Dynamic Island menampilkan spinner).
  3. Panggilan tool berisiko dikembalikan ke boundary Hime Core untuk evaluasi izin.
  4. Token asisten dialirkan secara bertahap via event `stream_delta`.
  5. Seluruh teks disanitasi menggunakan `redact_sensitive_text` untuk membersihkan tag `<think>` dan API key sebelum diserahkan ke antarmuka pengguna.
- **Pembatalan (*Cancellation*):** Mendukung penghentian generasi seketika melalui `cancel_active_turn()` via channel oneshot tokio tanpa merusak konsistensi sesi.

