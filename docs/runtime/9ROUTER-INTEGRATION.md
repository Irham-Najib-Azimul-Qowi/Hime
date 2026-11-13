# 9Router AI Gateway Integration Specification

- **Document ID:** HIME-RUNTIME-08-03
- **Phase Covered:** Phase 08 — AI Runtime Integration
- **Classification:** `[VERIFIED STATIC]` & `[VERIFIED AUTOMATED TEST]`
- **Status:** APPROVED TECHNICAL SPECIFICATION
- **Date:** 2026-11-10

---

## 1. Identifikasi Gateway 9Router

Berdasarkan pemeriksaan langsung terhadap kode sumber 9Router (`RE/9ROUTER/00-source-materials/9router`):
- **Teknologi Utama:** Node.js v24, Next.js standalone server, SQLite internal repository (`src/lib/localDb.js`).
- **Entry Points:** `custom-server.js`, `run-9router.bat`.
- **Port Listener Default:** TCP `20128` pada IPv4 `127.0.0.1`.
- **Topologi Integrasi:** Managed Background Service di bawah `JobSupervisor` Hime Core.

---

## 2. Model Discovery & Routing Flow

Hime Core berinteraksi dengan 9Router melalui modul `RouterGatewayAdapter`:
1. **Health Check Probes:** Adapter memeriksa ketersediaan port loopback `127.0.0.1:20128` dan mengukur latensi TCP.
2. **Koneksi Custom Provider Node:** 9Router dikonfigurasi menggunakan Custom Provider Node jenis *OpenAI-Compatible* dengan Base URL menunjuk ke Hime Loopback Adapter (`http://127.0.0.1:20129/v1`).
3. **Penyelarasan Virtual Combo:** 9Router mengelola load balancing dan fallback virtual model. Ketika request diarahkan ke node Hime, Hime Core menerima panggilan dan menyuntikkan kredensial resmi dari Vault DPAPI.
4. **Pencegahan Penulisan SQLite Langsung:** Hime Core **TIDAK PERNAH** menulis langsung ke SQLite internal `9router.db`. Konfigurasi diarahkan melalui antarmuka HTTP resmi dan Custom Node pointer.

---

## 3. Hasil Investigasi Empiris Celah Localhost (IB-02)

Pemeriksaan mendalam terhadap file middleware `src/dashboardGuard.js` membuktikan secara empiris:
```javascript
// Baris 161-165 src/dashboardGuard.js:
async function canAccessPublicLlmApi(request) {
  if (isLocalRequest(request)) return true; // <-- LOCALHOST BYPASS!
  if (await hasValidCliToken(request)) return true;
  return await hasValidApiKey(request);
}
```
- **Akar Kerentanan:** Setiap koneksi TCP yang berasal dari alamat loopback (`127.0.0.1` / `::1`) tanpa header `x-9r-via-proxy` diidentifikasi sebagai `isLocalRequest == true`. Akibatnya, rute publik LLM (`/v1/*`) **secara otomatis mengizinkan akses tanpa memvalidasi API key atau token autentikasi apa pun**.
- **Konfirmasi Status Blocker:** Karena kode sumber upstream 9Router tidak boleh dimodifikasi tanpa otorisasi Project Owner (ADR-006), celah ini **TETAP BERSTATUS `OPEN IMPLEMENTATION BLOCKER`**.
- **Mitigasi Pertahanan Berlapis (Defense-in-Depth):**
  1. Hime Core mengikat Provider Adapter pada port loopback terpisah (`20129`).
  2. Endpoint Hime Core memerlukan autentikasi token internal.
  3. Rekomendasi patch terkontrol disiapkan untuk persetujuan Project Owner.

