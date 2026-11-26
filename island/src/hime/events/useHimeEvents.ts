import { useEffect, useState, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { ApprovalItem, ApprovalResolutionResult, DownloadEventReport, HimeCoreStatus } from "../types";
import { getCoreStatus, listApprovals, resolveApproval as apiResolveApproval } from "../api";

export interface UseHimeEventsOptions {
	onDownloadDetected?: (report: DownloadEventReport) => void;
	onApprovalResolved?: (result: ApprovalResolutionResult) => void;
}

export function useHimeEvents(options?: UseHimeEventsOptions) {
	const optionsRef = useRef(options);
	optionsRef.current = options;

	const [status, setStatus] = useState<HimeCoreStatus>({
		connection_state: "Offline",
		is_online: false,
		core_version: null,
		uptime_secs: 0,
		active_approvals_count: 0,
		last_ping_ms: null
	});
	const [approvals, setApprovals] = useState<ApprovalItem[]>([]);
	const [approvalJustResolved, setApprovalJustResolved] = useState(false);
	const resolvedTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

	const markApprovalResolved = useCallback(() => {
		if (resolvedTimerRef.current) {
			clearTimeout(resolvedTimerRef.current);
		}
		setApprovalJustResolved(true);
		resolvedTimerRef.current = setTimeout(() => {
			setApprovalJustResolved(false);
			resolvedTimerRef.current = null;
		}, 3000);
	}, []);

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
		// STAGE H2: Event-driven IPC bridge (zero idle polling, no setInterval, zero churn on mount)

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
					markApprovalResolved();
					if (optionsRef.current?.onApprovalResolved) {
						optionsRef.current.onApprovalResolved(event.payload);
					}
				}
			);

			unlistenDownload = await listen<DownloadEventReport>(
				"hime-download-detected",
				(event) => {
					if (optionsRef.current?.onDownloadDetected) {
						optionsRef.current.onDownloadDetected(event.payload);
					}
				}
			);
		};

		setupListeners();

		return () => {
			if (unlistenStatus) unlistenStatus();
			if (unlistenPending) unlistenPending();
			if (unlistenResolved) unlistenResolved();
			if (unlistenDownload) unlistenDownload();
			if (resolvedTimerRef.current) {
				clearTimeout(resolvedTimerRef.current);
			}
		};
	}, [refresh, markApprovalResolved]);

	const handleResolve = async (requestId: string, decision: "approve" | "deny") => {
		try {
			await apiResolveApproval(requestId, decision);
			setApprovals((prev) => prev.filter((a) => a.request_id !== requestId));
			markApprovalResolved();
			refresh();
		} catch (err) {
			console.error("[Hime] Resolve approval error:", err);
		}
	};

	return {
		status,
		approvals,
		approvalJustResolved,
		refresh,
		resolveApproval: handleResolve
	};
}
