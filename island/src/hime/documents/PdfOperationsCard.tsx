import React, { useState } from "react";
import { Files } from "lucide-react";
import { mergePdfs } from "../api";

export const PdfOperationsCard: React.FC = () => {
	const [sources, setSources] = useState<string>("bab1.pdf, bab2.pdf, lampiran.pdf");
	const [dest, setDest] = useState<string>("gabungan_laporan.pdf");
	const [operating, setOperating] = useState(false);
	const [statusMsg, setStatusMsg] = useState<string | null>(null);

	const handleMerge = async () => {
		setOperating(true);
		setStatusMsg(null);
		try {
			const sourceList = sources.split(",").map((s) => s.trim()).filter(Boolean);
			const res = await mergePdfs(sourceList, dest);
			setStatusMsg(`Selesai: ${res.file_size_bytes} byte`);
		} catch (err: any) {
			setStatusMsg(`Gagal: ${err?.message || "Kesalahan"}`);
		} finally {
			setOperating(false);
		}
	};

	return (
		<div
			style={{
				background: "#161922",
				border: "1px solid #2B3142",
				borderRadius: "14px",
				padding: "14px",
				color: "#F3F4F6"
			}}
		>
			<div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "8px" }}>
				<Files size={16} color="#38BDF8" />
				<div>
					<div style={{ fontSize: "13px", fontWeight: 700, color: "#38BDF8" }}>
						{"Operasi PDF".slice(0, 16)}
					</div>
					<div style={{ fontSize: "11px", color: "#9CA3AF" }}>
						{"Gabung & kompres berkas PDF".slice(0, 36)}
					</div>
				</div>
			</div>

			<div style={{ display: "flex", flexDirection: "column", gap: "8px", marginTop: "10px" }}>
				<div>
					<label style={{ fontSize: "10px", color: "#9CA3AF", display: "block" }}>
						Berkas Sumber (pisahkan koma)
					</label>
					<input
						type="text"
						value={sources}
						onChange={(e) => setSources(e.target.value)}
						style={{
							width: "100%",
							background: "#0D0F14",
							border: "1px solid #2B3142",
							borderRadius: "6px",
							padding: "5px 8px",
							fontSize: "11px",
							color: "#F3F4F6",
							outline: "none"
						}}
					/>
				</div>

				<div>
					<label style={{ fontSize: "10px", color: "#9CA3AF", display: "block" }}>
						Tujuan Gabungan
					</label>
					<input
						type="text"
						value={dest}
						onChange={(e) => setDest(e.target.value)}
						style={{
							width: "100%",
							background: "#0D0F14",
							border: "1px solid #2B3142",
							borderRadius: "6px",
							padding: "5px 8px",
							fontSize: "11px",
							color: "#F3F4F6",
							outline: "none"
						}}
					/>
				</div>

				{statusMsg && (
					<div style={{ fontSize: "11px", color: "#34D399" }}>
						{statusMsg.slice(0, 36)}
					</div>
				)}

				<div style={{ display: "flex", justifyContent: "flex-end", gap: "6px", marginTop: "6px" }}>
					<button
						type="button"
						disabled={operating}
						onClick={handleMerge}
						style={{
							background: "#38BDF8",
							color: "#0F172A",
							border: "none",
							borderRadius: "6px",
							padding: "5px 12px",
							fontSize: "11px",
							fontWeight: 700,
							cursor: operating ? "not-allowed" : "pointer"
						}}
					>
						{"Gabung".slice(0, 10)}
					</button>
				</div>
			</div>
		</div>
	);
};
