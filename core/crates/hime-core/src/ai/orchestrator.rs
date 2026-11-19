use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

use hime_security::policy::PolicyEngine;
use hime_security::redaction::{redact_api_keys, redact_chain_of_thought};
use hime_storage::Database;

use super::conversation::ConversationService;
use super::event_engine::EventEngine;
use super::hermes_adapter::{AgentActivityEvent, HermesRuntimeAdapter, HermesRuntimeStatus};
use super::memory::MemoryEngine;
use super::provider_adapter::{ModelInfo, ProviderCredentialAdapter};
use super::resource::ResourceManager;
use super::router_gateway::{RouterGatewayAdapter, RouterHealthStatus};
use super::task_engine::TaskEngine;
use super::tools::{ToolGateway, ToolResult};
use super::translation::TranslationService;
use crate::bus::{EventBus, SystemEvent};

/// Aggregate status of the entire Hime AI runtime stack.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiRuntimeOverview {
    pub core_orchestration_active: bool,
    pub hermes_status: HermesRuntimeStatus,
    pub router_status: RouterHealthStatus,
    pub loopback_adapter_port: u16,
    pub configured_providers: Vec<String>,
    pub ib_01_status: String,
    pub ib_02_status: String,
}

/// A discrete planned step in an orchestrated multi-step workflow.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanStep {
    pub step_id: String,
    pub description: String,
    pub tool_id: String,
    pub params: serde_json::Value,
    pub idempotency_key: Option<String>,
    pub approval_ticket: Option<String>,
}

/// Execution outcome of an orchestrated multi-step plan.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanExecutionSummary {
    pub total_steps: usize,
    pub executed_steps: usize,
    pub success: bool,
    pub step_results: Vec<ToolResult>,
    pub synthesized_output: String,
    pub loop_detected: bool,
}

/// The central AI Orchestrator of Hime Core.
pub struct AiOrchestrator {
    pub provider_adapter: Arc<ProviderCredentialAdapter>,
    pub router_gateway: Arc<RouterGatewayAdapter>,
    pub hermes_adapter: Arc<HermesRuntimeAdapter>,
    pub conversation_service: Arc<ConversationService>,
    pub tool_gateway: Arc<ToolGateway>,
    pub task_engine: Arc<TaskEngine>,
    pub event_engine: Arc<EventEngine>,
    pub memory_engine: Arc<MemoryEngine>,
    pub resource_manager: Arc<ResourceManager>,
    pub translation_service: Arc<TranslationService>,
    pub event_bus: EventBus,
    pub storage: Arc<Database>,
    pub policy: Arc<PolicyEngine>,
}

impl AiOrchestrator {
    pub fn new(
        storage: Arc<Database>,
        policy: Arc<PolicyEngine>,
        event_bus: EventBus,
        loopback_adapter_port: u16,
    ) -> Self {
        let provider_adapter = Arc::new(ProviderCredentialAdapter::new(
            loopback_adapter_port,
            Arc::clone(&storage),
        ));
        let router_gateway = Arc::new(RouterGatewayAdapter::new(20128));
        let hermes_adapter = Arc::new(HermesRuntimeAdapter::new(
            format!("http://127.0.0.1:{}/v1", loopback_adapter_port),
            Arc::clone(&storage),
        ));
        let conversation_service = Arc::new(ConversationService::new(Arc::clone(&storage)));
        let resource_manager = Arc::new(ResourceManager::default());
        let tool_gateway = Arc::new(ToolGateway::new(
            Arc::clone(&storage),
            Arc::clone(&policy),
            event_bus.clone(),
        ));
        let task_engine = Arc::new(TaskEngine::new(
            Arc::clone(&storage),
            Arc::clone(&resource_manager),
            event_bus.clone(),
        ));
        let event_engine = Arc::new(EventEngine::new(event_bus.clone(), 200));
        let memory_engine = Arc::new(MemoryEngine::new(Arc::clone(&storage)));
        let translation_service = Arc::new(TranslationService::new(event_bus.clone(), true));

        Self {
            provider_adapter,
            router_gateway,
            hermes_adapter,
            conversation_service,
            tool_gateway,
            task_engine,
            event_engine,
            memory_engine,
            resource_manager,
            translation_service,
            event_bus,
            storage,
            policy,
        }
    }

