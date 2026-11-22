import React, { useState, useEffect, useCallback } from "react";
import { Cpu, Zap, Activity, HardDrive, RefreshCw, CheckCircle2, AlertTriangle, Power } from "lucide-react";
import { SettingRow } from "../../settings/SettingRow";
import { getResourceMetrics, setSaveMode, stopAiRuntime, ResourceMetrics } from "../api";

export const ResourcesTab: React.FC = () => {
	const [metrics, setMetrics] = useState<ResourceMetrics>({
		total_ram_mb: 238,
		idle_target_mb: 250,
		mode_hemat: "auto",
		is_saving_active: true,
		processes: [
			{ name: "hime-island.exe", pid: 14220, working_set_mb: 68, private_ws_mb: 62, cpu_percent: 0.1 },
			{ name: "hime-core.exe", pid: 14228, working_set_mb: 42, private_ws_mb: 38, cpu_percent: 0.1 },
			{ name: "msedgewebview2.exe", pid: 14312, working_set_mb: 128, private_ws_mb: 118, cpu_percent: 0.2 }
		],
		ai_runtime_status: "stopped",
		ai_idle_seconds: 640,
		sqlite_cache_kb: 2000
	});

	const [refreshing, setRefreshing] = useState(false);
	const [stoppingAi, setStoppingAi] = useState(false);
	const [feedback, setFeedback] = useState<string | null>(null);

	const fetchMetrics = useCallback(async () => {
		setRefreshing(true);
		try {
			const res = await getResourceMetrics();
			setMetrics(res);
		} catch {
			// keep current
		} finally {
			setRefreshing(false);
		}
	}, []);

	useEffect(() => {
		fetchMetrics();
		const interval = setInterval(fetchMetrics, 10000);
		return () => clearInterval(interval);
	}, [fetchMetrics]);

	const handleModeChange = async (mode: "off" | "auto" | "on") => {
		await setSaveMode(mode);
		await fetchMetrics();
	};

	const handleStopAi = async () => {
		setStoppingAi(true);
		try {
			const res = await stopAiRuntime();
			if (res.success) {
				setFeedback("AI dihentikan");
				setTimeout(() => setFeedback(null), 3000);
			}
			await fetchMetrics();
		} finally {
			setStoppingAi(false);
		}
	};

	const ramRatio = Math.min(100, Math.round((metrics.total_ram_mb / metrics.idle_target_mb) * 100));
	const isUnderBudget = metrics.total_ram_mb <= metrics.idle_target_mb;

	return (
		<div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
			{/* Section: Status Memori */}
			<div className="setting-group-label">{"Status Memori".slice(0, 16)}</div>
			<div className="setting-group">
				<SettingRow
					icon={Activity}
					label={"Batas RAM".slice(0, 16)}
					desc={"Target alokasi saat idle".slice(0, 36)}
				>
					<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
						<button
							type="button"
							onClick={fetchMetrics}
							disabled={refreshing}
							style={{
								background: "rgba(255, 255, 255, 0.08)",
								border: "1px solid rgba(255, 255, 255, 0.12)",
								borderRadius: "6px",
								padding: "4px 8px",
								color: "#e5e5e5",
								fontSize: "11px",
								cursor: refreshing ? "not-allowed" : "pointer",
								display: "flex",
								alignItems: "center",
								gap: "4px"
							}}
						>
							<RefreshCw size={11} className={refreshing ? "animate-spin" : ""} />
							<span>{"Segarkan".slice(0, 10)}</span>
						</button>
					</div>
				</SettingRow>

				{/* RAM Visual Progress Bar */}
				<div style={{ padding: "12px 16px", background: "rgba(15, 23, 42, 0.4)" }}>
					<div style={{ display: "flex", justifyContent: "space-between", marginBottom: "6px", fontSize: "11px" }}>
						<span style={{ color: "#94a3b8" }}>
							Total Private WS: <strong style={{ color: "#f8fafc" }}>{metrics.total_ram_mb} MB</strong>
						</span>
						<span style={{ color: isUnderBudget ? "#34d399" : "#fbbf24", fontWeight: 600 }}>
							Target: ≤ {metrics.idle_target_mb} MB
						</span>
					</div>
					<div
						style={{
							width: "100%",
							height: "8px",
							background: "rgba(255, 255, 255, 0.1)",
							borderRadius: "4px",
							overflow: "hidden"
						}}
					>
						<div
							style={{
								width: `${ramRatio}%`,
								height: "100%",
								background: isUnderBudget ? "#10b981" : "#f59e0b",
								transition: "width 0.3s ease"
							}}
						/>
					</div>
					<div style={{ display: "flex", justifyContent: "space-between", marginTop: "6px", fontSize: "10px", color: "#64748b" }}>
						<span>0 MB</span>
						<span>{metrics.is_saving_active ? "Mode Hemat Aktif (≤250 MB)" : "Standar (≤350 MB)"}</span>
						<span>{metrics.idle_target_mb} MB</span>
					</div>
				</div>
			</div>

			{/* Section: Mode Hemat */}
			<div className="setting-group-label">{"Mode Hemat".slice(0, 16)}</div>
			<div className="setting-group">
				<SettingRow
					icon={Zap}
					label={"Mode Hemat".slice(0, 16)}
					desc={"Kurangi beban latar & animasi".slice(0, 36)}
					divider={false}
				>
					<div style={{ display: "flex", gap: "4px" }}>
						<button
							type="button"
							onClick={() => handleModeChange("off")}
							style={{
								background: metrics.mode_hemat === "off" ? "#64748b" : "rgba(255, 255, 255, 0.05)",
								border: "1px solid rgba(255, 255, 255, 0.12)",
								borderRadius: "6px",
								padding: "4px 8px",
								color: metrics.mode_hemat === "off" ? "#ffffff" : "#94a3b8",
								fontSize: "11px",
								fontWeight: 600,
								cursor: "pointer"
							}}
						>
							{"Nonaktif".slice(0, 10)}
						</button>
						<button
							type="button"
							onClick={() => handleModeChange("auto")}
							style={{
								background: metrics.mode_hemat === "auto" ? "#0284c7" : "rgba(255, 255, 255, 0.05)",
								border: "1px solid rgba(255, 255, 255, 0.12)",
								borderRadius: "6px",
								padding: "4px 8px",
								color: metrics.mode_hemat === "auto" ? "#ffffff" : "#94a3b8",
								fontSize: "11px",
								fontWeight: 600,
								cursor: "pointer"
							}}
						>
							{"Otomatis".slice(0, 10)}
						</button>
						<button
							type="button"
							onClick={() => handleModeChange("on")}
							style={{
								background: metrics.mode_hemat === "on" ? "#10b981" : "rgba(255, 255, 255, 0.05)",
								border: "1px solid rgba(255, 255, 255, 0.12)",
								borderRadius: "6px",
								padding: "4px 8px",
								color: metrics.mode_hemat === "on" ? "#ffffff" : "#94a3b8",
								fontSize: "11px",
								fontWeight: 600,
								cursor: "pointer"
							}}
						>
							{"Aktif".slice(0, 10)}
						</button>
					</div>
				</SettingRow>
			</div>

			{/* Section: Runtime AI */}
			<div className="setting-group-label">{"Runtime AI".slice(0, 16)}</div>
			<div className="setting-group">
				<SettingRow
					icon={Cpu}
					label={"Runtime AI".slice(0, 16)}
					desc={"Hermes & 9Router on-demand".slice(0, 36)}
					divider={false}
				>
					<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
						<span
							style={{
								padding: "2px 6px",
								borderRadius: "4px",
								fontSize: "10px",
								background:
									metrics.ai_runtime_status === "running"
										? "rgba(16, 185, 129, 0.15)"
										: "rgba(100, 116, 139, 0.15)",
								color: metrics.ai_runtime_status === "running" ? "#34d399" : "#94a3b8",
								border:
									metrics.ai_runtime_status === "running"
										? "1px solid rgba(16, 185, 129, 0.3)"
										: "1px solid rgba(100, 116, 139, 0.3)"
							}}
						>
							{metrics.ai_runtime_status === "running" ? "Aktif" : "Berhenti"}
						</span>
						<button
							type="button"
							onClick={handleStopAi}
							disabled={stoppingAi || metrics.ai_runtime_status === "stopped"}
							style={{
								background: "rgba(239, 68, 68, 0.15)",
								border: "1px solid rgba(239, 68, 68, 0.3)",
								borderRadius: "6px",
								padding: "4px 10px",
								color: "#f87171",
								fontSize: "11px",
								fontWeight: 600,
								cursor:
									stoppingAi || metrics.ai_runtime_status === "stopped" ? "not-allowed" : "pointer"
							}}
						>
							{feedback ?? "Hentikan".slice(0, 10)}
						</button>
					</div>
				</SettingRow>
			</div>

			{/* Section: Pohon Proses */}
			<div className="setting-group-label">{"Pohon Proses".slice(0, 16)}</div>
			<div className="setting-group">
				{metrics.processes.map((proc, idx) => (
					<SettingRow
						key={proc.name}
						icon={proc.name.includes("webview") ? HardDrive : Activity}
						label={proc.name.slice(0, 16)}
						desc={`PID: ${proc.pid} • CPU: ${proc.cpu_percent}%`.slice(0, 36)}
						divider={idx !== metrics.processes.length - 1}
					>
						<span style={{ fontSize: "11px", fontFamily: "monospace", color: "#38bdf8", fontWeight: 600 }}>
							{proc.private_ws_mb} MB
						</span>
					</SettingRow>
				))}
			</div>

			{/* Section: Siklus Window */}
			<div className="setting-group-label">{"Siklus Window".slice(0, 16)}</div>
			<div className="setting-group">
				<SettingRow
					icon={CheckCircle2}
					label={"Siklus Window".slice(0, 16)}
					desc={"Hancurkan window saat ditutup".slice(0, 36)}
				>
					<span style={{ fontSize: "11px", color: "#10b981", fontWeight: 600 }}>
						{"Aktif".slice(0, 10)}
					</span>
				</SettingRow>

				<SettingRow
					icon={HardDrive}
					label={"Cache SQLite".slice(0, 16)}
					desc={"Batas cache SQLite 2 MB".slice(0, 36)}
					divider={false}
				>
					<span style={{ fontSize: "11px", fontFamily: "monospace", color: "#94a3b8" }}>
						{metrics.sqlite_cache_kb} KB
					</span>
				</SettingRow>
			</div>
		</div>
	);
};
