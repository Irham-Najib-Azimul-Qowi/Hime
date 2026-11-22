import React, { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { FileText, Check, X, BookOpen } from "lucide-react";
import { createAcademicReport } from "../api";
import { DocumentOpResult } from "./types";

export interface AcademicReportModalProps {
	isOpen: boolean;
	onClose: () => void;
	onGenerated?: (result: DocumentOpResult) => void;
}

export const AcademicReportModal: React.FC<AcademicReportModalProps> = ({
	isOpen,
	onClose,
	onGenerated
}) => {
	const [title, setTitle] = useState("Laporan Analisis");
	const [body, setBody] = useState("# Ringkasan Eksekutif\n\nAnalisis sistem personal AI desktop...");
	const [template, setTemplate] = useState("Akademik-ID");
	const [generating, setGenerating] = useState(false);
	const [result, setResult] = useState<DocumentOpResult | null>(null);

	if (!isOpen) return null;

	const handleGenerate = async () => {
		setGenerating(true);
		try {
			const res = await createAcademicReport(title, body);
			setResult(res);
			onGenerated?.(res);
		} catch (e) {
			console.error("Generate report failed:", e);
		} finally {
			setGenerating(false);
		}
	};

	return (
		<AnimatePresence>
			<motion.div
				className="academic-report-modal-backdrop"
				initial={{ opacity: 0 }}
				animate={{ opacity: 1 }}
				exit={{ opacity: 0 }}
				style={{
					position: "fixed",
					top: 0,
					left: 0,
					right: 0,
					bottom: 0,
					background: "rgba(0, 0, 0, 0.7)",
					backdropFilter: "blur(8px)",
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					zIndex: 9999
				}}
			>
				<motion.div
					className="academic-report-modal"
					initial={{ scale: 0.95, opacity: 0, y: 10 }}
					animate={{ scale: 1, opacity: 1, y: 0 }}
					exit={{ scale: 0.95, opacity: 0, y: 10 }}
					style={{
						background: "#161922",
						border: "1px solid #2B3142",
						borderRadius: "16px",
						padding: "20px",
						width: "440px",
						color: "#F3F4F6",
						boxShadow: "0 16px 36px rgba(0, 0, 0, 0.5)"
					}}
				>
					{/* Header */}
					<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "14px" }}>
						<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
							<div
								style={{
									width: "32px",
									height: "32px",
									borderRadius: "8px",
									background: "rgba(139, 92, 246, 0.2)",
									display: "flex",
									alignItems: "center",
									justifyContent: "center",
									color: "#8B5CF6"
								}}
							>
								<BookOpen size={18} />
							</div>
							<div>
								<div style={{ fontSize: "14px", fontWeight: 700, color: "#F3F4F6" }}>
									{"Laporan Akademik".slice(0, 16)}
								</div>
								<div style={{ fontSize: "11px", color: "#9CA3AF" }}>
									{"Format laporan standar Indonesia".slice(0, 36)}
								</div>
							</div>
						</div>
						<button
							type="button"
							onClick={onClose}
							style={{ background: "none", border: "none", color: "#6B7280", cursor: "pointer" }}
						>
							<X size={16} />
						</button>
					</div>

					{/* Title Input */}
					<div style={{ marginBottom: "10px" }}>
						<label style={{ fontSize: "11px", color: "#9CA3AF", display: "block", marginBottom: "4px" }}>
							Judul Laporan
						</label>
						<input
							type="text"
							value={title}
							onChange={(e) => setTitle(e.target.value)}
							style={{
								width: "100%",
								background: "#0D0F14",
								border: "1px solid #2B3142",
								borderRadius: "6px",
								padding: "6px 10px",
								color: "#F3F4F6",
								fontSize: "12px",
								outline: "none"
							}}
						/>
					</div>

					{/* Template Selector */}
					<div style={{ marginBottom: "10px" }}>
						<label style={{ fontSize: "11px", color: "#9CA3AF", display: "block", marginBottom: "4px" }}>
							Format Template
						</label>
						<select
							value={template}
							onChange={(e) => setTemplate(e.target.value)}
							style={{
								width: "100%",
								background: "#0D0F14",
								border: "1px solid #2B3142",
								borderRadius: "6px",
								padding: "6px 10px",
								color: "#F3F4F6",
								fontSize: "12px",
								outline: "none"
							}}
						>
							<option value="Akademik-ID">Akademik-ID (Times New Roman, 4-3-3-3cm)</option>
							<option value="Laporan-SKPL">Laporan-SKPL (Standar Rekayasa Perangkat Lunak)</option>
						</select>
					</div>

					{/* Markdown Body */}
					<div style={{ marginBottom: "14px" }}>
						<label style={{ fontSize: "11px", color: "#9CA3AF", display: "block", marginBottom: "4px" }}>
							Isi Markdown
						</label>
						<textarea
							rows={4}
							value={body}
							onChange={(e) => setBody(e.target.value)}
							style={{
								width: "100%",
								background: "#0D0F14",
								border: "1px solid #2B3142",
								borderRadius: "6px",
								padding: "6px 10px",
								color: "#F3F4F6",
								fontSize: "11px",
								fontFamily: "monospace",
								outline: "none",
								resize: "vertical"
							}}
						/>
					</div>

					{/* Result Preview if generated */}
					{result && (
						<div
							style={{
								background: "rgba(16, 185, 129, 0.12)",
								border: "1px solid rgba(16, 185, 129, 0.3)",
								borderRadius: "8px",
								padding: "8px 10px",
								marginBottom: "12px",
								fontSize: "11px",
								color: "#34D399"
							}}
						>
							<div style={{ fontWeight: 600 }}>Dokumen siap ({result.format.toUpperCase()})</div>
							<div style={{ fontSize: "10px", color: "#A7F3D0" }}>{result.output_path}</div>
						</div>
					)}

					{/* Actions */}
					<div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
						<button
							type="button"
							onClick={onClose}
							style={{
								background: "#1E222E",
								border: "none",
								borderRadius: "6px",
								padding: "6px 14px",
								color: "#9CA3AF",
								fontSize: "11px",
								cursor: "pointer"
							}}
						>
							{"Batal".slice(0, 10)}
						</button>
						<button
							type="button"
							disabled={generating}
							onClick={handleGenerate}
							style={{
								background: "#8B5CF6",
								border: "none",
								borderRadius: "6px",
								padding: "6px 16px",
								color: "#FFFFFF",
								fontSize: "11px",
								fontWeight: 700,
								cursor: generating ? "not-allowed" : "pointer"
							}}
						>
							{generating ? "Proses..." : "Buat".slice(0, 10)}
						</button>
					</div>
				</motion.div>
			</motion.div>
		</AnimatePresence>
	);
};
