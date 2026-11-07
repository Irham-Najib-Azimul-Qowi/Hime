# Hime License Register & Third-Party Provenance

- **Dokumen:** Register Lisensi Pihak Ketiga & Matriks Kepatuhan Distribusi Hime
- **Fase:** Phase 01 — Discovery & Environment Audit
- **Task ID:** TASK-01-03
- **Terakhir Diperbarui:** 2026-11-07
- **Otoritas Pemelihara:** Senior Infrastructure Architect & Compliance Specialist

---

## 1. Ikhtisar Register Lisensi & Kategori Kepatuhan

Register ini mencatat seluruh dependensi pihak ketiga, pustaka runtime, framework terencana, dan aset referensi dalam ekosistem Hime Assistant. Setiap entitas diklasifikasikan berdasarkan kategori lisensi resmi untuk memastikan kepatuhan penuh terhadap model distribusi desktop Windows.

### Kategori Lisensi (*License Categories*):
- **PERMISSIVE (MIT, Apache-2.0, BSD-2/3, ISC, PSF, HPND):** Sangat aman untuk didistribusikan bersama biner Hime. Kewajiban utama: mempertahankan pemberitahuan hak cipta (*copyright notice*) dan teks lisensi dalam dokumen atribusi (`THIRD_PARTY_LICENSES.md`).
- **WEAK COPYLEFT (MPL-2.0, LGPL-3.0):** Aman digunakan sebagai library eksternal atau modul terpisah (DLL/wheel). Jika file library tersebut dimodifikasi, perubahan pada file tersebut wajib dipublikasikan. Tidak menular (*non-infectious*) ke kode sumber Hime.
- **STRONG COPYLEFT (GPL, AGPL):** **DILARANG** untuk diintegrasikan secara statis ke dalam kode inti Hime. Tidak ditemukan dependensi runtime berlisensi GPL/AGPL pada komponen inti Hime.
- **PROPRIETARY / CLEAN-ROOM REFERENCE:** Hanya digunakan sebagai acuan perilaku dan desain; biner dan kode sumber dilarang disertakan dalam instalasi Hime.

---

## 2. Dependensi Langsung Hermes Agent (Python Ecosystem)

*Sumber Verifikasi: `RE/Hermes/00-source-materials/hermes-agent/pyproject.toml` & `uv.lock` (Python >=3.11,<3.15)*

