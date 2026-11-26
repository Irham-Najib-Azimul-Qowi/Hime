package ai.hime.mobile.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class QrPairingPayload(
    @SerialName("server_address") val serverAddress: String,
    @SerialName("server_port") val serverPort: Int,
    @SerialName("pairing_token") val pairingToken: String,
    @SerialName("confirmation_code") val confirmationCode: String,
    @SerialName("ca_fingerprint") val caFingerprint: String,
    @SerialName("expires_at") val expiresAt: Long
)

@Serializable
data class SecurityPosture(
    @SerialName("is_device_secure") val isDeviceSecure: Boolean,
    @SerialName("security_patch_level") val securityPatchLevel: String,
    @SerialName("storage_encrypted") val storageEncrypted: Boolean,
    @SerialName("unknown_sources_enabled") val unknownSourcesEnabled: Boolean,
    @SerialName("os_version") val osVersion: String
)

@Serializable
data class MobileDeviceStatus(
    @SerialName("device_id") val deviceId: String,
    @SerialName("battery_percent") val batteryPercent: Int,
    @SerialName("is_charging") val isCharging: Boolean,
    @SerialName("storage_free_bytes") val storageFreeBytes: Long,
    @SerialName("storage_total_bytes") val storageTotalBytes: Long,
    @SerialName("security_posture") val securityPosture: SecurityPosture,
    @SerialName("timestamp") val timestamp: Long
)

@Serializable
data class RemoteApprovalSubmission(
    @SerialName("ticket_id") val ticketId: String,
    @SerialName("decision") val decision: String, // "approve" or "deny"
    @SerialName("device_id") val deviceId: String,
    @SerialName("signed_token") val signedToken: String
)

@Serializable
data class RemoteChatSubmission(
    @SerialName("conversation_id") val conversationId: String,
    @SerialName("prompt") val prompt: String,
    @SerialName("device_id") val deviceId: String
)

@Serializable
data class TransferChunkPayload(
    @SerialName("transfer_id") val transferId: String,
    @SerialName("chunk_index") val chunkIndex: Int,
    @SerialName("total_chunks") val totalChunks: Int,
    @SerialName("data_base64") val dataBase64: String,
    @SerialName("chunk_sha256") val chunkSha256: String
)

@Serializable
data class ApprovalItem(
    val id: String,
    val actionType: String,
    val path: String,
    val description: String,
    val expiresAt: Long
)