    /// Returns the comprehensive status of the Hime AI runtime stack.
    pub async fn get_overview_status(&self) -> AiRuntimeOverview {
        let hermes = self.hermes_adapter.get_status();
        let router = self.router_gateway.check_health().await;
        let providers = self
            .storage
            .list_configured_providers()
            .unwrap_or_default();

        AiRuntimeOverview {
            core_orchestration_active: true,
            hermes_status: hermes,
            router_status: router,
            loopback_adapter_port: self.provider_adapter.port,
            configured_providers: providers,
            ib_01_status: "RESOLVED (Hime-Owned Provider Adapter protects secrets at-rest via DPAPI; 9Router DB never receives plaintext keys)".to_string(),
            ib_02_status: "OPEN (Localhost loopback bypass confirmed in 9Router dashboardGuard.js; isolated port & token defense active on Core)".to_string(),
        }
    }

    /// Lists all models discovered from the Provider Adapter and 9Router.
    pub fn list_models(&self) -> Vec<ModelInfo> {
        self.provider_adapter.list_models()
    }

    /// Stores a raw provider secret securely into the DPAPI vault.
    pub fn store_provider_credential(&self, provider: &str, secret: &str) -> Result<(), String> {
        self.provider_adapter
            .store_provider_secret(provider, secret.as_bytes().to_vec())
    }

    /// Dispatches a prompt through the end-to-end AI orchestration pipeline:
    /// 1. Persists user message in SQLite WAL.
    /// 2. Dispatches prompt to Hermes Agent via loopback adapter.
    /// 3. Streams back delta tokens and emits real-time activity events to EventBus.
    /// 4. Enforces SEC-09 (redacts private CoT).
    /// 5. Persists assistant response in SQLite WAL.
    pub async fn execute_chat_prompt(
        &self,
        conversation_id: &str,
        user_prompt: &str,
    ) -> Result<String, String> {
        self.execute_chat_prompt_with_model(conversation_id, user_prompt, None)
            .await
    }

