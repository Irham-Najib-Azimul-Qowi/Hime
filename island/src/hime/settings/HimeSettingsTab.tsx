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
	EyeOff
} from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { SettingRow } from "../../settings/SettingRow";
import { HimeCoreStatus } from "../types";
import { devTriggerApproval, pingCore, storeCredential } from "../api";

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

		setRouterTestResult("Konfigurasi 9Router & DPAPI Vault tersimpan aman.");
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
			{/* Section: Hime AI Assistant */}
			<div className="setting-group-label">Hime AI Assistant</div>
			<div className="setting-group">
				<SettingRow
					icon={Bot}
					label="Aktifkan Hime Assistant"
					desc="Integrasi personal AI desktop environment Hime Island"
				>
					<label className="toggle-switch">
						<input type="checkbox" checked={himeEnabled} onChange={toggleHimeEnabled} />
						<span className="slider" />
					</label>
				</SettingRow>

				<SettingRow
					icon={Radio}
					label="Status Layanan Hime Core"
					desc={
						isOnline
							? `Core v${status.core_version ?? "0.1.0"} · Named pipe terhubung (${status.last_ping_ms ?? 0} ms)`
							: "Core Offline — Named pipe IPC belum tersambung"
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
							<span>{pinging ? "Menguji..." : pingResult || "Uji Ping"}</span>
						</button>
					</div>
				</SettingRow>
			</div>

			{/* Section: 9Router AI Gateway */}
			<div className="setting-group-label">9Router AI Gateway & Model Provider</div>
			<div className="setting-group">
				<SettingRow
					icon={ExternalLink}
					label="Buka Dashboard Web UI 9Router"
					desc="Akses http://127.0.0.1:20128 di browser untuk login, kelola provider, dan sambungkan akun AI Anda"
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
						<span>Buka Web UI</span>
					</button>
				</SettingRow>

				<SettingRow
					icon={Server}
					label="9Router Base URL"
					desc="Endpoint HTTP 9Router lokal untuk merutekan permintaan model"
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
					label="Token / API Key 9Router"
					desc="Kunci otentikasi jika autentikasi diaktifkan pada 9Router"
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
					label="Model Default AI"
					desc="Model yang dipilih untuk merespon pertanyaan di Hime Assistant"
				>
					<div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
						<select
							className="settings-select"
							value={selectedModel}
							onChange={(e) => setSelectedModel(e.target.value)}
						>
							<option value="anthropic/claude-3-5-sonnet">Claude 3.5 Sonnet</option>
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
					label="Uji & Simpan Konfigurasi"
					desc={
						routerTestResult
							? routerTestResult
							: "Verifikasi koneksi gateway lokal dan simpan pengaturan"
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
							<span>{testingRouter ? "Menguji..." : "Uji Koneksi"}</span>
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
			<div className="setting-group-label">Layar & Perilaku Notch</div>
			<div className="setting-group">
				<SettingRow
					icon={PanelTop}
					label="Perilaku Fullscreen Notch"
					desc="Smart: garis tipis saat jendela fullscreen/tab; Fixed: selalu muncul"
					divider={false}
				>
					<select
						className="settings-select"
						value={currentNotchMode}
						onChange={(e) => handleNotchModeChange(e.target.value)}
					>
						<option value="smart">Smart (Otomatis garis tipis saat fullscreen)</option>
						<option value="fixed">Fixed (Selalu tampil di atas jendela)</option>
						<option value="peek">Peek (Hanya muncul saat disentuh kursor)</option>
					</select>
				</SettingRow>
			</div>

			{/* Section: Pengembang & Keamanan */}
			<div className="setting-group-label">Pengembang & Keamanan</div>
			<div className="setting-group">
				<SettingRow
					icon={ShieldAlert}
					label="Simulasi Approval PEP (VC-01)"
					desc="Kirim simulasi aksi berisiko ke Policy Enforcement Point untuk memunculkan Approval Bubble"
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
						{triggering ? "Mengirim..." : "Uji Approval"}
					</button>
				</SettingRow>
			</div>
		</>
	);
};
