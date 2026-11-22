import React, { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { ShieldAlert } from "lucide-react";
import { ApprovalItem } from "../types";

export interface ApprovalBubbleProps {
	approval: ApprovalItem;
	onClick: () => void;
	pillWidth?: number;
}

export const ApprovalBubble: React.FC<ApprovalBubbleProps> = ({
	approval,
	onClick
}) => {
	const [remainingSecs, setRemainingSecs] = useState<number>(() => {
		return Math.max(0, Math.floor((approval.expires_at - Date.now()) / 1000));
	});

	useEffect(() => {
		const timer = setInterval(() => {
			const diff = Math.max(0, Math.floor((approval.expires_at - Date.now()) / 1000));
			setRemainingSecs(diff);
			if (diff <= 0) clearInterval(timer);
		}, 1000);
		return () => clearInterval(timer);
	}, [approval.expires_at]);

	return (
		<motion.button
			key={`approval-bubble-${approval.request_id}`}
			type="button"
			className="activity-bubble vertical-approval-bubble"
			aria-label={`Pending Approval: ${remainingSecs}s remaining`}
			style={{
				left: "50%",
				display: "flex",
				alignItems: "center",
				gap: "6px",
				background: "rgba(245, 158, 11, 0.16)",
				border: "1px solid rgba(245, 158, 11, 0.35)",
				color: "#fbbf24",
				padding: "0 10px",
				height: "28px",
				borderRadius: "14px",
				cursor: "pointer",
				fontSize: "11px",
				fontWeight: 600,
				position: "absolute",
				top: "42px",
				backdropFilter: "blur(12px)",
				boxShadow: "0 4px 12px rgba(245, 158, 11, 0.2)",
				zIndex: 99
			}}
			initial={{ opacity: 0, scale: 0.6, y: -8, x: "-50%" }}
			animate={{ opacity: 1, scale: 1, y: 0, x: "-50%" }}
			exit={{ opacity: 0, scale: 0.6, y: -8, x: "-50%" }}
			transition={{ type: "spring", stiffness: 450, damping: 30 }}
			onClick={(e) => {
				e.stopPropagation();
				onClick();
			}}
		>
			<ShieldAlert size={13} strokeWidth={2.4} style={{ color: "#fbbf24" }} />
			<span>{remainingSecs}s</span>
		</motion.button>
	);
};
