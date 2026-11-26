# PHASE 08 — AI RUNTIME INTEGRATION
**Hermes Agent · 9Router Gateway · Credential Security (IB-01) · Localhost Security (IB-02) · End-to-End AI**

**Entry:** Fase 07 `QUALIFIED ACCEPTANCE` (owner sudah menjawab "ya" untuk melanjutkan). Audit independen Fase 00–07 selesai dan baseline Git (G1) ditetapkan.
**Status awal:** prompt Fase 08 sudah pernah dikirim ke agent sebelumnya tanpa laporan. **Periksa repository dulu** — mungkin sebagian sudah dikerjakan. Lanjutkan dari kondisi nyata, jangan mengulang pekerjaan yang sudah benar.
**Kewajiban dibawa:** G1, G3, G4, G5, G6, G7, G12 (lihat `05-…` bagian 4).

## 1. Tujuan

Hidupkan kemampuan AI nyata: pengguna membuka Hime Desktop, mengirim instruksi, menerima respons **nyata** dari model melalui **Hermes + 9Router**, melihat **Agent Activity Stream** real-time — semua di bawah boundary keamanan Hime Core. Hime Core tetap **control plane**, bukan proxy pasif.

> Target bukan sekadar "Hime bisa chatting". Targetnya **fondasi agentic yang aman**: streaming, session, tool execution lewat gateway, permission, cancellation, dan recovery semuanya bekerja.

## 2. Tiga keputusan yang tidak boleh dikompromikan
1. Jangan melemahkan keamanan demi AI cepat hidup — IB-01 dan IB-02 diuji nyata.
2. Mulai dengan **satu** Hermes runtime. Spesialis nanti lewat skill/tool (Fase 14).
3. Integrasi berhasil bukan karena menerima jawaban AI; **streaming, session, tool execution, permission, cancellation, recovery** harus bekerja.

## 3. Task

