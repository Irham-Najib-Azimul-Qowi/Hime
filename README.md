# Hime Assistant Workspace

**Hime Assistant** adalah proyek pengembangan aplikasi desktop Windows modern yang memadukan pengalaman antarmuka mengambang (*floating Dynamic Island*) yang terinspirasi dari Nusa Dynamic Island dengan kecerdasan eksekusi agen otonom tingkat lanjut dari **Hermes Agent** (Nous Research).

Melalui Hime Assistant, pengguna Windows dapat berinteraksi dengan AI Agent langsung dari desktop tanpa harus membuka terminal: mengirim instruksi teks/suara, memantau proses *reasoning* dan eksekusi alat (*tool calls*) secara real-time, menyetujui aksi berisiko, serta menerima hasil kerja agen.

---

## Struktur Direktori Workspace

```text
Hime/
│
├── README.md                 # Dokumentasi umum workspace Hime
├── PROJECT_STATUS.md         # Status dan roadmap pekerjaan proyek
│
├── RE/                       # Reverse Engineering Laboratory
│   ├── README.md             # Panduan dan standar riset REA
│   ├── research-index.md     # Indeks investigasi teknis
│   │
│   ├── Nusa/                 # Riset Antarmuka: Nusa Dynamic Island
│   │   ├── 00-source-materials/
│   │   ├── 01-initial-inspection/
│   │   ├── 02-binary-analysis/
│   │   ├── 03-application-architecture/
│   │   ├── 04-ui-and-interaction-analysis/
│   │   ├── 05-dependencies-and-frameworks/
│   │   ├── 06-modification-feasibility/
│   │   ├── findings/
│   │   └── reports/
│   │
│   ├── Hermes/               # Riset Otak Agen: NousResearch/hermes-agent
│   │   ├── 00-source-materials/
│   │   ├── 01-repository-analysis/ s/d 12-extension-and-customization/
│   │   ├── findings/
│   │   └── reports/
│   │
│   └── Shared/               # Diagram, arsitektur, dan referensi bersama
│       ├── architecture/
│       ├── diagrams/
│       ├── references/
│       └── reports/
│
├── Documentation/            # Dokumen arsitektur, RFC, dan spesifikasi produk
├── Projects/                 # Kode sumber implementasi Hime (akan datang)
├── Resources/                # Aset desain, mockups, ikon, dan resource grafis
└── Archive/                  # Arsip artefak lama yang sudah tidak aktif
```

---

## Komponen Kunci Riset

1. **Nusa Dynamic Island Investigation:**  
   Membedah sistem window management Win32, GPU rendering DirectComposition, dan interaksi kapsul mengambang di layar Windows.
2. **Hermes Agent Investigation:**  
   Membedah sistem reasoning loop, tool execution, memory/context window, serta antarmuka integrasi API/CLI Hermes Agent.
3. **Hime Integration Feasibility:**  
   Menghubungkan frontend floating island di desktop dengan backend Hermes Agent sebagai command center interaktif.
