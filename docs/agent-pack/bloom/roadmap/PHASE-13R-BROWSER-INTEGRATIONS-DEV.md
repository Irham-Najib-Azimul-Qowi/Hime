# PHASE 13R — BROWSER, INTEGRATIONS & DEVELOPER WORKSPACE

**Backend:** seluruh lingkup `legacy/roadmap/PHASE-13-BROWSER-AND-DEVELOPER-WORKSPACE.md` (Browser Service dengan Playwright default + kebijakan SSRF/injection, adapter GitHub dan Google Drive dengan OAuth + vault, Developer Workspace terkendali dengan allowlist/Job Object, alur software engineering untuk kuliah, pengujian injeksi/bypass). **Gate #6** untuk akun nyata; bangun dengan akun/repo uji.

## UI
| Fitur | Permukaan |
|---|---|
| Status tugas browser/riset, approval isian sensitif/unduhan | **Bubble** (status + approval) |
| Hasil riset dengan sitasi | Respons di Assistant (ringkas) → detail di Workspace |
| Koneksi akun GitHub/Drive (alur OAuth) | Tab **Integrasi** di Settings (membuka browser sistem untuk consent; token ke vault Core, tidak terlihat shell) |
| Repo/isu/PR, status build/test | **Workspace → Developer** (daftar dan diff) + **bubble** build/test |
| Workspace proyek dan allowlist perintah | Workspace → Developer + Settings (Izin) |
| Dokumen SE (SRS, desain, UML, ERD, test plan, DPPL) | Dihasilkan via Document Service (12R) dengan standar kualitas dokumen; tampil di Workspace → Documents |

Verifikasi: korpus prompt-injection (browser + repo), uji SSRF, uji bypass allowlist, OAuth (kedaluwarsa/dicabut/scope berlebih), visual + resource; **VC-08** (Developer + Integrasi).
