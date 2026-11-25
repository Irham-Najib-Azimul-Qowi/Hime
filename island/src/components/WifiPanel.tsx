import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import {
	ChevronLeft,
	LoaderCircle,
	Lock,
	MapPin,
	Network,
	RefreshCw,
	Wifi,
	WifiHigh,
	WifiLow,
	WifiZero
} from "lucide-react";
import "./QuickPanel.css";

export interface WifiNetwork {
	ssid: string;
	signal: number;
	security: string;
	connected: boolean;
	saved: boolean;
}

export interface WifiNetworkList {
	networks: WifiNetwork[];
	location_denied: boolean;
}

// Personal networks take a typed password; anything else that isn't open
// (work networks with sign-in, WEP) is handed to Windows Settings.
const PERSONAL = new Set(["wpa", "wpa2", "wpa3"]);

const ERROR_TEXT: Record<string, string> = {
	"wrong-password": "Wrong password",
	"password-required": "Enter the password",
	"password-length": "Wi-Fi passwords are 8 to 63 characters",
	unsupported: "Connect to this network in Windows Settings",
	"not-found": "This network is out of range",
	timeout: "Couldn't connect in time",
	"no-adapter": "No Wi-Fi adapter found",
	"location-denied": "Location access is off"
};

function SignalIcon({ signal }: { signal: number }) {
	if (signal >= 75) return <Wifi />;
	if (signal >= 50) return <WifiHigh />;
	if (signal >= 25) return <WifiLow />;
	return <WifiZero />;
}

export interface WifiPanelProps {
	enabled: boolean;
	onToggle: () => void;
	onBack: () => void;
	/** Keeps the notch open while a password is being typed. */
	onPinChange: (pinned: boolean) => void;
	/** Optional callback when network status or connectivity drops */
	onNetworkDrop?: (hasInternet: boolean, expression: "khawatir" | "sedih" | "netral") => void;
	/** Optional ethernet connection status */
	isEthernet?: boolean;
	ethernetName?: string | null;
	/** Optional internet connectivity status from backend */
	hasInternet?: boolean;
}

/**
 * Checks active internet connectivity via backend check and lightweight endpoint probe.
 */
export async function probeInternetConnectivity(): Promise<boolean> {
	if (typeof navigator !== "undefined" && typeof navigator.onLine === "boolean" && !navigator.onLine) {
		return false;
	}
	try {
		const res = await invoke<boolean>("check_internet_connectivity");
		return Boolean(res);
	} catch {
		try {
			const controller = new AbortController();
			const timeout = setTimeout(() => controller.abort(), 2500);
			await fetch("https://1.1.1.1/generate_204", {
				method: "HEAD",
				mode: "no-cors",
				cache: "no-store",
				signal: controller.signal
			});
			clearTimeout(timeout);
			return true;
		} catch {
			return typeof navigator !== "undefined" ? navigator.onLine : false;
		}
	}
}

