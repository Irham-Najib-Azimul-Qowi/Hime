use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

use hime_security::policy::PolicyEnforcer;
use hime_storage::Database;

use super::conversation::ConversationService;
use super::hermes_adapter::{AgentActivityEvent, HermesRuntimeAdapter, HermesRuntimeStatus};
use super::provider_adapter::{ModelInfo, ProviderCredentialAdapter};
use super::router_gateway::{RouterGatewayAdapter, RouterHealthStatus};
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

/// The central AI Orchestrator of Hime Core.
pub struct AiOrchestrator {
    pub provider_adapter: Arc<ProviderCredentialAdapter>,
    pub router_gateway: Arc<RouterGatewayAdapter>,
    pub hermes_adapter: Arc<HermesRuntimeAdapter>,
    pub conversation_service: Arc<ConversationService>,
    pub event_bus: EventBus,
    pub storage: Arc<Database>,
    pub policy: Arc<dyn PolicyEnforcer>,
}

impl AiOrchestrator {
    pub fn new(
        storage: Arc<Database>,
        policy: Arc<dyn PolicyEnforcer>,
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

        Self {
            provider_adapter,
            router_gateway,
            hermes_adapter,
            conversation_service,
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
            .execute_turn(conversation_id, user_prompt, event_tx)
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
}

