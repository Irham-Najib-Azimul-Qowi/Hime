import React, { useState, useEffect } from "react";
import { AlertTriangle } from "lucide-react";
import { Button } from "../ui/Button";
import { HimeEyes } from "../ui/HimeEyes";

export interface EnforcementAlertModalProps {
	isOpen: boolean;
	targetAppName: string;
	onSnooze5m?: () => void;
	onEmergencyPause?: () => void;
	onCountdownExpired?: () => void;
	hasUnsavedChangesModal?: boolean;
	snoozeCount?: number;
}

export const EnforcementAlertModal: React.FC<EnforcementAlertModalProps> = ({
	isOpen,
	targetAppName,
	onSnooze5m,
	onEmergencyPause,
	onCountdownExpired,
	hasUnsavedChangesModal = false,
	snoozeCount = 0,
}) => {
	const [countdown, setCountdown] = useState(10);

	useEffect(() => {
		if (!isOpen || hasUnsavedChangesModal) return;
		setCountdown(10);
		const interval = setInterval(() => {
			setCountdown((prev) => {
				if (prev <= 1) {
					clearInterval(interval);
					if (onCountdownExpired) onCountdownExpired();
					return 0;
				}
				return prev - 1;
			});
		}, 1000);

		return () => clearInterval(interval);
	}, [isOpen, hasUnsavedChangesModal]);

	if (!isOpen) return null;

	const maxSnoozesReached = snoozeCount >= 2;

	return (
		<div
			style={{
				position: "fixed",
				top: 48,
				left: "50%",
				transform: "translateX(-50%)",
				zIndex: 10000,
				width: 320,
				backgroundColor: "var(--surface-2, #14161C)",
				border: "1px solid var(--accent-red, #FF4D5E)",
				borderRadius: "var(--radius-card, 14px)",
				padding: "12px 14px",
				boxShadow: "0 12px 32px rgba(255, 77, 94, 0.2)",
				display: "flex",
				flexDirection: "column",
				gap: 8,
				color: "var(--text-1, #F4F6FA)",
				fontFamily: "inherit",
			}}
		>
			{/* Top Bar */}
			<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
				<div style={{ display: "flex", alignItems: "center", gap: 6 }}>
					<AlertTriangle size={16} color="var(--accent-red, #FF4D5E)" />
					<span style={{ fontSize: 13, fontWeight: 600 }}>
						{hasUnsavedChangesModal ? "Selesaikan dulu" : "Peringatan"}
					</span>
				</div>
				<HimeEyes expression={hasUnsavedChangesModal ? "khawatir" : "kesal"} size={20} />
			</div>

			{/* Description */}
			<span style={{ fontSize: 12, color: "var(--text-2, #A7AEBB)" }}>
				{hasUnsavedChangesModal
					? `Simpan pekerjaan di ${targetAppName}`
					: `Menutup ${targetAppName} dalam ${countdown}s`}
			</span>

			{/* Countdown Progress Bar */}
			{!hasUnsavedChangesModal && (
				<div
					style={{
						height: 4,
						width: "100%",
						backgroundColor: "var(--line, #262A34)",
						borderRadius: 2,
						overflow: "hidden",
					}}
				>
					<div
						style={{
							height: "100%",
							width: `${(countdown / 10) * 100}%`,
							backgroundColor: "var(--accent-red, #FF4D5E)",
							transition: "width 1s linear",
						}}
					/>
				</div>
			)}

			{/* Actions */}
			<div style={{ display: "flex", gap: 6, marginTop: 2 }}>
				<Button
					size="sm"
					variant="secondary"
					disabled={maxSnoozesReached}
					onClick={onSnooze5m}
					style={{ flex: 1 }}
				>
					Tunda 5m
				</Button>
				<Button
					size="sm"
					variant="danger"
					onClick={onEmergencyPause}
					style={{ flex: 1 }}
				>
					Jeda
				</Button>
			</div>
		</div>
	);
};
