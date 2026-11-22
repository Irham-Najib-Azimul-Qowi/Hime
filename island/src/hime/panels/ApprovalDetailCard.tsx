import React, { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { ShieldAlert, Check, X, Clock } from "lucide-react";
import { ApprovalItem } from "../types";

interface ApprovalDetailCardProps {
	approval: ApprovalItem;
	onResolve: (requestId: string, decision: "approve" | "deny") => void;
	onClose: () => void;
}

export const ApprovalDetailCard: React.FC<ApprovalDetailCardProps> = ({
	approval,
	onResolve,
	onClose
}) => {
	const [remainingSecs, setRemainingSecs] = useState<number>(() => {
		return Math.max(0, Math.floor((approval.expires_at - Date.now()) / 1000));
	});
	const [resolving, setResolving] = useState<string | null>(null);

	useEffect(() => {
		const timer = setInterval(() => {
			const diff = Math.max(0, Math.floor((approval.expires_at - Date.now()) / 1000));
			setRemainingSecs(diff);
			if (diff <= 0) {
				clearInterval(timer);
				onClose();
			}
		}, 1000);
		return () => clearInterval(timer);
	}, [approval.expires_at, onClose]);

	const handleAction = async (decision: "approve" | "deny") => {
		setResolving(decision);
		try {
			await onResolve(approval.request_id, decision);
		} finally {
			setTimeout(() => {
				onClose();
			}, 400);
		}
	};

	return (
		<motion.div
			className="approval-detail-card"
			initial={{ opacity: 0, scale: 0.98 }}
			animate={{ opacity: 1, scale: 1 }}
			exit={{ opacity: 0, scale: 0.98 }}
			transition={{ type: "spring", stiffness: 450, damping: 30 }}
			style={{
				display: "flex",
				flexDirection: "column",
				justifyContent: "space-between",
				height: "100%",
				width: "100%",
				padding: "14px 16px",
				color: "#f5f5f5",
				fontFamily: "inherit",
				userSelect: "none",
				boxSizing: "border-box"
			}}
			onClick={(e) => e.stopPropagation()}
		>
			{/* Header */}
			<div>
				<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
					<div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
						<ShieldAlert size={14} style={{ color: "#fbbf24" }} />
						<span style={{ fontSize: "12px", fontWeight: 600, color: "#fbbf24" }}>
							Izin Operasi
						</span>
					</div>
					<div
						style={{
							display: "flex",
							alignItems: "center",
							gap: "4px",
							fontSize: "11px",
							color: remainingSecs <= 10 ? "#ef4444" : "#a3a3a3",
							fontWeight: 500
						}}
					>
						<Clock size={11} />
						<span>{remainingSecs}s</span>
					</div>
				</div>

				{/* Description & Target */}
				<div style={{ marginTop: "8px" }}>
					<div style={{ fontSize: "12px", fontWeight: 500, color: "#ffffff", lineHeight: 1.3 }}>
						{approval.description}
					</div>
					{approval.path && (
						<div
							style={{
								fontSize: "10px",
								color: "#737373",
								fontFamily: "monospace",
								marginTop: "4px",
								overflow: "hidden",
								textOverflow: "ellipsis",
								whiteSpace: "nowrap"
							}}
						>
							Target: {approval.path}
						</div>
					)}
				</div>
			</div>

			{/* Action Buttons */}
			<div style={{ display: "flex", gap: "8px", marginTop: "12px" }}>
				<button
					type="button"
					disabled={resolving !== null}
					onClick={() => handleAction("deny")}
					style={{
						flex: 1,
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
						gap: "6px",
						background: "rgba(239, 68, 68, 0.12)",
						border: "1px solid rgba(239, 68, 68, 0.25)",
						color: "#f87171",
						borderRadius: "8px",
						padding: "7px 12px",
						fontSize: "11px",
						fontWeight: 600,
						cursor: resolving !== null ? "not-allowed" : "pointer",
						transition: "background 0.15s ease"
					}}
				>
					<X size={12} strokeWidth={2.5} />
					<span>Tolak</span>
				</button>

				<button
					type="button"
					disabled={resolving !== null}
					onClick={() => handleAction("approve")}
					style={{
						flex: 1,
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
						gap: "6px",
						background: "rgba(16, 185, 129, 0.15)",
						border: "1px solid rgba(16, 185, 129, 0.35)",
						color: "#34d399",
						borderRadius: "8px",
						padding: "7px 12px",
						fontSize: "11px",
						fontWeight: 600,
						cursor: resolving !== null ? "not-allowed" : "pointer",
						transition: "background 0.15s ease"
					}}
				>
					<Check size={12} strokeWidth={2.5} />
					<span>Setujui</span>
				</button>
			</div>
		</motion.div>
	);
};
