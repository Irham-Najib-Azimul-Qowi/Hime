use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

use hime_security::policy::RiskLevel;

/// Manifest schema for custom skills stored in `%APPDATA%\ai.hime.desktop\skills\<skill-id>\manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub id: String,
    #[serde(default, alias = "title")]
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, alias = "intents")]
    pub keywords: Vec<String>,
    #[serde(default = "default_risk_level", deserialize_with = "deserialize_risk_level")]
    pub risk_level: RiskLevel,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_risk_level() -> RiskLevel {
    RiskLevel::L1CreateSafe
}

fn default_enabled() -> bool {
    true
}

fn deserialize_risk_level<'de, D>(deserializer: D) -> Result<RiskLevel, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RiskInput {
        Enum(RiskLevel),
        Str(String),
        Num(u8),
    }

    match RiskInput::deserialize(deserializer)? {
        RiskInput::Enum(r) => Ok(r),
        RiskInput::Num(n) => Ok(match n {
            0 => RiskLevel::L0ReadOnly,
            1 => RiskLevel::L1CreateSafe,
            2 => RiskLevel::L2Modify,
            3 => RiskLevel::L3Destructive,
            _ => RiskLevel::L4System,
        }),
        RiskInput::Str(s) => match s.trim().to_ascii_lowercase().as_str() {
            "l0" | "l0readonly" | "readonly" | "read_only" | "low" => Ok(RiskLevel::L0ReadOnly),
            "l1" | "l1createsafe" | "createsafe" | "create_safe" | "safe" => {
                Ok(RiskLevel::L1CreateSafe)
            }
            "l2" | "l2modify" | "modify" | "medium" => Ok(RiskLevel::L2Modify),
            "l3" | "l3destructive" | "destructive" | "high" => Ok(RiskLevel::L3Destructive),
            "l4" | "l4system" | "system" | "critical" => Ok(RiskLevel::L4System),
            other => Err(serde::de::Error::custom(format!(
                "Unknown risk_level '{}'",
                other
            ))),
        },
    }
}

/// Complete runtime descriptor for a built-in or dynamic user skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDescriptor {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub instructions: String,
    pub keywords: Vec<String>,
    pub risk_level: RiskLevel,
    pub required_permissions: Vec<String>,
    pub is_builtin: bool,
    pub enabled: bool,
    pub source_path: Option<String>,
}

/// Result of matching a user prompt against registered skills.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMatch {
    pub skill: SkillDescriptor,
    pub score: f64,
    pub matched_keywords: Vec<String>,
}

/// Dynamic Skill Runtime & Registry.
///
/// Loads built-in skill descriptors plus user-installed skill folders from
/// `%APPDATA%\ai.hime.desktop\skills\<skill-id>\` (`SKILL.md` + `manifest.json`).
pub struct SkillsRegistry {
    skills_dir: PathBuf,
    skills: Arc<RwLock<HashMap<String, SkillDescriptor>>>,
}

impl SkillsRegistry {
    /// Creates a registry pointing to `%APPDATA%\ai.hime.desktop\skills` and loads all skills.
    pub fn new() -> Self {
        Self::with_dir(Self::default_skills_dir())
    }

    /// Creates a registry with a custom skills directory (useful for testing).
    pub fn with_dir<P: Into<PathBuf>>(skills_dir: P) -> Self {
        let registry = Self {
            skills_dir: skills_dir.into(),
            skills: Arc::new(RwLock::new(HashMap::new())),
        };
        let _ = registry.reload_skills();
        registry
    }

