//! Native drag-and-drop File Shelf (`FileShelf`) with JSON persistence at
//! `%APPDATA%\ai.hime.desktop\shelf.json`.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use windows::core::Result;

use crate::d2d::{NativeSurface, Rect, TextAlign};
use crate::theme::{
    ACCENT_CYAN, CORNER_RADIUS, GLASS_BORDER, GLASS_SURFACE, SAFE_PADDING, SURFACE_ELEVATED,
    TEXT_PRIMARY, TEXT_SECONDARY,
};

/// Coarse category for a stashed file or folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ShelfItemKind {
    #[default]
    File,
    Folder,
    Image,
    Audio,
    Video,
    Document,
    Archive,
    Other,
}

/// A single stashed file or directory in [`FileShelf`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShelfItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub kind: ShelfItemKind,
}

/// Drag-and-drop temporary file stash with JSON persistence.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileShelf {
    items: Vec<ShelfItem>,
}

impl FileShelf {
    /// Creates an empty `FileShelf`.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn items(&self) -> &[ShelfItem] {
        &self.items
    }

    /// Returns `%APPDATA%\ai.hime.desktop\shelf.json` on Windows.
    pub fn default_storage_path() -> Option<PathBuf> {
        std::env::var_os("APPDATA").map(|dir| {
            PathBuf::from(dir)
                .join("ai.hime.desktop")
                .join("shelf.json")
        })
    }

    /// Adds paths to the shelf, deduplicating by normalized path string.
    /// Returns the number of newly added items.
    pub fn add_paths<P: AsRef<Path>>(&mut self, paths: &[P]) -> usize {
        let mut added = 0;
        for p in paths {
            let path = p.as_ref();
            let path_str = path.to_string_lossy().to_string();
            if path_str.is_empty() || self.items.iter().any(|it| it.path == path_str) {
                continue;
            }

            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| path_str.clone());

            let meta = fs::metadata(path).ok();
            let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or_else(|| path.is_dir());
            let size_bytes = if is_dir {
                0
            } else {
                meta.map(|m| m.len()).unwrap_or(0)
            };
            let kind = if is_dir {
                ShelfItemKind::Folder
            } else {
                classify_extension(path)
            };

            self.items.push(ShelfItem {
                id: stable_item_id(&path_str),
                name,
                path: path_str,
                size_bytes,
                kind,
            });
            added += 1;
        }
        added
    }

    /// Removes an item by `id`. Returns `true` if found and removed.
    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|it| it.id != id);
        self.items.len() != before
    }

    /// Removes all items from the shelf.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Persists the shelf to `%APPDATA%\ai.hime.desktop\shelf.json`.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::default_storage_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "APPDATA is not set")
        })?;
        self.save_to(&path)
    }

    /// Persists the shelf as JSON to `path`, creating parent directories as needed.
    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.items)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }

    /// Loads the shelf from `%APPDATA%\ai.hime.desktop\shelf.json`, returning empty if absent.
    pub fn load() -> std::io::Result<Self> {
        let Some(path) = Self::default_storage_path() else {
            return Ok(Self::new());
        };
        Self::load_from(&path)
    }

    /// Loads the shelf from `path`, returning an empty `FileShelf` if the file does not exist.
    pub fn load_from(path: &Path) -> std::io::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let raw = fs::read_to_string(path)?;
        let items: Vec<ShelfItem> = serde_json::from_str(&raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Self { items })
    }

    /// Renders the file shelf onto `surface`.
    pub fn render(&self, surface: &NativeSurface) -> Result<()> {
        let (sw, sh) = surface.size();
        let rect = Rect::new(8.0, 8.0, (sw as f32 - 16.0).max(80.0), (sh as f32 - 16.0).max(80.0));

        surface.fill_rounded_rect(rect, CORNER_RADIUS, GLASS_SURFACE)?;
        surface.stroke_rounded_rect(rect, CORNER_RADIUS, GLASS_BORDER, 1.0)?;

        let header = Rect::new(
            rect.x + SAFE_PADDING,
            rect.y + 12.0,
            rect.width - SAFE_PADDING * 2.0,
            24.0,
        );
        surface.draw_text(
            &format!("File Shelf ({})", self.items.len()),
            header,
            14.0,
            TEXT_PRIMARY,
            TextAlign::Leading,
        )?;

        if self.items.is_empty() {
            let empty_rect = Rect::new(
                rect.x + SAFE_PADDING,
                rect.y + 48.0,
                rect.width - SAFE_PADDING * 2.0,
                (rect.height - 64.0).max(24.0),
            );
            surface.draw_text(
                "Drop files here to stash",
                empty_rect,
                12.5,
                TEXT_SECONDARY,
                TextAlign::Center,
            )?;
            return Ok(());
        }

        let mut y = rect.y + 44.0;
        let row_h = 34.0;
        for item in &self.items {
            if y + row_h > rect.y + rect.height - 12.0 {
                break;
            }
            let row = Rect::new(
                rect.x + SAFE_PADDING,
                y,
                rect.width - SAFE_PADDING * 2.0,
                row_h - 4.0,
            );
            surface.fill_rounded_rect(row, 8.0, SURFACE_ELEVATED)?;

            let badge_rect = Rect::new(row.x + 8.0, row.y + 5.0, 44.0, 20.0);
            surface.draw_text(
                kind_tag(item.kind),
                badge_rect,
                10.5,
                ACCENT_CYAN,
                TextAlign::Leading,
            )?;

            let name_rect = Rect::new(row.x + 56.0, row.y + 4.0, (row.width - 130.0).max(40.0), 22.0);
            surface.draw_text(
                &item.name,
                name_rect,
                12.0,
                TEXT_PRIMARY,
                TextAlign::Leading,
            )?;

            let size_rect = Rect::new(row.x + row.width - 72.0, row.y + 4.0, 64.0, 22.0);
            surface.draw_text(
                &format_size(item.size_bytes),
                size_rect,
                11.0,
                TEXT_SECONDARY,
                TextAlign::Trailing,
            )?;

            y += row_h;
        }

        Ok(())
    }
}

