import React, { useState } from "react";
import { GitBranch, RefreshCw } from "lucide-react";
import { GitStatusInfo } from "./types";

export const GitStatusCard: React.FC = () => {
	const [status] = useState<GitStatusInfo>({
		branch: "hime/h1-monorepo",
		ahead: 0,
		behind: 0,
		staged_count: 2,
		unstaged_count: 1,
		untracked_count: 0,
		is_clean: false
	});
	const [loading, setLoading] = useState(false);

	const handleRefresh = () => {
		setLoading(true);
		setTimeout(() => {
			setLoading(false);
		}, 500);
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
					<GitBranch size={16} color="#8B5CF6" />
					<div>
						<div style={{ fontSize: "13px", fontWeight: 700, color: "#8B5CF6" }}>
							{"Status Git".slice(0, 16)}
						</div>
						<div style={{ fontSize: "11px", color: "#9CA3AF" }}>
							{"Pantau branch & perubahan kode".slice(0, 36)}
						</div>
					</div>
				</div>

				<button
					type="button"
					onClick={handleRefresh}
					disabled={loading}
					style={{
						background: "#1E222E",
						border: "none",
						borderRadius: "6px",
						padding: "4px 8px",
						color: "#9CA3AF",
						cursor: loading ? "not-allowed" : "pointer"
					}}
				>
					<RefreshCw size={11} className={loading ? "animate-spin" : ""} />
				</button>
			</div>

			<div style={{ display: "flex", alignItems: "center", gap: "8px", marginTop: "12px" }}>
				<div
					style={{
						background: "rgba(139, 92, 246, 0.15)",
						borderRadius: "6px",
						padding: "4px 8px",
						fontSize: "11px",
						fontFamily: "monospace",
						color: "#C4B5FD",
						fontWeight: 600
					}}
				>
					{status.branch}
				</div>

				<span style={{ fontSize: "11px", color: "#9CA3AF" }}>
					{status.is_clean ? (
						<span style={{ color: "#10B981" }}>Bersih</span>
					) : (
						`${status.staged_count} staged, ${status.unstaged_count} modified`
					)}
				</span>
			</div>

			<div style={{ display: "flex", justifyContent: "flex-end", gap: "6px", marginTop: "12px" }}>
				<button
					type="button"
					style={{
						background: "#1E222E",
						color: "#E2E8F0",
						border: "1px solid #2B3142",
						borderRadius: "6px",
						padding: "5px 10px",
						fontSize: "11px",
						cursor: "pointer"
					}}
				>
					{"Diff".slice(0, 10)}
				</button>
				<button
					type="button"
					style={{
						background: "#8B5CF6",
						color: "#FFFFFF",
						border: "none",
						borderRadius: "6px",
						padding: "5px 12px",
						fontSize: "11px",
						fontWeight: 700,
						cursor: "pointer"
					}}
				>
					{"Commit".slice(0, 10)}
				</button>
			</div>
		</div>
	);
};