### 08.1 Discovery runtime (bukti dulu, kode kemudian)
Periksa **instalasi aktual** (jangan asumsi lokasi/nama executable):
- Hermes: versi, entry point, interpreter/venv, konfigurasi, environment variables, mekanisme sesi, mekanisme tool/skill, protokol streaming, endpoint (jika ada), cara dihentikan.
- 9Router: versi, entry point, port, konfigurasi, endpoint (OpenAI-compatible), model discovery, Custom Node, autentikasi, penyimpanan config, `dashboardGuard.js`, middleware, binding IPv4/IPv6.
- Jika belum terpasang: pasang di direktori terkelola Hime dengan versi **terkunci** (butuh jaringan; bila pemasangan memerlukan admin → Owner Gate #4).
- Klasifikasikan temuan: `VERIFIED STATIC/RUNTIME`, `INFERRED`, `UNKNOWN`, `NOT TESTED`.
- Hasil: `docs/runtime/RUNTIME-DISCOVERY.md` + **compatibility manifest** awal (`config/compatibility-manifest.json`: versi Hermes, 9Router, protokol IPC, skema DB).

### 08.2 Process Supervisor untuk runtime
Gunakan supervisor Fase 06 (Job Object, `CREATE_SUSPENDED` → assign → resume). Wajib: deteksi instalasi, start, health check, deteksi crash, restart policy (backoff, batas percobaan), shutdown terkendali, shutdown saat Hime ditutup, pembersihan proses dalam Job, penanganan konflik port, isolasi environment, working directory, timeout, batas resource, laporan kegagalan. **Tanpa hak administrator.** Kerjakan G5: uji breakaway/nested job/assignment failure; klaim hanya untuk proses yang terbukti masuk Job.

### 08.3 IB-01 — Credential security (lihat `06-…` bagian 5)
Implementasikan **Hime-Owned Provider Adapter** (`ProviderCredentialAdapter`) bila contract test Custom Node lulus. Jalankan seluruh contract test (discovery, chat, SSE, non-stream, auth forwarding, error mapping, retry, timeout, cancel, model selection, fallback). Jika Custom Node tak memadai → evaluasi alternatif, tulis ADR, **jangan** aktifkan jalur kredensial sensitif. Periksa DB 9Router setelah operasi. Gunakan **kredensial uji aman** — bila owner belum menyediakan, tandai uji nyata `BLOCKED` dan buat permintaan di `OWNER-DECISIONS-NEEDED.md`; lanjutkan pekerjaan lain dengan provider tiruan **yang jelas diberi label mock** (mock tidak boleh dilaporkan sebagai end-to-end nyata).

### 08.4 IB-02 — Localhost security (lihat `06-…` bagian 6)
Investigasi jalur penuh middleware 9Router; buktikan urutan autentikasi vs bypass lokal; pilih mekanisme (patch terkendali [Gate #2] / isolasi proses+jaringan / integrasi alternatif); uji bypass langsung, IPv4/IPv6, LAN. Jika penyelesaian butuh modifikasi upstream: tulis rekomendasi teknis + patch diusulkan (tidak diterapkan) + permintaan owner, lalu kerjakan hal lain.

### 08.5 Hermes Adapter (`HermesRuntimeAdapter`)
Terisolasi di satu modul: inisialisasi runtime, session management, kirim request, kumpulkan respons, **streaming event**, cancellation, status runtime, normalisasi error. Pisahkan konfigurasi: executable/interpreter, Python env, config Hermes, model endpoint, environment, logging. **Jangan** menyebar komunikasi Hermes ke modul lain.

### 08.6 9Router Adapter (`RouterGatewayAdapter`)
Daftar provider dan model, konfigurasi provider (yang didukung), routing model, health, request forwarding, streaming, error, timeout, retry, fallback. **Jangan** menulis langsung ke SQLite 9Router; gunakan API resmi/konfigurasi yang didukung; jangan bergantung pada endpoint internal yang belum terverifikasi (jika terpaksa → adapter tunggal + contract test + catatan risiko). Fondasi model selection (model aktif, per kategori task, fallback, status koneksi) — **tanpa fitur palsu**.

### 08.7 Orkestrasi AI di Hime Core
Boundary minimum (nama boleh disesuaikan): `HermesRuntimeAdapter`, `RouterGatewayAdapter`, `ProviderCredentialAdapter`, `ConversationService`, `AgentSessionManager`, `AgentEventBridge`, `RuntimeHealthService`, `ToolExecutionGateway`, `AIRequestPolicyService`.

**Alur request:** UI → IPC → Core validasi → Policy (izin+konteks) → buat/lanjutkan agent session → Hermes → model via 9Router → tool invocation kembali ke Core → evaluasi kebijakan → eksekusi via Tool Gateway (bila diizinkan) → hasil ke Hermes → respons dan activity event ke Desktop → persistensi percakapan + audit. Jika Core tidak tersedia → operasi berisiko **gagal tertutup**. Klasifikasi tool: read-only / reversible / destructive / sensitive; pakai Permission Approval lifecycle Fase 07. ID idempoten per tool call (cegah eksekusi ganda saat retry).

### 08.8 Streaming dan real-time
Hermes → Core → IPC → Desktop: streaming teks, activity events, tool events, approval requests, completion, error, cancellation. Event punya correlation ID dan nomor urut; reconnect tidak menggandakan output; cancel menghentikan pekerjaan sesuai kemampuan runtime (dokumentasikan batasnya); UI tidak menampilkan state basi; error tidak membocorkan kredensial. **Tampilkan Agent Activity Stream, bukan chain-of-thought privat.**

### 08.9 Integrasi pengalaman percakapan
Hubungkan Conversation Interface Fase 07: percakapan baru/lanjut, riwayat, persistensi pesan (di Core/SQLite, bukan hanya frontend), streaming, stop generation, pemulihan error, state sesi, info model, state "runtime tidak tersedia". Percakapan dapat dipulihkan setelah aplikasi dibuka ulang. Jika Hermes belum mendukung fitur tertentu → dokumentasikan dan beri fallback jujur.

### 08.10 Observability dan resource
Monitor: status Core, proses Hermes, status 9Router, koneksi provider, sesi aktif, request aktif, latensi, durasi stream, token usage (jika tersedia), error count, restart count. **Hanya metrik yang dapat diukur.** Log terstruktur dengan correlation ID; uji redaksi dengan **kredensial kanari**. Ukur CPU/RAM/latensi aktual (G12): idle, chat, tool call, restart. Satu Hermes, satu reasoning task aktif; backpressure, timeout, cancel, degradasi anggun.

### 08.11 Pengujian
- **Hermes:** startup, shutdown, crash recovery, health, session, request, streaming, cancel.
- **9Router:** health, discovery, pemilihan provider, request API, streaming, error, autentikasi, **penolakan akses langsung**.
- **Kredensial:** round-trip vault, inspeksi penyimpanan, retrieval, redaksi, kredensial tak tersedia/kedaluwarsa, request provider tanpa kebocoran.
- **Core:** siklus orkestrasi, boundary kebijakan, eksekusi tool, approval, streaming IPC, cancel, recovery.
- **IPC (G6):** fragmentasi, konkurensi, reconnect, frame cacat, replay.
- **End-to-end (15 langkah):** launch Core → launch Hermes → launch 9Router → health → discovery model → buka Desktop → kirim prompt nyata (kredensial uji) → terima respons nyata → amati streaming → amati Activity Stream → uji cancel → uji kegagalan runtime → uji recovery → verifikasi keamanan kredensial → verifikasi boundary akses langsung. **Tidak boleh mock untuk menyatakan E2E berhasil.** Bagian yang tak bisa dijalankan (tanpa kredensial uji) → `BLOCKED`/`NOT TESTED`.
- **Smoke GUI (G2):** bila sesi interaktif tersedia, ambil screenshot Island dan Conversation.

## 4. Governance keamanan (ulang: wajib dipatuhi)
Core = PEP; Hermes bukan otoritas; 9Router bukan batas; localhost bukan autentikasi; Named Pipe ≠ perlindungan HTTP 9Router; DPAPI bukan perlindungan dari malware se-user; tanpa shell/FS tak terbatas; tanpa kredensial terekspos; tanpa CoT privat. **IB-01/IB-02 hanya RESOLVED bila seluruh kriteria terbukti**; bila tidak, pertahankan OPEN, jangan sembunyikan, jangan turunkan standar uji, jangan aktifkan jalur kredensial sensitif, dokumentasikan residual risk, dan mintakan keputusan owner.

## 5. Deliverables
- Kode: adapter Hermes, adapter 9Router, provider credential adapter, layanan orkestrasi, event bridge, health service, integrasi UI.
- `docs/runtime/`: `AI-RUNTIME-ARCHITECTURE.md`, `HERMES-INTEGRATION.md`, `9ROUTER-INTEGRATION.md`, `CREDENTIAL-SECURITY.md`, `AI-ORCHESTRATION.md`, `STREAMING-AND-EVENTS.md`, `RUNTIME-OPERATIONS.md`, `AI-RUNTIME-TESTING.md`, `RUNTIME-DISCOVERY.md`.
- `project-management/phases/PHASE-08-COMPLETION-REPORT.md`, `PHASE-08-SECURITY-VERIFICATION-REPORT.md`; pembaruan `PROJECT_STATUS.md`, `RISK_REGISTER.md`, ADR baru, `config/compatibility-manifest.json`.

## 6. Definition of Done
- [ ] Hermes dikelola Core (start/health/restart/stop) — `VERIFIED RUNTIME`.
- [ ] 9Router terintegrasi lewat adapter; health + model discovery berfungsi.
- [ ] Conversation Interface terhubung ke Hermes; **respons nyata** (kredensial uji) — atau `BLOCKED` dengan alasan.
- [ ] Streaming tampil di Desktop; Activity Stream menampilkan event aktual.
- [ ] Cancel berfungsi sesuai kemampuan runtime; recovery dari crash diuji.
- [ ] Semua tool lewat Hime Core; approval lifecycle tetap berlaku; fail-closed diuji.
- [ ] Kredensial diverifikasi (kanari); status IB-01 dan IB-02 jujur (RESOLVED atau OPEN dengan bukti).
- [ ] Metrik resource nyata tercatat.
- [ ] Semua test lama + baru lulus (jumlah direkonsiliasi); dokumentasi lengkap; Git dilaporkan.

**Status akhir yang diperbolehkan:** `READY FOR OWNER ACCEPTANCE` atau `BLOCKED - OWNER DECISION REQUIRED` (lalu lanjutkan fase berikutnya yang tidak terblokir; Fase 09 tidak bergantung pada IB-01/IB-02 selama tidak memakai kredensial sensitif).

## 7. Di luar cakupan
Banyak agent/specialist; storage/file intelligence; mobile; dokumen; browser; mengubah source upstream tanpa Gate #2; fitur produk fase berikutnya.