| Package | Versi Terkunci | Tipe Dependensi | Lisensi | Bukti Lisensi | Repository / Sumber | Kewajiban Distribusi | Risiko | Status |
|:---|:---:|:---:|:---:|:---|:---|:---|:---:|:---:|
| **hermes-agent** (Core) | `c5f347eb` | Root Component | **MIT** | `LICENSE` | `NousResearch/hermes-agent` | Sertakan notice Nous Research | LOW | `[VERIFIED]` |
| `openai` | `2.54.0` | Direct Runtime | **Apache-2.0** | PyPI Metadata | PyPI | Sertakan notice Apache-2.0 | LOW | `[VERIFIED]` |
| `certifi` | `2026.10.19` | Direct Runtime | **MPL-2.0** | PyPI Metadata | PyPI / certifi.io | Sertakan lisensi MPL-2.0 jika dimodifikasi | LOW | `[VERIFIED]` |
| `truststore` | `0.10.5` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / sethmlarson | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `python-dotenv` | `1.2.4` | Direct Runtime | **BSD-3-Clause** | PyPI Metadata | PyPI / theskumar | Sertakan notice BSD-3 | LOW | `[VERIFIED]` |
| `fire` | `0.7.1` | Direct Runtime | **Apache-2.0** | PyPI Metadata | PyPI / Google | Sertakan notice Apache-2.0 | LOW | `[VERIFIED]` |
| `httpx` (incl. `socks`) | `0.28.1` | Direct Runtime | **BSD-3-Clause** | PyPI Metadata | PyPI / encode | Sertakan notice BSD-3 | LOW | `[VERIFIED]` |
| `rich` | `14.3.4` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / Textualize | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `tenacity` | `9.2.1` | Direct Runtime | **Apache-2.0** | PyPI Metadata | PyPI / jd | Sertakan notice Apache-2.0 | LOW | `[VERIFIED]` |
| `tomli-w` | `1.2.0` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / hukkin | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `ruamel.yaml` | `0.18.16` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / ruamel | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `ruamel.yaml.clib` | `0.2.15` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / ruamel | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `requests` | `2.34.2` | Direct Runtime | **Apache-2.0** | PyPI Metadata | PyPI / psf | Sertakan notice Apache-2.0 | LOW | `[VERIFIED]` |
| `jinja2` | `3.1.6` | Direct Runtime | **BSD-3-Clause** | PyPI Metadata | PyPI / pallets | Sertakan notice BSD-3 | LOW | `[VERIFIED]` |
| `firecrawl-anydoc` | `0.2.4` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / mendableai | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `pydantic` | `2.14.1` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / pydantic | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `prompt_toolkit` | `3.0.53` | Direct Runtime | **BSD-3-Clause** | PyPI Metadata | PyPI / jonathanslenders | Sertakan notice BSD-3 | LOW | `[VERIFIED]` |
| `croniter` | `6.2.4` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / kiorky | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `snowballstemmer` | `3.1.1` | Direct Runtime | **BSD-3-Clause** | PyPI Metadata | PyPI / snowball | Sertakan notice BSD-3 | LOW | `[VERIFIED]` |
| `packaging` | `26.3` | Direct Runtime | **Apache-2.0 / BSD** | PyPI Metadata | PyPI / pypa | Dual license; sertakan notice | LOW | `[VERIFIED]` |
| `Markdown` | `3.11.2` | Direct Runtime | **BSD-3-Clause** | PyPI Metadata | PyPI / Python-Markdown | Sertakan notice BSD-3 | LOW | `[VERIFIED]` |
| `PyJWT` | `2.15.1` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / jpadilla | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `urllib3` | `2.8.0` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / urllib3 | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `cryptography` | `50.0.2` | Direct Runtime | **Apache-2.0 / BSD** | PyPI Metadata | PyPI / pyca | Dual license; sertakan notice | LOW | `[VERIFIED]` |
| `tzdata` | `2025.3` | Direct Runtime (Win) | **Apache-2.0** | PyPI Metadata | PyPI / pypa | Sertakan notice Apache-2.0 | LOW | `[VERIFIED]` |
| `psutil` | `7.2.2` | Direct Runtime | **BSD-3-Clause** | PyPI Metadata | PyPI / giampaolo | Sertakan notice BSD-3 | LOW | `[VERIFIED]` |
| `websockets` | `15.0.1` | Direct Runtime | **BSD-3-Clause** | PyPI Metadata | PyPI / aaugustin | Sertakan notice BSD-3 | LOW | `[VERIFIED]` |
| `browser-harness` | `0.1.13` | Direct Runtime | **MIT** | PyPI Metadata | PyPI | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `pathspec` | `1.1.1` | Direct Runtime | **MPL-2.0** | PyPI Metadata | PyPI / cpburnz | Sertakan lisensi MPL-2.0 jika dimodifikasi | LOW | `[VERIFIED]` |
| `fastapi` | `0.104.0+` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / tiangolo | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `uvicorn` | `0.31.0+` | Direct Runtime | **BSD-3-Clause** | PyPI Metadata | PyPI / encode | Sertakan notice BSD-3 | LOW | `[VERIFIED]` |
| `httptools` | `0.6.3+` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / MagicStack | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `watchfiles` | `0.20+` | Direct Runtime | **MIT** | PyPI Metadata | PyPI / samuelcolvin | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `python-multipart` | `0.0.9+` | Direct Runtime | **Apache-2.0** | PyPI Metadata | PyPI / andrew-d | Sertakan notice Apache-2.0 | LOW | `[VERIFIED]` |
| `ptyprocess` | `0.7.0+` | Direct Runtime (Non-Win) | **ISC** | PyPI Metadata | PyPI / pexpect | Sertakan notice ISC | LOW | `[VERIFIED]` |
| `pywinpty` | `3.0.5+` | Direct Runtime (Win) | **MIT** | PyPI Metadata | PyPI / spyder-ide | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `pywin32` | `306+` | Direct Runtime (Win) | **PSF** | PyPI Metadata | PyPI / mhammond | Sertakan notice PSF | LOW | `[VERIFIED]` |
| `winrt-windows-*` (4 pkgs) | `3.2.1+` | Direct Runtime (Win) | **MIT** | PyPI Metadata | PyPI / Microsoft | Sertakan notice MIT Microsoft | LOW | `[VERIFIED]` |
| `Pillow` | `13.0.0` | Direct Runtime | **HPND (Permissive)** | PyPI Metadata | PyPI / python-pillow | Sertakan notice HPND | LOW | `[VERIFIED]` |
| `pillow-heif` | `1.4.0+` | Direct Runtime | **Apache-2.0 (libheif LGPL-3.0)** | PyPI Metadata | PyPI / bigcat88 | DLL terpisah memenuhi syarat relinking LGPL | MEDIUM | `[VERIFIED]` |
| `resvg-py` | `0.5.0` | Direct Runtime | **MPL-2.0** | PyPI Metadata | PyPI / RazrFalcon | Sertakan lisensi MPL-2.0 jika dimodifikasi | LOW | `[VERIFIED]` |
| `concurrent-log-handler` | `0.9.30` | Direct Runtime (Win) | **Apache-2.0** | PyPI Metadata | PyPI / Preston-Landers | Sertakan notice Apache-2.0 | LOW | `[VERIFIED]` |
| `nemo-relay` | `0.11+` | Direct Runtime | **Apache-2.0** | PyPI Metadata | PyPI | Sertakan notice Apache-2.0 | LOW | `[VERIFIED]` |

