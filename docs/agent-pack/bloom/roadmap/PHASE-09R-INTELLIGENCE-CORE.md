# PHASE 09R — INTELLIGENCE CORE, TRANSLATION, WORKSPACE WINDOW

**Backend:** seluruh lingkup `legacy/roadmap/PHASE-09-INTELLIGENCE-CORE.md` §09.1–09.7 (Policy engine, Tool Gateway, Task Engine, Event Engine, Context & Memory, Orchestrator + quality controls, Resource Manager). Tetap berlaku.
**Entry:** 08R `READY FOR OWNER ACCEPTANCE` (atau BLOCKED hanya pada kredensial nyata).

## UI (menggantikan legacy §09.8–09.9 yang bersifat UI)

### 9R.1 Window **Hime Workspace** (baru)
Ikuti aturan Bloom untuk menambah window (berkas 06 §4–5): `hime.html`, `src/hime/workspace/main.tsx`, `vite.config.ts`, `tauri.conf.json`, `capabilities/default.json`; **dibuat on-demand**, bergaya window Settings Bloom (bukan Island). Isi awal (navigasi sisi sederhana ala Settings): **Activity** (stream + riwayat audit/approval), **Percakapan**, **AI Control Center** (status Hermes/9Router/model/penggunaan/diagnostik), **Memori** (lihat/hapus item memori). Hanya menampilkan data dari Core.

### 9R.2 Task engine di UI
Panel **Tasks** (Island): hari ini, berikutnya, tambah cepat; **integrasi dengan Kalender+Pomodoro Bloom** (timer fokus Bloom dipakai ulang untuk sesi tugas, bukan dibuat ulang). Kanban penuh di Workspace (12R). Bubble progres task.

### 9R.3 Terjemahan teks terpilih (UR-16)
Backend: Core + PAL (UIA `TextPattern`, fallback clipboard dengan pemulihan) sesuai legacy §09.8. UI: **bubble/ekspansi Island** sesuai berkas 07 §5; pengaturan opt-in, hotkey, bahasa target, aplikasi dikecualikan di tab Hime Settings. Uji lintas aplikasi (≥3), teks panjang, offline.

### 9R.4 Memori dan konteks
UI untuk melihat/menghapus memori di Workspace; opt-in penyimpanan memori otomatis.

### 9R.5 Verifikasi GUI (G2) — **wajib di fase ini**
Smoke interaktif lengkap: semua mode panel Hime + bubble + Workspace window pada 100%/125% DPI, 1366×768, dan monitor kedua bila ada; mode visibilitas Fixed/Smart/Peek; multi-monitor. Screenshot + checklist Bloom-native. **VC-03** (Translation + Workspace window).

## Akseptansi tambahan
Semua exit gate legacy PHASE-09 (bypass Gateway tidak terbukti, retry tidak menggandakan efek, task pulih setelah restart, memori punya provenance) + resource: Workspace window dibuat/ditutup on-demand (RAM kembali ke baseline setelah ditutup, terukur).
