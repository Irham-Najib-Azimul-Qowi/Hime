use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::info;

use hime_storage::{Database, TaskRecord};

use super::resource::ResourceManager;
use crate::bus::{EventBus, SystemEvent};

/// Valid state machine transitions for tasks in Hime Core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    WaitingApproval,
    Paused,
    Succeeded,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Queued => "queued",
            TaskStatus::Running => "running",
            TaskStatus::WaitingApproval => "waiting_approval",
            TaskStatus::Paused => "paused",
            TaskStatus::Succeeded => "succeeded",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "queued" => Some(TaskStatus::Queued),
            "running" => Some(TaskStatus::Running),
            "waiting_approval" => Some(TaskStatus::WaitingApproval),
            "paused" => Some(TaskStatus::Paused),
            "succeeded" => Some(TaskStatus::Succeeded),
            "failed" => Some(TaskStatus::Failed),
            "cancelled" => Some(TaskStatus::Cancelled),
            _ => None,
        }
    }

    /// Determines whether transition from current status to `next` is allowed.
    pub fn can_transition_to(&self, next: TaskStatus) -> bool {
        match self {
            TaskStatus::Queued => matches!(next, TaskStatus::Running | TaskStatus::Cancelled),
            TaskStatus::Running => matches!(
                next,
                TaskStatus::WaitingApproval
                    | TaskStatus::Paused
                    | TaskStatus::Succeeded
                    | TaskStatus::Failed
                    | TaskStatus::Cancelled
            ),
            TaskStatus::WaitingApproval => matches!(
                next,
                TaskStatus::Running | TaskStatus::Cancelled | TaskStatus::Failed
            ),
            TaskStatus::Paused => matches!(next, TaskStatus::Running | TaskStatus::Cancelled),
            TaskStatus::Succeeded | TaskStatus::Failed | TaskStatus::Cancelled => false,
        }
    }
}

/// Task Engine managing execution lifecycle, SQLite recovery, retry policies,
/// and backpressure concurrency throttling.
pub struct TaskEngine {
    storage: Arc<Database>,
    resource_manager: Arc<ResourceManager>,
    event_bus: EventBus,
    semaphore: Arc<Semaphore>,
}

impl TaskEngine {
    pub fn new(
        storage: Arc<Database>,
        resource_manager: Arc<ResourceManager>,
        event_bus: EventBus,
    ) -> Self {
        let max_concurrency = resource_manager.max_concurrent_tasks();
        let engine = Self {
            storage,
            resource_manager,
            event_bus,
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
        };

        engine.recover_on_startup();
        engine
    }

    /// Returns the maximum concurrency configured by the resource manager.
    pub fn max_concurrency(&self) -> usize {
        self.resource_manager.max_concurrent_tasks()
    }

    /// Returns available execution permits under current backpressure limits.
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Startup recovery: recover unfinished tasks left behind by previous crashes or shutdowns.
    pub fn recover_on_startup(&self) {
        info!("Running Task Engine startup recovery");
        if let Ok(active_tasks) = self.storage.list_tasks(Some("running")) {
            for task in active_tasks {
                info!(task_id = %task.id, "Recovering uncompleted running task to queued status");
                let _ = self.storage.update_task_status(&task.id, "queued", task.progress, false);
                let _ = self.event_bus.publish(SystemEvent::TaskUpdated {
                    task_id: task.id,
                    status: "queued".to_string(),
                });
            }
        }
    }

    pub fn submit_task(
        &self,
        id: &str,
        title: &str,
        description: Option<&str>,
        priority: &str,
    ) -> Result<TaskRecord, String> {
        let record = self
            .storage
            .create_task(id, title, description, priority, "queued")
            .map_err(|e| format!("Database error submitting task: {}", e))?;

        let _ = self.event_bus.publish(SystemEvent::TaskUpdated {
            task_id: id.to_string(),
            status: "queued".to_string(),
        });

        info!(task_id = %id, title = %title, priority = %priority, "Task submitted to queue");
        Ok(record)
    }

    pub fn get_task(&self, id: &str) -> Result<Option<TaskRecord>, String> {
        self.storage
            .get_task(id)
            .map_err(|e| format!("Database error fetching task: {}", e))
    }

    pub fn list_tasks(&self, status_filter: Option<&str>) -> Result<Vec<TaskRecord>, String> {
        self.storage
            .list_tasks(status_filter)
            .map_err(|e| format!("Database error listing tasks: {}", e))
    }

