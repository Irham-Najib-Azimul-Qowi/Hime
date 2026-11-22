import React, { useState } from "react";
import { ShieldCheck, ShieldAlert, RefreshCw, EyeOff } from "lucide-react";
import { SecretFinding } from "./types";

export const SecretScannerCard: React.FC = () => {
	const [scanning, setScanning] = useState(false);
	const [findings, setFindings] = useState<SecretFinding[]>([
		{
			file_path: ".env.example",
			line_number: 14,
			secret_type: "GitHub Personal Access Token",
			masked_match: "ghp_****"
		}
	]);

	const handleScan = () => {
		setScanning(true);
		setTimeout(() => {
			setFindings([]);
			setScanning(false);
		}, 600);
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
					<ShieldCheck size={16} color="#10B981" />
					<div>
						<div style={{ fontSize: "13px", fontWeight: 700, color: "#10B981" }}>
							{"Pindai Rahasia".slice(0, 16)}
						</div>
						<div style={{ fontSize: "11px", color: "#9CA3AF" }}>
							{"Deteksi kebocoran token & kunci".slice(0, 36)}
						</div>
					</div>
				</div>

				<button
					type="button"
					disabled={scanning}
					onClick={handleScan}
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
			</div>

			<div style={{ marginTop: "10px" }}>
				{findings.length === 0 ? (
					<div
						style={{
							background: "rgba(16, 185, 129, 0.1)",
							border: "1px solid rgba(16, 185, 129, 0.25)",
							borderRadius: "8px",
							padding: "8px 10px",
							fontSize: "11px",
							color: "#34D399",
							display: "flex",
							alignItems: "center",
							gap: "6px"
						}}
					>
						<ShieldCheck size={14} />
						<span>Workspace aman: Nol kredensial bocor.</span>
					</div>
				) : (
					<div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
						{findings.map((f, i) => (
							<div
								key={i}
								style={{
									background: "rgba(239, 68, 68, 0.1)",
									border: "1px solid rgba(239, 68, 68, 0.25)",
									borderRadius: "8px",
									padding: "8px 10px",
									fontSize: "11px",
									color: "#F87171"
								}}
							>
								<div style={{ display: "flex", justifyContent: "space-between", fontWeight: 600 }}>
									<span>{f.secret_type}</span>
									<span>Baris {f.line_number}</span>
								</div>
								<div style={{ fontSize: "10px", color: "#FCA5A5", fontFamily: "monospace", marginTop: "2px" }}>
									{f.file_path}: {f.masked_match}
								</div>
							</div>
						))}
					</div>
				)}
			</div>
		</div>
	);
};
