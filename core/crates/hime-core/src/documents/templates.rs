use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTemplateSpec {
    pub name: String,
    pub description: String,
    pub font_family: String,
    pub font_size_pt: f32,
    pub line_spacing: f32,
    pub margin_left_cm: f32,
    pub margin_right_cm: f32,
    pub margin_top_cm: f32,
    pub margin_bottom_cm: f32,
    pub paper_size: String,
    pub requires_table_of_contents: bool,
}

pub struct TemplateRegistry;

impl TemplateRegistry {
    /// Academic-ID Template according to standard Indonesian university thesis/paper guidelines (UR-14 / B3):
    /// Times New Roman 12, line spacing 1.5, margin left 4cm, others 3cm, A4.
    pub fn academic_id() -> DocumentTemplateSpec {
        DocumentTemplateSpec {
            name: "Akademik-ID".to_string(),
            description: "Standar penulisan karya ilmiah/makalah/skripsi Indonesia (TNR 12, Spasi 1.5, Margin 4-3-3-3)".to_string(),
            font_family: "Times New Roman".to_string(),
            font_size_pt: 12.0,
            line_spacing: 1.5,
            margin_left_cm: 4.0,
            margin_right_cm: 3.0,
            margin_top_cm: 3.0,
            margin_bottom_cm: 3.0,
            paper_size: "A4".to_string(),
            requires_table_of_contents: true,
        }
    }

    /// Modern Technical & Business Report Template:
    /// Calibri/Aptos 11, line spacing 1.15, standard 2.5cm margins, A4.
    pub fn modern_report() -> DocumentTemplateSpec {
        DocumentTemplateSpec {
            name: "Modern-Report".to_string(),
            description: "Format laporan teknis dan profesional modern dengan tipografi bersih".to_string(),
            font_family: "Calibri".to_string(),
            font_size_pt: 11.0,
            line_spacing: 1.15,
            margin_left_cm: 2.54,
            margin_right_cm: 2.54,
            margin_top_cm: 2.54,
            margin_bottom_cm: 2.54,
            paper_size: "A4".to_string(),
            requires_table_of_contents: true,
        }
    }
}

