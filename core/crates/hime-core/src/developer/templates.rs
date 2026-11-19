use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeTemplateMetadata {
    pub id: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub sections: Vec<String>,
}

pub struct SoftwareEngineeringTemplates;

impl SoftwareEngineeringTemplates {
    pub fn list_available() -> Vec<SeTemplateMetadata> {
        vec![
            SeTemplateMetadata {
                id: "srs-skpl".to_string(),
                title: "Spesifikasi Kebutuhan Perangkat Lunak (SKPL / SRS)".to_string(),
                category: "Requirements".to_string(),
                description: "Standar IEEE 830 / SKPL perkuliahan informatika (Fungsional, Non-Fungsional, Aktor, Use Case).".to_string(),
                sections: vec![
                    "1. Pendahuluan".to_string(),
                    "2. Deskripsi Umum Sistem".to_string(),
                    "3. Kebutuhan Fungsional & Non-Fungsional".to_string(),
                    "4. Pemodelan Use Case".to_string(),
                ],
            },
            SeTemplateMetadata {
                id: "sad-dppl".to_string(),
                title: "Deskripsi Perancangan Perangkat Lunak (DPPL / SAD)".to_string(),
                category: "Design".to_string(),
                description: "Arsitektur sistem, dekomposisi modul, skema database ERD, diagram kelas, dan rancangan API.".to_string(),
                sections: vec![
                    "1. Gambaran Umum Arsitektur".to_string(),
                    "2. Dekomposisi Komponen & Modul".to_string(),
                    "3. Skema Basis Data & ERD".to_string(),
                    "4. Spesifikasi Kontrak API".to_string(),
                    "5. Model Ancaman & Keamanan".to_string(),
                ],
            },
            SeTemplateMetadata {
                id: "test-plan".to_string(),
                title: "Rencana & Matriks Kasus Pengujian (Test Plan)".to_string(),
                category: "Testing".to_string(),
                description: "Rencana uji terstruktur, matriks skenario pengujian, dan bukti verifikasi hasil.".to_string(),
                sections: vec![
                    "1. Lingkup & Sasaran Uji".to_string(),
                    "2. Matriks Kasus Uji & Skenario".to_string(),
                    "3. Hasil & Bukti Eksekusi".to_string(),
                ],
            },
            SeTemplateMetadata {
                id: "security-review".to_string(),
                title: "Checklist Tinjauan Kode & Keamanan (Code Review)".to_string(),
                category: "Security".to_string(),
                description: "Checklist keamanan perangkat lunak berbasis OWASP Top 10 (Validasi input, kredensial, otorisasi).".to_string(),
                sections: vec![
                    "1. Validasi Input & Sanitasi".to_string(),
                    "2. Proteksi Rahasia & Kredensial".to_string(),
                    "3. Penanganan Error & Logging".to_string(),
                    "4. Audit Dependensi Pihak Ketiga".to_string(),
                ],
            },
        ]
    }

    pub fn render_template(template_id: &str, project_title: &str, author: &str) -> Option<String> {
        match template_id {
            "srs-skpl" => Some(format!(
                "# SPESIFIKASI KEBUTUHAN PERANGKAT LUNAK (SKPL)\n\n\
                 **Judul Proyek:** {}\n\
                 **Penyusun:** {}\n\
                 **Standar:** IEEE Std 830-1998 / Format Akademik Indonesia\n\n\
                 ---\n\n\
                 ## 1. PENDAHULUAN\n\
                 ### 1.1 Tujuan Dokumen\n\
                 Dokumen ini mendefinisikan seluruh kebutuhan fungsional dan non-fungsional untuk proyek {}.\n\n\
                 ### 1.2 Ruang Lingkup Produk\n\
                 Sistem ini berfungsi sebagai personal desktop environment mandiri dengan integrasi AI terkelola.\n\n\
                 ## 2. DESKRIPSI UMUM\n\
                 ### 2.1 Karakteristik Pengguna\n\
                 Pengguna utama adalah akademisi, pengembang perangkat lunak, dan profesional.\n\n\
                 ### 2.2 Batasan Desain & Implementasi\n\
                 - Menjamin kedaulatan data lokal (100% data tersimpan di mesin pengguna).\n\
                 - Tanpa eksekusi shell string mentah tanpa batas.\n\n\
                 ## 3. KEBUTUHAN SPESIFIK\n\
                 ### 3.1 Kebutuhan Fungsional\n\
                 - **SKPL-F-01:** Sistem wajib mendukung isolasi profil penelusuran web.\n\
                 - **SKPL-F-02:** Sistem wajib menyediakan antarmuka Kanban dan penjadwalan bebas tabrakan.\n\n\
                 ### 3.2 Kebutuhan Non-Fungsional\n\
                 - **SKPL-NF-01 (Keamanan):** Seluruh kredensial tersimpan dalam vault terenkripsi DPAPI.\n\
                 - **SKPL-NF-02 (Kinerja):** Alokasi memori runtime dibatasi maksimal 800 MB.\n",
                project_title, author, project_title
            )),
            "sad-dppl" => Some(format!(
                "# DESKRIPSI PERANCANGAN PERANGKAT LUNAK (DPPL)\n\n\
                 **Judul Proyek:** {}\n\
                 **Arsitek:** {}\n\n\
                 ---\n\n\
                 ## 1. GAMBARAN UMUM ARSITEKTUR\n\
                 Arsitektur mengadopsi pola Clean-Room Multi-Crate Workspace dengan Core sebagai Policy Enforcement Point.\n\n\
                 ## 2. DEKOMPOSISI KOMPONEN\n\
                 - **Hime Core:** Server kendali utama berbasis Rust.\n\
                 - **Hime Desktop:** Shell antarmuka berbasis Tauri 2.0 dan React.\n\
                 - **Hime Storage:** Mesin persistensi lokal berbasis SQLite dengan hash-chain audit.\n\n\
                 ## 3. MODEL KEAMANAN\n\
                 - Zero raw shell strings.\n\
                 - Sandbox quarantine untuk berkas unduhan.\n",
                project_title, author
            )),
            _ => None,
        }
    }
}

