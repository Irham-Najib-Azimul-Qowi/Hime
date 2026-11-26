import React, { useState } from "react";
import { Palette, ExternalLink, RefreshCw } from "lucide-react";

export const ArtcraftCard: React.FC = () => {
	const [isInstalled, setIsInstalled] = useState(true);
	const [scanning, setScanning] = useState(false);

	const handleScan = () => {
		setScanning(true);
		setTimeout(() => {
			setIsInstalled(true);
			setScanning(false);
		}, 800);
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
			<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
				<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
					<Palette size={16} color="#F59E0B" />
					<div>
						<div style={{ fontSize: "13px", fontWeight: 700, color: "#F59E0B" }}>
							{"Desain Artcraft".slice(0, 16)}
						</div>
						<div style={{ fontSize: "11px", color: "#9CA3AF" }}>
							{"Integrasi Artcraft desainer".slice(0, 36)}
						</div>
					</div>
				</div>

				<div style={{ display: "flex", alignItems: "center", gap: "4px" }}>
					<span
						style={{
							width: "6px",
							height: "6px",
							borderRadius: "50%",
							backgroundColor: isInstalled ? "#10B981" : "#6B7280"
						}}
					/>
					<span style={{ fontSize: "10px", color: isInstalled ? "#10B981" : "#6B7280" }}>
						{isInstalled ? "Terdeteksi" : "Tidak Ada"}
					</span>
				</div>
			</div>

			<div
				style={{
					marginTop: "10px",
					fontSize: "11px",
					color: "#9CA3AF",
					background: "#0D0F14",
					padding: "8px 10px",
					borderRadius: "6px"
				}}
			>
				Alur kerja Artcraft: Token visual sinkron dengan Hime Desktop secara lokal.
			</div>

			<div style={{ display: "flex", justifyContent: "flex-end", gap: "6px", marginTop: "10px" }}>
				<button
					type="button"
					onClick={handleScan}
					disabled={scanning}
					style={{
						background: "#1E222E",
						color: "#9CA3AF",
						border: "none",
						borderRadius: "6px",
						padding: "4px 10px",
						fontSize: "11px",
						cursor: scanning ? "not-allowed" : "pointer",
						display: "flex",
						alignItems: "center",
						gap: "4px"
					}}
				>
					<RefreshCw size={11} className={scanning ? "animate-spin" : ""} />
					<span>{"Pindai".slice(0, 10)}</span>
				</button>
				<button
					type="button"
					style={{
						background: "#F59E0B",
						color: "#0F172A",
						border: "none",
						borderRadius: "6px",
						padding: "4px 12px",
						fontSize: "11px",
						fontWeight: 700,
						cursor: "pointer",
						display: "flex",
						alignItems: "center",
						gap: "4px"
					}}
				>
					<ExternalLink size={11} />
					<span>{"Buka".slice(0, 10)}</span>
				</button>
			</div>
		</div>
	);
};
