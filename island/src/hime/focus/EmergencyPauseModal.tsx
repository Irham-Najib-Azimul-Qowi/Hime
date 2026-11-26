import React, { useState } from "react";
import { ShieldAlert, X } from "lucide-react";
import { Button } from "../ui/Button";
import { HimeEyes } from "../ui/HimeEyes";
import { triggerEmergencyPause } from "../api";
import type { ReasonPill } from "./types";

export interface EmergencyPauseModalProps {
	isOpen: boolean;
	onClose: () => void;
	onPaused?: (durationMin: number, reason: string) => void;
}

const DURATIONS = [15, 30, 60];
const REASONS: ReasonPill[] = ["Sibuk", "Sakit", "Libur", "Lainnya"];

export const EmergencyPauseModal: React.FC<EmergencyPauseModalProps> = ({
	isOpen,
	onClose,
	onPaused,
}) => {
	const [selectedDuration, setSelectedDuration] = useState<number>(15);
	const [selectedReason, setSelectedReason] = useState<ReasonPill>("Sibuk");
	const [isSubmitting, setIsSubmitting] = useState(false);

	if (!isOpen) return null;

	const handleActivate = async () => {
		setIsSubmitting(true);
		try {
			await triggerEmergencyPause(selectedDuration, selectedReason);
			if (onPaused) onPaused(selectedDuration, selectedReason);
			onClose();
		} catch {
			if (onPaused) onPaused(selectedDuration, selectedReason);
			onClose();
		} finally {
			setIsSubmitting(false);
		}
	};

	return (
		<div
			style={{
				position: "fixed",
				inset: 0,
				zIndex: 9999,
				backgroundColor: "rgba(0, 0, 0, 0.65)",
				backdropFilter: "blur(4px)",
				display: "flex",
				alignItems: "center",
				justifyContent: "center",
				padding: 16,
			}}
			onClick={onClose}
		>
			<div
				style={{
					backgroundColor: "var(--surface-2, #14161C)",
					border: "1px solid var(--line, #262A34)",
					borderRadius: "var(--radius-card, 14px)",
					width: 340,
					maxWidth: "100%",
					padding: "16px 18px",
					display: "flex",
					flexDirection: "column",
					gap: 12,
					color: "var(--text-1, #F4F6FA)",
					boxShadow: "0 16px 40px rgba(0,0,0,0.5)",
				}}
				onClick={(e) => e.stopPropagation()}
			>
				{/* Header */}
				<div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
					<div style={{ display: "flex", alignItems: "center", gap: 8 }}>
						<ShieldAlert size={18} color="var(--accent-coral, #FF7A59)" />
						<span style={{ fontSize: 14, fontWeight: 600 }}>Jeda Darurat</span>
					</div>
					<div style={{ display: "flex", alignItems: "center", gap: 6 }}>
						<HimeEyes expression="khawatir" size={20} />
						<button
							type="button"
							onClick={onClose}
							style={{
								background: "none",
								border: "none",
								color: "var(--text-2, #A7AEBB)",
								cursor: "pointer",
								padding: 2,
								display: "flex",
							}}
						>
							<X size={16} />
						</button>
					</div>
				</div>

				<span style={{ fontSize: 12, color: "var(--text-2, #A7AEBB)" }}>
					Hentikan semua penegakan
				</span>

				{/* Duration Pills */}
				<div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
					<span style={{ fontSize: 11, color: "var(--text-3, #6F7785)" }}>Durasi</span>
					<div style={{ display: "flex", gap: 6 }}>
						{DURATIONS.map((dur) => (
							<Button
								key={dur}
								size="sm"
								variant={selectedDuration === dur ? "primary" : "ghost"}
								onClick={() => setSelectedDuration(dur)}
								style={{ flex: 1 }}
							>
								{`${dur}m`}
							</Button>
						))}
					</div>
				</div>

				{/* Reason Pills */}
				<div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
					<span style={{ fontSize: 11, color: "var(--text-3, #6F7785)" }}>Alasan</span>
					<div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
						{REASONS.map((r) => (
							<Button
								key={r}
								size="sm"
								variant={selectedReason === r ? "secondary" : "ghost"}
								onClick={() => setSelectedReason(r)}
							>
								{r}
							</Button>
						))}
					</div>
				</div>

				{/* Footer Controls */}
				<div style={{ display: "flex", gap: 8, marginTop: 4 }}>
					<Button
						variant="danger"
						size="md"
						disabled={isSubmitting}
						onClick={handleActivate}
						style={{ flex: 1 }}
					>
						Aktifkan
					</Button>
					<Button variant="ghost" size="md" onClick={onClose}>
						Batal
					</Button>
				</div>
			</div>
		</div>
	);
};
