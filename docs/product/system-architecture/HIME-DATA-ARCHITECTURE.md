# Hime Data Architecture Specification

- **Document ID:** HIME-DATA-ARCHITECTURE
- **Task ID:** TASK-02-05 (System Architecture & Technical Design)
- **Phase:** Phase 02 — Nusa Reverse Engineering & Architecture Discovery
- **Authority:** Principal Data Architect & Senior Database Reliability Engineer
- **Status:** **APPROVED ARCHITECTURAL SPECIFICATION — PENDING OWNER REVIEW**
- **Date:** 2026-11-08

---

## 1. Executive Summary & Core Data Strategy

Dokumen ini mendefinisikan arsitektur data (*Data Architecture*) untuk sistem **Hime**, mencakup desain basis data primer, batas kepemilikan skema data domain (*schema ownership*), konkurensi transaksi, strategi migrasi skema, indeks pencarian teks penuh (FTS5), dan mekanisme persistensi rahasia.

### Prinsip Fondasi Arsitektur Data:
1. **Local-First & Data Sovereignty:** Seluruh data operasional pengguna disimpan secara lokal di mesin pengguna. Tidak ada data pribadi atau riwayat obrolan yang dikirimkan ke cloud pihak ketiga tanpa otorisasi eksplisit.
2. **SQLite sebagai Mesin Basis Data Utama MVP:** Sesuai evaluasi pada ADR-004, SQLite 3.45+ dengan mode Write-Ahead Logging (WAL) ditetapkan sebagai mesin basis data primer Hime Core karena efisiensi memori yang luar biasa ($\le 15\text{ MB}$ footprint), keandalan ACID, dan ketiadaan overhead pengelolaan service background terpisah.
3. **Pemisahan Kepemilikan Domain yang Ketat (*Strict Schema Boundaries*):** Hime Core dilarang memanipulasi basis data internal 9Router (`data.sqlite`) atau basis data internal Hermes Agent secara langsung. Setiap komponen memiliki domain persistensi terisolasi.
4. **Pemisahan Data Terbuka vs Rahasia Terenkripsi:** Konfigurasi umum disimpan dalam tabel biasa; seluruh kunci API, token OAuth, dan password wajib disimpan dalam bentuk *ciphertext blob* terenkripsi OS (Windows DPAPI / Linux Secret Service).

---

## 2. Batas Kepemilikan Data Lintas-Komponen (Data Ownership Boundaries)

Sistem membagi ekosistem persistensi ke dalam empat domain kepemilikan data yang terisolasi secara fisik:

```mermaid
graph TD
    subgraph DOMAIN_HIME_CORE ["1. HIME CORE DATA DOMAIN (%USERPROFILE%\\.hime\\data\\hime.db)"]
        HIME_CONV["conversations & messages"]
        HIME_FTS["context_fts (FTS5 Virtual Table)"]
        HIME_TASKS["user_tasks & reminders"]
        HIME_JOURNAL["storage_journal (Safe Move & Undo Log)"]
        HIME_AUDIT["tool_audit_log (Security Trail)"]
        HIME_SECRETS["secrets_vault (DPAPI / libsecret Encrypted Blobs)"]
    end

    subgraph DOMAIN_9ROUTER ["2. 9ROUTER DATA DOMAIN (%APPDATA%\\9router\\db\\data.sqlite)"]
        ROUTER_PROV["providerConnections (Plaintext JSON - CWE-312)"]
        ROUTER_CONF["configurations & model_routes"]
        ROUTER_SESS["sessions & access_keys"]
    end

    subgraph DOMAIN_HERMES ["3. HERMES DATA DOMAIN (.hermes_cache/)"]
        HERMES_MEM["agent_scratchpad & reasoning_checkpoints"]
        HERMES_VENV["python_virtualenv_packages (uv managed)"]
    end

    subgraph DOMAIN_USER ["4. USER FILESYSTEM DOMAIN (Host OS Volumes)"]
        USER_DOWNLOADS["Folder Downloads (%USERPROFILE%\\Downloads)"]
        USER_WORKSPACE["Folder Proyek Kerja Pengguna"]
        USER_DOCS["Dokumen PDF, Markdown, & Kode Sumber"]
    end

    %% Batas Kepemilikan
    DOMAIN_HIME_CORE -.->|DILARANG Mengubah Langsung / Hanya via Wrapper| DOMAIN_9ROUTER
    DOMAIN_HIME_CORE <-->|IPC Task Messages| DOMAIN_HERMES
    DOMAIN_HIME_CORE <-->|Safe Move & File Watcher (Zero Source Loss)| DOMAIN_USER
```

