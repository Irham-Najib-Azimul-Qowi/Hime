# 07 — UX SPEC: HIME DI ATAS BLOOM

Semua fitur Hime dipetakan ke **permukaan yang sudah ada di Bloom**. Prinsip Bloom: *dock untuk aplikasi, notch untuk informasi ringkas dan kontrol, fallback native lebih baik daripada menduplikasi aplikasi utuh.* Spesifikasi ini memakai **kosakata Bloom**: pill, panel, live-activity bubble, peek, klik-untuk-buka, scroll/swipe pindah mode, spring.

> Nilai visual konkret (ukuran, radius, warna, spring) **diambil dari kode Bloom** (Stage H0), bukan dari dokumen ini. Dokumen ini menetapkan *apa* dan *di mana*, bukan angkanya.

## 1. Permukaan Bloom dan perannya bagi Hime

| Permukaan Bloom | Peran Hime | Contoh |
|---|---|---|
| **Pill (status strip Island)** | Indikator Hime ringkas: siap / bekerja / menunggu approval / offline | titik status kecil + teks pendek, mengikuti gaya strip Bloom |
| **Panel Island** (mode yang berganti dengan scroll/swipe) | Interaksi ringkas | **Assistant**, **Tasks**, **Storage ringkas**, **Translate**, **Nudge/Fokus** |
| **Live-activity bubble** (rounded-square di samping pill) | Hal yang berjalan/butuh perhatian | bubble **agent bekerja**, **approval**, **terjemahan**, **unduhan duplikat**, **timer fokus/nudge**, **transfer perangkat** |
| **Overlay window** | Momen modal singkat yang perlu menonjol | check-in strict saat laptop dibuka (opsional; lihat §6) |
| **Dock** | Aplikasi | ikon Hime Workspace (opsional); tidak ada konten AI di dock |
| **Settings window** | Konfigurasi | tab Hime: AI, Izin & Keamanan, Perangkat, Produktivitas, Storage, Integrasi, Privasi |
| **Window baru `hime` (Workspace)** | UI besar | Kanban, riwayat Activity/Audit, dashboard Storage, Documents, Devices, AI Control Center |

**Aturan:** informasi/aksi yang muat ringkas → Island/bubble. Hanya yang benar-benar butuh ruang → Workspace. Tidak ada UI besar di Island.

## 2. Pemetaan fitur → permukaan

| Fitur Hime (UR) | Island/Bubble | Workspace | Settings |
|---|---|---|---|
| **Assistant** chat/instruksi cepat | Panel Assistant: kotak input satu baris + respons streaming ringkas; ekspansi tinggi sesuai konten | Riwayat percakapan penuh, lampiran | Model/penyedia (via Core) |
| **Status agent** | Bubble "bekerja" (ikon + langkah singkat), klik → panel Activity ringkas | Activity stream lengkap | — |
| **Approval** aksi berisiko (UR-12, keamanan) | Bubble approval → ekspansi Island: apa yang akan dilakukan, pratinjau, tombol Setujui/Tolak, hitung mundur TTL | Riwayat approval | Kebijakan izin |
| **Terjemahan teks terpilih** (UR-16) | Bubble/ekspansi Island: teks asli, terjemahan, salin/tutup | — | Bahasa target, hotkey, opt-in |
| **Tasks/Kanban** (UR-09) | Panel Tasks ringkas (hari ini, berikutnya, tambah cepat) + integrasi **Pomodoro/Kalender Bloom** | Kanban penuh, proyek, kalender | Mode ketegasan |
| **Nudge & mode ketegasan** (UR-10, UR-11) | Bubble timer fokus/nudge; escalation bertahap | Statistik, riwayat | Santai/Biasa/Strict, ambang, kategori |
| **Unduhan duplikat/versi** (UR-03) | Bubble pertanyaan: "Sudah ada. Ini versi terbaru?" + pilihan ringkas | Antrean proposal file | Aturan penyimpanan |
| **Storage laptop/HP** (UR-01, 02, 04) | Panel Storage ringkas (ruang, saran satu-dua) | Dashboard Storage, duplikat, aturan, timeline operasi | Root yang diizinkan, retensi Recycle |
| **Perangkat HP** (UR-02, 11, 12) | Bubble transfer/pairing, status baterai HP | Devices: daftar, storage HP, transfer, revoke | Pairing, izin |
| **Dokumen** (UR-06, 07, 14, 15) | Bubble progres pembuatan/konversi, hasil → "Buka" | Documents: antrean, hasil, templat | Templat, tool |
| **Browser/GitHub/Drive** (UR-05, 08) | Bubble status tugas dan approval | Developer & Integrations | Akun/token (via Core vault), kebijakan domain |
| **Software engineering** (UR-13) | Bubble status build/test | Developer workspace, dokumen SE | Allowlist perintah |
| **AI Control Center** | — | Status Hermes/9Router, model, penggunaan, diagnostik | Model, mode resource |
| **Shelf Bloom** (drop file/teks) | Sudah ada; **integrasi:** "kirim ke Hime" untuk file/teks di shelf (mis. tanya, ringkas, arsipkan) | — | — |

