# Phase 08 Security Verification Report — AI Runtime & Credential Isolation

- **Document ID:** PHASE-08-SECURITY-VERIFICATION-REPORT
- **Phase Covered:** Phase 08 — AI Runtime Integration (Hermes Agent, 9Router Gateway, Credential Security & End-to-End AI Execution)
- **Date:** 2026-11-10
- **Authority:** System Architect & Technical Lead
- **Project Owner:** User (Satoshi)
- **Security Auditor & Engineering Executor:** AI Coding Agent (Senior Security Engineer, AI Systems Architect, Windows Systems Engineer)
- **Status Akhir:** **`READY FOR ARCHITECT REVIEW`**
- **Klasifikasi Keamanan:**
  * **IB-01 (Penyimpanan Kredensial):** **`RESOLVED`** `[VERIFIED AUTOMATED TEST]`
  * **IB-02 (Bypass Autentikasi Localhost 9Router):** **`OPEN IMPLEMENTATION BLOCKER`** `[VERIFIED STATIC]`
  * **SEC-09 (Zero Private CoT Exposure):** **`ENFORCED`** `[VERIFIED AUTOMATED TEST]`
  * **Process Supervision (Job Object):** **`ENFORCED`** `[VERIFIED AUTOMATED TEST]`

---

## 1. Ringkasan Eksekutif Keamanan (*Executive Security Summary*)

Sesuai instruksi tata kelola rekayasa (AGENTS.md) dan arahan arsitektur Phase 08, laporan ini menyajikan hasil verifikasi keamanan empiris terhadap integrasi runtime AI Hime yang mencakup:
1. **Resolusi Tuntas IB-01 (Kredensial Aman via DPAPI Vault & Loopback Egress Gateway):** Menghilangkan risiko penyimpanan API key plaintext di `9router.db` dengan memindahkan seluruh pengelolaan rahasia ke Windows DPAPI vault Hime Core. 9Router diposisikan murni sebagai node perute tanpa memegang kunci asli.
2. **Audit Empiris & Pembuktian Celah IB-02 (Localhost Bypass pada 9Router):** Memverifikasi keberadaan celah autentikasi pada `src/dashboardGuard.js` 9Router. Menegaskan status **`OPEN IMPLEMENTATION BLOCKER`** tanpa melakukan modifikasi sepihak terhadap kode upstream sebelum disetujui Project Owner.
3. **Penegakan Perlindungan Penalaran Agen (SEC-09 Zero Private CoT Exposure):** Memverifikasi bahwa seluruh token penalaran internal (`<think>`, `<thought>`) disaring secara server-side sebelum meninggalkan proses Hime Core.
4. **Pencegahan Proses Yatim (*Orphan Process Prevention*):** Menegakkan isolasi subprocess Hermes dan 9Router di bawah kendali kernel Win32 Job Object dengan atribut `KILL_ON_JOB_CLOSE`.

---

## 2. Resolusi Mendalam Kerentanan IB-01 (Plaintext Credentials in 9Router SQLite)

### A. Analisis Kerentanan Upstream
Pemeriksaan kode sumber 9Router (`RE/9ROUTER/00-source-materials/9router/src/lib/db/repos/connectionsRepo.js` dan `src/lib/localDb.js`) mengonfirmasi bahwa saat pengguna menambahkan provider (OpenAI, Anthropic, dll.), 9Router menyimpan payload JSON berisi `apiKey` mentah secara langsung ke dalam file SQLite disk `%APPDATA%\9router\db\data.sqlite` (atau `9router.db`). Jika laptop dicuri atau terdapat proses lokal yang mengekstrak file database, seluruh kunci API pengguna akan bocor (*CWE-312: Cleartext Storage of Sensitive Information*).

### B. Arsitektur Solusi Hime Core: Loopback Egress Gateway
Alih-alih memodifikasi skema internal 9Router yang berisiko merusak kompatibilitas rilis upstream, Hime Core membangun adapter loopback berpemilik (*Hime-Owned Provider Credential Adapter*):

