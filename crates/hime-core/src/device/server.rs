use crate::bus::{EventBus, SystemEvent};
use crate::device::pairing::{PairingError, PairingManager};
use crate::device::pki::LocalPkiAuthority;
use crate::device::protocol::MobileDeviceStatus;
use crate::device::transfer::TransferEngine;
use crate::server::ApprovalRecord;
use hime_ipc::protocol::{JsonRpcRequest, JsonRpcResponse, INVALID_REQUEST, METHOD_NOT_FOUND, UNAUTHORIZED};
use hime_storage::{Database, DeviceStorageSummaryRecord};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DeviceGatewayServer {
    db: Arc<Database>,
    pki: Arc<LocalPkiAuthority>,
    pairing: Arc<PairingManager>,
    transfer: Arc<TransferEngine>,
    event_bus: EventBus,
    approvals: Arc<Mutex<HashMap<String, ApprovalRecord>>>,
    is_enabled: Mutex<bool>,
}

impl DeviceGatewayServer {
    pub fn new(
        db: Arc<Database>,
        event_bus: EventBus,
        approvals: Arc<Mutex<HashMap<String, ApprovalRecord>>>,
        transfer: Arc<TransferEngine>,
        server_address: &str,
        server_port: u16,
    ) -> Self {
        let pki = Arc::new(LocalPkiAuthority::new());
        let pairing = Arc::new(PairingManager::new(
            db.clone(),
            server_address.to_string(),
            server_port,
            pki.ca_fingerprint().to_string(),
        ));

        Self {
            db,
            pki,
            pairing,
            transfer,
            event_bus,
            approvals,
            is_enabled: Mutex::new(true), // Enabled by default in Phase 11
        }
    }

    pub fn pairing_manager(&self) -> Arc<PairingManager> {
        self.pairing.clone()
    }

    pub fn pki_authority(&self) -> Arc<LocalPkiAuthority> {
        self.pki.clone()
    }

    pub fn transfer_engine(&self) -> Arc<TransferEngine> {
        self.transfer.clone()
    }

