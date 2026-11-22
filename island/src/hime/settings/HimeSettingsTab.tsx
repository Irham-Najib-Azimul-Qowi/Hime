import React, { useState } from "react";
import {
	Bot,
	Radio,
	RefreshCw,
	ShieldAlert,
	Server,
	Activity,
	Check,
	PanelTop,
	ExternalLink,
	Key,
	Cpu,
	Eye,
	EyeOff,
	HelpCircle,
	Sparkles,
	Clock,
	RotateCcw
} from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { SettingRow } from "../../settings/SettingRow";
import { HimeCoreStatus } from "../types";
import { devTriggerApproval, pingCore, storeCredential } from "../api";
import { ExpressionTester } from "./ExpressionTester";
import { CATALOG_TEMPLATES } from "../proactive/copyLint";
import { AttentionBudget } from "../proactive/types";

interface HimeSettingsTabProps {
	status: HimeCoreStatus;
	onRefresh: () => void;
}

export const HimeSettingsTab: React.FC<HimeSettingsTabProps> = ({ status, onRefresh }) => {
	const isOnline = status.is_online;

	const [himeEnabled, setHimeEnabled] = useState<boolean>(() => {
		return localStorage.getItem("hime-enabled") !== "false";
	});
	const [pingResult, setPingResult] = useState<string | null>(null);
	const [pinging, setPinging] = useState(false);
	const [triggering, setTriggering] = useState(false);

	// 9Router & Model Configuration State
	const [routerUrl, setRouterUrl] = useState<string>(() => {
		return localStorage.getItem("hime-9router-url") || "http://127.0.0.1:20128/v1";
	});
	const [routerKey, setRouterKey] = useState<string>(() => {
		return localStorage.getItem("hime-9router-key") || "";
	});
	const [selectedModel, setSelectedModel] = useState<string>(() => {
		return localStorage.getItem("hime-9router-model") || "anthropic/claude-3-5-sonnet";
	});
	const [customModel, setCustomModel] = useState<string>(() => {
		return localStorage.getItem("hime-9router-custom-model") || "";
	});
	const [showKey, setShowKey] = useState(false);
	const [testingRouter, setTestingRouter] = useState(false);
	const [routerTestResult, setRouterTestResult] = useState<string | null>(null);

	// Notch Behavior State
	const [currentNotchMode, setCurrentNotchMode] = useState<string>(() => {
		const raw = localStorage.getItem("bloom-notch-mode") || "smart";
		return raw === "auto-hide" ? "smart" : raw;
	});

	// Proactive Interaction & Attention Budget State
	const [proactiveBudget, setProactiveBudget] = useState<AttentionBudget>(() => {
		const raw = localStorage.getItem("hime-proactive-budget") || localStorage.getItem("hime-chatty");
		if (raw === "pendiam" || raw === "seimbang" || raw === "cerewet") return raw;
		return "seimbang";
	});
	const [quietHours, setQuietHours] = useState<boolean>(() => {
		return localStorage.getItem("hime-proactive-quiet-hours") !== "false";
	});
	const [backoffResetNotice, setBackoffResetNotice] = useState<string | null>(null);

	const handleBudgetChange = (val: AttentionBudget) => {
		setProactiveBudget(val);
		localStorage.setItem("hime-proactive-budget", val);
		localStorage.setItem("hime-chatty", val);
		try {
			const raw = localStorage.getItem("hime-proactive-settings");
			const parsed = raw ? JSON.parse(raw) : {};
			parsed.budget = val;
			localStorage.setItem("hime-proactive-settings", JSON.stringify(parsed));
		} catch {}
		invoke("save_setting", { key: "hime-chatty", value: val }).catch(() => {});
	};

	const handleToggleQuietHours = () => {
		const next = !quietHours;
		setQuietHours(next);
		localStorage.setItem("hime-proactive-quiet-hours", next ? "true" : "false");
		try {
			const raw = localStorage.getItem("hime-proactive-settings");
			const parsed = raw ? JSON.parse(raw) : {};
			parsed.quietHoursEnabled = next;
			localStorage.setItem("hime-proactive-settings", JSON.stringify(parsed));
		} catch {}
	};

	const handleResetBackoff = () => {
		try {
			const raw = localStorage.getItem("hime-proactive-settings");
			const parsed = raw ? JSON.parse(raw) : {};
			parsed.dismissCounts = {};
			parsed.backoffMultipliers = {};
			localStorage.setItem("hime-proactive-settings", JSON.stringify(parsed));
			setBackoffResetNotice("Riwayat reset!");
			setTimeout(() => setBackoffResetNotice(null), 3000);
		} catch {}
	};

	const handleSimulateProactive = (type: "bisik" | "colek" | "tanya" | "penting") => {
		let item;
		if (type === "bisik") {
			item = {
				id: `sim-bisik-${Date.now()}`,
				situationId: "file_download_new",
				level: "bisik" as const,
				title: "Berkas Baru",
				description: "Simpan ke mana?",
				bubbleText: "Simpan ke mana?",
				expression: "penasaran" as const,
				createdAt: Date.now()
			};
		} else if (type === "colek") {
			const tmpl = CATALOG_TEMPLATES.eveningRecap;
			item = {
				id: `sim-colek-${Date.now()}`,
				situationId: tmpl.situationId,
				level: tmpl.level,
				title: tmpl.title,
				description: tmpl.description,
				bubbleText: tmpl.bubbleText,
				expression: tmpl.expression,
				actions: tmpl.actions,
				autoDismissSecs: tmpl.autoDismissSecs,
				createdAt: Date.now()
			};
		} else if (type === "tanya") {
			const tmpl = CATALOG_TEMPLATES.fileDuplicate;
			item = {
				id: `sim-tanya-${Date.now()}`,
				situationId: tmpl.situationId,
				level: tmpl.level,
				title: tmpl.title,
				description: tmpl.description,
				bubbleText: tmpl.bubbleText,
				expression: tmpl.expression,
				actions: tmpl.actions,
				hasRememberOption: tmpl.hasRememberOption,
				createdAt: Date.now()
			};
		} else {
			const tmpl = CATALOG_TEMPLATES.morningBriefing;
			item = {
				id: `sim-penting-${Date.now()}`,
				situationId: tmpl.situationId,
				level: tmpl.level,
				title: tmpl.title,
				description: tmpl.description,
				bubbleText: tmpl.bubbleText,
				expression: tmpl.expression,
				actions: tmpl.actions,
				createdAt: Date.now()
			};
		}

		emit("hime-proactive-event", item).catch(() => {});
	};

	const toggleHimeEnabled = () => {
		const next = !himeEnabled;
		setHimeEnabled(next);
		localStorage.setItem("hime-enabled", next ? "true" : "false");
	};

	const handleNotchModeChange = (mode: string) => {
		setCurrentNotchMode(mode);
		localStorage.setItem("bloom-notch-mode", mode);
		invoke("save_setting", { key: "bloom-notch-mode", value: mode }).catch(() => {});
	};

	const handleOpen9Router = async () => {
		const target = "http://127.0.0.1:20128";
		try {
			await openUrl(target);
		} catch {
			window.open(target, "_blank");
		}
	};

	const handleSaveRouterConfig = () => {
		localStorage.setItem("hime-9router-url", routerUrl.trim());
		localStorage.setItem("hime-9router-key", routerKey.trim());
		const effectiveModel = selectedModel === "custom" ? customModel.trim() : selectedModel;
		localStorage.setItem("hime-9router-model", selectedModel);
		if (selectedModel === "custom") {
			localStorage.setItem("hime-9router-custom-model", customModel.trim());
		}
		invoke("save_setting", { key: "hime-9router-url", value: routerUrl.trim() }).catch(() => {});
		invoke("save_setting", { key: "hime-9router-model", value: effectiveModel }).catch(() => {});

		if (routerKey.trim()) {
			const provider = selectedModel.includes("gemini")
				? "gemini"
				: selectedModel.includes("claude")
				? "anthropic"
				: selectedModel.includes("gpt")
				? "openai"
				: "9router";
			storeCredential(provider, routerKey.trim()).catch(() => {});
		}

		setRouterTestResult("Konfigurasi tersimpan aman.");
		setTimeout(() => setRouterTestResult(null), 3000);
	};

	const handleTestRouterConnection = async () => {
		setTestingRouter(true);
		setRouterTestResult(null);
		try {
			const targetUrl = routerUrl.trim().replace(/\/$/, "");
			const endpoint = targetUrl.endsWith("/v1") ? `${targetUrl}/models` : `${targetUrl}/v1/models`;
			const res = await fetch(endpoint, {
				method: "GET",
				headers: routerKey.trim() ? { Authorization: `Bearer ${routerKey.trim()}` } : {}
			});
			if (res.ok) {
				setRouterTestResult("9Router online & terhubung!");
			} else {
				setRouterTestResult(`Gateway merespon HTTP ${res.status}`);
			}
		} catch (err: any) {
			setRouterTestResult(`Koneksi gagal: ${err?.message || "Gateway offline"}`);
		} finally {
			setTestingRouter(false);
		}
	};

	const handlePing = async () => {
		setPinging(true);
		try {
			const ms = await pingCore();
			setPingResult(`Latency: ${ms} ms`);
			onRefresh();
		} catch (err) {
			setPingResult(`Ping error: ${err}`);
		} finally {
			setPinging(false);
		}
	};

	const handleTestApproval = async () => {
		setTriggering(true);
		try {
			await devTriggerApproval(
				"file.delete",
				"X:/Downloads/temp_sandbox_artifact.tmp",
				"Permintaan persetujuan: Hapus file sementara dari sandbox"
			);
			onRefresh();
		} catch (err) {
			console.error("Trigger test approval error:", err);
		} finally {
			setTriggering(false);
		}
	};

	return (
		<>
			{/* Section: Asisten Hime */}
			<div className="setting-group-label">Asisten Hime</div>
			<div className="setting-group">
				<SettingRow
					icon={Bot}
					label="Asisten Hime"
					desc="Personal AI desktop Hime"
				>
					<label className="toggle-switch">
						<input type="checkbox" checked={himeEnabled} onChange={toggleHimeEnabled} />
						<span className="slider" />
					</label>
				</SettingRow>

				<SettingRow
					icon={Radio}
					label="Status Core"
					desc={
						isOnline
							? `Core v${status.core_version ?? "0.1.0"} (${status.last_ping_ms ?? 0} ms)`
							: "Core offline, IPC terputus"
					}
					divider={false}
				>
					<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
						<span
							style={{
								width: "8px",
								height: "8px",
								borderRadius: "50%",
								backgroundColor: isOnline ? "#10b981" : "#737373",
								boxShadow: isOnline ? "0 0 6px rgba(16, 185, 129, 0.6)" : "none"
							}}
						/>
						<button
							type="button"
							disabled={pinging}
							onClick={handlePing}
							style={{
								display: "flex",
								alignItems: "center",
								gap: "4px",
								background: "rgba(255, 255, 255, 0.08)",
								border: "1px solid rgba(255, 255, 255, 0.12)",
								borderRadius: "6px",
								padding: "4px 8px",
								color: "#e5e5e5",
								fontSize: "11px",
								cursor: pinging ? "not-allowed" : "pointer"
							}}
							title="Uji latency IPC Win32 Named Pipe"
						>
							<RefreshCw size={11} className={pinging ? "animate-spin" : ""} />
							<span>{pinging ? "Menguji..." : pingResult ? "Online" : "Uji Ping"}</span>
						</button>
					</div>
				</SettingRow>
			</div>

			{/* Section: 9Router AI Gateway */}
			<div className="setting-group-label">9Router Gateway</div>
			<div className="setting-group">
				<SettingRow
					icon={ExternalLink}
					label="Web UI 9Router"
					desc="Buka 127.0.0.1:20128"
					action
					onClick={handleOpen9Router}
				>
					<button
						type="button"
						onClick={(e) => {
							e.stopPropagation();
							handleOpen9Router();
						}}
						style={{
							display: "flex",
							alignItems: "center",
							gap: "6px",
							background: "#0284c7",
							border: "none",
							borderRadius: "6px",
							padding: "5px 12px",
							color: "#ffffff",
							fontSize: "11px",
							fontWeight: 600,
							cursor: "pointer"
						}}
					>
						<ExternalLink size={12} />
						<span>Buka UI</span>
					</button>
				</SettingRow>

				<SettingRow
					icon={Server}
					label="URL 9Router"
					desc="Endpoint HTTP gateway lokal"
				>
					<input
						type="text"
						value={routerUrl}
						onChange={(e) => setRouterUrl(e.target.value)}
						placeholder="http://127.0.0.1:20128/v1"
						style={{
							background: "var(--bloom-group-bg, rgba(255, 255, 255, 0.08))",
							border: "1px solid var(--bloom-border, rgba(255, 255, 255, 0.12))",
							borderRadius: "6px",
							padding: "4px 8px",
							color: "var(--bloom-text, #ffffff)",
							fontSize: "11px",
							fontFamily: "monospace",
							width: "190px",
							outline: "none"
						}}
					/>
				</SettingRow>

				<SettingRow
					icon={Key}
					label="Token 9Router"
					desc="Kunci autentikasi 9Router"
				>
					<div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
						<input
							type={showKey ? "text" : "password"}
							value={routerKey}
							onChange={(e) => setRouterKey(e.target.value)}
							placeholder="Opsional / Token..."
							style={{
								background: "var(--bloom-group-bg, rgba(255, 255, 255, 0.08))",
								border: "1px solid var(--bloom-border, rgba(255, 255, 255, 0.12))",
								borderRadius: "6px",
								padding: "4px 8px",
								color: "var(--bloom-text, #ffffff)",
								fontSize: "11px",
								fontFamily: "monospace",
								width: "140px",
								outline: "none"
							}}
						/>
						<button
							type="button"
							onClick={() => setShowKey(!showKey)}
							style={{
								background: "none",
								border: "none",
								color: "#a3a3a3",
								cursor: "pointer",
								padding: "4px",
								display: "flex",
								alignItems: "center"
							}}
							title={showKey ? "Sembunyikan Token" : "Tampilkan Token"}
						>
							{showKey ? <EyeOff size={13} /> : <Eye size={13} />}
						</button>
					</div>
				</SettingRow>

				<SettingRow
					icon={Cpu}
					label="Model Default"
					desc="Model utama respon chat"
				>
					<div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
						<select
							className="settings-select"
							value={selectedModel}
							onChange={(e) => setSelectedModel(e.target.value)}
						>
							<option value="anthropic/claude-3-5-sonnet">Claude 3.5</option>
							<option value="openai/gpt-4o">GPT-4o</option>
							<option value="google/gemini-1.5-pro">Gemini 1.5 Pro</option>
							<option value="deepseek/deepseek-chat">DeepSeek Chat V3</option>
							<option value="nousresearch/hermes-3-llama-3.1-8b">Hermes 3 (Local)</option>
							<option value="custom">Model Kustom...</option>
						</select>
						{selectedModel === "custom" && (
							<input
								type="text"
								value={customModel}
								onChange={(e) => setCustomModel(e.target.value)}
								placeholder="provider/model-id"
								style={{
									background: "var(--bloom-group-bg, rgba(255, 255, 255, 0.08))",
									border: "1px solid var(--bloom-border, rgba(255, 255, 255, 0.12))",
									borderRadius: "6px",
									padding: "4px 8px",
									color: "var(--bloom-text, #ffffff)",
									fontSize: "11px",
									width: "120px",
									outline: "none"
								}}
							/>
						)}
					</div>
				</SettingRow>

				<SettingRow
					icon={Activity}
					label="Uji & Simpan"
					desc={
						routerTestResult
							? routerTestResult.slice(0, 36)
							: "Verifikasi gateway dan simpan"
					}
					divider={false}
				>
					<div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
						<button
							type="button"
							disabled={testingRouter}
							onClick={handleTestRouterConnection}
							style={{
								display: "flex",
								alignItems: "center",
								gap: "4px",
								background: "rgba(56, 189, 248, 0.12)",
								border: "1px solid rgba(56, 189, 248, 0.25)",
								borderRadius: "6px",
								padding: "4px 8px",
								color: "#38bdf8",
								fontSize: "11px",
								cursor: testingRouter ? "not-allowed" : "pointer"
							}}
						>
							<Activity size={11} className={testingRouter ? "animate-spin" : ""} />
							<span>{testingRouter ? "Menguji..." : "Uji Coba"}</span>
						</button>

						<button
							type="button"
							onClick={handleSaveRouterConfig}
							style={{
								display: "flex",
								alignItems: "center",
								gap: "4px",
								background: "#10b981",
								border: "none",
								borderRadius: "6px",
								padding: "4px 10px",
								color: "#ffffff",
								fontSize: "11px",
								fontWeight: 600,
								cursor: "pointer"
							}}
						>
							<Check size={11} />
							<span>Simpan</span>
						</button>
					</div>
				</SettingRow>
			</div>

			{/* Section: Layar & Notch */}
			<div className="setting-group-label">Layar & Notch</div>
			<div className="setting-group">
				<SettingRow
					icon={PanelTop}
					label="Perilaku Layar"
					desc="Otomatis tipis saat layar penuh"
					divider={false}
				>
					<select
						className="settings-select"
						value={currentNotchMode}
						onChange={(e) => handleNotchModeChange(e.target.value)}
					>
						<option value="smart">Smart (Otomatis tipis fullscreen)</option>
						<option value="fixed">Fixed (Selalu tampil di atas)</option>
						<option value="peek">Peek (Hanya saat disentuh kursor)</option>
					</select>
				</SettingRow>
			</div>

			{/* Section: Hime Proaktif & Anggaran Perhatian */}
			<div className="setting-group-label">Hime Proaktif</div>
			<div className="setting-group">
				<SettingRow
					icon={HelpCircle}
					label="Tingkat Sapa"
					desc="Anggaran perhatian per jam"
				>
					<select
						className="settings-select"
						value={proactiveBudget}
						onChange={(e) => handleBudgetChange(e.target.value as AttentionBudget)}
					>
						<option value="pendiam">Pendiam (1 colek, 1 tanya/jam)</option>
						<option value="seimbang">Seimbang (3 colek, 3 tanya/jam)</option>
						<option value="cerewet">Cerewet (6 colek, 5 tanya/jam)</option>
					</select>
				</SettingRow>

				<SettingRow
					icon={Clock}
					label="Jam Tenang"
					desc="Hanya P0 pada 23:00–06:00"
				>
					<label className="toggle-switch">
						<input type="checkbox" checked={quietHours} onChange={handleToggleQuietHours} />
						<span className="slider" />
					</label>
				</SettingRow>

				<SettingRow
					icon={RotateCcw}
					label="Reset Backoff"
					desc={backoffResetNotice ? backoffResetNotice.slice(0, 36) : "Reset aturan dan penolakan"}
					action
					onClick={handleResetBackoff}
					divider={false}
				>
					<button
						type="button"
						onClick={(e) => {
							e.stopPropagation();
							handleResetBackoff();
						}}
						style={{
							background: "rgba(255, 255, 255, 0.08)",
							border: "1px solid rgba(255, 255, 255, 0.15)",
							borderRadius: "6px",
							padding: "4px 10px",
							color: "#e2e8f0",
							fontSize: "11px",
							cursor: "pointer"
						}}
					>
						{"Reset".slice(0, 10)}
					</button>
				</SettingRow>
			</div>

			{/* Section: Maskot & Mata */}
			<div className="setting-group-label">Maskot & Mata</div>
			<div className="setting-group">
				<SettingRow
					icon={Bot}
					label="Uji Ekspresi"
					desc="Pratinjau 22 ekspresi maskot"
					divider={false}
				>
					<div style={{ width: "100%", marginTop: "8px" }}>
						<ExpressionTester />
					</div>
				</SettingRow>
			</div>

			{/* Section: Pengembang & Keamanan */}
			<div className="setting-group-label">Pengembang</div>
			<div className="setting-group">
				<SettingRow
					icon={Sparkles}
					label="Tes Proaktif"
					desc="Uji 4 tingkat interaksi"
				>
					<div style={{ display: "flex", gap: "4px" }}>
						<button
							type="button"
							onClick={() => handleSimulateProactive("bisik")}
							style={{
								background: "rgba(168, 85, 247, 0.15)",
								border: "1px solid rgba(168, 85, 247, 0.3)",
								borderRadius: "6px",
								padding: "4px 8px",
								color: "#c084fc",
								fontSize: "10px",
								fontWeight: 600,
								cursor: "pointer"
							}}
							title="Bisik: Titik di pill + mata penasaran"
						>
							Bisik
						</button>
						<button
							type="button"
							onClick={() => handleSimulateProactive("colek")}
							style={{
								background: "rgba(245, 158, 11, 0.15)",
								border: "1px solid rgba(245, 158, 11, 0.3)",
								borderRadius: "6px",
								padding: "4px 8px",
								color: "#fbbf24",
								fontSize: "10px",
								fontWeight: 600,
								cursor: "pointer"
							}}
							title="Colek: Rekap malam (auto-hilang)"
						>
							Colek
						</button>
						<button
							type="button"
							onClick={() => handleSimulateProactive("tanya")}
							style={{
								background: "rgba(56, 189, 248, 0.15)",
								border: "1px solid rgba(56, 189, 248, 0.3)",
								borderRadius: "6px",
								padding: "4px 8px",
								color: "#38bdf8",
								fontSize: "10px",
								fontWeight: 600,
								cursor: "pointer"
							}}
							title="Tanya: File ganda (<= 3 opsi + Ingat)"
						>
							Tanya
						</button>
						<button
							type="button"
							onClick={() => handleSimulateProactive("penting")}
							style={{
								background: "rgba(239, 68, 68, 0.15)",
								border: "1px solid rgba(239, 68, 68, 0.3)",
								borderRadius: "6px",
								padding: "4px 8px",
								color: "#f87171",
								fontSize: "10px",
								fontWeight: 600,
								cursor: "pointer"
							}}
							title="Penting: Ringkasan pagi + ekspansi + suara"
						>
							Penting
						</button>
					</div>
				</SettingRow>

				<SettingRow
					icon={ShieldAlert}
					label="Tes Approval"
					desc="Kirim simulasi bubble izin"
					action
					onClick={handleTestApproval}
					divider={false}
				>
					<button
						type="button"
						disabled={triggering}
						onClick={(e) => {
							e.stopPropagation();
							handleTestApproval();
						}}
						style={{
							background: "rgba(245, 158, 11, 0.15)",
							border: "1px solid rgba(245, 158, 11, 0.3)",
							borderRadius: "6px",
							padding: "4px 10px",
							color: "#fbbf24",
							fontSize: "11px",
							fontWeight: 600,
							cursor: triggering ? "not-allowed" : "pointer"
						}}
					>
						{triggering ? "Kirim..." : "Tes Izin"}
					</button>
				</SettingRow>
			</div>
		</>
	);
};