    /// Resolves `%APPDATA%\ai.hime.desktop\skills` on Windows (or fallback home config dir).
    pub fn default_skills_dir() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            if !appdata.trim().is_empty() {
                return PathBuf::from(appdata).join("ai.hime.desktop").join("skills");
            }
        }
        if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            return PathBuf::from(home).join(".hime").join("skills");
        }
        PathBuf::from("skills")
    }

    pub fn skills_dir(&self) -> &Path {
        &self.skills_dir
    }

    /// Reloads all built-in skills and scans the dynamic skills directory.
    /// Returns the total number of loaded skills.
    pub fn reload_skills(&self) -> Result<usize, String> {
        let mut loaded = HashMap::new();

        for builtin in builtin_skill_descriptors() {
            loaded.insert(builtin.id.clone(), builtin);
        }

        if self.skills_dir.exists() && self.skills_dir.is_dir() {
            let entries = fs::read_dir(&self.skills_dir).map_err(|e| {
                format!(
                    "Failed to read skills directory '{}': {}",
                    self.skills_dir.display(),
                    e
                )
            })?;

            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                match load_skill_from_dir(&path) {
                    Ok(Some(skill)) => {
                        info!(skill_id = %skill.id, path = %path.display(), "Loaded dynamic skill");
                        loaded.insert(skill.id.clone(), skill);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        warn!(path = %path.display(), error = %e, "Skipping invalid skill folder");
                    }
                }
            }
        }

        let count = loaded.len();
        let mut guard = self
            .skills
            .write()
            .map_err(|_| "SkillsRegistry lock poisoned".to_string())?;
        *guard = loaded;
        Ok(count)
    }

    pub fn list_skills(&self) -> Vec<SkillDescriptor> {
        let guard = self.skills.read().unwrap();
        let mut items: Vec<SkillDescriptor> = guard.values().cloned().collect();
        items.sort_by(|a, b| a.id.cmp(&b.id));
        items
    }

    pub fn get_skill(&self, id: &str) -> Option<SkillDescriptor> {
        let guard = self.skills.read().unwrap();
        guard.get(id).cloned()
    }

    /// Matches enabled skills against `prompt` using keyword/intent overlap and name/description hits.
    pub fn match_skills(&self, prompt: &str) -> Vec<SkillMatch> {
        let lower_prompt = prompt.trim().to_ascii_lowercase();
        if lower_prompt.is_empty() {
            return Vec::new();
        }

        let guard = self.skills.read().unwrap();
        let mut matches = Vec::new();

        for skill in guard.values() {
            if !skill.enabled {
                continue;
            }

            let mut matched_keywords = Vec::new();
            let mut score = 0.0;

            for kw in &skill.keywords {
                let kw_lower = kw.trim().to_ascii_lowercase();
                if kw_lower.is_empty() {
                    continue;
                }
                if lower_prompt.contains(&kw_lower) {
                    matched_keywords.push(kw.clone());
                    // Multi-word intent phrases carry slightly higher confidence
                    let weight = if kw_lower.contains(' ') { 1.5 } else { 1.0 };
                    score += weight;
                }
            }

            let name_lower = skill.name.to_ascii_lowercase();
            if !name_lower.is_empty() && lower_prompt.contains(&name_lower) {
                score += 1.5;
            }

            if score > 0.0 {
                matches.push(SkillMatch {
                    skill: skill.clone(),
                    score,
                    matched_keywords,
                });
            }
        }

        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.skill.id.cmp(&b.skill.id))
        });

        matches
    }

    /// Formats the top `max_skills` matching skills into a concise system-prompt injection block.
    pub fn build_skills_prompt_injection(&self, prompt: &str, max_skills: usize) -> String {
        let matched = self.match_skills(prompt);
        if matched.is_empty() || max_skills == 0 {
            return String::new();
        }

        let mut out = String::from("## Active Relevant Skills\n");
        for item in matched.into_iter().take(max_skills) {
            out.push_str(&format!(
                "### {} (`{}`) [Risk: {:?}]\n{}\n\n",
                item.skill.name, item.skill.id, item.skill.risk_level, item.skill.instructions
            ));
        }
        out
    }
}

impl Default for SkillsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn load_skill_from_dir(dir: &Path) -> Result<Option<SkillDescriptor>, String> {
    let manifest_path = dir.join("manifest.json");
    let skill_md_path = dir.join("SKILL.md");

    if !manifest_path.exists() && !skill_md_path.exists() {
        return Ok(None);
    }

    let folder_id = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("custom_skill")
        .to_string();

    let manifest = if manifest_path.exists() {
        let raw = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read '{}': {}", manifest_path.display(), e))?;
        serde_json::from_str::<SkillManifest>(&raw)
            .map_err(|e| format!("Invalid manifest JSON in '{}': {}", manifest_path.display(), e))?
    } else {
        SkillManifest {
            id: folder_id.clone(),
            name: folder_id.clone(),
            version: default_version(),
            description: String::new(),
            keywords: vec![folder_id.replace(['-', '_'], " ")],
            risk_level: default_risk_level(),
            permissions: Vec::new(),
            enabled: true,
        }
    };

    let instructions = if skill_md_path.exists() {
        fs::read_to_string(&skill_md_path)
            .map_err(|e| format!("Failed to read '{}': {}", skill_md_path.display(), e))?
    } else {
        manifest.description.clone()
    };

    let id = if manifest.id.trim().is_empty() {
        folder_id.clone()
    } else {
        manifest.id
    };
    let name = if manifest.name.trim().is_empty() {
        id.clone()
    } else {
        manifest.name
    };

    Ok(Some(SkillDescriptor {
        id,
        name,
        version: manifest.version,
        description: manifest.description,
        instructions,
        keywords: manifest.keywords,
        risk_level: manifest.risk_level,
        required_permissions: manifest.permissions,
        is_builtin: false,
        enabled: manifest.enabled,
        source_path: Some(dir.to_string_lossy().to_string()),
    }))
}

