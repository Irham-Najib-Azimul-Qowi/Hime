import React, { useState } from "react";
import { motion } from "framer-motion";
import {
	Sun,
	Moon,
	Copy,
	HardDrive,
	CheckCircle2,
	Calendar
} from "lucide-react";
import { ProactiveItem } from "./types";
import { HimeEyes } from "../ui/HimeEyes";

export interface ProactiveCardProps {
	item: ProactiveItem;
	onAction?: (actionId: string, remember?: boolean) => void;
	onClose: () => void;
}

export const ProactiveCard: React.FC<ProactiveCardProps> = ({ item, onAction, onClose }) => {
	const [rememberRule, setRememberRule] = useState(false);

	// Dedicated Morning Briefing Card
	if (item.situationId === "morning_briefing") {
		return (
			<motion.div
				className="proactive-card morning-briefing-card"
				initial={{ opacity: 0, scale: 0.95, y: -6 }}
				animate={{ opacity: 1, scale: 1, y: 0 }}
				exit={{ opacity: 0, scale: 0.95, y: -6 }}
				transition={{ type: "spring", stiffness: 400, damping: 28 }}
				style={{
					background: "linear-gradient(135deg, rgba(254, 240, 138, 0.12), rgba(217, 119, 6, 0.08))",
					border: "1px solid rgba(250, 204, 21, 0.3)",
					borderRadius: "16px",
					padding: "14px 16px",
					color: "#f8fafc",
					backdropFilter: "blur(20px)",
					width: "320px",
					boxShadow: "0 8px 24px rgba(0, 0, 0, 0.35)"
				}}
			>
				{/* Header */}
				<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "10px" }}>
					<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
						<div
							style={{
								width: "28px",
								height: "28px",
								borderRadius: "8px",
								background: "rgba(250, 204, 21, 0.2)",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
								color: "#facc15"
							}}
						>
							<Sun size={16} />
						</div>
						<div>
							<div style={{ fontSize: "13px", fontWeight: 700, color: "#fef08a" }}>
								{item.title.slice(0, 16)}
							</div>
							<div style={{ fontSize: "11px", color: "#94a3b8" }}>
								{item.description.slice(0, 36)}
							</div>
						</div>
					</div>
					<HimeEyes expression="senang" size={22} />
				</div>

				{/* 3 Tasks Today & Study Block */}
				<div style={{ display: "flex", flexDirection: "column", gap: "6px", marginBottom: "12px" }}>
					<div
						style={{
							display: "flex",
							alignItems: "center",
							gap: "6px",
							fontSize: "11px",
							color: "#e2e8f0",
							background: "rgba(255, 255, 255, 0.05)",
							padding: "5px 8px",
							borderRadius: "6px"
						}}
					>
						<CheckCircle2 size={12} color="#38bdf8" />
						<span>1. Selesaikan laporan analisis</span>
					</div>
					<div
						style={{
							display: "flex",
							alignItems: "center",
							gap: "6px",
							fontSize: "11px",
							color: "#e2e8f0",
							background: "rgba(255, 255, 255, 0.05)",
							padding: "5px 8px",
							borderRadius: "6px"
						}}
					>
						<CheckCircle2 size={12} color="#38bdf8" />
						<span>2. Review arsitektur Hime</span>
					</div>
					<div
						style={{
							display: "flex",
							alignItems: "center",
							gap: "6px",
							fontSize: "11px",
							color: "#e2e8f0",
							background: "rgba(255, 255, 255, 0.05)",
							padding: "5px 8px",
							borderRadius: "6px"
						}}
					>
						<CheckCircle2 size={12} color="#38bdf8" />
						<span>3. Uji coba modul proaktif</span>
					</div>
					<div
						style={{
							display: "flex",
							alignItems: "center",
							gap: "6px",
							fontSize: "11px",
							color: "#facc15",
							background: "rgba(250, 204, 21, 0.1)",
							border: "1px dashed rgba(250, 204, 21, 0.3)",
							padding: "5px 8px",
							borderRadius: "6px",
							fontWeight: 600
						}}
					>
						<Calendar size={12} />
						<span>Sesi Belajar: 19:00 - 20:30</span>
					</div>
				</div>

				{/* Actions */}
				<div style={{ display: "flex", justifyContent: "flex-end", gap: "6px" }}>
					<button
						type="button"
						onClick={onClose}
						style={{
							background: "rgba(255, 255, 255, 0.1)",
							border: "none",
							borderRadius: "6px",
							padding: "5px 12px",
							color: "#cbd5e1",
							fontSize: "11px",
							cursor: "pointer"
						}}
					>
						{"Tutup".slice(0, 10)}
					</button>
					<button
						type="button"
						onClick={() => onAction?.("view")}
						style={{
							background: "#facc15",
							border: "none",
							borderRadius: "6px",
							padding: "5px 14px",
							color: "#0f172a",
							fontSize: "11px",
							fontWeight: 700,
							cursor: "pointer"
						}}
					>
						{"Lihat".slice(0, 10)}
					</button>
				</div>
			</motion.div>
		);
	}

	// Dedicated Evening Recap Card
	if (item.situationId === "evening_recap") {
		return (
			<motion.div
				className="proactive-card evening-recap-card"
				initial={{ opacity: 0, scale: 0.95, y: -6 }}
				animate={{ opacity: 1, scale: 1, y: 0 }}
				exit={{ opacity: 0, scale: 0.95, y: -6 }}
				transition={{ type: "spring", stiffness: 400, damping: 28 }}
				style={{
					background: "linear-gradient(135deg, rgba(99, 102, 241, 0.14), rgba(59, 130, 246, 0.08))",
					border: "1px solid rgba(129, 140, 248, 0.35)",
					borderRadius: "16px",
					padding: "14px 16px",
					color: "#f8fafc",
					backdropFilter: "blur(20px)",
					width: "320px",
					boxShadow: "0 8px 24px rgba(0, 0, 0, 0.35)"
				}}
			>
				{/* Header */}
				<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "10px" }}>
					<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
						<div
							style={{
								width: "28px",
								height: "28px",
								borderRadius: "8px",
								background: "rgba(129, 140, 248, 0.2)",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
								color: "#a5b4fc"
							}}
						>
							<Moon size={16} />
						</div>
						<div>
							<div style={{ fontSize: "13px", fontWeight: 700, color: "#e0e7ff" }}>
								{item.title.slice(0, 16)}
							</div>
							<div style={{ fontSize: "11px", color: "#94a3b8" }}>
								{item.description.slice(0, 36)}
							</div>
						</div>
					</div>
					<HimeEyes expression="senang" size={22} />
				</div>

				{/* Recap details */}
				<div style={{ display: "flex", gap: "8px", marginBottom: "12px" }}>
					<div
						style={{
							flex: 1,
							background: "rgba(16, 185, 129, 0.12)",
							border: "1px solid rgba(16, 185, 129, 0.25)",
							borderRadius: "8px",
							padding: "8px",
							textAlign: "center"
						}}
					>
						<div style={{ fontSize: "16px", fontWeight: 700, color: "#34d399" }}>2</div>
						<div style={{ fontSize: "10px", color: "#a7f3d0" }}>Selesai</div>
					</div>
					<div
						style={{
							flex: 1,
							background: "rgba(245, 158, 11, 0.12)",
							border: "1px solid rgba(245, 158, 11, 0.25)",
							borderRadius: "8px",
							padding: "8px",
							textAlign: "center"
						}}
					>
						<div style={{ fontSize: "16px", fontWeight: 700, color: "#fbbf24" }}>1</div>
						<div style={{ fontSize: "10px", color: "#fde68a" }}>Geser Besok</div>
					</div>
				</div>

				{/* Actions */}
				<div style={{ display: "flex", justifyContent: "flex-end", gap: "6px" }}>
					<button
						type="button"
						onClick={onClose}
						style={{
							background: "rgba(255, 255, 255, 0.1)",
							border: "none",
							borderRadius: "6px",
							padding: "5px 12px",
							color: "#cbd5e1",
							fontSize: "11px",
							cursor: "pointer"
						}}
					>
						{"Tutup".slice(0, 10)}
					</button>
					<button
						type="button"
						onClick={() => onAction?.("view")}
						style={{
							background: "#818cf8",
							border: "none",
							borderRadius: "6px",
							padding: "5px 14px",
							color: "#0f172a",
							fontSize: "11px",
							fontWeight: 700,
							cursor: "pointer"
						}}
					>
						{"Lihat".slice(0, 10)}
					</button>
				</div>
			</motion.div>
		);
	}

	// Dedicated Duplicate Download Prompt Card
	if (item.situationId === "file_duplicate") {
		return (
			<motion.div
				className="proactive-card duplicate-download-card"
				initial={{ opacity: 0, scale: 0.95, y: -6 }}
				animate={{ opacity: 1, scale: 1, y: 0 }}
				exit={{ opacity: 0, scale: 0.95, y: -6 }}
				transition={{ type: "spring", stiffness: 400, damping: 28 }}
				style={{
					background: "rgba(30, 41, 59, 0.85)",
					border: "1px solid rgba(56, 189, 248, 0.35)",
					borderRadius: "16px",
					padding: "14px 16px",
					color: "#f8fafc",
					backdropFilter: "blur(20px)",
					width: "320px",
					boxShadow: "0 8px 24px rgba(0, 0, 0, 0.35)"
				}}
			>
				{/* Header */}
				<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "10px" }}>
					<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
						<div
							style={{
								width: "28px",
								height: "28px",
								borderRadius: "8px",
								background: "rgba(56, 189, 248, 0.2)",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
								color: "#38bdf8"
							}}
						>
							<Copy size={16} />
						</div>
						<div>
							<div style={{ fontSize: "13px", fontWeight: 700, color: "#38bdf8" }}>
								{item.title.slice(0, 16)}
							</div>
							<div style={{ fontSize: "11px", color: "#94a3b8" }}>
								{item.description.slice(0, 36)}
							</div>
						</div>
					</div>
					<HimeEyes expression="penasaran" size={22} />
				</div>

				{/* File details */}
				<div
					style={{
						background: "rgba(255, 255, 255, 0.05)",
						borderRadius: "8px",
						padding: "8px",
						fontSize: "11px",
						color: "#cbd5e1",
						marginBottom: "10px"
					}}
				>
					<div style={{ fontWeight: 600, color: "#f1f5f9" }}>
						{item.payload?.filename || "document-v2.pdf"}
					</div>
					<div style={{ fontSize: "10px", color: "#64748b" }}>
						Hash identik dengan berkas di Unduhan
					</div>
				</div>

				{/* "Ingat pilihan" checkbox */}
				<label
					style={{
						display: "flex",
						alignItems: "center",
						gap: "6px",
						fontSize: "11px",
						color: "#94a3b8",
						marginBottom: "12px",
						cursor: "pointer",
						userSelect: "none"
					}}
				>
					<input
						type="checkbox"
						checked={rememberRule}
						onChange={(e) => setRememberRule(e.target.checked)}
						style={{ accentColor: "#38bdf8" }}
					/>
					<span>Ingat pilihan untuk berkas sejenis</span>
				</label>

				{/* <= 3 Actions (default first) */}
				<div style={{ display: "flex", justifyContent: "flex-end", gap: "6px" }}>
					<button
						type="button"
						onClick={() => onAction?.("delete_new", rememberRule)}
						style={{
							background: "rgba(239, 68, 68, 0.15)",
							border: "1px solid rgba(239, 68, 68, 0.3)",
							borderRadius: "6px",
							padding: "5px 10px",
							color: "#f87171",
							fontSize: "11px",
							cursor: "pointer"
						}}
					>
						{"Hapus".slice(0, 10)}
					</button>
					<button
						type="button"
						onClick={() => onAction?.("keep_both", rememberRule)}
						style={{
							background: "rgba(255, 255, 255, 0.08)",
							border: "1px solid rgba(255, 255, 255, 0.12)",
							borderRadius: "6px",
							padding: "5px 10px",
							color: "#e2e8f0",
							fontSize: "11px",
							cursor: "pointer"
						}}
					>
						{"Simpan Dua".slice(0, 10)}
					</button>
					<button
						type="button"
						onClick={() => onAction?.("replace", rememberRule)}
						style={{
							background: "#38bdf8",
							border: "none",
							borderRadius: "6px",
							padding: "5px 12px",
							color: "#0f172a",
							fontSize: "11px",
							fontWeight: 700,
							cursor: "pointer"
						}}
					>
						{"Ganti".slice(0, 10)}
					</button>
				</div>
			</motion.div>
		);
	}

	// Dedicated Storage Low Warning Card
	if (item.situationId === "storage_low") {
		return (
			<motion.div
				className="proactive-card storage-low-card"
				initial={{ opacity: 0, scale: 0.95, y: -6 }}
				animate={{ opacity: 1, scale: 1, y: 0 }}
				exit={{ opacity: 0, scale: 0.95, y: -6 }}
				transition={{ type: "spring", stiffness: 400, damping: 28 }}
				style={{
					background: "linear-gradient(135deg, rgba(239, 68, 68, 0.14), rgba(245, 158, 11, 0.08))",
					border: "1px solid rgba(239, 68, 68, 0.35)",
					borderRadius: "16px",
					padding: "14px 16px",
					color: "#f8fafc",
					backdropFilter: "blur(20px)",
					width: "320px",
					boxShadow: "0 8px 24px rgba(0, 0, 0, 0.35)"
				}}
			>
				{/* Header */}
				<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "10px" }}>
					<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
						<div
							style={{
								width: "28px",
								height: "28px",
								borderRadius: "8px",
								background: "rgba(239, 68, 68, 0.2)",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
								color: "#f87171"
							}}
						>
							<HardDrive size={16} />
						</div>
						<div>
							<div style={{ fontSize: "13px", fontWeight: 700, color: "#f87171" }}>
								{item.title.slice(0, 16)}
							</div>
							<div style={{ fontSize: "11px", color: "#94a3b8" }}>
								{item.description.slice(0, 36)}
							</div>
						</div>
					</div>
					<HimeEyes expression="khawatir" size={22} />
				</div>

				{/* Progress meter */}
				<div style={{ marginBottom: "12px" }}>
					<div style={{ display: "flex", justifyContent: "space-between", fontSize: "10px", color: "#94a3b8", marginBottom: "4px" }}>
						<span>Kapasitas Drive C:</span>
						<span style={{ color: "#f87171", fontWeight: 600 }}>Tersisa 8% (12 GB)</span>
					</div>
					<div
						style={{
							width: "100%",
							height: "6px",
							background: "rgba(255, 255, 255, 0.1)",
							borderRadius: "3px",
							overflow: "hidden"
						}}
					>
						<div
							style={{
								width: "92%",
								height: "100%",
								background: "linear-gradient(90deg, #f59e0b, #ef4444)",
								borderRadius: "3px"
							}}
						/>
					</div>
				</div>

				{/* Actions */}
				<div style={{ display: "flex", justifyContent: "flex-end", gap: "6px" }}>
					<button
						type="button"
						onClick={onClose}
						style={{
							background: "rgba(255, 255, 255, 0.1)",
							border: "none",
							borderRadius: "6px",
							padding: "5px 12px",
							color: "#cbd5e1",
							fontSize: "11px",
							cursor: "pointer"
						}}
					>
						{"Nanti".slice(0, 10)}
					</button>
					<button
						type="button"
						onClick={() => onAction?.("view")}
						style={{
							background: "#ef4444",
							border: "none",
							borderRadius: "6px",
							padding: "5px 14px",
							color: "#ffffff",
							fontSize: "11px",
							fontWeight: 700,
							cursor: "pointer"
						}}
					>
						{"Lihat".slice(0, 10)}
					</button>
				</div>
			</motion.div>
		);
	}

	// Generic Catalog Card
	return (
		<motion.div
			className="proactive-card generic-proactive-card"
			initial={{ opacity: 0, scale: 0.95, y: -6 }}
			animate={{ opacity: 1, scale: 1, y: 0 }}
			exit={{ opacity: 0, scale: 0.95, y: -6 }}
			transition={{ type: "spring", stiffness: 400, damping: 28 }}
			style={{
				background: "rgba(30, 41, 59, 0.85)",
				border: "1px solid rgba(255, 255, 255, 0.15)",
				borderRadius: "16px",
				padding: "14px 16px",
				color: "#f8fafc",
				backdropFilter: "blur(20px)",
				width: "320px",
				boxShadow: "0 8px 24px rgba(0, 0, 0, 0.35)"
			}}
		>
			<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "10px" }}>
				<div>
					<div style={{ fontSize: "13px", fontWeight: 700 }}>
						{item.title.slice(0, 16)}
					</div>
					<div style={{ fontSize: "11px", color: "#94a3b8" }}>
						{item.description.slice(0, 36)}
					</div>
				</div>
				<HimeEyes expression={item.expression} size={22} />
			</div>

			<div style={{ display: "flex", justifyContent: "flex-end", gap: "6px", marginTop: "12px" }}>
				{item.actions && item.actions.length > 0 ? (
					item.actions.slice(0, 3).map((act, idx) => (
						<button
							key={act.id}
							type="button"
							onClick={() => onAction?.(act.id, rememberRule)}
							style={{
								background: idx === 0 ? "#38bdf8" : "rgba(255, 255, 255, 0.1)",
								color: idx === 0 ? "#0f172a" : "#f1f5f9",
								fontWeight: idx === 0 ? 700 : 500,
								border: "none",
								borderRadius: "6px",
								padding: "5px 12px",
								fontSize: "11px",
								cursor: "pointer"
							}}
						>
							{act.label.slice(0, 10)}
						</button>
					))
				) : (
					<button
						type="button"
						onClick={onClose}
						style={{
							background: "rgba(255, 255, 255, 0.1)",
							border: "none",
							borderRadius: "6px",
							padding: "5px 12px",
							color: "#cbd5e1",
							fontSize: "11px",
							cursor: "pointer"
						}}
					>
						{"Tutup".slice(0, 10)}
					</button>
				)}
			</div>
		</motion.div>
	);
};
