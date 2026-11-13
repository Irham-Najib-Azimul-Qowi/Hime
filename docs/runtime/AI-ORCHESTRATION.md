# AI Request Lifecycle & Tool Execution Orchestration

- **Document ID:** HIME-RUNTIME-08-05
- **Phase Covered:** Phase 08 — AI Runtime Integration
- **Classification:** `[VERIFIED AUTOMATED TEST]` & `[VERIFIED STATIC]`
- **Status:** APPROVED TECHNICAL SPECIFICATION
- **Date:** 2026-11-10

---

## 1. Siklus Hidup Permintaan Interaksi AI (*AI Request Lifecycle*)

Setiap interaksi pengguna yang dikirimkan melalui Dynamic Island, Omnibox, atau Conversation View melewati siklus kendali terpusat di dalam Hime Core:

```mermaid
sequenceDiagram
    autonumber
    actor User as Pengguna (Windows Desktop)
    participant UI as Hime Desktop (React UI)
    participant Tauri as Tauri IPC Backend
    participant Pipe as Named Pipe Transport
    participant Core as AiOrchestrator (Hime Core)
    participant PEP as Security PEP / PDP
    participant Conv as ConversationService (SQLite)
    participant Hermes as Hermes Runtime Adapter
    participant Egress as Provider Adapter (:20129)

    User->>UI: Ketik instruksi & kirim prompt
    UI->>Tauri: invoke("conversation_send_message", {conv_id, prompt})
    Tauri->>Pipe: JSON-RPC "conversation.send_message"
    Pipe->>Core: Dispatch RPC method
    Core->>Conv: Simpan pesan pengguna (role: "user")
    Core-->>UI: Event: AgentStateChanged("thinking")
    Core->>Hermes: execute_turn(conv_id, prompt)
    
    alt Pemanggilan Model AI
        Hermes->>Egress: POST /v1/chat/completions (SSE Stream)
        Egress-->>Hermes: Stream delta tokens
        Hermes->>Core: Emit stream_delta
        Core-->>UI: Event: AgentStreamChunk(delta)
    end

    alt Eksekusi Tool Berbahaya (File / System)
        Hermes->>Core: Request Tool Invocation (e.g. "delete_file")
        Core->>PEP: evaluate_action(target_resource, risk_level)
        PEP-->>Core: Result: RequiresUserConfirmation
        Core-->>UI: Event: AgentStateChanged("waiting_approval")
        User->>UI: Klik Otorisasi Izin
        UI->>Core: IPC "permissions.respond" (approved)
        Core->>PEP: Catat Audit Token Persetujuan
        Core->>Hermes: Lanjutkan eksekusi tool dengan hak akses
    end

    Hermes->>Core: Turn selesai (Final sanitized text)
    Core->>Conv: Simpan pesan asisten (role: "assistant")
    Core-->>UI: Event: AgentStateChanged("idle")
    UI-->>User: Tampilkan respons akhir yang utuh & bersih
```

---

## 2. Gatekeeping Eksekusi Tool (*Tool Execution Gateway*)

Hermes Agent dirancang memiliki kemampuan menjalankan tool native (*function calling*). Namun, dalam arsitektur Hime, agen **DILARANG KERAS** mengeksekusi aksi sistem operasi secara langsung tanpa pengawasan Hime Core.

### Mekanisme Pengawasan PEP/PDP:
1. **Penetapan Tingkat Risiko (*Risk Level Classification*):**
   - **Tingkat Rendah (*Low / Read-Only*):** Pembacaan status memori, pencarian model terdaftar, kalkulasi lokal matematika. Diizinkan secara otomatis oleh PDP (*Policy Decision Point*).
   - **Tingkat Sedang (*Medium / Non-Destructive Write*):** Pembuatan catatan sementara, eksekusi query baca file non-kritis. Diberikan izin berbasis durasi sesi aktif.
   - **Tingkat Tinggi (*High / Destructive System Actions*):** Modifikasi file kode, penghapusan data, perubahan konfigurasi jaringan, pemanggilan perintah shell PowerShell. Memerlukan konfirmasi interaktif pengguna secara eksplisit melalui prompt izin Hime Desktop.
2. **Audit Trail Tak Terbantahkan (*Tamper-Resistant Audit Trail*):** Setiap permintaan eksekusi dicatat di dalam tabel `audit_logs` Hime Core lengkap dengan timestamp UTC, nama tool, hash parameter, dan status evaluasi kebijakan.

---

## 3. Manajemen Konkurensi & Pembatalan Interaktif (*Interactive Cancellation*)

Untuk memastikan antarmuka tetap responsif dan konsumsi token tidak sia-sia saat pengguna mengubah pikiran:

1. **Correlation ID Unik:** Setiap sesi turn menghasilkan ID pelacakan UUIDv4 acak.
2. **Channel Pembatalan Instan:** Saat pengguna menekan tombol **"Stop Generation"** di UI atau menekan `Escape` di Omnibox:
   - Frontend memanggil perintah `conversation_cancel`.
   - Hime Core memicu pembatalan pada `tokio::sync::oneshot` sender yang terhubung langsung ke request stream Hermes.
   - Koneksi TCP SSE diputuskan segera, agen beralih seketika ke status `idle`, dan fragmen respons yang sudah diterima tetap disimpan secara aman di SQLite dengan metadata status `cancelled: true`.

---

## 4. Bukti Verifikasi Automatis

Orkestrasi AI diverifikasi secara lengkap melalui:
- **`test_conversation_cancellation` (`crates/hime-core/tests/ai_integration.rs`):** Menguji penghentian generasi secara deterministik tanpa deadlock atau kebocoran resource.
- **`test_desktop_ai_runtime_integration_flow` (`apps/hime-desktop/src-tauri/tests/desktop_integration.rs`):** Menguji transmisi round-trip dari pemanggilan antarmuka desktop, transit pipa bernama, orkestrasi Hime Core, hingga pengembalian payload respons.
- **Status Orkestrasi:** **`VERIFIED AUTOMATED TEST`**.

