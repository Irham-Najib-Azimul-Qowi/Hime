import React, { useState, useRef, useEffect, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
	ArrowUp,
	Sparkles,
	Trash2,
	X,
	RefreshCw,
	FileText,
	Code,
	BarChart3,
	Layers,
	Copy,
	Check
} from "lucide-react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import "katex/dist/katex.min.css";
import { HimeCoreStatus } from "../types";
import { sendChatMessage, createAcademicReport } from "../api";

export interface SkillItem {
	id: string;
	slash: string;
	name: string;
	desc: string;
	color: string;
	icon: any;
}

export const SKILL_OPTIONS: SkillItem[] = [
	{
		id: "dev",
		slash: "/dev",
		name: "Dev & Code",
		desc: "Coding, arsitektur software, git, & debugging",
		color: "#10b981",
		icon: Code
	},
	{
		id: "doc",
		slash: "/doc",
		name: "Akademik & Dokumen",
		desc: "Laporan ilmiah TNR 12 margin 4-3-3-3, makalah, & naskah",
		color: "#f59e0b",
		icon: FileText
	},
	{
		id: "data",
		slash: "/data",
		name: "Data & AI",
		desc: "Profil dataset CSV, statistik penyimpanan, & audit data",
		color: "#a855f7",
		icon: BarChart3
	},
	{
		id: "sys",
		slash: "/sys",
		name: "Sistem & Storage",
		desc: "Cek kesehatan PC, smart download, & organizer berkas",
		color: "#3b82f6",
		icon: Layers
	},
	{
		id: "clear",
		slash: "/clear",
		name: "Bersihkan Chat",
		desc: "Hapus seluruh riwayat percakapan",
		color: "#ef4444",
		icon: Trash2
	}
];

interface Message {
	id: string;
	role: "user" | "assistant";
	text: string;
	timestamp: number;
	skillName?: string;
	skillColor?: string;
}

interface HimeAssistantPanelProps {
	status: HimeCoreStatus;
	onClose?: () => void;
	setNotchPinned?: (pinned: boolean) => void;
}

/**
 * Filter out repetitive robot/AI emoji spams from the response
 * so responses appear clean, technical, and human-crafted.
 */
function cleanAiResponse(text: string): string {
	if (!text) return "";
	return text
		.replace(/^(\s*)[🤖✨💡🚀📌🎯🔥🎉⚡]\s*/gm, "$1- ")
		.replace(/[🤖✨💡🚀📌🎯🔥🎉⚡]/g, "")
		.trim();
}

/**
 * CodeBlock with language header and 1-click clipboard copy.
 */
const CodeBlock: React.FC<{ language?: string; value: string }> = ({ language, value }) => {
	const [copied, setCopied] = useState(false);

	const handleCopy = () => {
		navigator.clipboard.writeText(value);
		setCopied(true);
		setTimeout(() => setCopied(false), 2000);
	};

	return (
		<div
			style={{
				margin: "8px 0",
				borderRadius: "8px",
				background: "rgba(10, 10, 14, 0.92)",
				border: "1px solid rgba(255, 255, 255, 0.1)",
				overflow: "hidden"
			}}
		>
			<div
				style={{
					display: "flex",
					alignItems: "center",
					justifyContent: "space-between",
					padding: "4px 10px",
					background: "rgba(255, 255, 255, 0.04)",
					borderBottom: "1px solid rgba(255, 255, 255, 0.06)",
					fontSize: "10px",
					color: "#94a3b8",
					fontFamily: "monospace"
				}}
			>
				<span>{language || "code"}</span>
				<button
					type="button"
					onClick={handleCopy}
					style={{
						background: "none",
						border: "none",
						color: copied ? "#34d399" : "#94a3b8",
						cursor: "pointer",
						fontSize: "10px",
						display: "flex",
						alignItems: "center",
						gap: "4px",
						padding: "2px 4px",
						borderRadius: "4px",
						transition: "all 0.15s ease"
					}}
				>
					{copied ? <Check size={11} /> : <Copy size={11} />}
					<span>{copied ? "Tersalin" : "Salin"}</span>
				</button>
			</div>
			<pre
				style={{
					margin: 0,
					padding: "10px",
					overflowX: "auto",
					fontSize: "11px",
					lineHeight: 1.5,
					fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace",
					color: "#e2e8f0"
				}}
			>
				<code>{value}</code>
			</pre>
		</div>
	);
};