fn builtin_skill_descriptors() -> Vec<SkillDescriptor> {
    vec![
        SkillDescriptor {
            id: "builtin.task_deadline_reminder".to_string(),
            name: "Task & Deadline Reminder".to_string(),
            version: "1.0.0".to_string(),
            description: "Manage tasks, deadlines, study schedules, and proactive reminders.".to_string(),
            instructions: "Extract concrete task titles, due dates, and priority levels. Create or update tasks via task tools and confirm reminder timing concisely.".to_string(),
            keywords: vec![
                "remind", "reminder", "deadline", "due", "schedule", "todo", "task",
                "ingatkan", "jadwal", "tugas", "besok",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            risk_level: RiskLevel::L1CreateSafe,
            required_permissions: vec![
                "tasks.read".to_string(),
                "tasks.write".to_string(),
                "notifications.send".to_string(),
            ],
            is_builtin: true,
            enabled: true,
            source_path: None,
        },
        SkillDescriptor {
            id: "builtin.file_organizer".to_string(),
            name: "File Organizer".to_string(),
            version: "1.0.0".to_string(),
            description: "Organize, categorize, rename, and move local files safely according to user folder rules.".to_string(),
            instructions: "Inspect target directories first (read-only), respect user folder rules from memory, and preview move/rename plans before modifying files.".to_string(),
            keywords: vec![
                "file", "folder", "directory", "organize", "sort", "move", "rename",
                "cleanup", "downloads", "documents", "rapikan", "pindahkan", "simpan",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            risk_level: RiskLevel::L2Modify,
            required_permissions: vec![
                "fs.read".to_string(),
                "fs.write".to_string(),
                "fs.move".to_string(),
            ],
            is_builtin: true,
            enabled: true,
            source_path: None,
        },
        SkillDescriptor {
            id: "builtin.app_cli_controller".to_string(),
            name: "App & CLI Controller".to_string(),
            version: "1.0.0".to_string(),
            description: "Launch applications, run bounded CLI commands, and control active desktop windows.".to_string(),
            instructions: "Verify command safety against PolicyEngine before execution. Never run destructive shell commands without explicit user approval.".to_string(),
            keywords: vec![
                "open app", "launch", "run command", "cli", "terminal", "powershell",
                "cmd", "process", "kill", "restart", "buka aplikasi", "jalankan",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            risk_level: RiskLevel::L4System,
            required_permissions: vec![
                "process.launch".to_string(),
                "cli.exec".to_string(),
                "window.control".to_string(),
            ],
            is_builtin: true,
            enabled: true,
            source_path: None,
        },
        SkillDescriptor {
            id: "builtin.document_web_builder".to_string(),
            name: "Document & Web Builder".to_string(),
            version: "1.0.0".to_string(),
            description: "Generate structured documents (Markdown, DOCX, PDF, reports) and static web pages.".to_string(),
            instructions: "Produce clean, well-structured documents or self-contained HTML/CSS/JS artifacts in the user's designated output directory.".to_string(),
            keywords: vec![
                "document", "markdown", "pdf", "docx", "report", "html", "webpage",
                "website", "landing page", "build web", "buat dokumen", "laporan",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            risk_level: RiskLevel::L1CreateSafe,
            required_permissions: vec![
                "fs.write".to_string(),
                "doc.generate".to_string(),
                "web.build".to_string(),
            ],
            is_builtin: true,
            enabled: true,
            source_path: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_skills_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hime_skills_test_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_builtin_skills_loaded_and_matched() {
        let dir = temp_skills_dir("builtin");
        let registry = SkillsRegistry::with_dir(&dir);

        let all = registry.list_skills();
        assert_eq!(all.len(), 4);
        assert!(registry.get_skill("builtin.task_deadline_reminder").is_some());
        assert!(registry.get_skill("builtin.file_organizer").is_some());
        assert!(registry.get_skill("builtin.app_cli_controller").is_some());
        assert!(registry.get_skill("builtin.document_web_builder").is_some());

        let matches = registry.match_skills("Please organize my downloads folder and rename files");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].skill.id, "builtin.file_organizer");
        assert_eq!(matches[0].skill.risk_level, RiskLevel::L2Modify);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_dynamic_skill_loading_and_reload() {
        let dir = temp_skills_dir("dynamic");
        let registry = SkillsRegistry::with_dir(&dir);
        assert_eq!(registry.list_skills().len(), 4);

        // Add a custom skill folder
        let custom_dir = dir.join("git-helper");
        fs::create_dir_all(&custom_dir).unwrap();
        fs::write(
            custom_dir.join("manifest.json"),
            r#"{
                "id": "custom.git_helper",
                "name": "Git Helper",
                "version": "0.2.0",
                "description": "Assists with git commits and branch cleanup",
                "keywords": ["git", "commit", "branch", "rebase"],
                "risk_level": "L3Destructive",
                "permissions": ["git.read", "git.write"]
            }"#,
        )
        .unwrap();
        fs::write(
            custom_dir.join("SKILL.md"),
            "Always check git status before committing or rebasing.",
        )
        .unwrap();

        let total = registry.reload_skills().unwrap();
        assert_eq!(total, 5);

        let custom = registry.get_skill("custom.git_helper").unwrap();
        assert!(!custom.is_builtin);
        assert_eq!(custom.risk_level, RiskLevel::L3Destructive);
        assert!(custom.instructions.contains("git status"));

        let matches = registry.match_skills("Help me rebase this git branch");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].skill.id, "custom.git_helper");

        let injection = registry.build_skills_prompt_injection("git commit", 2);
        assert!(injection.contains("Git Helper"));
        assert!(injection.contains("L3Destructive"));

        let _ = fs::remove_dir_all(&dir);
    }
}