*Catatan: 356 locked/resolved packages di `uv.lock` (direct + transitive; transitif murni: `NOT SEPARATELY COUNTED`). Hash SHA-256 dan versi seluruh paket telah terkunci secara deterministik, namun teks lisensi individual dari paket transitif berstatus **`NOT FULLY AUDITED / UNVERIFIED`** hingga pemindaian otomatis berskala penuh dijalankan pada fase rilis.*

---

## 3. Dependensi Langsung 9Router Gateway (Node.js Ecosystem)

*Sumber Verifikasi: `RE/9ROUTER/00-source-materials/9router/package.json` & `cli/package.json`*

| Package | Versi Manifest | Tipe Dependensi | Lisensi | Bukti Lisensi | Repository / Sumber | Kewajiban Distribusi | Risiko | Status |
|:---|:---:|:---:|:---:|:---|:---|:---|:---:|:---:|
| **9router** (Core) | `v0.5.112` (`1fe586bb`) | Root Component | **MIT** | `LICENSE` | `decolua/9router` | Sertakan notice decolua | LOW | `[VERIFIED]` |
| `next` | `^16.4.2` | Runtime Prod | **MIT** | npm package.json | Vercel | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `react` | `19.2.4` | Runtime Prod | **MIT** | npm package.json | Meta | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `react-dom` | `19.2.4` | Runtime Prod | **MIT** | npm package.json | Meta | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `@next/third-parties` | `^16.2.9` | Runtime Prod | **MIT** | npm package.json | Vercel | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `@dnd-kit/*` (4 pkgs) | `^6.3.1 - ^10.0.0` | Runtime Prod | **MIT** | npm package.json | clauderic/dnd-kit | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `@xyflow/react` | `^12.10.1` | Runtime Prod | **MIT** | npm package.json | xyflow | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `monaco-editor` | `^0.57.0` | Runtime Prod | **MIT** | npm package.json | Microsoft | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `@monaco-editor/react` | `^4.7.0` | Runtime Prod | **MIT** | npm package.json | suryakaps | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `material-symbols` | `^0.44.6` | Runtime Prod | **Apache-2.0** | npm package.json | Google | Sertakan notice Apache-2.0 | LOW | `[VERIFIED]` |
| `recharts` | `^3.7.0` | Runtime Prod | **MIT** | npm package.json | recharts | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `undici` | `^7.30.1` | Runtime Prod | **MIT** | npm package.json | nodejs/undici | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `socks-proxy-agent` | `^8.0.5` | Runtime Prod | **MIT** | npm package.json | TooTallNate | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `bcryptjs` | `^3.0.3` | Runtime Prod | **MIT** | npm package.json | dcodeIO | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `jose` | `^6.1.3` | Runtime Prod | **MIT** | npm package.json | panva/jose | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `node-forge` | `^1.3.3` | Runtime Prod | **BSD-3-Clause / GPL-2.0** | npm package.json | digitalbazaar | Pilih BSD-3-Clause; sertakan notice | LOW | `[VERIFIED]` |
| `node-machine-id` | `^1.1.12` | Runtime Prod | **BSD-3-Clause** | npm package.json | automation-stack | Sertakan notice BSD-3 | LOW | `[VERIFIED]` |
| `@node-saml/node-saml` | `^5.1.0` | Runtime Prod | **MIT** | npm package.json | node-saml | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `zustand` | `^5.0.10` | Runtime Prod | **MIT** | npm package.json | pmndrs | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `sql.js` | `^1.14.1` | Runtime Prod (Fallback) | **MIT** | npm package.json | sql-js | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `uuid` | `^13.0.0` | Runtime Prod | **MIT** | npm package.json | uuidjs | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `marked` | `^18.0.1` | Runtime Prod | **MIT** | npm package.json | markedjs | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `chalk` | `^5.6.2` | Runtime Prod | **MIT** | npm package.json | chalk | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `confbox` | `^0.2.4` | Runtime Prod | **MIT** | npm package.json | unjs | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `enquirer` | `^2.4.1` | Runtime Prod | **MIT** | npm package.json | enquirer | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `open` | `^11.0.0` | Runtime Prod | **MIT** | npm package.json | sindresorhus | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `ora` | `^9.1.0` | Runtime Prod | **MIT** | npm package.json | sindresorhus | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `prop-types` | `^15.8.1` | Runtime Prod | **MIT** | npm package.json | facebook | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `@aws-sdk/credential-providers` | `^3.1135.0` | Optional Runtime | **Apache-2.0** | npm package.json | AWS | Sertakan notice Apache-2.0 | LOW | `[VERIFIED]` |
| `ajv` | `^8.17.1` | Optional Runtime | **MIT** | npm package.json | ajv-validator | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `better-sqlite3` | `^12.6.2` | Optional Native | **MIT** | npm package.json | JoshuaWise | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `ws` | `^8.18.0` | Optional Runtime | **MIT** | npm package.json | websockets | Sertakan notice MIT | LOW | `[VERIFIED]` |
| `playwright-core` | `^1.62.1` | Dev / Lazy Runtime | **Apache-2.0** | npm package.json | Microsoft | Diunduh dinamis jika fitur CDP aktif | MEDIUM | `[VERIFIED]` |
| `wreq-js` | `^3.2.0` | Dev / Lazy Runtime | **MIT / Custom** | `open-sse/services/` | wreq-js | Diunduh dinamis via `lazyRuntime.js` | MEDIUM | `[VERIFIED]` |