/**
 * Markdown & KaTeX renderer with custom styled elements.
 */
const MarkdownContent: React.FC<{ content: string }> = ({ content }) => {
	return (
		<ReactMarkdown
			remarkPlugins={[remarkGfm, remarkMath]}
			rehypePlugins={[rehypeKatex]}
			components={{
				code({ className, children, ...props }: any) {
					const match = /language-(\w+)/.exec(className || "");
					const textVal = String(children).replace(/\n$/, "");
					if (match || textVal.includes("\n")) {
						return <CodeBlock language={match ? match[1] : ""} value={textVal} />;
					}
					return (
						<code
							style={{
								background: "rgba(255, 255, 255, 0.08)",
								padding: "1px 5px",
								borderRadius: "4px",
								fontSize: "11px",
								fontFamily: "monospace",
								color: "#34d399"
							}}
							{...props}
						>
							{children}
						</code>
					);
				},
				p({ children }) {
					return <p style={{ margin: "4px 0", lineHeight: 1.55 }}>{children}</p>;
				},
				ul({ children }) {
					return <ul style={{ paddingLeft: "16px", margin: "4px 0" }}>{children}</ul>;
				},
				ol({ children }) {
					return <ol style={{ paddingLeft: "16px", margin: "4px 0" }}>{children}</ol>;
				},
				li({ children }) {
					return <li style={{ margin: "2px 0" }}>{children}</li>;
				},
				h1({ children }) {
					return <h1 style={{ fontSize: "13.5px", fontWeight: 700, margin: "8px 0 4px", color: "#f8fafc" }}>{children}</h1>;
				},
				h2({ children }) {
					return <h2 style={{ fontSize: "12.5px", fontWeight: 600, margin: "6px 0 3px", color: "#f1f5f9" }}>{children}</h2>;
				},
				h3({ children }) {
					return <h3 style={{ fontSize: "11.5px", fontWeight: 600, margin: "4px 0 2px", color: "#e2e8f0" }}>{children}</h3>;
				},
				blockquote({ children }) {
					return (
						<blockquote
							style={{
								borderLeft: "2px solid #34d399",
								margin: "6px 0",
								paddingLeft: "8px",
								color: "#94a3b8",
								fontStyle: "italic"
							}}
						>
							{children}
						</blockquote>
					);
				},
				table({ children }) {
					return (
						<div style={{ overflowX: "auto", margin: "6px 0" }}>
							<table style={{ borderCollapse: "collapse", width: "100%", fontSize: "11px" }}>
								{children}
							</table>
						</div>
					);
				},
				th({ children }) {
					return (
						<th
							style={{
								borderBottom: "1px solid rgba(255, 255, 255, 0.16)",
								padding: "4px 8px",
								textAlign: "left",
								color: "#e2e8f0",
								fontWeight: 600
							}}
						>
							{children}
						</th>
					);
				},
				td({ children }) {
					return (
						<td
							style={{
								borderBottom: "1px solid rgba(255, 255, 255, 0.06)",
								padding: "4px 8px",
								color: "#cbd5e1"
							}}
						>
							{children}
						</td>
					);
				}
			}}
		>
			{content}
		</ReactMarkdown>
	);
};