## 3. Pola interaksi (ikuti Bloom)

- **Klik untuk membuka**; hover hanya *peek* pada strip status; **memori panel terakhir** mengikuti pengaturan Notch Bloom.
- **Scroll/swipe** berganti mode panel; Hime menambah mode ke daftar yang sama (urutan dan jumlah mode dibuat bisa diatur agar Island tidak penuh).
- **Bubble** mengikuti aturan prioritas/ukuran bubble Bloom; Hime tidak boleh menggusur musik/timer secara sembarangan — **prioritas bubble** didefinisikan (approval > nudge strict > agent bekerja > terjemahan > lain), dan dicatat.
- **Approval selalu menang** atas bubble lain dan tidak dapat tertutup tak sengaja (butuh aksi eksplisit atau kedaluwarsa TTL → ditolak otomatis dan dicatat).
- **Fokus:** Island tidak mencuri fokus keyboard dari aplikasi pengguna kecuali pengguna mengklik input Assistant atau memakai hotkey.
- **Keyboard:** hotkey global untuk membuka Assistant (konfigurabel; pastikan tidak bentrok hotkey Bloom/Windows); Esc menutup; Tab/Enter pada approval.
- **Offline:** bila Core mati → pill menampilkan status offline yang ringkas, panel Hime menjelaskan dan menawarkan "coba lagi"; Bloom lain tetap normal.

## 4. State wajib per komponen
idle, peek, terbuka, loading, streaming, kosong, error (tanpa bocor rahasia), offline, kedaluwarsa (approval), dibatalkan. Bahasa Indonesia, nada ringkas.

## 5. Terjemahan teks terpilih (UR-16) — spesifikasi UX
1. Pengguna memblok teks asing di aplikasi apa pun.
2. Pemicu: (a) hotkey konfigurabel (default aman), atau (b) otomatis bila opt-in dan UI Automation tersedia.
3. Island menampilkan bubble: bahasa terdeteksi → terjemahan; tombol salin; tutup otomatis setelah jeda atau Esc; tidak mencuri fokus.
4. Privasi: **opt-in**, indikator bahwa teks dikirim ke penyedia cloud, daftar aplikasi dikecualikan, tidak aktif pada kolom password.
5. Teknis (Core + PAL): UIA `TextPattern`, fallback clipboard dengan pemulihan clipboard (legacy `roadmap/PHASE-09` §09.8).

## 6. Mode ketegasan dan check-in (UR-10)
- **Santai/Biasa:** bubble nudge di Island, ringkasan harian di panel Tasks.
- **Strict:** saat laptop dibuka, Island otomatis membuka panel Tasks dengan tugas prioritas dan tombol "Mulai"; bila diabaikan, eskalasi bertahap (bubble lebih menonjol → overlay window singkat). **Overlay hanya sebagai eskalasi terakhir**, selalu punya "Tunda" dengan alasan singkat. Hime **menagih, tidak mengunci** (tidak mengunci layar, tidak mematikan aplikasi).
- Nada manusiawi dan mendukung; tidak menghakimi.

## 7. Paritas Nusa (referensi)
Daftar fitur Nusa (parity matrix dari Fase 02 di `docs/reference/nusa/`) yang berstatus Preserve/Improve harus **dipetakan** ke permukaan Bloom pada tabel `docs/hime/NUSA-PARITY-ON-BLOOM.md`: fitur → permukaan → status (sudah dimiliki Bloom / ditambahkan Hime / diganti / dibuang beralasan). Banyak fitur Nusa (Dynamic Island, kanban, pemutar musik, terjemahan, kontrol sistem) sudah dipenuhi Bloom atau ditambahkan Hime.

## 8. Yang sengaja tidak dilakukan
Dashboard AI di Island; chat panjang di Island (pindah ke Workspace); duplikasi aplikasi penuh di notch; tema/skin baru; plugin/CSS bebas; menambah mode Island tanpa batas; konten AI di dock.
