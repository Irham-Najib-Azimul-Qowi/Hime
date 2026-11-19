# PHASE 08R — AI RUNTIME (Hermes + 9Router) DI ATAS SHELL BLOOM

**Backend/keamanan:** kerjakan **seluruh lingkup** `legacy/roadmap/PHASE-08-AI-RUNTIME-INTEGRATION.md` (discovery runtime, supervisor Hermes/9Router, **IB-01**, **IB-02**, adapter Hermes dan 9Router, orkestrasi, streaming, observability, resource, pengujian 15 langkah E2E, DoD) dan `legacy/06-SECURITY-SPECIFICATION.md`. Semua tetap berlaku, termasuk aturan: Core = PEP; Hermes bukan otoritas; localhost bukan autentikasi; kredensial uji aman; tanpa mock yang dilaporkan sebagai nyata.
**Mulai lebih awal:** bagian backend boleh berjalan **paralel dengan menunggu VC-01** (Stage H2).
**Entry:** H2 selesai (VC-01 terkirim). Periksa apakah ada pekerjaan Fase 08 lama yang sudah dipanen.

## Bagian UI (menggantikan UI di legacy PHASE-08)

| Kebutuhan | Permukaan Bloom | Perilaku |
|---|---|---|
| Percakapan baru/lanjut, streaming, stop | **Panel Assistant** (Island) | Input satu baris; respons streaming ringkas; ekspansi tinggi dengan spring; tombol stop; markdown tersanitasi; **tanpa chain-of-thought**, hanya ringkasan aktivitas |
| Aktivitas agent (tool dipanggil, status) | **Bubble agent bekerja** → ringkasan di panel; detail di Workspace (09R) | Event `hime-agent-status`, `hime-tool-event` |
| Approval tool berisiko | **Bubble approval** (dari H2) | Terhubung ke Tool Gateway nyata |
| Status runtime, model, penyedia | Tab **AI** di Settings (ringkas) + (09R) AI Control di Workspace | Status Hermes/9Router/koneksi provider dari Core; pemilihan model nyata, bukan palsu |
| Input kredensial uji | Tab AI di Settings | Field → command → **vault Core**; dikosongkan; tidak ada di settings.json/localStorage/log/event; uji kanari |
| Runtime tidak tersedia | Pill + panel | State offline/"runtime mati" jujur dengan "coba lagi" |
| Riwayat percakapan | Daftar ringkas di panel; penuh di Workspace | Persisten di Core (bukan frontend) |

## Tambahan akseptansi
- Semua yang ada di DoD legacy PHASE-08 **plus**: E2E nyata (kredensial uji) **melalui UI Bloom**: prompt → streaming di Island → bubble aktivitas → approval nyata → hasil; cancel; matikan Hermes/9Router/Core secara paksa → shell tetap hidup, state jujur, recovery.
- Verifikasi visual + resource sesuai berkas 04 §6 (panel Assistant, bubble agent, state streaming/error/offline). **VC-02** ke owner (Assistant + bubble aktivitas/approval).
- Gerbang owner: kredensial nyata (Gate #1), modifikasi upstream untuk IB-02 (Gate #2).
- Status IB-01/IB-02 jujur; jika OPEN, jalur kredensial sensitif **dinonaktifkan di UI** dan dijelaskan.
