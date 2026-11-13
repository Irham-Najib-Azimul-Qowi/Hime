# AI Runtime Architecture & Topology

- **Document ID:** HIME-RUNTIME-08-01
- **Phase Covered:** Phase 08 — AI Runtime Integration
- **Classification:** `[VERIFIED STATIC]` & `[VERIFIED AUTOMATED TEST]`
- **Status:** APPROVED ARCHITECTURAL SPECIFICATION
- **Date:** 2026-11-10

---

## 1. Visi Arsitektur Runtime AI

Hime Assistant mengintegrasikan kecerdasan buatan otonom dengan desktop Windows 11 melalui topologi modular terisolasi. Arsitektur runtime ini tidak menempatkan model AI langsung di dalam antarmuka desktop, melainkan menempatkan **Hime Core sebagai pusat kendali (*Central Control Plane*)** yang mengorkestrasi dua runtime terpilih:

1. **Hermes Agent (Nous Research):** Runtime penalaran agen otonom (*agent reasoning & tool execution*).
2. **9Router AI Gateway (decolua):** Gerbang perutean multi-model dan multi-provider (*OpenAI, Anthropic, Gemini, local models*).
3. **Hime Core (Rust):** Pengawas proses native (Win32 Job Object), penegak kebijakan keamanan (PEP/PDP), kubah kredensial terlindungi DPAPI, dan perantara streaming IPC.
4. **Hime Desktop (Tauri 2.0 + React 18):** Antarmuka pengguna Dynamic Island, Omnibox, dan Conversation View.

```mermaid
flowchart TD
    subgraph UI_Layer ["Hime Desktop (Tauri 2 + React 18)"]
        UI_Chat["Conversation View"]
        UI_Island["Dynamic Island (Agent Activity)"]
        UI_Omnibox["Universal Omnibox (Ctrl+K)"]
    end

    subgraph Core_Control_Plane ["Hime Core Security Domain"]
        IPC_Server["IPC Server (Named Pipe)"]
        Orchestrator["AiOrchestrator"]
        ConvService["ConversationService (SQLite WAL)"]
        Supervisor["Process Supervisor (Job Objects)"]
        PEP["Policy Enforcement Point (PEP/PDP)"]
        DPAPI_Vault["Windows DPAPI Secret Vault"]
        EgressAdapter["Loopback Provider Adapter (:20129)"]
    end

    subgraph External_Runtimes ["Supervised External Subprocesses"]
        Hermes["Hermes Agent Runtime (Python uv)"]
        Router["9Router Gateway (:20128)"]
    end

    subgraph Upstream_Clouds ["AI Cloud Providers"]
        OpenAI["OpenAI API"]
        Anthropic["Anthropic API"]
        Nous["Nous Research API"]
    end

    UI_Chat -->|1. IPC Request (conv_send)| IPC_Server
    IPC_Server --> Orchestrator
    Orchestrator --> ConvService
    Orchestrator -->|2. Supervised Turn| Hermes
    Hermes -->|3. Model Chat Request| EgressAdapter
    EgressAdapter -->|4. Decrypt on-the-fly| DPAPI_Vault
    EgressAdapter -->|5. Outbound HTTPS| Upstream_Clouds
    EgressAdapter -->|6. SSE Token Stream| Hermes
    Hermes -->|7. Tool Invocation| PEP
    PEP -->|8. Tool Result| Hermes
    Hermes -->|9. Stream Delta & Events| Orchestrator
    Orchestrator -->|10. Real-time IPC Frames| UI_Chat
    Orchestrator -->|11. Activity State| UI_Island
```

---

## 2. Pemisahan Domain & Tanggung Jawab

| Komponen | Bahasa / Lingkungan | Peran & Batasan Tanggung Jawab |
|:---|:---|:---|
| **Hime Desktop** | TypeScript / React / Tauri | Antarmuka interaksi pengguna, visualisasi streaming token, tampilan aktivitas agen, dan tombol otorisasi persetujuan. Tidak menyimpan kredensial di `localStorage`. |
| **Hime Core** | Rust 2021 | Mengelola siklus hidup proses Hermes dan 9Router di bawah Win32 Job Object; mengevaluasi kebijakan eksekusi tool; mengamankan rahasia via Windows DPAPI; mengelola database SQLite WAL. |
| **Hermes Agent** | Python 3.11+ (`uv`) | Menjalankan *reasoning loop*, dekomposisi instruksi pengguna, dan sintesis respons. Mengakses model melalui base URL loopback Hime Core; panggilan tool dialihkan kembali ke Hime Core. |
| **Hime Provider Adapter** | Rust (Tokio TCP / HTTP) | Menyediakan endpoint OpenAI-compatible loopback (`127.0.0.1:20129/v1`). Mendekripsi kunci API on-demand dengan memori zeroization, mengalirkan token SSE. Menjamin database 9Router 0% plaintext (IB-01). |
| **9Router Gateway** | Node.js (`custom-server.js`) | Gateway perutean model dan fallback virtual combo. Dibatasi pada loopback port lokal di bawah isolasi pengawasan Hime Core. |

---

## 3. Protokol Aliran Data & Zero-CoT (SEC-09)

Sesuai aturan keamanan **SEC-09**, privasi penalaran internal (*Chain-of-Thought*) dilindungi secara mutlak:
1. Blok tag penalaran `<think>...</think>` dan `<thought>...</thought>` difilter menggunakan engine redaksi regex Hime Core sebelum token dipancarkan ke frontend.
2. Pengguna hanya melihat:
   - Status aktivitas agen (*"Hermes is formulating plan..."*).
   - Indikator pemanggilan tool (*"Tool Invocation: file_read"*).
   - Hasil akhir respons yang bersih dan terformat.
3. Kredensial provider tidak pernah dipaparkan melalui log IPC, UI event, maupun pesan galat.