```text
+-----------------------+           +------------------------+           +--------------------------+
|  9Router AI Gateway   |           |  Hime Core Egress GW   |           |  Windows OS DPAPI Vault  |
|  (Loopback :20128)    |           |  (Loopback :20129)     |           |  (Kernel Crypt32 API)    |
+-----------+-----------+           +-----------+------------+           +------------+-------------+
            |                                   |                                     |
            | 1. HTTP POST /v1/chat/completions |                                     |
            |    Bearer "hime-internal-dummy"   |                                     |
            |---------------------------------->|                                     |
            |                                   | 2. Query Encrypted Secret Blob      |
            |                                   |------------------------------------>|
            |                                   | 3. CryptUnprotectData (RAM only)    |
            |                                   |<------------------------------------|
            |                                   |                                     |
            |                                   | 4. Inject Real Header:              |
            |                                   |    Authorization: Bearer sk-live... |
            |                                   | 5. Zeroize RAM buffer immediately   |
            |                                   |                                     |
            |                                   | 6. Forward to Upstream Cloud API    |
            |                                   |------------------------------------> [ api.openai.com ]
            |                                   | 7. Relay SSE Chunks                 |
            |<----------------------------------|                                     |
```

### C. Jaminan Keamanan yang Dicapai:
1. **0% Plaintext Keys di 9Router:** Database `9router.db` hanya berisi URL `http://127.0.0.1:20129/v1` dan token dummy `"hime-internal"`.
2. **Enkripsi At-Rest Berbasis Kunci Pengguna OS:** Byte rahasia dienkripsi dengan Windows DPAPI (`CryptProtectData`), terikat pada kredensial login Windows pengguna saat ini. File database `hime.db` tidak dapat didekripsi dari akun pengguna lain atau instalasi OS terpisah.
3. **Pembersihan Memori (*Memory Zeroization*):** Segera setelah header HTTP keluar dikirim, buffer string plaintext dihancurkan menggunakan trait `Zeroize` dan `ZeroizeOnDrop` dari crate `zeroize`.
4. **Verifikasi Automatis:** Diuji melalui `test_storage_credentials` (hime-storage) dan `test_dpapi_credential_storage_and_redaction` (hime-core).
5. **Kesimpulan Status IB-01:** **`RESOLVED`** `[VERIFIED AUTOMATED TEST]`.

---

## 3. Investigasi & Pembuktian Empiris Celah IB-02 (Localhost Bypass)

### A. Kode Sumber Kerentanan
Pemeriksaan pada `RE/9ROUTER/00-source-materials/9router/src/dashboardGuard.js`:

```javascript
// Baris 161-165:
async function canAccessPublicLlmApi(request) {
  if (isLocalRequest(request)) return true; // <-- KERENTANAN UTAMA
  if (await hasValidCliToken(request)) return true;
  return await hasValidApiKey(request);
}

// Baris 14-22:
function isLocalRequest(request) {
  const forwardedFor = request.headers.get("x-forwarded-for");
  const remoteAddress = request.socket?.remoteAddress;
  const isLoopback = remoteAddress === "127.0.0.1" || remoteAddress === "::1" || remoteAddress === "::ffff:127.0.0.1";
  return isLoopback && !request.headers.has("x-9r-via-proxy");
}
```

### B. Bukti Dampak Eksploitasi
- Setiap permintaan HTTP yang diinisiasi oleh proses apa pun pada mesin host (misalnya script Node.js, Python, curl, atau malware yang berjalan di bawah pengguna yang sama) yang ditujukan ke `http://127.0.0.1:20128/v1/chat/completions` **secara otomatis diizinkan tanpa token otentikasi apa pun**.
- Mekanisme ini dirancang upstream untuk memudahkan CLI lokal, namun secara fundamental merusak batas keamanan lokal (*CWE-306: Missing Authentication for Critical Function*).

### C. Tata Kelola & Batasan Otoritas (ADR-006 & AGENTS.md)
Sesuai aturan AGENTS.md Bagian A & G:
> *"Agents do NOT modify external dependency source code (Hermes, 9Router) without a documented reason and approval."*
> *"Do NOT claim a vulnerability is fixed if it has only been moved or masked."*

Oleh karena itu:
- Agent **TIDAK MENGUBAH** kode sumber 9Router secara sepihak.
- Celah **IB-02 DITETAPKAN SECARA FORMAL BERSTATUS: `OPEN IMPLEMENTATION BLOCKER`**.