---

## 4. Pustaka Curated Agent Skills (`.agent/skills/`)

| Skill | Path Kanonikal | Format Berkas | Lisensi | Sumber Asal | Kewajiban Redistribusi | Risiko | Status |
|:---|:---|:---:|:---:|:---|:---|:---:|:---:|
| `animate` | `.agent/skills/animate/` | Pure Markdown | **MIT** | `emilkowalski/skills` | Pertahankan atribusi Emil Kowalski | LOW | `[VERIFIED]` |
| `emil-design-eng` | `.agent/skills/emil-design-eng/` | Pure Markdown | **MIT** | `emilkowalski/skills` | Pertahankan atribusi Emil Kowalski | LOW | `[VERIFIED]` |
| `ponytail` | `.agent/skills/ponytail/` | Pure Markdown | **MIT** | `DietrichGebert/ponytail` | Pertahankan atribusi Dietrich Gebert | LOW | `[VERIFIED]` |
| `rea` | `.agent/skills/rea/` | Pure Markdown (8 files) | **MIT** | `morluto/rea` | Pertahankan atribusi morluto | LOW | `[VERIFIED]` |
| `systematic-debugging` | `.agent/skills/systematic-debugging/` | Pure Markdown | **MIT** | `obra/superpowers` | Pertahankan atribusi Jesse Vincent | LOW | `[VERIFIED]` |
| `verification-before-completion` | `.agent/skills/verification-before-completion/` | Pure Markdown | **MIT** | `obra/superpowers` | Pertahankan atribusi Jesse Vincent | LOW | `[VERIFIED]` |
| `web-design-guidelines` | `.agent/skills/web-design-guidelines/` | Pure Markdown (2 files) | **MIT** | `vercel-labs/agent-skills` | Pertahankan atribusi Vercel Labs | LOW | `[VERIFIED]` |