export function WifiPanel({
	enabled,
	onToggle,
	onBack,
	onPinChange,
	onNetworkDrop,
	isEthernet: propIsEthernet,
	ethernetName: propEthernetName,
	hasInternet: propHasInternet
}: WifiPanelProps) {
	const [list, setList] = useState<WifiNetworkList | null>(null);
	const [isEthernet, setIsEthernet] = useState<boolean>(propIsEthernet ?? false);
	const [ethernetName, setEthernetName] = useState<string | null>(propEthernetName ?? null);
	const [scanning, setScanning] = useState(false);
	const [selected, setSelected] = useState<string | null>(null);
	// A saved network whose stored password stopped working also gets a field.
	const [askPassword, setAskPassword] = useState<string | null>(null);
	const [password, setPassword] = useState("");
	const [busy, setBusy] = useState<string | null>(null);
	const [error, setError] = useState<{ ssid: string; text: string } | null>(null);
	const [hasInternet, setHasInternet] = useState<boolean>(() => {
		if (typeof propHasInternet === "boolean") return propHasInternet;
		if (typeof navigator !== "undefined" && typeof navigator.onLine === "boolean") {
			return navigator.onLine;
		}
		return true;
	});
	const scanFallback = useRef<number | null>(null);
	const lastOnlineState = useRef<boolean>(true);

	useEffect(() => {
		if (typeof propIsEthernet === "boolean") {
			setIsEthernet(propIsEthernet);
		} else {
			invoke<[boolean, string | null]>("get_ethernet_status")
				.then(([connected, name]) => {
					setIsEthernet(connected);
					if (name) setEthernetName(name);
				})
				.catch(() => {});
		}
		if (propEthernetName !== undefined) {
			setEthernetName(propEthernetName);
		}
	}, [propIsEthernet, propEthernetName]);

	useEffect(() => {
		if (typeof propHasInternet === "boolean") {
			setHasInternet(propHasInternet);
		}
	}, [propHasInternet]);

	const notifyNetworkDrop = useCallback(
		(online: boolean, connectedSsid?: string) => {
			const effectiveOnline = online || isEthernet;
			if (lastOnlineState.current === effectiveOnline) return;
			lastOnlineState.current = effectiveOnline;

			const expr = effectiveOnline ? "netral" : "khawatir";
			onNetworkDrop?.(effectiveOnline, expr);

			if (typeof window !== "undefined" && typeof window.dispatchEvent === "function") {
				// 1. Dispatch custom event for Hime network status
				window.dispatchEvent(
					new CustomEvent("hime:network-drop", {
						detail: {
							hasInternet: effectiveOnline,
							expression: expr,
							ssid: connectedSsid || (isEthernet ? (ethernetName || "Ethernet") : undefined),
							status: effectiveOnline ? "connected" : "Tidak ada koneksi internet"
						}
					})
				);

				// 2. Dispatch eye expression override for notch mascot
				window.dispatchEvent(
					new CustomEvent("hime:expression", {
						detail: { expression: expr }
					})
				);

				// 3. Emit / retract NCM critical problem alert notification
				if (!effectiveOnline) {
					window.dispatchEvent(
						new CustomEvent("hime:ncm-submit", {
							detail: {
								id: "network-drop-alert",
								module: "wifi",
								kind: "alert",
								priority: 0,
								ttl: 8000,
								expression: "khawatir",
								payload: {
									title: "Tidak ada koneksi internet",
									text: connectedSsid
										? `Terhubung ke "${connectedSsid}" tanpa akses internet.`
										: "Tidak ada koneksi internet pada perangkat."
								},
								createdAt: Date.now()
							}
						})
					);
				} else {
					window.dispatchEvent(
						new CustomEvent("hime:ncm-retract", {
							detail: { id: "network-drop-alert" }
						})
					);
				}
			}

			// 4. Emit Tauri event for cross-window and desktop shell sync
			try {
				emit("hime-network-status", {
					hasInternet: effectiveOnline,
					expression: expr,
					ssid: connectedSsid || (isEthernet ? (ethernetName || "Ethernet") : undefined),
					status: effectiveOnline ? "connected" : "Tidak ada koneksi internet"
				}).catch(() => {});
			} catch {}
		},
		[onNetworkDrop, isEthernet, ethernetName]
	);

	const checkAndSyncConnectivity = useCallback(async () => {
		const online = await probeInternetConnectivity();
		setHasInternet(online);
		const connected = list?.networks.find((n) => n.connected);
		notifyNetworkDrop(online, connected?.ssid);
		return online;
	}, [list, notifyNetworkDrop]);

	const refresh = useCallback(async () => {
		try {
			const res = await invoke<WifiNetworkList>("wifi_list_networks");
			setList(res);
			const connected = res.networks.find((n) => n.connected);
			void probeInternetConnectivity().then((online) => {
				setHasInternet(online);
				notifyNetworkDrop(online, connected?.ssid);
			});
		} catch (e) {
			console.error("[wifi] list failed:", e);
			setList({ networks: [], location_denied: false });
		}
	}, [notifyNetworkDrop]);

	const scan = useCallback(async () => {
		setScanning(true);
		if (scanFallback.current !== null) window.clearTimeout(scanFallback.current);
		// Results arrive as a wifi-changed event; stop the spinner regardless.
		scanFallback.current = window.setTimeout(() => setScanning(false), 8000);
		try {
			await invoke("wifi_scan");
		} catch (e) {
			console.error("[wifi] scan failed:", e);
			setScanning(false);
		}
	}, []);

	useEffect(() => {
		if (!enabled) {
			setHasInternet(false);
			notifyNetworkDrop(false);
			return;
		}
		void refresh();
		void scan();
	}, [enabled, refresh, scan, notifyNetworkDrop]);

	// Listen to browser network changes
	useEffect(() => {
		const handleOnline = () => {
			void checkAndSyncConnectivity();
		};
		const handleOffline = () => {
			setHasInternet(false);
			const connected = list?.networks.find((n) => n.connected);
			notifyNetworkDrop(false, connected?.ssid);
		};

		if (typeof window !== "undefined") {
			window.addEventListener("online", handleOnline);
			window.addEventListener("offline", handleOffline);
		}

		return () => {
			if (typeof window !== "undefined") {
				window.removeEventListener("online", handleOnline);
				window.removeEventListener("offline", handleOffline);
			}
		};
	}, [checkAndSyncConnectivity, list, notifyNetworkDrop]);

	useEffect(() => {
		let debounce: number | null = null;
		const unlisten = listen("wifi-changed", () => {
			if (debounce !== null) window.clearTimeout(debounce);
			debounce = window.setTimeout(() => {
				debounce = null;
				setScanning(false);
				void refresh();
			}, 250);
		});
		return () => {
			if (debounce !== null) window.clearTimeout(debounce);
			if (scanFallback.current !== null) window.clearTimeout(scanFallback.current);
			unlisten.then((f) => f());
			onPinChange(false);
		};
	}, [refresh, onPinChange]);

	const collapse = () => {
		setSelected(null);
		setAskPassword(null);
		setPassword("");
		setError(null);
		onPinChange(false);
	};

	const toggleRow = (ssid: string) => {
		if (selected === ssid) {
			collapse();
			return;
		}
		collapse();
		setSelected(ssid);
	};

	const connect = async (network: WifiNetwork, withPassword: boolean) => {
		setBusy(network.ssid);
		setError(null);
		try {
			await invoke("wifi_connect", {
				ssid: network.ssid,
				password: withPassword ? password : null
			});
			collapse();
			void checkAndSyncConnectivity();
		} catch (e) {
			const code = String(e);
			if (network.saved && PERSONAL.has(network.security) && !withPassword) {
				setAskPassword(network.ssid);
			}
			setError({ ssid: network.ssid, text: ERROR_TEXT[code] ?? code });
		} finally {
			setBusy(null);
			void refresh();
		}
	};

	const disconnect = async () => {
		try {
			await invoke("wifi_disconnect");
			collapse();
			notifyNetworkDrop(false);
		} catch (e) {
			console.error("[wifi] disconnect failed:", e);
		}
	};

	const forget = async (network: WifiNetwork) => {
		try {
			await invoke("wifi_forget", { ssid: network.ssid });
			collapse();
			void refresh();
		} catch (e) {
			setError({ ssid: network.ssid, text: ERROR_TEXT[String(e)] ?? String(e) });
		}
	};

	const statusText = (n: WifiNetwork) => {
		if (busy === n.ssid) return "Menghubungkan…";
		if (n.connected) {
			return hasInternet ? "Connected" : "Terhubung, Tanpa Internet";
		}
		if (n.saved) return "Tersimpan";
		return n.security === "open" ? "Terbuka" : "Aman";
	};

	const renderActions = (n: WifiNetwork) => {
		const isBusy = busy === n.ssid;
		const needsPassword = (!n.saved && PERSONAL.has(n.security)) || askPassword === n.ssid;
		const handledByWindows = !n.saved && n.security !== "open" && !PERSONAL.has(n.security);

		if (handledByWindows) {
			return (
				<div className="qp-row-actions">
					<span className="qp-note">Jaringan ini membutuhkan proses sign-in Windows.</span>
					<button className="qp-btn" onClick={() => invoke("wifi_open_settings", { page: "wifi" })}>
						Buka Pengaturan
					</button>
				</div>
			);
		}

		return (
			<div className="qp-row-actions">
				{needsPassword && !n.connected && (
					<input
						className="qp-input"
						type="password"
						placeholder="Kata Sandi"
						autoFocus
						value={password}
						disabled={isBusy}
						onChange={(e) => setPassword(e.target.value)}
						onFocus={() => onPinChange(true)}
						onBlur={() => onPinChange(false)}
						onKeyDown={(e) => {
							e.stopPropagation();
							if (e.key === "Enter" && password) void connect(n, true);
							if (e.key === "Escape") collapse();
						}}
					/>
				)}
				<div className="qp-btn-row">
					{n.connected ? (
						<button className="qp-btn" onClick={disconnect}>
							Putuskan
						</button>
					) : (
						<button
							className="qp-btn primary"
							disabled={isBusy || (needsPassword && !password)}
							onClick={() => void connect(n, needsPassword)}
						>
							{isBusy ? <LoaderCircle className="qp-spin" /> : "Hubungkan"}
						</button>
					)}
					{n.saved && (
						<button className="qp-btn" disabled={isBusy} onClick={() => void forget(n)}>
							Lupakan
						</button>
					)}
				</div>
			</div>
		);
	};

	const renderBody = () => {
		if (!enabled) {
			if (isEthernet) {
				return (
					<div className="qp-ethernet-card">
						<span className="qp-row-icon qp-ethernet-icon">
							<Network size={14} />
						</span>
						<div className="qp-row-text">
							<span className="qp-row-name">{ethernetName || "Ethernet"}</span>
							<span className="qp-row-sub text-success">Terhubung ke LAN</span>
						</div>
						<span className="qp-ethernet-badge">LAN</span>
					</div>
				);
			}
			return <div className="qp-empty">Tidak terhubung (Wi-Fi mati)</div>;
		}
		if (list?.location_denied) {
			return (
				<div className="qp-empty">
					<MapPin />
					<span>Windows hanya menampilkan daftar Wi-Fi untuk aplikasi dengan izin lokasi.</span>
					<button
						className="qp-btn"
						onClick={() => invoke("wifi_open_settings", { page: "location" })}
					>
						Pengaturan Lokasi
					</button>
				</div>
			);
		}
		if (list === null) {
			return (
				<div className="qp-empty">
					<LoaderCircle className="qp-spin" />
				</div>
			);
		}
		if (list.networks.length === 0) {
			if (isEthernet) {
				return (
					<div className="qp-ethernet-card">
						<span className="qp-row-icon qp-ethernet-icon">
							<Network size={14} />
						</span>
						<div className="qp-row-text">
							<span className="qp-row-name">{ethernetName || "Ethernet"}</span>
							<span className="qp-row-sub text-success">Terhubung ke LAN</span>
						</div>
						<span className="qp-ethernet-badge">LAN</span>
					</div>
				);
			}
			return (
				<div className="qp-empty">{scanning ? "Mencari jaringan…" : "Tidak terhubung"}</div>
			);
		}

		const connectedNet = list.networks.find((n) => n.connected);

		return (
			<>
				{isEthernet && (
					<div className="qp-ethernet-card">
						<span className="qp-row-icon qp-ethernet-icon">
							<Network size={14} />
						</span>
						<div className="qp-row-text">
							<span className="qp-row-name">{ethernetName || "Ethernet"}</span>
							<span className={`qp-row-sub ${!hasInternet ? "text-danger" : "text-success"}`}>
								{hasInternet ? "Terhubung ke LAN" : "Terhubung, Tanpa Internet"}
							</span>
						</div>
						<span className="qp-ethernet-badge">LAN</span>
					</div>
				)}
				{!hasInternet && (
					<div className="qp-warning-banner" role="alert">
						<WifiZero className="qp-warning-icon" />
						<div className="qp-warning-text">
							<span className="qp-warning-title">Tanpa Internet</span>
							{connectedNet ? (
								<span className="qp-warning-sub">Terhubung ke {connectedNet.ssid} tanpa akses internet</span>
							) : isEthernet ? (
								<span className="qp-warning-sub">Terhubung ke LAN tanpa akses internet</span>
							) : (
								<span className="qp-warning-sub">Tidak ada koneksi internet pada perangkat</span>
							)}
						</div>
					</div>
				)}
				<ul className="qp-list">
					{list.networks.map((n) => {
						const expanded = selected === n.ssid;
						const isNoInternet = n.connected && !hasInternet;
						return (
							<li
								key={n.ssid}
								className={`qp-row ${expanded ? "expanded" : ""} ${n.connected ? "active" : ""} ${isNoInternet ? "no-internet" : ""}`}
							>
								<button
									className="qp-row-main"
									onClick={() => toggleRow(n.ssid)}
									aria-expanded={expanded}
								>
									<span className="qp-row-icon">
										{isNoInternet ? <WifiZero /> : <SignalIcon signal={n.signal} />}
									</span>
									<span className="qp-row-text">
										<span className="qp-row-name">{n.ssid}</span>
										<span className={`qp-row-sub ${isNoInternet ? "text-danger" : ""}`}>
											{statusText(n)}
										</span>
									</span>
									{n.security !== "open" && <Lock className="qp-row-badge" />}
								</button>
								{expanded && renderActions(n)}
								{error?.ssid === n.ssid && <div className="qp-error">{error.text}</div>}
							</li>
						);
					})}
				</ul>
			</>
		);
	};

	return (
		<div
			className="qp"
			style={{
				position: "relative",
				top: 0,
				left: 0,
				width: "100%",
				height: "100%",
				borderBottomLeftRadius: "inherit",
				borderBottomRightRadius: "inherit",
				boxSizing: "border-box"
			}}
			onClick={(e) => e.stopPropagation()}
		>
			<div className="qp-header">
				<button className="qp-icon-btn" onClick={onBack} aria-label="Back to controls">
					<ChevronLeft />
				</button>
				<span className="qp-title" style={{ display: "inline-flex", alignItems: "center", gap: 5 }}>
					{isEthernet && <Network size={14} style={{ color: "#38D6F5" }} />}
					{isEthernet ? "Ethernet / Wi-Fi" : "Wi-Fi"}
				</span>
				<button
					className="qp-icon-btn"
					onClick={() => void scan()}
					disabled={!enabled || scanning}
					aria-label="Scan for networks"
				>
					<RefreshCw className={scanning ? "qp-spin" : ""} />
				</button>
				<button
					className={`qp-switch qp-switch--wifi ${enabled ? "on" : ""}`}
					role="switch"
					aria-checked={enabled}
					aria-label="Wi-Fi"
					onClick={onToggle}
				>
					<span />
				</button>
			</div>
			<div className="qp-body">{renderBody()}</div>
			<button className="qp-footer" onClick={() => invoke("wifi_open_settings", { page: "wifi" })}>
				Pengaturan Wi-Fi Lainnya
			</button>
		</div>
	);
}
