//! Hime Island Module SDK and Extension Lifecycle Runtime
//!
//! Provides the core traits, manifest parser, and zero-cost ModuleRegistry
//! defined in 14-MODULARITY-AND-CODE-QUALITY.md.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModuleError {
    #[error("Module '{0}' not found in registry")]
    NotFound(String),
    #[error("Dependency cycle detected involving '{0}'")]
    DependencyCycle(String),
    #[error("Missing dependency '{0}' required by '{1}'")]
    MissingDependency(String, String),
    #[error("Failed to start module '{0}': {1}")]
    StartFailed(String, String),
    #[error("Failed to stop module '{0}': {1}")]
    StopFailed(String, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleStatus {
    Disabled,
    Stopped,
    Starting,
    Running,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHealth {
    pub status: ModuleStatus,
    pub message: Option<String>,
}

impl ModuleHealth {
    pub fn running() -> Self {
        Self {
            status: ModuleStatus::Running,
            message: None,
        }
    }

    pub fn stopped() -> Self {
        Self {
            status: ModuleStatus::Stopped,
            message: None,
        }
    }

    pub fn disabled() -> Self {
        Self {
            status: ModuleStatus::Disabled,
            message: None,
        }
    }

    pub fn failed(msg: impl Into<String>) -> Self {
        Self {
            status: ModuleStatus::Failed,
            message: Some(msg.into()),
        }
    }
}

/// The core trait that all Hime backend modules implement.
pub trait Module: Send + Sync {
    fn id(&self) -> &str;
    fn version(&self) -> &str;
    fn deps(&self) -> &[&str] {
        &[]
    }
    fn start(&mut self) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    fn health(&self) -> ModuleHealth;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub id: String,
    pub version: String,
    pub title: String,
    pub desc: String,
    pub icon: String,
    pub category: String,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    pub default: ModuleDefaultConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDefaultConfig {
    pub enabled: bool,
}

pub struct ModuleEntry {
    pub module: Box<dyn Module>,
    pub enabled: bool,
    pub status: ModuleStatus,
}

/// Central ModuleRegistry that enforces topological dependency ordering,
/// zero-cost disabled modules, and safe start/stop lifecycles.
pub struct ModuleRegistry {
    modules: Arc<Mutex<HashMap<String, ModuleEntry>>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Registers a module into the registry. Starts in Stopped or Disabled state.
    pub fn register(&self, module: Box<dyn Module>, enabled: bool) {
        let id = module.id().to_string();
        let status = if enabled {
            ModuleStatus::Stopped
        } else {
            ModuleStatus::Disabled
        };

        let mut lock = self.modules.lock().unwrap();
        lock.insert(
            id,
            ModuleEntry {
                module,
                enabled,
                status,
            },
        );
    }

    /// Starts an individual enabled module. Fails if dependencies are not running.
    pub fn start_module(&self, id: &str) -> Result<(), ModuleError> {
        let mut lock = self.modules.lock().unwrap();

        // 1. Verify dependencies are Running
        let deps: Vec<String> = {
            let entry = lock.get(id).ok_or_else(|| ModuleError::NotFound(id.to_string()))?;
            if !entry.enabled {
                return Ok(()); // Zero cost when disabled
            }
            entry.module.deps().iter().map(|s| s.to_string()).collect()
        };

        for dep in &deps {
            match lock.get(dep) {
                Some(dep_entry) if dep_entry.status == ModuleStatus::Running => {}
                Some(_) => {
                    return Err(ModuleError::StartFailed(
                        id.to_string(),
                        format!("Dependency '{}' is not running", dep),
                    ));
                }
                None => {
                    return Err(ModuleError::MissingDependency(dep.clone(), id.to_string()));
                }
            }
        }

        // 2. Start module
        let entry = lock.get_mut(id).unwrap();
        entry.status = ModuleStatus::Starting;
        match entry.module.start() {
            Ok(()) => {
                entry.status = ModuleStatus::Running;
                Ok(())
            }
            Err(e) => {
                entry.status = ModuleStatus::Failed;
                Err(ModuleError::StartFailed(id.to_string(), e))
            }
        }
    }

    /// Stops a module cleanly. Zero-cost when stopped.
    pub fn stop_module(&self, id: &str) -> Result<(), ModuleError> {
        let mut lock = self.modules.lock().unwrap();
        let entry = lock.get_mut(id).ok_or_else(|| ModuleError::NotFound(id.to_string()))?;

        if entry.status != ModuleStatus::Running && entry.status != ModuleStatus::Starting {
            return Ok(());
        }

        match entry.module.stop() {
            Ok(()) => {
                entry.status = if entry.enabled {
                    ModuleStatus::Stopped
                } else {
                    ModuleStatus::Disabled
                };
                Ok(())
            }
            Err(e) => {
                entry.status = ModuleStatus::Failed;
                Err(ModuleError::StopFailed(id.to_string(), e))
            }
        }
    }

    /// Gets current health of all modules.
    pub fn get_health_report(&self) -> HashMap<String, ModuleHealth> {
        let lock = self.modules.lock().unwrap();
        let mut report = HashMap::new();
        for (id, entry) in lock.iter() {
            report.insert(id.clone(), entry.module.health());
        }
        report
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyModule {
        id: String,
        running: bool,
    }

    impl DummyModule {
        fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                running: false,
            }
        }
    }

    impl Module for DummyModule {
        fn id(&self) -> &str {
            &self.id
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn start(&mut self) -> Result<(), String> {
            self.running = true;
            Ok(())
        }

        fn stop(&mut self) -> Result<(), String> {
            self.running = false;
            Ok(())
        }

        fn health(&self) -> ModuleHealth {
            if self.running {
                ModuleHealth::running()
            } else {
                ModuleHealth::stopped()
            }
        }
    }

    #[test]
    fn test_register_and_lifecycle() {
        let registry = ModuleRegistry::new();
        let dummy = Box::new(DummyModule::new("test_mod"));
        registry.register(dummy, true);

        assert!(registry.start_module("test_mod").is_ok());
        let health = registry.get_health_report();
        assert_eq!(health.get("test_mod").unwrap().status, ModuleStatus::Running);

        assert!(registry.stop_module("test_mod").is_ok());
        let health_after = registry.get_health_report();
        assert_eq!(health_after.get("test_mod").unwrap().status, ModuleStatus::Stopped);
    }

    #[test]
    fn test_zero_cost_disabled() {
        let registry = ModuleRegistry::new();
        let dummy = Box::new(DummyModule::new("disabled_mod"));
        registry.register(dummy, false);

        // Starting disabled module is a no-op
        assert!(registry.start_module("disabled_mod").is_ok());
        let health = registry.get_health_report();
        assert_eq!(health.get("disabled_mod").unwrap().status, ModuleStatus::Stopped);
    }
}
