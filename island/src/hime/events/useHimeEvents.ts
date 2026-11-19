import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { ApprovalItem, ApprovalResolutionResult, DownloadEventReport, HimeCoreStatus } from "../types";
import { getCoreStatus, listApprovals, resolveApproval as apiResolveApproval } from "../api";

export interface UseHimeEventsOptions {
	onDownloadDetected?: (report: DownloadEventReport) => void;
}

export function useHimeEvents(options?: UseHimeEventsOptions) {
	const [status, setStatus] = useState<HimeCoreStatus>({
		connection_state: "Offline",
		is_online: false,
		core_version: null,
		uptime_secs: 0,
		active_approvals_count: 0,
		last_ping_ms: null
	});
	const [approvals, setApprovals] = useState<ApprovalItem[]>([]);

	const refresh = useCallback(async () => {
		try {
			const currentStatus = await getCoreStatus();
			setStatus(currentStatus);
			const currentApprovals = await listApprovals();
			setApprovals(currentApprovals);
		} catch {
			setStatus((prev) => ({
				...prev,
				connection_state: "Offline",
				is_online: false
			}));
		}
	}, []);

	useEffect(() => {
		refresh();
		const pollTimer = setInterval(refresh, 3000);

		let unlistenStatus: (() => void) | undefined;
		let unlistenPending: (() => void) | undefined;
		let unlistenResolved: (() => void) | undefined;
		let unlistenDownload: (() => void) | undefined;

		const setupListeners = async () => {
			unlistenStatus = await listen<HimeCoreStatus>("hime-core-status", (event) => {
				setStatus(event.payload);
			});

			unlistenPending = await listen<ApprovalItem>("hime-approval-pending", (event) => {
				setApprovals((prev) => {
					const exists = prev.some((a) => a.request_id === event.payload.request_id);
					if (exists) return prev;
					return [event.payload, ...prev];
				});
			});

			unlistenResolved = await listen<ApprovalResolutionResult>(
				"hime-approval-resolved",
				(event) => {
					setApprovals((prev) => prev.filter((a) => a.request_id !== event.payload.request_id));
				}
			);

			unlistenDownload = await listen<DownloadEventReport>(
				"hime-download-detected",
				(event) => {
					if (options?.onDownloadDetected) {
						options.onDownloadDetected(event.payload);
					}
				}
			);
		};

		setupListeners();

		return () => {
			clearInterval(pollTimer);
			if (unlistenStatus) unlistenStatus();
			if (unlistenPending) unlistenPending();
			if (unlistenResolved) unlistenResolved();
			if (unlistenDownload) unlistenDownload();
		};
	}, [refresh, options]);

	const handleResolve = async (requestId: string, decision: "approve" | "deny") => {
		try {
			await apiResolveApproval(requestId, decision);
			setApprovals((prev) => prev.filter((a) => a.request_id !== requestId));
			refresh();
		} catch (err) {
			console.error("[Hime] Resolve approval error:", err);
		}
	};

	return {
		status,
		approvals,
		refresh,
		resolveApproval: handleResolve
	};
}
