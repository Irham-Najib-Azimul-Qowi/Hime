import React, { useState } from "react";
import { BookOpen, Plus, FileText } from "lucide-react";
import { AcademicReportModal } from "./AcademicReportModal";
import { PdfOperationsCard } from "./PdfOperationsCard";
import { ArtcraftCard } from "./ArtcraftCard";

export const DocumentPanel: React.FC = () => {
	const [isModalOpen, setIsModalOpen] = useState(false);

	return (
		<div
			className="hime-document-panel"
			style={{
				padding: "16px",
				color: "#F3F4F6",
				display: "flex",
				flexDirection: "column",
				gap: "12px",
				maxWidth: "380px"
			}}
		>
			{/* Panel Header */}
			<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
				<div>
					<div style={{ fontSize: "14px", fontWeight: 700, color: "#F3F4F6" }}>
						{"Dokumen & Cetak".slice(0, 16)}
					</div>
					<div style={{ fontSize: "11px", color: "#9CA3AF" }}>
						{"Alur dokumen & laporan Hime".slice(0, 36)}
					</div>
				</div>

				<button
					type="button"
					onClick={() => setIsModalOpen(true)}
					style={{
						display: "flex",
						alignItems: "center",
						gap: "4px",
						background: "#8B5CF6",
						color: "#FFFFFF",
						border: "none",
						borderRadius: "8px",
						padding: "6px 10px",
						fontSize: "11px",
						fontWeight: 700,
						cursor: "pointer"
					}}
				>
					<Plus size={12} />
					<span>{"Buat Baru".slice(0, 10)}</span>
				</button>
			</div>

			{/* Cards */}
			<PdfOperationsCard />
			<ArtcraftCard />

			{/* Modal */}
			<AcademicReportModal
				isOpen={isModalOpen}
				onClose={() => setIsModalOpen(false)}
			/>
		</div>
	);
};
