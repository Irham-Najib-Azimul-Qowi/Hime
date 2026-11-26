import React, { useState } from "react";
import { Terminal, GitBranch, ShieldCheck } from "lucide-react";
import { GitStatusCard } from "./GitStatusCard";
import { CommandRunnerCard } from "./CommandRunnerCard";
import { SecretScannerCard } from "./SecretScannerCard";

export const DeveloperPanel: React.FC = () => {
	const [activeTab, setActiveTab] = useState<"git" | "runner" | "secrets">("git");

	return (
		<div
			style={{
				display: "flex",
				flexDirection: "column",
				gap: "12px",
				padding: "16px",
				background: "#0D1117",
				borderRadius: "16px",
				color: "#F3F4F6",
				fontFamily: "system-ui, -apple-system, sans-serif"
			}}
		>
			<div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
				<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
					<Terminal size={18} color="#38BDF8" />
					<div>
						<div style={{ fontSize: "14px", fontWeight: 700, color: "#38BDF8" }}>
							{"Workspace Dev".slice(0, 16)}
						</div>
						<div style={{ fontSize: "11px", color: "#9CA3AF" }}>
							{"Git, terminal aman & audit kode".slice(0, 36)}
						</div>
					</div>
				</div>

				{/* Tab Selector */}
				<div
					style={{
						display: "flex",
						gap: "4px",
						background: "#161B22",
						padding: "3px",
						borderRadius: "8px",
						border: "1px solid #30363D"
					}}
				>
					<button
						type="button"
						onClick={() => setActiveTab("git")}
						style={{
							background: activeTab === "git" ? "#8B5CF6" : "transparent",
							border: "none",
							borderRadius: "6px",
							padding: "4px 8px",
							color: "#FFFFFF",
							fontSize: "11px",
							fontWeight: 600,
							cursor: "pointer",
							display: "flex",
							alignItems: "center",
							gap: "4px"
						}}
					>
						<GitBranch size={12} />
						<span>{"Git".slice(0, 10)}</span>
					</button>

					<button
						type="button"
						onClick={() => setActiveTab("runner")}
						style={{
							background: activeTab === "runner" ? "#38BDF8" : "transparent",
							border: "none",
							borderRadius: "6px",
							padding: "4px 8px",
							color: "#FFFFFF",
							fontSize: "11px",
							fontWeight: 600,
							cursor: "pointer",
							display: "flex",
							alignItems: "center",
							gap: "4px"
						}}
					>
						<Terminal size={12} />
						<span>{"Perintah".slice(0, 10)}</span>
					</button>

					<button
						type="button"
						onClick={() => setActiveTab("secrets")}
						style={{
							background: activeTab === "secrets" ? "#F59E0B" : "transparent",
							border: "none",
							borderRadius: "6px",
							padding: "4px 8px",
							color: "#FFFFFF",
							fontSize: "11px",
							fontWeight: 600,
							cursor: "pointer",
							display: "flex",
							alignItems: "center",
							gap: "4px"
						}}
					>
						<ShieldCheck size={12} />
						<span>{"Audit".slice(0, 10)}</span>
					</button>
				</div>
			</div>

			{/* Active Card Body */}
			<div style={{ marginTop: "4px" }}>
				{activeTab === "git" && <GitStatusCard />}
				{activeTab === "runner" && <CommandRunnerCard />}
				{activeTab === "secrets" && <SecretScannerCard />}
			</div>
		</div>
	);
};
