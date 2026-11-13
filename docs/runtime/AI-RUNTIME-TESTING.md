# AI Runtime Testing Strategy & Verification Matrix

- **Document ID:** HIME-RUNTIME-08-08
- **Phase Covered:** Phase 08 — AI Runtime Integration
- **Classification:** `[VERIFIED AUTOMATED TEST]`
- **Status:** APPROVED TEST SPECIFICATION
- **Date:** 2026-11-10

---

## 1. Filosofi & Strategi Pengujian Runtime AI

Pengujian subsistem AI pada Hime mematuhi prinsip **Evidence-First Engineering** (AGENTS.md):
1. **Zero Flaky Tests:** Pengujian tidak boleh bergantung pada jaringan internet luar yang tidak deterministik atau API cloud komersial pihak ketiga yang dapat berubah sewaktu-waktu.
2. **Pemisahan Boundary Pengujian:**
   - **Automated Mock/Loopback:** Memverifikasi kontrak protokol HTTP, SSE stream parsing, DPAPI encryption/decryption, dan IPC framing secara 100% lokal.
   - **Supervised Subprocess Smoke:** Memverifikasi integritas peluncuran Win32 Job Object dan integrasi port loopback.
3. **Cakupan Berlapis (*Layered Test Coverage*):** Dari tingkat enkripsi storage terendah hingga dispatching antarmuka grafis desktop tertinggi.

---

## 2. Matriks Pengujian Lengkap (Complete Test Matrix)

| Kategori | Target Crate / Modul | Test Identifier | Deskripsi & Verifikasi | Hasil Aktual |
|:---|:---|:---|:---|:---|
| **Storage / Vault** | `hime-storage` | `test_storage_credentials` | Enkripsi DPAPI, penyimpanan tabel `credentials`, pembacaan dan pencegahan duplikasi | `PASSED` |
| **Storage / Conv** | `hime-storage` | `test_storage_conversations_and_messages` | Pembuatan percakapan, penyimpanan multi-turn pesan SQLite WAL, pengurutan timestamp | `PASSED` |
| **Security / CoT** | `hime-security` | `test_chain_of_thought_and_key_redaction` | Redaksi mutlak tag `<think>` dan sensor token API key `sk-...` (SEC-09) | `PASSED` |
| **AI Integration** | `hime-core` | `test_ai_status_and_capabilities` | Evaluasi status gateway AI, deteksi adapter aktif, pelaporan model | `PASSED` |
| **AI Integration** | `hime-core` | `test_ai_models_listing` | Pengambilan daftar model virtual dan metadata provider | `PASSED` |
| **AI Integration** | `hime-core` | `test_dpapi_credential_storage_and_redaction` | Penyimpanan kunci via RPC dan jaminan redaksi nilai sensitif (IB-01) | `PASSED` |
| **AI Integration** | `hime-core` | `test_conversation_lifecycle_and_persistence` | Siklus lengkap pembuatan percakapan, pengiriman pesan, pembaruan turn, persistensi | `PASSED` |
| **AI Integration** | `hime-core` | `test_conversation_cancellation` | Pembatalan turn interaktif melalui channel oneshot tokio | `PASSED` |
| **AI Integration** | `hime-core` | `test_provider_adapter_loopback_streaming` | Eksekusi server HTTP loopback, pengaliran token Server-Sent Events (SSE) | `PASSED` |
| **Desktop E2E** | `hime-desktop` | `test_desktop_ai_runtime_integration_flow` | Round-trip Named Pipe IPC: Desktop Tauri backend -> Core -> Adapters -> Response | `PASSED` |

---

## 3. Hasil Rekonsiliasi Pengujian Workspace

Berdasarkan eksekusi aktual perintah:
```powershell
cargo test --workspace
```

| Crate | Unit Tests | Integration Tests | Total Tests | Status |
|:---|:---:|:---:|:---:|:---:|
| `crates/hime-core` | 2 | 12 (5 AI + 7 Foundation) | 14 | `14 PASSED / 0 FAILED` |
| `apps/hime-desktop` (src-tauri) | 0 | 7 | 7 | `7 PASSED / 0 FAILED` |
| `crates/hime-ipc` | 14 | 0 | 14 | `14 PASSED / 0 FAILED` |
| `crates/hime-pal` | 1 | 0 | 1 | `1 PASSED / 0 FAILED` |
| `crates/hime-security` | 5 | 0 | 5 | `5 PASSED / 0 FAILED` |
| `crates/hime-storage` | 4 | 0 | 4 | `4 PASSED / 0 FAILED` |
| **TOTAL KESELURUHAN** | **26** | **19** | **45** | **45 PASSED / 0 FAILED / 0 IGNORED** |

---

## 4. Prosedur Reproduksi Pengujian Mandiri

Setiap pengembang atau agen dapat mereproduksi seluruh hasil pengujian secara mandiri dengan langkah-langkah berikut:

```powershell
# 1. Jalankan seluruh test suite Rust
cargo test --workspace

# 2. Jalankan test khusus AI runtime integration dengan log terperinci
cargo test -p hime-core --test ai_integration -- --nocapture

# 3. Jalankan test integrasi desktop end-to-end
cargo test -p hime-desktop --test desktop_integration -- --nocapture

# 4. Verifikasi kompilasi frontend TypeScript / React
pnpm --dir apps/hime-desktop build
```