---

## 5. Baseline Dependensi Terencana Hime Desktop & Hime Core (Phase 06+)

*Status: PLANNED (Belum diinstal; implementasi dimulai pada Phase 06–07)*

| Komponen / Crate Terencana | Target Ekosistem | Kategori Lisensi | Penggunaan yang Direncanakan | Kompatibilitas Distribusi |
|:---|:---:|:---:|:---|:---:|
| **Tauri 2.0 Shell (`tauri`, `tauri-build`)** | Rust / Cargo | **MIT OR Apache-2.0** | Shell aplikasi desktop native & DirectComposition window | Sangat Kompatibel (Permissive) |
| **`windows` / `windows-sys` (Crates.io)** | Rust / Cargo | **MIT OR Apache-2.0** | Win32 API bindings (Named Pipes, DPI, DWM blur, HWND management) | Sangat Kompatibel (Microsoft official) |
| **`tokio`** | Rust / Cargo | **MIT** | Async event runtime untuk supervision proses & IPC broker | Sangat Kompatibel |
| **`rusqlite`** | Rust / Cargo | **MIT** | SQLite embedding untuk cache lokal Hime (SQLite engine = Public Domain) | Sangat Kompatibel |
| **`serde` & `serde_json`** | Rust / Cargo | **MIT OR Apache-2.0** | Serialisasi konfigurasi & pesan envelope IPC JSON | Sangat Kompatibel |
| **`ring` / `getrandom`** | Rust / Cargo | **MIT / ISC / Apache-2.0** | CSPRNG password generation untuk child process 9Router | Sangat Kompatibel |
| **`tracing` / `tracing-subscriber`** | Rust / Cargo | **MIT** | Structured logging untuk diagnostic Hime Core | Sangat Kompatibel |
| **React 19 / TypeScript / Tailwind CSS** | Node / Web Frontend | **MIT / Apache-2.0** | UI rendering Dynamic Island di dalam WebView2 | Sangat Kompatibel |
| **Microsoft Edge WebView2** | Windows Native | **Microsoft Software License Terms** | Runtime browser engine (Evergreen OS Component di Windows 11) | Bawaan OS (Zero redistributable binary required) |

---

## 6. Batasan Penggunaan Aset Aplikasi Nusa (`NusaSetup.exe` v1.2.0.0)

| Aset Nusa | Jenis / Status | Lisensi | Batasan Hak Cipta & Ketentuan Penggunaan Hime |
|:---|:---|:---:|:---|
| **Biner `NusaSetup.exe` & `nusa.exe`** | Executable Biner | **Proprietary (Rama)** | **DILARANG DIDISTRIBUSIKAN.** Hanya digunakan di folder `RE/` untuk analisis reverse engineering tertutup. Tidak boleh dimasukkan ke dalam paket rilis Hime. |
| **Kode Sumber Dekompilasi / Disassembli** | Kode Mesin PE32+ | **Proprietary (Rama)** | **DILARANG MENYALIN KODE SUMBER.** Hime Desktop wajib dibangun secara *clean-room* menggunakan Rust dan API resmi Microsoft Windows. |
| **Aset Grafis, Ikon, Gambar BMP** | Visual Assets | **Proprietary (Rama)** | **DILARANG DIGUNAKAN.** Seluruh ikon, logo, dan visual Hime wajib dibuat orisinal. |
| **Nama Merek "Nusa"** | Identitas Brand | **Proprietary (Rama)** | **DILARANG DIGUNAKAN SEBAGAI NAMA PRODUK.** Produk ini dinamai "Hime Assistant". |
| **Ide Fungsional Dynamic Island Windows** | Konsep Interaksi UI | **Domain Publik / Ide Fungsional** | **DAPAT DIIMPLEMENTASIKAN SECARA MANDIRI.** Konsep jendela mengambang yang responsif terhadap audio/notifikasi bukan merupakan ekspresi yang dapat dipatenkan secara sepihak oleh aplikasi freeware; implementasi teknis Hime menggunakan DirectComposition independen. |