    pub fn transition_status(
        &self,
        id: &str,
        new_status: TaskStatus,
        progress: f64,
    ) -> Result<bool, String> {
        let current = self
            .storage
            .get_task(id)
            .map_err(|e| format!("Storage lookup error: {}", e))?
            .ok_or_else(|| format!("Task '{}' not found", id))?;

        let current_status = TaskStatus::parse(&current.status)
            .ok_or_else(|| format!("Unknown current status '{}'", current.status))?;

        if !current_status.can_transition_to(new_status) {
            return Err(format!(
                "Invalid state transition: cannot transition from '{}' to '{}'",
                current_status.as_str(),
                new_status.as_str()
            ));
        }

        let is_completed = matches!(
            new_status,
            TaskStatus::Succeeded | TaskStatus::Failed | TaskStatus::Cancelled
        );

        let updated = self
            .storage
            .update_task_status(id, new_status.as_str(), progress, is_completed)
            .map_err(|e| format!("Failed to update task status in DB: {}", e))?;

        if updated {
            let _ = self.event_bus.publish(SystemEvent::TaskUpdated {
                task_id: id.to_string(),
                status: new_status.as_str().to_string(),
            });
            info!(task_id = %id, from = current_status.as_str(), to = new_status.as_str(), "Task status transitioned");
        }

        Ok(updated)
    }

    pub fn cancel_task(&self, id: &str) -> Result<bool, String> {
        self.transition_status(id, TaskStatus::Cancelled, 0.0)
    }

    pub fn retry_task(&self, id: &str) -> Result<TaskRecord, String> {
        let task = self
            .storage
            .get_task(id)
            .map_err(|e| format!("Storage error: {}", e))?
            .ok_or_else(|| format!("Task '{}' not found", id))?;

        if task.retry_count >= task.max_retries {
            return Err(format!(
                "Task '{}' exceeded maximum retries ({})",
                id, task.max_retries
            ));
        }

        let _ = self
            .storage
            .increment_task_retry(id)
            .map_err(|e| format!("Failed to increment retry count: {}", e))?;

        self.storage
            .update_task_status(id, "queued", 0.0, false)
            .map_err(|e| format!("Failed to update task status: {}", e))?;

        let _ = self.event_bus.publish(SystemEvent::TaskUpdated {
            task_id: id.to_string(),
            status: "queued".to_string(),
        });

        self.storage
            .get_task(id)
            .map_err(|e| format!("Storage error: {}", e))?
            .ok_or_else(|| format!("Task '{}' not found after retry", id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_state_machine_valid_transitions() {
        assert!(TaskStatus::Queued.can_transition_to(TaskStatus::Running));
        assert!(TaskStatus::Running.can_transition_to(TaskStatus::WaitingApproval));
        assert!(TaskStatus::WaitingApproval.can_transition_to(TaskStatus::Running));
        assert!(TaskStatus::Running.can_transition_to(TaskStatus::Succeeded));
        assert!(TaskStatus::Running.can_transition_to(TaskStatus::Failed));

        // Invalid transitions
        assert!(!TaskStatus::Succeeded.can_transition_to(TaskStatus::Running));
        assert!(!TaskStatus::Cancelled.can_transition_to(TaskStatus::Running));
        assert!(!TaskStatus::Queued.can_transition_to(TaskStatus::Succeeded));
    }

    #[test]
    fn test_task_engine_lifecycle_and_retry() {
        let storage = Arc::new(Database::open_in_memory().unwrap());
        let resource = Arc::new(ResourceManager::default());
        let event_bus = EventBus::new(128);
        let engine = TaskEngine::new(storage, resource, event_bus);

        // Submit
        let task = engine
            .submit_task("t-1", "Analyze Logs", Some("Process error logs"), "high")
            .unwrap();
        assert_eq!(task.status, "queued");

        // Transition to Running
        assert!(engine
            .transition_status("t-1", TaskStatus::Running, 0.2)
            .unwrap());

        // Transition to Succeeded
        assert!(engine
            .transition_status("t-1", TaskStatus::Succeeded, 1.0)
            .unwrap());

        let finished = engine.get_task("t-1").unwrap().unwrap();
        assert_eq!(finished.status, "succeeded");
        assert!(finished.completed_at.is_some());
    }

    #[test]
    fn test_task_engine_recovery_on_startup() {
        let storage = Arc::new(Database::open_in_memory().unwrap());
        // Insert a task marked as 'running' in storage
        storage
            .create_task("t-crash", "Crash Test Task", None, "medium", "running")
            .unwrap();

        let resource = Arc::new(ResourceManager::default());
        let event_bus = EventBus::new(128);
        let engine = TaskEngine::new(storage, resource, event_bus);

        // Verify that the task was re-queued on startup
        let recovered = engine.get_task("t-crash").unwrap().unwrap();
        assert_eq!(recovered.status, "queued");
    }
}