    /// Dispatches a prompt through the end-to-end AI orchestration pipeline with optional model override.
    pub async fn execute_chat_prompt_with_model(
        &self,
        conversation_id: &str,
        user_prompt: &str,
        model_override: Option<&str>,
    ) -> Result<String, String> {
        // Step 1: Persist user message
        let _ = self
            .conversation_service
            .add_user_message(conversation_id, user_prompt)
            .map_err(|e| format!("Failed to record user message: {}", e))?;

        // Step 2: Channel for capturing real-time agent activity events
        let (event_tx, mut event_rx) = mpsc::channel::<AgentActivityEvent>(64);
        let event_bus = self.event_bus.clone();

        // Spawn a background task to relay agent events to Hime Core's system event bus
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                let _ = event_bus.publish(SystemEvent::AgentStateChanged {
                    new_state: event.event_type.clone(),
                });
            }
        });

        // Step 3: Execute turn through Hermes adapter
        let response_text = self
            .hermes_adapter
            .execute_turn_with_model(conversation_id, user_prompt, model_override, event_tx)
            .await?;

        // Step 4: Persist assistant message in SQLite WAL
        let _ = self
            .conversation_service
            .add_assistant_message(conversation_id, &response_text, Some(response_text.len() as i64 / 4))
            .map_err(|e| format!("Failed to record assistant message: {}", e))?;

        info!(
            conversation_id = %conversation_id,
            chars = response_text.len(),
            "Completed AI conversation turn successfully"
        );

        Ok(response_text)
    }

    /// Cancels active turn.
    pub async fn cancel_active_turn(&self) -> bool {
        self.hermes_adapter.cancel_active_turn().await
    }

    /// Executes an orchestrated multi-step plan with Quality Controls:
    /// - Max step iteration limit (max 10)
    /// - Consecutive loop detection
    /// - Tool verification before claiming step success
    /// - Hash-chained audit trail
    /// - SEC-09 output redaction
    pub async fn execute_orchestrated_plan(
        &self,
        client_id: &str,
        steps: Vec<PlanStep>,
    ) -> Result<PlanExecutionSummary, String> {
        if steps.is_empty() {
            return Err("Plan contains no steps".to_string());
        }

        if steps.len() > 10 {
            return Err("Plan exceeds maximum allowed steps limit (10)".to_string());
        }

        let mut executed_results = Vec::new();
        let mut last_call_sig = String::new();
        let mut repeat_count = 0;

        for step in &steps {
            // Quality Control 1: Loop Detection
            let call_sig = format!("{}:{}", step.tool_id, step.params);
            if call_sig == last_call_sig {
                repeat_count += 1;
                if repeat_count >= 2 {
                    warn!(tool_id = %step.tool_id, "Loop detected during plan execution; aborting");
                    let _ = self.event_engine.emit(
                        "agent.loop_detected",
                        serde_json::json!({ "step_id": step.step_id, "tool_id": step.tool_id }),
                        None,
                    ).await;

                    return Ok(PlanExecutionSummary {
                        total_steps: steps.len(),
                        executed_steps: executed_results.len(),
                        success: false,
                        step_results: executed_results,
                        synthesized_output: "Aborted: Quality control detected infinite loop of identical tool calls".to_string(),
                        loop_detected: true,
                    });
                }
            } else {
                last_call_sig = call_sig;
                repeat_count = 0;
            }

            // Quality Control 2: Execute step through Tool Gateway
            let res = self
                .tool_gateway
                .execute_tool(
                    client_id,
                    &step.tool_id,
                    step.params.clone(),
                    step.approval_ticket.as_deref(),
                    step.idempotency_key.as_deref(),
                )
                .await?;

            let success = res.success;
            executed_results.push(res);

            // Quality Control 3: Verification of step success
            if !success {
                warn!(step_id = %step.step_id, "Plan execution halted due to step failure");
                return Ok(PlanExecutionSummary {
                    total_steps: steps.len(),
                    executed_steps: executed_results.len(),
                    success: false,
                    step_results: executed_results,
                    synthesized_output: format!("Execution halted at step '{}'", step.step_id),
                    loop_detected: false,
                });
            }
        }

        // Quality Control 4: Synthesize output with SEC-09 redaction
        let raw_summary = format!(
            "Successfully completed all {} plan steps without error.",
            steps.len()
        );
        let sanitized_summary = redact_api_keys(&redact_chain_of_thought(&raw_summary));

        Ok(PlanExecutionSummary {
            total_steps: steps.len(),
            executed_steps: executed_results.len(),
            success: true,
            step_results: executed_results,
            synthesized_output: sanitized_summary,
            loop_detected: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_multi_step_plan_execution() {
        let storage = Arc::new(Database::open_in_memory().unwrap());
        let policy = Arc::new(PolicyEngine::new());
        let event_bus = EventBus::new(128);
        let orchestrator = AiOrchestrator::new(storage, policy, event_bus, 20129);

        let plan = vec![
            PlanStep {
                step_id: "s1".to_string(),
                description: "Get System Info".to_string(),
                tool_id: "system.info".to_string(),
                params: serde_json::json!({}),
                idempotency_key: None,
                approval_ticket: None,
            },
            PlanStep {
                step_id: "s2".to_string(),
                description: "Create Follow-up Task".to_string(),
                tool_id: "tasks.create".to_string(),
                params: serde_json::json!({ "title": "Orchestrated Task", "priority": "medium" }),
                idempotency_key: None,
                approval_ticket: None,
            },
        ];

        let summary = orchestrator
            .execute_orchestrated_plan("test_runner", plan)
            .await
            .expect("plan execution succeeds");

        assert!(summary.success);
        assert_eq!(summary.total_steps, 2);
        assert_eq!(summary.executed_steps, 2);
        assert!(!summary.loop_detected);
    }

    #[tokio::test]
    async fn test_orchestrator_loop_detection() {
        let storage = Arc::new(Database::open_in_memory().unwrap());
        let policy = Arc::new(PolicyEngine::new());
        let event_bus = EventBus::new(128);
        let orchestrator = AiOrchestrator::new(storage, policy, event_bus, 20129);

        // Three identical calls in a row
        let looping_plan = vec![
            PlanStep {
                step_id: "loop-1".to_string(),
                description: "Repeated Tool Call".to_string(),
                tool_id: "system.info".to_string(),
                params: serde_json::json!({}),
                idempotency_key: None,
                approval_ticket: None,
            },
            PlanStep {
                step_id: "loop-2".to_string(),
                description: "Repeated Tool Call".to_string(),
                tool_id: "system.info".to_string(),
                params: serde_json::json!({}),
                idempotency_key: None,
                approval_ticket: None,
            },
            PlanStep {
                step_id: "loop-3".to_string(),
                description: "Repeated Tool Call".to_string(),
                tool_id: "system.info".to_string(),
                params: serde_json::json!({}),
                idempotency_key: None,
                approval_ticket: None,
            },
        ];

        let summary = orchestrator
            .execute_orchestrated_plan("test_runner", looping_plan)
            .await
            .expect("loop detected handled");

        assert!(!summary.success);
        assert!(summary.loop_detected);
        assert!(summary.synthesized_output.contains("infinite loop"));
    }
}
