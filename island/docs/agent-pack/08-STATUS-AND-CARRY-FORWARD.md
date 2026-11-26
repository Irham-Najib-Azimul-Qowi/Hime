# 08 — STATUS & CARRY-FORWARD

## 1. Status pekerjaan lama (menurut ekspor percakapan; periksa nyata)

| Fase lama | Hasil | Nasib pada pivot Bloom |
|---|---|---|
| 00 Governance | Accepted | Prinsip dan dokumen tetap; `PROJECT_STATUS.md`/`BACKLOG.md` ditulis ulang |
| 01 Discovery | Accepted | Baseline hardware, risiko, security baseline **dipakai** |
| 02–03 Nusa RE, PRD, arsitektur | Qualified | **Dipakai** (PRD, FR/NFR, ADR, constraints, referensi Nusa). Bagian "Desktop buatan sendiri" digantikan ADR-013 |
| 04 Engineering Planning | Qualified | **Dipakai** (IPC, supervisor, keamanan, filesystem, observability, QA) |
| 05 Dev Readiness | Qualified | Skrip diadaptasi |
| 06 Foundation | Qualified (33/33) | **Core dipakai**; shell Tauri lama **dibuang** |
| 07 Desktop Experience | Qualified (37/37) tapi **gagal visual** | **Logika Core dipakai** (approval lifecycle, task+audit); **seluruh UI dibuang** (referensi perilaku saja) |
| 08 AI Runtime | Prompt dikirim, tanpa laporan | **Dikerjakan ulang sebagai PHASE-08R** setelah Stage H0–H2. Periksa apakah ada kode backend Fase 08 di repo lama dan **panen** bila ada |

Jumlah test (33/37) hanya untuk bagian yang dipindahkan; hitung ulang di repo baru.

## 2. Kewajiban yang dibawa (diperbarui dari carry-forward lama)

| ID | Kewajiban | Status baru | Ditutup di |
|---|---|---|---|
| G1 | Baseline Git | **Selesai secara struktural**: repo baru = klon Bloom; commit baseline setelah panen | H1 |
| G2 | Verifikasi GUI interaktif | **Menjadi syarat mutlak setiap perubahan UI** (berkas 04 §6), bukan tugas akhir | H2 dan seterusnya |
| G3 | IB-01 (kredensial 9Router) | **Tetap OPEN** | 08R |
| G4 | IB-02 (localhost 9Router) | **Tetap OPEN** | 08R |
| G5 | Job Object: breakaway/nested | Tetap | 08R |
| G6 | Test vector IPC (fragmentasi, konkurensi, reconnect, frame cacat, replay) | Tetap; juga mencakup klien shell | 08R |
| G7 | Kebijakan lint | **Digantikan**: shell → gerbang Bloom; Core → legacy 04 | H1 |
| G8 | Traceability FR→test | Tetap (Core) | 09R+ |
| G9 | Kebijakan Recycle/pemulihan | Tetap | 10R |
| G10 | Linux di PAL | **Dibatalkan untuk v1.0** (D8). PAL tetap menjaga portabilitas Core | — |
| G11 | Lisensi/aset | **Diperluas**: kepatuhan GPL-3.0 Bloom + tanpa aset Nusa | H0, 15R |
| G12 | Benchmark resource | **Baseline Bloom** (H0) lalu delta Hime | H0, tiap fase |
| G13 | `hime-pal`/`hime-storage` diperdalam | Tetap | 09R–10R |
| G14 | **Baru:** updater/announcement Bloom dinonaktifkan; audit jaringan keluar | | H0 |
| G15 | **Baru:** `restore-taskbar.ps1` + `run-safe.ps1` | | H0 |
| G16 | **Baru:** touchpoint registry + upstream tag/SHA tercatat | | H0–H2 |
| G17 | **Baru:** akseptansi visual owner pada milestone UI | | H2, 08R+, 15R |

## 3. Blocker implementasi yang masih berlaku
**IB-01** dan **IB-02** (lihat `legacy/06-SECURITY-SPECIFICATION.md` §5–6). Semua kriteria RESOLVED di sana tetap berlaku. Tanpa penyelesaian, jalur kredensial sensitif **tetap dinonaktifkan**.

## 4. Pelajaran dari kegagalan Fase 07 (wajib dibawa)
1. Build dan test hijau **bukan** bukti UI baik. Pengujian GUI/screenshot nyata wajib.
2. Jangan menafsirkan "Island" sebagai jendela dengan elemen di dalamnya; Island adalah bentuk yang bermorfosis.
3. Jangan membangun sistem desain sendiri; ikuti yang sudah ada dan matang.
4. Klaim sekunder ("laporan agent") bukan bukti; periksa langsung.
