use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtcraftDiscoveryResult {
    pub is_installed: bool,
    pub cli_available: bool,
    pub detected_path: Option<String>,
    pub supported_formats: Vec<String>,
    pub integration_mode: String,
    pub limitations: Vec<String>,
}

pub struct ArtcraftAdapter;

impl ArtcraftAdapter {
    /// Discovers local availability of "Artcraft" tools (Wordcraft, Figcraft) per UR-06 / B4.
    pub fn discover() -> ArtcraftDiscoveryResult {
        // Probe common install locations or PATH on Windows
        let probe_locations = [
            r"C:\Program Files\ArtCraft\artcraft.exe",
            r"C:\Program Files (x86)\ArtCraft\artcraft.exe",
            r"C:\Program Files\WordCraft\wordcraft.exe",
            r"C:\Program Files\FigCraft\figcraft.exe",
        ];

        let mut detected_path = None;
        for loc in &probe_locations {
            if Path::new(loc).exists() {
                detected_path = Some(loc.to_string());
                break;
            }
        }

        let is_installed = detected_path.is_some();
        let supported_formats = vec![
            "docx".to_string(),
            "pptx".to_string(),
            "pdf".to_string(),
            "svg".to_string(),
        ];

        let limitations = vec![
            "Direct GUI blind automation is strictly prohibited by Hime safety policy".to_string(),
            "Communication proceeds via standard open interchange formats (DOCX/PPTX/PDF/SVG)".to_string(),
        ];

        ArtcraftDiscoveryResult {
            is_installed,
            cli_available: false, // Discovered via standard document exchange
            detected_path,
            supported_formats,
            integration_mode: if is_installed { "direct_file_exchange".to_string() } else { "standard_format_fallback".to_string() },
            limitations,
        }
    }
}