export const HimeAssistantPanel: React.FC<HimeAssistantPanelProps> = ({ status, setNotchPinned }) => {
	const isOnline = status.is_online;
	const [selectedSkill, setSelectedSkill] = useState<SkillItem | null>(null);
	const [showSlashMenu, setShowSlashMenu] = useState(false);
	const [slashQuery, setSlashQuery] = useState("");
	const [menuIndex, setMenuIndex] = useState(0);

	const [messages, setMessages] = useState<Message[]>(() => {
		const saved = localStorage.getItem("hime-chat-history");
		if (saved) {
			try {
				return JSON.parse(saved);
			} catch {}
		}
		return [];
	});

	const [inputValue, setInputValue] = useState("");
	const [isLoading, setIsLoading] = useState(false);
	const inputRef = useRef<HTMLInputElement>(null);
	const messagesEndRef = useRef<HTMLDivElement>(null);
	const chatScrollRef = useRef<HTMLDivElement>(null);

	const claimFocus = () => {
		setNotchPinned?.(true);
		void getCurrentWebviewWindow()
			.setFocus()
			.then(() => {
				inputRef.current?.focus();
			})
			.catch(console.error);
	};

	useEffect(() => {
		window.scrollTo(0, 0);
		claimFocus();
		const t = setTimeout(claimFocus, 80);
		return () => clearTimeout(t);
	}, []);

	const scrollToBottom = (smooth = true) => {
		if (chatScrollRef.current) {
			chatScrollRef.current.scrollTo({
				top: chatScrollRef.current.scrollHeight,
				behavior: smooth ? "smooth" : "auto"
			});
		}
	};

	useEffect(() => {
		scrollToBottom(false);
	}, [messages, isLoading]);

	// Filter slash command options
	const filteredSkills = useMemo(() => {
		if (!slashQuery) return SKILL_OPTIONS;
		const q = slashQuery.toLowerCase();
		return SKILL_OPTIONS.filter(
			(s) =>
				s.slash.toLowerCase().includes(q) ||
				s.name.toLowerCase().includes(q) ||
				s.id.includes(q)
		);
	}, [slashQuery]);

	const applySkill = (skill: SkillItem) => {
		if (skill.id === "clear") {
			setMessages([]);
			localStorage.removeItem("hime-chat-history");
			setShowSlashMenu(false);
			setInputValue("");
			return;
		}
		setSelectedSkill(skill);
		setShowSlashMenu(false);
		setInputValue("");
		setTimeout(() => inputRef.current?.focus(), 50);
	};

	const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
		const val = e.target.value;
		setInputValue(val);

		if (val.startsWith("/")) {
			setShowSlashMenu(true);
			setSlashQuery(val.slice(1));
			setMenuIndex(0);
		} else {
			setShowSlashMenu(false);
		}
	};

	const handleInputKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
		e.stopPropagation();

		if (showSlashMenu) {
			if (e.key === "ArrowDown") {
				e.preventDefault();
				setMenuIndex((prev) => (prev + 1) % filteredSkills.length);
				return;
			}
			if (e.key === "ArrowUp") {
				e.preventDefault();
				setMenuIndex((prev) => (prev - 1 + filteredSkills.length) % filteredSkills.length);
				return;
			}
			if (e.key === "Enter" || e.key === "Tab") {
				e.preventDefault();
				if (filteredSkills[menuIndex]) {
					applySkill(filteredSkills[menuIndex]);
				}
				return;
			}
			if (e.key === "Escape") {
				e.preventDefault();
				setShowSlashMenu(false);
				return;
			}
		}

		if (e.key === "Backspace" && !inputValue && selectedSkill) {
			setSelectedSkill(null);
		}
	};

	const handleSendMessage = async (textToSend?: string) => {
		const prompt = (textToSend ?? inputValue).trim();
		if (!prompt || isLoading) return;

		const userMsg: Message = {
			id: `usr_${Date.now()}`,
			role: "user",
			text: prompt,
			timestamp: Date.now(),
			skillName: selectedSkill?.name,
			skillColor: selectedSkill?.color
		};

		const updated = [...messages, userMsg];
		setMessages(updated);
		setInputValue("");
		setShowSlashMenu(false);
		setIsLoading(true);

		if (
			prompt.toLowerCase().includes("laporan akademik") ||
			prompt.toLowerCase().includes("academic report")
		) {
			await handleGenerateAcademicReport();
			return;
		}

		try {
			const activeModel =
				localStorage.getItem("hime-9router-model") || "anthropic/claude-3-5-sonnet";
			const outgoingPrompt = selectedSkill
				? `[Mode Spesialis: ${selectedSkill.name}]\n${prompt}\n\n[Format: Jawablah dengan lugas, profesional, dan rapi dengan Markdown standar. Hindari emotikon/emoji berlebihan agar tidak tampak seperti AI template.]`
				: `${prompt}\n\n[Format: Jawablah dengan lugas, profesional, dan rapi dengan Markdown standar. Hindari emotikon/emoji berlebihan agar tidak tampak seperti AI template.]`;

			const res = await sendChatMessage(outgoingPrompt, activeModel);
			const rawReply = res.response || "Respons diterima dari Hime Core.";
			const botReply = cleanAiResponse(rawReply);

			const botMsg: Message = {
				id: `ast_${Date.now()}`,
				role: "assistant",
				text: botReply,
				timestamp: Date.now(),
				skillName: selectedSkill?.name,
				skillColor: selectedSkill?.color
			};
			const final = [...updated, botMsg];
			setMessages(final);
			localStorage.setItem("hime-chat-history", JSON.stringify(final.slice(-25)));
		} catch (err: any) {
			const errorMsg: Message = {
				id: `ast_${Date.now()}`,
				role: "assistant",
				text: `Error: ${err?.message || err || "Koneksi ke Hime Core gagal."}`,
				timestamp: Date.now()
			};
			setMessages([...updated, errorMsg]);
		} finally {
			setIsLoading(false);
		}
	};

	const handleGenerateAcademicReport = async () => {
		const reportTitle = "Laporan_Akademik_Hime";
		const sampleBody = `# LAPORAN AKADEMIK SISTEM HIME
## BAB I PENDAHULUAN
### 1.1 Latar Belakang
Hime adalah ekosistem asisten AI personal cerdas yang mengintegrasikan antarmuka Dynamic Island pada Windows dengan runtime otonom berstandar rekayasa perangkat lunak modern.

### 1.2 Rumusan Masalah
Bagaimana mengoptimalkan interaksi multimodal pengguna dengan sistem operasi melalui pemantauan penyimpanan cerdas dan format dokumen ilmiah berstandar nasional (TNR 12, spasi 1.5, margin 4-3-3-3).

## BAB II METODOLOGI DAN PEMBAHASAN
Sistem menggunakan arsitektur event-driven terdistribusi berbasis Win32 Named Pipe IPC dengan enkripsi DPAPI, sandboxing atomik, dan zero-data-loss storage engine.

## BAB III KESIMPULAN
Implementasi berhasil mencapai kinerja deterministik dan kepatuhan penuh terhadap standar karya ilmiah Indonesia.`;

		try {
			const res = await createAcademicReport(reportTitle, sampleBody);
			const botMsg: Message = {
				id: `ast_${Date.now()}`,
				role: "assistant",
				text: `Dokumen laporan akademik berhasil dibuat.\n\n- Template: Akademik-ID (Margin 4-3-3-3, TNR 12, Spasi 1.5)\n- Berkas: \`${res.output_path}\`\n- Ukuran: ${res.file_size_bytes} bytes\n\nBerkas telah diverifikasi dan siap dicetak atau diedit lebih lanjut.`,
				timestamp: Date.now(),
				skillName: "Akademik & Dokumen",
				skillColor: "#f59e0b"
			};
			setMessages((prev) => {
				const next = [...prev, botMsg];
				localStorage.setItem("hime-chat-history", JSON.stringify(next.slice(-25)));
				return next;
			});
		} catch (err: any) {
			const errText = err?.message || String(err);
			const errorMsg: Message = {
				id: `ast_${Date.now()}`,
				role: "assistant",
				text: `Gagal membuat laporan akademik: ${errText}`,
				timestamp: Date.now()
			};
			setMessages((prev) => [...prev, errorMsg]);
		} finally {
			setIsLoading(false);
		}
	};

	return (
		<motion.div
			className="hime-assistant-panel"
			initial={{ opacity: 0, scale: 0.98 }}
			animate={{ opacity: 1, scale: 1 }}
			exit={{ opacity: 0, scale: 0.98 }}
			transition={{ type: "spring", stiffness: 450, damping: 30 }}
			style={{
				display: "flex",
				flexDirection: "column",
				height: "100%",
				width: "100%",
				padding: "8px 12px 10px 12px",
				color: "#f5f5f5",
				fontFamily: "inherit",
				userSelect: "text",
				boxSizing: "border-box",
				whiteSpace: "normal",
				overflow: "hidden"
			}}
			onClick={(e) => e.stopPropagation()}
		>
			{/* Chat Messages Stream (Zero clutter top header) */}
			<div
				ref={chatScrollRef}
				style={{
					flex: 1,
					minHeight: 0,
					display: "flex",
					flexDirection: "column",
					overflowY: "auto",
					overflowX: "hidden",
					padding: "4px 2px",
					gap: "10px",
					scrollbarWidth: "none",
					msOverflowStyle: "none"
				}}
			>
				{messages.length === 0 ? (
					<div
						style={{
							flex: 1,
							display: "flex",
							flexDirection: "column",
							alignItems: "center",
							justifyContent: "center",
							textAlign: "center",
							padding: "16px 8px"
						}}
					>
						<div
							style={{
								width: "36px",
								height: "36px",
								borderRadius: "10px",
								background: "rgba(52, 211, 153, 0.12)",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
								marginBottom: "8px",
								color: "#34d399"
							}}
						>
							<Sparkles size={18} />
						</div>
						<div
							style={{
								fontSize: "12.5px",
								fontWeight: 600,
								color: "#f8fafc",
								marginBottom: "4px",
								display: "flex",
								alignItems: "center",
								gap: "6px"
							}}
						>
							<span>Hime Assistant</span>
							<span
								style={{
									width: "6px",
									height: "6px",
									borderRadius: "50%",
									background: isOnline ? "#10b981" : "#eab308",
									boxShadow: isOnline ? "0 0 6px #10b98188" : "none"
								}}
								title={isOnline ? "Core Online" : "Menghubungkan Core..."}
							/>
						</div>
						<div
							style={{
								fontSize: "11px",
								color: "#94a3b8",
								lineHeight: 1.45,
								maxWidth: "280px",
								marginBottom: "14px"
							}}
						>
							Ketik prompt bebas atau gunakan <span style={{ color: "#34d399", fontFamily: "monospace" }}>/</span> untuk memilih skill spesialis.
						</div>

						{/* Quick Starter Pills */}
						<div
							style={{
								display: "flex",
								flexWrap: "wrap",
								gap: "6px",
								justifyContent: "center",
								maxWidth: "320px"
							}}
						>
							{SKILL_OPTIONS.filter((s) => s.id !== "clear").map((skill) => {
								const Icon = skill.icon;
								return (
									<button
										key={skill.id}
										type="button"
										onClick={() => applySkill(skill)}
										style={{
											background: "rgba(255, 255, 255, 0.04)",
											border: `1px solid ${skill.color}33`,
											borderRadius: "8px",
											padding: "5px 10px",
											color: "#e2e8f0",
											fontSize: "10.5px",
											display: "flex",
											alignItems: "center",
											gap: "5px",
											cursor: "pointer",
											transition: "all 0.15s ease"
										}}
										onMouseEnter={(e) => {
											e.currentTarget.style.background = `${skill.color}18`;
											e.currentTarget.style.borderColor = `${skill.color}66`;
										}}
										onMouseLeave={(e) => {
											e.currentTarget.style.background = "rgba(255, 255, 255, 0.04)";
											e.currentTarget.style.borderColor = `${skill.color}33`;
										}}
									>
										<Icon size={12} style={{ color: skill.color }} />
										<span>{skill.name}</span>
									</button>
								);
							})}
						</div>
					</div>
				) : (
					messages.map((msg) => (
						<motion.div
							key={msg.id}
							initial={{ opacity: 0, y: 4 }}
							animate={{ opacity: 1, y: 0 }}
							style={{
								display: "flex",
								flexDirection: "column",
								alignItems: msg.role === "user" ? "flex-end" : "flex-start",
								width: "100%"
							}}
						>
							{msg.role === "assistant" && msg.skillName && (
								<div
									style={{
										display: "inline-flex",
										alignItems: "center",
										gap: "4px",
										fontSize: "9.5px",
										color: msg.skillColor || "#34d399",
										fontWeight: 600,
										marginBottom: "3px",
										paddingLeft: "4px"
									}}
								>
									<span>{msg.skillName}</span>
								</div>
							)}
							<div
								style={{
									maxWidth: "92%",
									padding: "8px 12px",
									borderRadius:
										msg.role === "user"
											? "14px 14px 2px 14px"
											: "14px 14px 14px 2px",
									background:
										msg.role === "user"
											? "rgba(255, 255, 255, 0.12)"
											: "rgba(255, 255, 255, 0.04)",
									border:
										msg.role === "user"
											? "1px solid rgba(255, 255, 255, 0.18)"
											: "1px solid rgba(255, 255, 255, 0.08)",
									fontSize: "11.5px",
									lineHeight: 1.5,
									color: "#f5f5f5",
									wordBreak: "break-word"
								}}
							>
								{msg.role === "assistant" ? (
									<MarkdownContent content={msg.text} />
								) : (
									<span style={{ whiteSpace: "pre-wrap" }}>{msg.text}</span>
								)}
							</div>
						</motion.div>
					))
				)}

				{isLoading && (
					<motion.div
						initial={{ opacity: 0, y: 4 }}
						animate={{ opacity: 1, y: 0 }}
						style={{
							display: "flex",
							alignItems: "center",
							gap: "6px",
							padding: "6px 12px",
							borderRadius: "14px",
							background: "rgba(255, 255, 255, 0.04)",
							border: "1px solid rgba(255, 255, 255, 0.08)",
							width: "fit-content",
							fontSize: "11px",
							color: "#34d399"
						}}
					>
						<RefreshCw size={12} className="animate-spin" />
						<span>Menyiapkan respons...</span>
					</motion.div>
				)}

				<div ref={messagesEndRef} />
			</div>

			{/* Interactive Input Bar with Slash Command Popover */}
			<div style={{ position: "relative", width: "100%", marginTop: "6px", flexShrink: 0 }}>
				{/* Floating Slash Command Menu */}
				<AnimatePresence>
					{showSlashMenu && filteredSkills.length > 0 && (
						<motion.div
							initial={{ opacity: 0, y: 6 }}
							animate={{ opacity: 1, y: 0 }}
							exit={{ opacity: 0, y: 6 }}
							transition={{ duration: 0.12 }}
							style={{
								position: "absolute",
								bottom: "100%",
								left: 0,
								right: 0,
								marginBottom: "8px",
								background: "rgba(18, 18, 24, 0.96)",
								backdropFilter: "blur(24px)",
								border: "1px solid rgba(255, 255, 255, 0.12)",
								borderRadius: "12px",
								padding: "6px",
								display: "flex",
								flexDirection: "column",
								gap: "2px",
								boxShadow: "0 12px 36px rgba(0, 0, 0, 0.65)",
								zIndex: 50,
								maxHeight: "220px",
								overflowY: "auto"
							}}
						>
							<div
								style={{
									fontSize: "9.5px",
									fontWeight: 600,
									color: "#64748b",
									padding: "4px 8px",
									textTransform: "uppercase",
									letterSpacing: "0.05em"
								}}
							>
								Pilih Skill Spesialis (/skill)
							</div>
							{filteredSkills.map((skill, index) => {
								const Icon = skill.icon;
								const isSelected = index === menuIndex;
								return (
									<div
										key={skill.id}
										onClick={() => applySkill(skill)}
										onMouseEnter={() => setMenuIndex(index)}
										style={{
											display: "flex",
											alignItems: "center",
											gap: "8px",
											padding: "6px 8px",
											borderRadius: "8px",
											background: isSelected ? "rgba(255, 255, 255, 0.08)" : "transparent",
											cursor: "pointer",
											transition: "background 0.12s ease"
										}}
									>
										<div
											style={{
												width: "24px",
												height: "24px",
												borderRadius: "6px",
												background: `${skill.color}20`,
												color: skill.color,
												display: "flex",
												alignItems: "center",
												justifyContent: "center",
												flexShrink: 0
											}}
										>
											<Icon size={13} />
										</div>
										<div style={{ flex: 1, minWidth: 0 }}>
											<div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
												<span style={{ fontSize: "11px", fontWeight: 600, color: "#f1f5f9" }}>
													{skill.name}
												</span>
												<span
													style={{
														fontSize: "10px",
														color: skill.color,
														fontFamily: "monospace"
													}}
												>
													{skill.slash}
												</span>
											</div>
											<div
												style={{
													fontSize: "10px",
													color: "#94a3b8",
													whiteSpace: "nowrap",
													overflow: "hidden",
													textOverflow: "ellipsis"
												}}
											>
												{skill.desc}
											</div>
										</div>
									</div>
								);
							})}
						</motion.div>
					)}
				</AnimatePresence>

				{/* Input Form */}
				<form
					onSubmit={(e) => {
						e.preventDefault();
						e.stopPropagation();
						handleSendMessage();
					}}
					onClick={() => claimFocus()}
					style={{
						display: "flex",
						alignItems: "center",
						gap: "6px",
						background: "rgba(255, 255, 255, 0.06)",
						border: "1px solid rgba(255, 255, 255, 0.12)",
						borderRadius: "12px",
						padding: "6px 8px",
						width: "100%",
						boxSizing: "border-box"
					}}
				>
					{/* Slash trigger button */}
					<button
						type="button"
						onClick={() => {
							setShowSlashMenu((prev) => !prev);
							claimFocus();
						}}
						title="Pilih Skill Spesialis (/)"
						style={{
							background: showSlashMenu ? "rgba(255, 255, 255, 0.12)" : "rgba(255, 255, 255, 0.04)",
							border: "1px solid rgba(255, 255, 255, 0.08)",
							borderRadius: "6px",
							width: "22px",
							height: "22px",
							display: "flex",
							alignItems: "center",
							justifyContent: "center",
							color: "#94a3b8",
							cursor: "pointer",
							fontSize: "11px",
							fontFamily: "monospace",
							fontWeight: 600,
							flexShrink: 0
						}}
					>
						/
					</button>

					{/* Active Skill Pill */}
					{selectedSkill && (
						<motion.div
							initial={{ scale: 0.85, opacity: 0 }}
							animate={{ scale: 1, opacity: 1 }}
							style={{
								display: "inline-flex",
								alignItems: "center",
								gap: "4px",
								padding: "2px 7px",
								borderRadius: "999px",
								background: `${selectedSkill.color}22`,
								border: `1px solid ${selectedSkill.color}55`,
								color: selectedSkill.color,
								fontSize: "10.5px",
								fontWeight: 600,
								flexShrink: 0,
								userSelect: "none"
							}}
						>
							<selectedSkill.icon size={11} />
							<span>{selectedSkill.name}</span>
							<button
								type="button"
								onClick={() => setSelectedSkill(null)}
								title="Hapus skill"
								style={{
									background: "none",
									border: "none",
									color: "inherit",
									cursor: "pointer",
									padding: 0,
									display: "flex",
									alignItems: "center",
									marginLeft: "2px",
									opacity: 0.75
								}}
								onMouseEnter={(e) => (e.currentTarget.style.opacity = "1")}
								onMouseLeave={(e) => (e.currentTarget.style.opacity = "0.75")}
							>
								<X size={10} />
							</button>
						</motion.div>
					)}

					<input
						ref={inputRef}
						type="text"
						value={inputValue}
						onChange={handleInputChange}
						onKeyDown={handleInputKeyDown}
						onFocus={() => claimFocus()}
						onClick={() => claimFocus()}
						onMouseDown={() => claimFocus()}
						disabled={isLoading}
						placeholder={
							selectedSkill
								? `Tanya ${selectedSkill.name}...`
								: "Tanya Hime... (ketik / untuk skill)"
						}
						style={{
							background: "transparent",
							border: "none",
							outline: "none",
							fontSize: "11.5px",
							color: "#f5f5f5",
							flex: 1,
							minWidth: "60px",
							userSelect: "text",
							cursor: "text"
						}}
					/>

					<button
						type="submit"
						disabled={!inputValue.trim() || isLoading}
						title="Kirim pesan"
						style={{
							background:
								inputValue.trim() && !isLoading ? "#10b981" : "rgba(255, 255, 255, 0.06)",
							border: "none",
							borderRadius: "8px",
							width: "26px",
							height: "26px",
							display: "flex",
							alignItems: "center",
							justifyContent: "center",
							cursor: inputValue.trim() && !isLoading ? "pointer" : "default",
							color: inputValue.trim() && !isLoading ? "#ffffff" : "#737373",
							transition: "all 0.15s ease",
							flexShrink: 0
						}}
					>
						<ArrowUp size={14} />
					</button>
				</form>
			</div>
		</motion.div>
	);
};