### Tabel Rincian Batas Kepemilikan:
| Domain Data | Lokasi Fisik di Disk | Mesin Basis Data | Pemilik Mutlak (*Sole Owner*) | Aturan Akses Lintas-Domain |
|:---|:---|:---|:---:|:---|
| **Hime Core Data** | `%USERPROFILE%\.hime\data\hime.db` (Win) / `~/.hime/data/hime.db` (Linux) | SQLite 3.45+ (WAL Mode) | Hime Core Daemon | Hanya diakses oleh Hime Core. Desktop UI mengakses via Tauri IPC. |
| **9Router Data** | `%APPDATA%\9router\db\data.sqlite` (Win) / `~/.9router/db/data.sqlite` (Linux) | SQLite (Bawaan 9Router) | 9Router Engine | **DILARANG DIMODIFIKASI LANGSUNG OLEH HIME CORE**. Konfigurasi dialirkan via HTTP API 9Router. |
| **Hermes Memory** | `%USERPROFILE%\.hime\hermes_state\` | Internal Python Cache | Hermes Agent Runtime | Dikelola mandiri oleh Hermes Agent reasoning loop. |
| **User Files** | Partisi NTFS lokal / ext4 mount points | File Asli Pengguna | Pengguna Manusia | Hanya diakses melalui `StorageService` dengan verifikasi SHA-256 dan izin PEP. |

---

## 3. Skema Basis Data Hime Core (`hime.db`)

Basis data utama Hime Core memuat tabel-tabel terstruktur berikut:

### A. Tabel Manajemen Percakapan & Riwayat Pesan
```sql
CREATE TABLE conversations (
    conversation_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    created_at_utc INTEGER NOT NULL,
    updated_at_utc INTEGER NOT NULL,
    active_workspace_path TEXT,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    total_token_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE messages (
    message_id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    sender_type TEXT NOT NULL,         -- 'USER', 'ASSISTANT', 'SYSTEM'
    content TEXT NOT NULL,
    created_at_utc INTEGER NOT NULL,
    token_count INTEGER,
    referenced_files_json TEXT,        -- Array of file metadata paths
    FOREIGN KEY (conversation_id) REFERENCES conversations(conversation_id) ON DELETE CASCADE
);
CREATE INDEX idx_messages_conv ON messages(conversation_id, created_at_utc ASC);
```

### B. Virtual Table Pencarian Teks Penuh (SQLite FTS5)
```sql
-- Virtual Table FTS5 untuk pencarian konteks instan <= 15 ms
CREATE VIRTUAL TABLE context_fts USING fts5(
    document_id UNINDEXED,             -- message_id atau file_path
    source_type UNINDEXED,             -- 'CHAT_MESSAGE', 'LOCAL_DOC', 'TASK'
    title,
    body_text,
    tokenize = 'porter unicode61 remove_diacritics 2'
);
```

### C. Tabel Jurnal Operasi Berkas & Pembatalan 30 Hari (`storage_journal`)
```sql
CREATE TABLE storage_journal (
    operation_id TEXT PRIMARY KEY,
    correlation_id TEXT NOT NULL,
    source_path TEXT NOT NULL,
    destination_path TEXT NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    sha256_hash TEXT NOT NULL,
    operation_type TEXT NOT NULL,      -- 'SAME_VOLUME_RENAME', 'CROSS_VOLUME_MOVE'
    is_cross_volume INTEGER NOT NULL,  -- 0 = False, 1 = True
    last_completed_step TEXT NOT NULL, -- 'PENDING', 'COPIED_TEMP', 'VERIFIED_HASH', 'COMMITTED', 'COMPLETED'
    created_at_utc INTEGER NOT NULL,
    expires_at_utc INTEGER NOT NULL,   -- Default created_at + 30 Hari (2.592.000 detik)
    status TEXT NOT NULL,              -- 'SUCCESS', 'UNDONE', 'EXPIRED', 'CORRUPT_BLOCKED'
    undone_at_utc INTEGER
);
CREATE INDEX idx_journal_status_expiry ON storage_journal(status, expires_at_utc);
CREATE INDEX idx_journal_dest_hash ON storage_journal(destination_path, sha256_hash);
```

### D. Tabel Indeks Pemantau Berkas (`file_index`)
```sql
CREATE TABLE file_index (
    file_path TEXT PRIMARY KEY,
    file_name TEXT NOT NULL,
    file_extension TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    sha256_hash TEXT NOT NULL,
    mtime_utc INTEGER NOT NULL,
    indexed_at_utc INTEGER NOT NULL,
    mime_type TEXT,
    category TEXT                      -- 'DOCUMENT', 'IMAGE', 'CODE', 'ARCHIVE', 'MEDIA'
);
CREATE INDEX idx_file_hash_size ON file_index(sha256_hash, size_bytes);
CREATE INDEX idx_file_category ON file_index(category);
```

### E. Tabel Tugas Pengguna (`user_tasks`)
```sql
CREATE TABLE user_tasks (
    task_id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    is_completed INTEGER NOT NULL DEFAULT 0,
    due_date_utc INTEGER,
    priority TEXT NOT NULL DEFAULT 'MEDIUM', -- 'LOW', 'MEDIUM', 'HIGH'
    created_at_utc INTEGER NOT NULL,
    completed_at_utc INTEGER
);
CREATE INDEX idx_tasks_pending ON user_tasks(is_completed, due_date_utc ASC);
```

### F. Tabel Brankas Kredensial Terenkripsi (`secrets_vault`)
```sql
CREATE TABLE secrets_vault (
    secret_key TEXT PRIMARY KEY,       -- Identifier, misal: 'provider:openai:api_key'
    encrypted_blob BLOB NOT NULL,      -- Ciphertext hasil Windows DPAPI / Linux libsecret
    key_derivation_meta TEXT,          -- Metadata enkripsi (Scope: CurrentUser, Alg: DPAPI)
    created_at_utc INTEGER NOT NULL,
    updated_at_utc INTEGER NOT NULL
);
```

### G. Tabel Jejak Audit Keamanan (`tool_audit_log`)
```sql
CREATE TABLE tool_audit_log (
    audit_id TEXT PRIMARY KEY,
    correlation_id TEXT NOT NULL,
    timestamp_utc INTEGER NOT NULL,
    tool_name TEXT NOT NULL,
    permission_type TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    command_payload TEXT NOT NULL,
    target_path TEXT,
    authorization_decision TEXT NOT NULL,
    execution_duration_ms INTEGER,
    exit_code INTEGER,
    error_message TEXT
);
CREATE INDEX idx_audit_time ON tool_audit_log(timestamp_utc DESC);
CREATE INDEX idx_audit_corr ON tool_audit_log(correlation_id);
```

---

## 4. Konfigurasi Konkurensi & Pengaturan PRAGMA SQLite

Untuk memastikan performa optimal dan stabilitas ACID pada perangkat keras pengembang riil:

```sql
-- Pengaturan Konkurensi & Performa Wajib saat Membuka Koneksi
PRAGMA journal_mode = WAL;            -- Memungkinkan pembaca konkuren tanpa memblokir penulis
PRAGMA synchronous = NORMAL;          -- Menjamin integritas transaksi pada SSD tanpa penalti fsync berlebih
PRAGMA busy_timeout = 5000;           -- Menunggu 5 detik saat terjadi lock contention sebelum melempar busy error
PRAGMA foreign_keys = ON;             -- Penegakan integritas relasional relasi tabel
PRAGMA temp_store = MEMORY;           -- Menyimpan tabel sementara di RAM untuk kecepatan query
PRAGMA cache_size = -8000;            -- Alokasi cache memori ~8 MB (8.000 halaman 1 KB)
PRAGMA mmap_size = 67108864;          -- Menggunakan memory-mapped I/O hingga 64 MB untuk pembacaan cepat
```

---

## 5. Strategi Migrasi Skema Basis Data (Versioned Migration Strategy)

Hime Core menerapkan skema migrasi terurut maju (*forward-only numbered migrations*) yang dieksekusi secara otomatis saat aplikasi melakukan cold startup:

### A. Tabel Pelacak Versi Migrasi
```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    migration_name TEXT NOT NULL,
    applied_at_utc INTEGER NOT NULL,
    checksum TEXT NOT NULL
);
```

### B. Prosedur Eksekusi Migrasi:
1. Saat inisialisasi modul database, Hime Core memeriksa nilai maksimum `version` dari tabel `schema_migrations`.
2. Hime Core membaca berkas migrasi tersemat (*embedded Rust SQL assets* via `include_str!`):
   - `001_initial_schema.sql`
   - `002_fts5_indexing.sql`
   - `003_storage_journal.sql`
   - `004_secrets_vault.sql`
3. Setiap skrip migrasi dibungkus di dalam satu transaksi terisolasi (`BEGIN IMMEDIATE TRANSACTION; ... COMMIT;`).
4. Jika terjadi kegagalan di tengah jalan, transaksi di-rollback secara otomatis dan startup Hime Core dihentikan dengan pesan error diagnostik yang jelas (*fail-safe startup*).

---

## 6. Strategi Pencadangan, Pemulihan & Integritas Data

### A. Prosedur Pencadangan Rutin (*Automated Vacuum & Backup*)
- **Online Backup API:** Memanfaatkan fungsi SQLite native `sqlite3_backup_init()` untuk membuat salinan berkas `.hime\data\hime_backup.db` secara transparan tanpa mengunci aktivitas baca/tulis aplikasi.
- **Jadwal Pemangkasan (*Pruning Schedule*):** Jurnal operasi pemindahan berkas `storage_journal` yang berumur $> 30\text{ hari}$ dipangkas secara otomatis seminggu sekali untuk menjaga ukuran database tetap berada di bawah 20 MB.

### B. Pemulihan Kerusakan Pasca-Crash (*Post-Crash Recovery*)
1. Jika proses Hime Core mati mendadak saat transaksi berlangsung, SQLite secara otomatis membaca berkas `hime.db-wal` saat startup berikutnya dan melakukan rekonsiliasi checkpoint ke berkas utama `hime.db`.
2. `StorageEngine` memindai tabel `storage_journal` untuk operasi yang berstatus selain `COMPLETED` atau `SUCCESS`:
   - Jika `last_completed_step` = `'COPIED_TEMP'`, berkas sisa `.hime_tmp` dihapus dari volume tujuan, dan entri ditandai `ABORTED_CLEANED`. Berkas sumber dipastikan 100% aman (*Zero Source Loss*).
   - Jika `last_completed_step` = `'COMMITTED'`, sistem memverifikasi hash destinasi. Jika sah, sistem menyelesaikan penghapusan berkas asal yang tertunda.