fn stable_item_id(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let digest = hasher.finalize();
    format!("{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}", digest[0], digest[1], digest[2], digest[3], digest[4], digest[5])
}

fn classify_extension(path: &Path) -> ShelfItemKind {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" => ShelfItemKind::Image,
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => ShelfItemKind::Audio,
        "mp4" | "mkv" | "webm" | "mov" | "avi" => ShelfItemKind::Video,
        "pdf" | "doc" | "docx" | "txt" | "md" | "xlsx" | "pptx" => ShelfItemKind::Document,
        "zip" | "7z" | "rar" | "tar" | "gz" => ShelfItemKind::Archive,
        "" => ShelfItemKind::Other,
        _ => ShelfItemKind::File,
    }
}

fn kind_tag(kind: ShelfItemKind) -> &'static str {
    match kind {
        ShelfItemKind::File => "FILE",
        ShelfItemKind::Folder => "DIR",
        ShelfItemKind::Image => "IMG",
        ShelfItemKind::Audio => "AUD",
        ShelfItemKind::Video => "VID",
        ShelfItemKind::Document => "DOC",
        ShelfItemKind::Archive => "ZIP",
        ShelfItemKind::Other => "OBJ",
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_remove_clear_and_deduplicate_paths() {
        let temp_dir = std::env::temp_dir().join("hime_shelf_unit_test");
        let _ = fs::create_dir_all(&temp_dir);
        let sample_file = temp_dir.join("notes.md");
        fs::write(&sample_file, b"hello hime").unwrap();

        let mut shelf = FileShelf::new();
        let added = shelf.add_paths(&[&sample_file, &sample_file]);
        assert_eq!(added, 1);
        assert_eq!(shelf.items().len(), 1);
        assert_eq!(shelf.items()[0].name, "notes.md");
        assert_eq!(shelf.items()[0].size_bytes, 10);
        assert_eq!(shelf.items()[0].kind, ShelfItemKind::Document);

        let id = shelf.items()[0].id.clone();
        assert!(shelf.remove(&id));
        assert!(shelf.items().is_empty());

        shelf.add_paths(&[&temp_dir]);
        assert_eq!(shelf.items()[0].kind, ShelfItemKind::Folder);
        shelf.clear();
        assert!(shelf.items().is_empty());

        let _ = fs::remove_file(&sample_file);
        let _ = fs::remove_dir(&temp_dir);
    }

    #[test]
    fn save_to_and_load_from_json_roundtrip() {
        let temp_dir = std::env::temp_dir().join("hime_shelf_json_test");
        let json_path = temp_dir.join("shelf.json");
        let _ = fs::remove_file(&json_path);

        let mut shelf = FileShelf::new();
        shelf.add_paths(&[PathBuf::from(r"C:\fake\image.png")]);
        shelf.save_to(&json_path).unwrap();

        let loaded = FileShelf::load_from(&json_path).unwrap();
        assert_eq!(loaded.items().len(), 1);
        assert_eq!(loaded.items()[0].name, "image.png");
        assert_eq!(loaded.items()[0].kind, ShelfItemKind::Image);

        let _ = fs::remove_file(&json_path);
        let _ = fs::remove_dir(&temp_dir);
    }

    #[test]
    fn default_storage_path_ends_with_expected_suffix() {
        if let Some(p) = FileShelf::default_storage_path() {
            assert!(p.ends_with(Path::new("ai.hime.desktop").join("shelf.json")));
        }
    }
}
