# Standar & Pedoman Pelaksanaan Reverse Engineering (RE)
## Hime Assistant Project

Direktori `RE/` berfungsi sebagai laboratorium riset teknis dan investigasi reverse engineering terpusat untuk proyek **Hime Assistant**.

---

## 1. Prinsip Utama Investigasi (REA Protocol)

Dalam setiap investigasi biner, kode sumber, atau sistem eksternal, seluruh kegiatan wajib mematuhi standar **REA (Reverse Engineering Anything)**:

1. **Evidence-Based (Berbasis Bukti Nyata):**  
   Setiap kesimpulan teknis harus didukung oleh bukti konkret: nomor baris, offset biner, pemanggilan API, nama simbol, atau hasil eksekusi log. Jangan mengarang arsitektur atau fitur yang tidak terverifikasi.
2. **Kategori Kepastian:**
   - **CONFIRMED**: Terbukti secara langsung melalui pembacaan kode sumber, disassembly, atau pengukuran live.
   - **HIGH CONFIDENCE**: Didukung oleh indikator jamak yang sangat kuat.
   - **POSSIBLE**: Hipotesis teknis rasional yang masih memerlukan pengujian lanjutan.
   - **UNKNOWN**: Belum dapat dipastikan karena keterbatasan data atau proteksi biner.
3. **Non-Destructive & Safe Execution:**  
   Tidak mengubah biner asli, tidak menjalankan instruksi yang berpotensi merusak sistem operasi host, dan tidak membocorkan kredensial atau rahasia sensitif.
4. **Clean-Room Engineering:**  
   Hasil reverse engineering digunakan murni untuk memahami protokol antarmuka, arsitektur, dan perilaku teknis agar implementasi Hime Assistant dapat dibangun secara independen, legal, dan aman dari pelanggaran hak cipta.

---

## 2. Struktur Laboratorium RE

```text
RE/
├── README.md                 (Standar & pedoman riset)
├── research-index.md         (Indeks target & status investigasi)
├── Nusa/                     (Investigasi desktop UI & window management)
│   ├── 00-source-materials/
│   ├── 01-initial-inspection/
│   ├── 02-binary-analysis/
│   ├── 03-application-architecture/
│   ├── 04-ui-and-interaction-analysis/
│   ├── 05-dependencies-and-frameworks/
│   ├── 06-modification-feasibility/
│   ├── findings/
│   └── reports/
├── Hermes/                   (Investigasi NousResearch/hermes-agent)
│   ├── 00-source-materials/
│   ├── 01-repository-analysis/
│   ├── 02-codebase-architecture/
│   ├── 03-agent-lifecycle/
│   ├── 04-agent-core/
│   ├── 05-tool-system/
│   ├── 06-memory-and-context/
│   ├── 07-session-and-conversation/
│   ├── 08-model-and-provider-system/
│   ├── 09-api-and-integration/
│   ├── 10-execution-and-runtime/
│   ├── 11-security-and-permissions/
│   ├── 12-extension-and-customization/
│   ├── findings/
│   └── reports/
└── Shared/                   (Diagram arsitektur & referensi lintas target)
    ├── architecture/
    ├── diagrams/
    ├── references/
    └── reports/
```

