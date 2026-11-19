export type CoreConnectionState = "Offline" | "Connecting" | "Online" | "Error";

export interface HimeCoreStatus {
	connection_state: CoreConnectionState;
	is_online: boolean;
	core_version: string | null;
	uptime_secs: number;
	active_approvals_count: number;
	last_ping_ms: number | null;
}

export interface ApprovalItem {
	request_id: string;
	action_type: string;
	path: string;
	description: string;
	status: "pending" | "approved" | "denied" | "expired";
	created_at: number;
	expires_at: number;
}

export interface ApprovalResolutionResult {
	request_id: string;
	decision: "approve" | "deny";
	status: string;
}

export interface DownloadEventReport {
	download_path: string;
	file_name: string;
	status:
		| "InProgress"
		| "NewUniqueFile"
		| { ExactDuplicate: { existing_path: string } }
		| { NewVersionCandidate: { existing_path: string; suggested_action: string } };
	notification_prompt: string | null;
	timestamp_ms: number;
}

export interface DocumentOpResult {
	status: string;
	output_path: string;
	file_size_bytes: number;
	metadata?: Record<string, any>;
}
