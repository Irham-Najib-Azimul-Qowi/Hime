import React from "react";
import { X, Flame, MessageSquare } from "lucide-react";
import { useFocusSession } from "./useFocusSession";
import { HimeEyes } from "../ui/HimeEyes";
import { Pill } from "../ui/Pill";
import { Button } from "../ui/Button";
import type { ReasonPill } from "./types";

export interface HimeFocusPanelProps {
	onClose?: () => void;
	onOpenChat?: () => void;
}

const REASONS: ReasonPill[] = ["Sibuk", "Sakit", "Libur", "Lainnya"];

export const HimeFocusPanel: React.FC<HimeFocusPanelProps> = ({ onClose, onOpenChat }) => {
	const {
		activeRoutine,
		sessionState,
		remainingSecs,
		durationMin,
		progressPct,
		snoozeCount,
		maxSnoozes,
		isSelectingReason,
		startSession,
		completeSession,
		promptSnooze,
		confirmSnooze,
		promptSkip,
		confirmSkip,
		resetSession,
		setIsSelectingReason,
	} = useFocusSession();

	// Format mm:ss
	const minutes = Math.floor(remainingSecs / 60);
	const seconds = remainingSecs % 60;
	const formattedTime = `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;

	// Eyes expression
	const eyesExpression =
		sessionState === "active"
			? "fokus"
			: sessionState === "completed"
			? "senang"
			: sessionState === "skipped"
			? "sedih"
			: sessionState === "snoozed"
			? "kesal"
			: "senang";

	// Circular progress ring math
	const radius = 38;
	const circumference = 2 * Math.PI * radius;
	const strokeDashoffset = circumference - (Math.min(100, Math.max(0, progressPct)) / 100) * circumference;

	return (
		<div
			style={{
				display: "flex",
				flexDirection: "column",
				height: "100%",
				minHeight: 220,
				color: "var(--text-1, #F4F6FA)",
				fontFamily: "inherit",
			}}
		>
			{/* Header */}
			<div
				style={{
					display: "flex",
					alignItems: "center",
					justifyContent: "space-between",
					padding: "8px 12px 6px",
					borderBottom: "1px solid var(--line, #262A34)",
				}}
			>
				<div style={{ display: "flex", alignItems: "center", gap: 8 }}>
					<span style={{ fontSize: 13, fontWeight: 600 }}>Sesi Fokus</span>
					{activeRoutine.streak && activeRoutine.streak.count > 0 && (
						<Pill
							label={`${activeRoutine.streak.count} hari`}
							accent="amber"
							variant="tint"
							icon={<Flame size={11} />}
						/>
					)}
				</div>

				<div style={{ display: "flex", alignItems: "center", gap: 6 }}>
					{onOpenChat && (
						<button
							type="button"
							onClick={onOpenChat}
							title="Chat"
							style={{
								background: "none",
								border: "none",
								borderRadius: 6,
								width: 22,
								height: 22,
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
								cursor: "pointer",
								color: "var(--text-2, #A7AEBB)",
							}}
						>
							<MessageSquare size={13} />
						</button>
					)}
					<HimeEyes expression={eyesExpression} size={20} />
					{onClose && (
						<button
							type="button"
							onClick={onClose}
							title="Tutup"
							style={{
								background: "none",
								border: "none",
								borderRadius: 6,
								width: 22,
								height: 22,
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
								cursor: "pointer",
								color: "var(--text-2, #A7AEBB)",
							}}
						>
							<X size={14} />
						</button>
					)}
				</div>
			</div>

			{/* Center Ring & Timer Display */}
			<div
				style={{
					display: "flex",
					flexDirection: "column",
					alignItems: "center",
					justifyContent: "center",
					padding: "12px 16px 8px",
					gap: 8,
				}}
			>
				<div style={{ position: "relative", width: 92, height: 92 }}>
					<svg width="92" height="92" viewBox="0 0 92 92" style={{ transform: "rotate(-90deg)" }}>
						<circle
							cx="46"
							cy="46"
							r={radius}
							fill="none"
							stroke="var(--line, #262A34)"
							strokeWidth="4"
						/>
						<circle
							cx="46"
							cy="46"
							r={radius}
							fill="none"
							stroke={sessionState === "completed" ? "var(--accent-lime, #A8EE4B)" : "var(--accent-mint, #34E3A6)"}
							strokeWidth="4"
							strokeDasharray={circumference}
							strokeDashoffset={strokeDashoffset}
							strokeLinecap="round"
							style={{ transition: "stroke-dashoffset 0.3s ease" }}
						/>
					</svg>

					<div
						style={{
							position: "absolute",
							inset: 0,
							display: "flex",
							flexDirection: "column",
							alignItems: "center",
							justifyContent: "center",
						}}
					>
						<span style={{ fontSize: 18, fontWeight: 700, letterSpacing: "-0.5px" }}>
							{formattedTime}
						</span>
						<span style={{ fontSize: 10, color: "var(--text-3, #6F7785)" }}>
							{sessionState === "active" ? "tersisa" : `${durationMin} mnt`}
						</span>
					</div>
				</div>

				<div style={{ display: "flex", alignItems: "center", gap: 6, marginTop: 2 }}>
					<span style={{ fontSize: 13, fontWeight: 600 }}>{activeRoutine.title}</span>
					<Pill label={activeRoutine.category} accent="sky" variant="tint" />
					<Pill label={activeRoutine.enforcement} accent="coral" variant="outline" />
				</div>
			</div>

			{/* Action Area & Reason Selection */}
			<div
				style={{
					marginTop: "auto",
					padding: "8px 12px 10px",
					display: "flex",
					flexDirection: "column",
					gap: 6,
				}}
			>
				{isSelectingReason ? (
					<div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
						<div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
							<span style={{ fontSize: 11, color: "var(--text-2, #A7AEBB)" }}>Pilih alasan:</span>
							<Button size="sm" variant="ghost" onClick={() => setIsSelectingReason(null)}>
								Batal
							</Button>
						</div>
						<div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
							{REASONS.map((r) => (
								<Button
									key={r}
									size="sm"
									variant="secondary"
									onClick={() => (isSelectingReason === "skip" ? confirmSkip(r) : confirmSnooze(r))}
								>
									{r}
								</Button>
							))}
						</div>
					</div>
				) : (
					<div style={{ display: "flex", gap: 6, justifyContent: "center" }}>
						{sessionState === "idle" && (
							<Button variant="primary" size="md" onClick={startSession} style={{ flex: 1 }}>
								Mulai
							</Button>
						)}

						{sessionState === "active" && (
							<>
								<Button variant="primary" size="md" onClick={completeSession} style={{ flex: 1 }}>
									Selesai
								</Button>
								<Button
									variant="secondary"
									size="md"
									disabled={activeRoutine.enforcement === "tegas" && snoozeCount >= maxSnoozes}
									onClick={promptSnooze}
								>
									Tunda 10m
								</Button>
								<Button variant="ghost" size="md" onClick={promptSkip}>
									Lewati
								</Button>
							</>
						)}

						{sessionState === "completed" && (
							<Button variant="primary" size="md" onClick={resetSession} style={{ flex: 1 }}>
								Selesai
							</Button>
						)}

						{(sessionState === "skipped" || sessionState === "snoozed") && (
							<Button variant="primary" size="md" onClick={startSession} style={{ flex: 1 }}>
								Mulai
							</Button>
						)}
					</div>
				)}
			</div>
		</div>
	);
};