### D. Rekomendasi Solusi untuk Disetujui Project Owner:
| Opsi | Pendekatan | Kelebihan | Kekurangan |
|:---|:---|:---|:---|
| **Opsi 1 (Disarankan)** | **Controlled Upstream Patch:** Menghapus baris `if (isLocalRequest(request)) return true;` pada fork terkelola 9Router Hime dan mewajibkan token `x-hime-internal-auth`. | Menutup celah bypass secara total pada layer HTTP. | Memerlukan dokumentasi patch terkelola di repositori. |
| **Opsi 2** | **Windows Filtering Platform (WFP) / PID Guard:** Menggunakan firewall lokal untuk membatasi koneksi ke port 20128 hanya dari PID Hime Core. | Kode 9Router tetap murni 100% tanpa sentuhan. | Kompleksitas tinggi dan memerlukan hak administrator. |
| **Opsi 3** | **Formal Residual Risk Acceptance:** Project Owner menerima risiko localhost bypass dengan kompensasi bahwa mesin desktop adalah single-user workstation terlindungi. | Tidak ada overhead engineering tambahan. | Risiko tetap terbuka terhadap script lokal berbahaya. |

---

## 4. Penegakan Privasi Penalaran Agen Mutlak (SEC-09 Zero Private CoT Exposure)

### A. Pengujian Redaksi Server-Side
Pengujian otomatis `test_chain_of_thought_and_key_redaction` menguji payload teks yang mengandung blok penalaran dan kredensial:

```rust
let raw_output = "<think>Analisis strategi internal: Kunci API sk-live9876543210abcdefghij harus disembunyikan.</think>Halo! Saya siap membantu Anda.";
let clean_output = redact_sensitive_text(raw_output);
assert_eq!(clean_output, "Halo! Saya siap membantu Anda.");
assert!(!clean_output.contains("<think>"));
assert!(!clean_output.contains("sk-live"));
```

### B. Audit Antarmuka Pengguna & Log Tracing
- **Named Pipe IPC Frames:** Chunk streaming yang dikirim ke Hime Desktop telah dibersihkan secara mutlak di dalam thread Hime Core sebelum encoding biner frame dilakukan.
- **Dynamic Island:** Hanya menampilkan status operasional semantik (`AgentStateChanged::Thinking`, `AgentStateChanged::ExecutingTool`) tanpa mengekspos teks monolog penalaran ke elemen DOM.
- **Audit Logging:** Tracing SQLite WAL hanya mencatat metadata interaksi dan respons final yang telah melalui sensor redaksi.

---

## 5. Matriks Verifikasi Keamanan Subproses & Transpor

| Komponen Keamanan | Target Proteksi | Mekanisme & Parameter | Klasifikasi Bukti | Status |
|:---|:---|:---|:---:|:---:|
| **Win32 Job Object** | Hermes & 9Router Subprocesses | `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` + `CREATE_SUSPENDED` | `[VERIFIED AUTOMATED TEST]` | **ENFORCED** |
| **Localhost Binding** | Jaringan LAN / Wi-Fi Eksternal | Strict socket binding `127.0.0.1` (Tolak `0.0.0.0`) | `[VERIFIED STATIC]` | **ENFORCED** |
| **Named Pipe ACL** | IPC Desktop ke Core | Token Keamanan Pengguna Windows (`\\.\pipe\hime-ipc`) | `[VERIFIED RUNTIME]` | **ENFORCED** |
| **Tool Execution Gate** | Aksi Sistem Berbahaya | PDP/PEP Interaktif Hime Core (Confirmation Dialog) | `[VERIFIED AUTOMATED TEST]` | **ENFORCED** |
| **DPAPI Key Storage** | Kredensial AI Penyedia | Windows `CryptProtectData` + Memory Zeroization | `[VERIFIED AUTOMATED TEST]` | **RESOLVED** |
| **Localhost Auth Guard** | Endpoint Inferensi 9Router | Pemeriksaan `src/dashboardGuard.js` (Bypass `isLocalRequest`) | `[VERIFIED STATIC]` | **OPEN (IB-02)** |

---

## 6. Pernyataan Penutup Auditor Keamanan

Implementasi Phase 08 telah berhasil mengamankan penyimpanan kredensial secara mutlak melalui DPAPI Vault (IB-01 RESOLVED), melindungi privasi penalaran pengguna (SEC-09 ENFORCED), dan mengunci siklus hidup proses di bawah pengawasan kernel (Job Objects). 

Terkait **IB-02**, keputusan sadar diambil untuk **TIDAK MEREKAYASA STATUS** dan menyajikan kondisi kode apa adanya kepada Project Owner untuk penentuan opsi resolusi final.

