import { invoke } from "@tauri-apps/api/core";
import { ApprovalItem, DocumentOpResult, DownloadEventReport, HimeCoreStatus } from "../types";

export const getCoreStatus = async (): Promise<HimeCoreStatus> => {
	return await invoke<HimeCoreStatus>("hime_core_status");
};

export const pingCore = async (): Promise<number> => {
	return await invoke<number>("hime_ping");
};

export const listApprovals = async (): Promise<ApprovalItem[]> => {
	return await invoke<ApprovalItem[]>("hime_approval_list");
};

export const resolveApproval = async (
	requestId: string,
	decision: "approve" | "deny"
): Promise<string> => {
	return await invoke<string>("hime_approval_resolve", {
		requestId,
		decision
	});
};

export const devTriggerApproval = async (
	actionType: string = "file.delete",
	path: string = "X:/Downloads/temp_artifact.tmp",
	description: string = "Permintaan persetujuan: Hapus file sementara dari sandbox"
): Promise<ApprovalItem> => {
	return await invoke<ApprovalItem>("hime_dev_trigger_approval", {
		actionType,
		path,
		description
	});
};

export interface ChatResponse {
	status: string;
	conversation_id: string;
	response: string;
}

export const sendChatMessage = async (
	prompt: string,
	model?: string
): Promise<ChatResponse> => {
	return await invoke<ChatResponse>("hime_send_chat", {
		prompt,
		model
	});
};

export const storeCredential = async (
	provider: string,
	secret: string
): Promise<{ status: string; provider: string; vault: string }> => {
	return await invoke("hime_store_credential", { provider, secret });
};

export const pollNewDownloads = async (): Promise<DownloadEventReport[]> => {
	return await invoke<DownloadEventReport[]>("hime_poll_new_downloads");
};

export const inspectDownload = async (
	downloadPath: string
): Promise<DownloadEventReport> => {
	return await invoke<DownloadEventReport>("hime_inspect_download", {
		downloadPath
	});
};

export const replaceAndArchiveDownload = async (
	newDownload: string,
	existingFile: string
): Promise<{ status: string; operation_id: string }> => {
	return await invoke("hime_replace_and_archive_download", {
		newDownload,
		existingFile
	});
};

export const createAcademicReport = async (
	title: string,
	markdownBody: string,
	destinationPath?: string
): Promise<DocumentOpResult> => {
	return await invoke<DocumentOpResult>("hime_create_academic_report", {
		title,
		markdownBody,
		destinationPath
	});
};

export const mergePdfs = async (
	sourcePaths: string[],
	destinationPath: string
): Promise<DocumentOpResult> => {
	return await invoke<DocumentOpResult>("hime_merge_pdfs", {
		sourcePaths,
		destinationPath
	});
};