    /// Dispatches an incoming JSON-RPC request from an authenticated Mobile Device client.
    pub async fn dispatch_device_rpc(
        &self,
        req: JsonRpcRequest,
        client_cert_fingerprint: &str,
    ) -> JsonRpcResponse {
        let req_id = req.id.clone();

        // Check if device service is enabled
        {
            let enabled = self.is_enabled.lock().await;
            if !*enabled {
                return JsonRpcResponse::error(
                    req_id,
                    UNAUTHORIZED,
                    "Hime Mobile Device Gateway is currently disabled in Settings",
                    None,
                );
            }
        }

        // Method whitelist for unauthenticated pairing
        if req.method == "device.pairing.start" {
            match self.pairing.create_pairing_session(300) {
                Ok(qr) => return JsonRpcResponse::success(req_id, serde_json::to_value(qr).unwrap_or_default()),
                Err(e) => return JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
            }
        }

        if req.method == "device.pairing.confirm" {
            let params = req.params.unwrap_or_default();
            let token = params.get("token").and_then(|v| v.as_str()).unwrap_or("");
            let code = params.get("confirmation_code").and_then(|v| v.as_str()).unwrap_or("");
            let dev_id = params.get("device_id").and_then(|v| v.as_str()).unwrap_or("");
            let dev_name = params.get("device_name").and_then(|v| v.as_str()).unwrap_or("Android Companion");
            let dev_type = params.get("device_type").and_then(|v| v.as_str()).unwrap_or("android");
            let fingerprint = params.get("client_cert_fingerprint").and_then(|v| v.as_str()).unwrap_or(client_cert_fingerprint);

            match self.pairing.complete_pairing(token, code, dev_id, dev_name, dev_type, fingerprint) {
                Ok(dev) => return JsonRpcResponse::success(req_id, serde_json::to_value(dev).unwrap_or_default()),
                Err(e) => return JsonRpcResponse::error(req_id, UNAUTHORIZED, e.to_string(), None),
            }
        }

        // All other device methods require valid client_cert_fingerprint
        let params = req.params.unwrap_or_default();
        let device_id = params.get("device_id").and_then(|v| v.as_str()).unwrap_or("");

        match self.pairing.verify_device_active(device_id, client_cert_fingerprint) {
            Ok(true) => {}
            Ok(false) => {
                return JsonRpcResponse::error(
                    req_id,
                    UNAUTHORIZED,
                    "Client certificate fingerprint not authorized for this device",
                    None,
                );
            }
            Err(PairingError::DeviceRevoked) => {
                return JsonRpcResponse::error(
                    req_id,
                    UNAUTHORIZED,
                    "Device access has been revoked by Desktop administrator",
                    None,
                );
            }
            Err(e) => {
                return JsonRpcResponse::error(req_id, UNAUTHORIZED, e.to_string(), None);
            }
        }

        match req.method.as_str() {
            "device.status.report" => {
                match serde_json::from_value::<MobileDeviceStatus>(params) {
                    Ok(status) => {
                        let summary = DeviceStorageSummaryRecord {
                            device_id: status.device_id.clone(),
                            total_bytes: status.storage_total_bytes,
                            free_bytes: status.storage_free_bytes,
                            indexed_items_count: 0,
                            updated_at: status.timestamp,
                        };
                        let _ = self.db.set_device_storage_summary(&summary);
                        JsonRpcResponse::success(req_id, serde_json::json!({ "acknowledged": true }))
                    }
                    Err(e) => JsonRpcResponse::error(req_id, INVALID_REQUEST, e.to_string(), None),
                }
            }

            "device.approvals.list" => {
                let map = self.approvals.lock().await;
                let pending: Vec<ApprovalRecord> = map
                    .values()
                    .filter(|a| a.status == "pending" && !a.consumed)
                    .cloned()
                    .collect();
                JsonRpcResponse::success(req_id, serde_json::to_value(pending).unwrap_or_default())
            }

            "device.approvals.resolve" => {
                let ticket_id = params.get("ticket_id").and_then(|v| v.as_str()).unwrap_or("");
                let decision = params.get("decision").and_then(|v| v.as_str()).unwrap_or("");

                let mut map = self.approvals.lock().await;
                if let Some(record) = map.get_mut(ticket_id) {
                    if record.consumed {
                        return JsonRpcResponse::error(req_id, INVALID_REQUEST, "Approval ticket already consumed", None);
                    }
                    record.consumed = true;
                    record.status = if decision == "approve" { "approved".to_string() } else { "denied".to_string() };

                    self.event_bus.publish(SystemEvent::ApprovalResolved {
                        request_id: ticket_id.to_string(),
                        decision: record.status.clone(),
                    });

                    JsonRpcResponse::success(req_id, serde_json::json!({
                        "ticket_id": ticket_id,
                        "status": record.status,
                        "resolved_by_device": device_id
                    }))
                } else {
                    JsonRpcResponse::error(req_id, INVALID_REQUEST, "Approval ticket not found", None)
                }
            }

            "device.transfer.init" => {
                let transfer_id = params.get("transfer_id").and_then(|v| v.as_str()).unwrap_or("");
                let file_name = params.get("file_name").and_then(|v| v.as_str()).unwrap_or("");
                let file_size = params.get("file_size").and_then(|v| v.as_i64()).unwrap_or(0);
                let sha256 = params.get("sha256").and_then(|v| v.as_str()).unwrap_or("");

                match self.transfer.init_incoming_transfer(transfer_id, device_id, file_name, file_size, sha256) {
                    Ok(rec) => JsonRpcResponse::success(req_id, serde_json::to_value(rec).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "device.transfer.chunk" => {
                let transfer_id = params.get("transfer_id").and_then(|v| v.as_str()).unwrap_or("");
                let chunk_b64 = params.get("data_base64").and_then(|v| v.as_str()).unwrap_or("");
                let chunk_sha = params.get("chunk_sha256").and_then(|v| v.as_str()).unwrap_or("");

                // Base64 decode (or simple byte parse)
                let chunk_bytes = match base64_simple_decode(chunk_b64) {
                    Some(b) => b,
                    None => return JsonRpcResponse::error(req_id, INVALID_REQUEST, "Invalid base64 chunk payload", None),
                };

                match self.transfer.write_chunk(transfer_id, &chunk_bytes, chunk_sha) {
                    Ok(bytes) => JsonRpcResponse::success(req_id, serde_json::json!({ "bytes_transferred": bytes })),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            "device.transfer.finalize" => {
                let transfer_id = params.get("transfer_id").and_then(|v| v.as_str()).unwrap_or("");
                let dest_str = params.get("destination_path").and_then(|v| v.as_str()).unwrap_or("");

                match self.transfer.finalize_transfer(transfer_id, Path::new(dest_str)) {
                    Ok(op_res) => JsonRpcResponse::success(req_id, serde_json::to_value(op_res).unwrap_or_default()),
                    Err(e) => JsonRpcResponse::error(req_id, hime_ipc::protocol::INTERNAL_ERROR, e.to_string(), None),
                }
            }

            other => JsonRpcResponse::error(req_id, METHOD_NOT_FOUND, format!("Device method '{}' not found", other), None),
        }
    }
}

fn base64_simple_decode(input: &str) -> Option<Vec<u8>> {
    // Standard base64 decoding helper or byte conversion
    let clean = input.trim();
    if clean.is_empty() {
        return Some(Vec::new());
    }
    // Using simple standard hex or base64 chars
    let mut out = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0u8;

    for &b in clean.as_bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' | b'\r' | b'\n' => continue,
            _ => return None,
        };
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    Some(out)
}
