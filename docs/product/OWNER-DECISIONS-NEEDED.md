# Owner Decisions Needed — Hime Project
## Register Keputusan Pemilik Proyek (Owner Decision Gates)

Dokumen ini memuat seluruh permohonan keputusan resmi yang berada di bawah otoritas eksklusif **Project Owner** (Pengguna).  
Sesuai aturan tata kelola, agen rekayasa **tidak boleh** mengambil keputusan sepihak atas gerbang-gerbang ini, namun **pekerjaan lain yang tidak terblokir tetap berjalan** tanpa menunggu dalam keadaan diam.

---

### OD-01 — Resolusi Celah Keamanan Localhost 9Router (IB-02)

- **Gate:** #2 (Memodifikasi source upstream Hermes atau 9Router)
- **Fase/Task Terdampak:** Phase 08 (Task 08.4), Phase 09
- **Mengapa Dibutuhkan:**  
  Audit empiris membuktikan bahwa file `src/dashboardGuard.js` pada 9Router membebaskan seluruh pemeriksaan kunci API untuk setiap koneksi yang berasal dari loopback lokal (`isLocalRequest`). Akibatnya, skrip atau aplikasi lain pada akun pengguna yang sama dapat memanggil endpoint `/v1/*` tanpa autentikasi.
- **Pilihan:**
  - **A (Direkomendasikan):** Otorisasi *controlled upstream patch* pada repositori terkelola 9Router Hime untuk menghapus baris `if (isLocalRequest(request)) return true;` dan mewajibkan token header internal `x-hime-auth`.
  - **B:** Menerapkan pembatasan firewall Windows (Windows Filtering Platform) untuk mengisolasi port 20128 hanya bagi PID resmi Hime Core.
  - **C:** Menerima risiko residual (*Formal Residual Risk Acceptance*) dengan catatan bahwa desktop adalah workstation pribadi single-user yang terlindungi login Windows.
- **Rekomendasi Saya:** **Pilihan A**  
  *Alasan:* Pilihan A menutup celah keamanan langsung pada akarnya di layer protokol HTTP tanpa memerlukan hak administrator untuk konfigurasi firewall tingkat rendah.
- **Risiko Bila Ditunda / Bila Disetujui:**  
  Bila ditunda, endpoint 9Router tetap terbuka bagi proses lokal lain (jalur kredensial sensitif tetap diblokir). Bila disetujui, integritas isolasi keamanan localhost tertutup 100%.
- **Pekerjaan yang Tetap Berjalan Sambil Menunggu:**  
  Seluruh orkestrasi Hime Core, streaming UI, persistensi percakapan, dan eksekusi Phase 09 tetap berjalan normal dengan mitigasi Hime Provider Adapter (:20129).
- **Cara Memberi Jawaban:**  
  Ketik pada chat: `OD-01: A` (atau pilihan lain).
- **Status:** **`OPEN`**

---

### OD-02 — Otorisasi Penggunaan Kunci API AI Cloud Komersial

- **Gate:** #1 (Memakai API key sungguhan atau berbayar)
- **Fase/Task Terdampak:** Phase 08 (Task 08.11), Phase 09
- **Mengapa Dibutuhkan:**  
  Pengujian kontrak transmisi, streaming Server-Sent Events (SSE), dan enkripsi DPAPI telah berhasil diuji 100% menggunakan loopback provider lokal. Pengujian end-to-end langsung ke server OpenAI/Anthropic/Nous di internet memerlukan kunci API berbayar.
- **Pilihan:**
  - **A:** Project Owner memasukkan kunci API uji bertarif rendah (misal saldo $1) ke dalam pengaturan Hime Desktop.
  - **B (Direkomendasikan Sementara):** Tetap menggunakan Loopback Provider Adapter lokal Hime Core untuk seluruh pengembangan otomatis sampai sistem siap untuk User Acceptance Testing (UAT).
- **Rekomendasi Saya:** **Pilihan B**  
  *Alasan:* Menjaga biaya nol rupiah selama pengembangan dan pengujian otomatis, serta mencegah kebocoran saldo tidak sengaja.
- **Risiko Bila Ditunda / Bila Disetujui:**  
  Tidak ada risiko teknis; pengujian fungsional dan automated test tetap lulus 100% menggunakan provider adapter mock terverifikasi.
- **Pekerjaan yang Tetap Berjalan Sambil Menunggu:**  
  Pengembangan Phase 09 (Intelligence Core), Phase 10 (Storage), dan seterusnya berjalan penuh.
- **Cara Memberi Jawaban:**  
  Ketik pada chat: `OD-02: B` (atau `OD-02: A` jika ingin menguji dengan API key asli).
- **Status:** **`OPEN`**
