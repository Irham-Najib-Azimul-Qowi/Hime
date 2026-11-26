import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ChevronDown, Check, Plus } from "lucide-react";
import type { ParsedTaskInput } from "./types";
import { CATEGORIES } from "./types";

interface QuickAddBarProps {
	onAdd: (
		title: string,
		priority?: string,
		dueDate?: number,
		labels?: string,
	) => Promise<unknown>;
}

export function QuickAddBar({ onAdd }: QuickAddBarProps) {
	const [input, setInput] = useState("");
	const [parsed, setParsed] = useState<ParsedTaskInput | null>(null);
	const [selectedCategory, setSelectedCategory] = useState("");
	const [isCategoryOpen, setIsCategoryOpen] = useState(false);
	const [submitting, setSubmitting] = useState(false);
	const inputRef = useRef<HTMLInputElement>(null);
	const dropdownRef = useRef<HTMLDivElement>(null);
	const debounceRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

	// Close category menu on click outside or Escape
	useEffect(() => {
		if (!isCategoryOpen) return;
		const handleClickOutside = (e: MouseEvent) => {
			if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
				setIsCategoryOpen(false);
			}
		};
		const handleEscape = (e: KeyboardEvent) => {
			if (e.key === "Escape") setIsCategoryOpen(false);
		};
		document.addEventListener("mousedown", handleClickOutside);
		document.addEventListener("keydown", handleEscape);
		return () => {
			document.removeEventListener("mousedown", handleClickOutside);
			document.removeEventListener("keydown", handleEscape);
		};
	}, [isCategoryOpen]);

	const doParse = useCallback(async (text: string) => {
		if (text.trim().length < 2) {
			setParsed(null);
			return;
		}
		try {
			const result = await invoke<ParsedTaskInput>("hime_parse_task_natural", { text });
			setParsed(result);
		} catch {
			// Ignore parse errors — user can still submit raw
		}
	}, []);

	const handleInputChange = (val: string) => {
		setInput(val);
		if (debounceRef.current) clearTimeout(debounceRef.current);
		debounceRef.current = setTimeout(() => doParse(val), 300);
	};

	const handleSubmit = async () => {
		const text = input.trim();
		if (!text || submitting) return;
		setSubmitting(true);
		try {
			const title = parsed?.title || text;
			const priority = parsed?.priority || "medium";
			const dueDate = parsed?.due_date ?? undefined;
			await onAdd(title, priority, dueDate, selectedCategory || undefined);
			setInput("");
			setParsed(null);
			setSelectedCategory("");
			setIsCategoryOpen(false);
			inputRef.current?.focus();
		} finally {
			setSubmitting(false);
		}
	};

	const handleKeyDown = (e: React.KeyboardEvent) => {
		if (e.key === "Enter" && !e.shiftKey) {
			e.preventDefault();
			handleSubmit();
		}
	};

	const currentCatObj = CATEGORIES.find((c) => c.id === selectedCategory);

	return (
		<div style={{ padding: "8px 12px 10px", borderTop: "1px solid rgba(255,255,255,0.08)", position: "relative" }}>
			{/* Live pills */}
			{parsed && input.trim().length > 2 && (
				<div style={{ display: "flex", gap: 6, marginBottom: 6, flexWrap: "wrap" }}>
					{parsed.priority !== "medium" && (
						<span
							style={{
								fontSize: 10,
								padding: "2px 8px",
								borderRadius: 9999,
								background:
									parsed.priority === "critical"
										? "#ef444433"
										: parsed.priority === "high"
											? "#ec489933"
											: "#6b728033",
								color:
									parsed.priority === "critical"
										? "#ef4444"
										: parsed.priority === "high"
											? "#ec4899"
											: "#6b7280",
								border: `1px solid ${
									parsed.priority === "critical"
										? "#ef444466"
										: parsed.priority === "high"
											? "#ec489966"
											: "#6b728066"
								}`,
							}}
						>
							{parsed.priority}
						</span>
					)}
					{parsed.due_date && (
						<span
							style={{
								fontSize: 10,
								padding: "2px 8px",
								borderRadius: 9999,
								background: "#0ea5e933",
								color: "#0ea5e9",
								border: "1px solid #0ea5e966",
							}}
						>
							{new Date(parsed.due_date * 1000).toLocaleDateString("id-ID", {
								weekday: "short",
								day: "numeric",
								month: "short",
							})}
						</span>
					)}
				</div>
			)}

			<div style={{ display: "flex", gap: 6, alignItems: "center" }}>
				{/* Input field with rounded pill geometry */}
				<input
					ref={inputRef}
					type="text"
					value={input}
					onChange={(e) => handleInputChange(e.target.value)}
					onKeyDown={handleKeyDown}
					placeholder="+ Tambah tugas…"
					style={{
						flex: 1,
						minWidth: 0,
						background: "rgba(255,255,255,0.06)",
						border: "1px solid rgba(255,255,255,0.12)",
						borderRadius: 9999,
						padding: "6px 14px",
						fontSize: 12,
						height: 32,
						color: "#fff",
						outline: "none",
						boxSizing: "border-box",
						transition: "all 0.15s ease",
					}}
				/>

				{/* Custom Category dropdown */}
				<div ref={dropdownRef} style={{ position: "relative" }}>
					<button
						type="button"
						onClick={() => setIsCategoryOpen((prev) => !prev)}
						title="Pilih Kategori"
						style={{
							background: currentCatObj ? "rgba(255,255,255,0.1)" : "rgba(255,255,255,0.06)",
							border: currentCatObj
								? `1px solid ${currentCatObj.color}88`
								: "1px solid rgba(255,255,255,0.12)",
							borderRadius: 9999,
							padding: "0 12px",
							height: 32,
							fontSize: 12,
							color: currentCatObj ? "#fff" : "rgba(255,255,255,0.7)",
							outline: "none",
							cursor: "pointer",
							display: "inline-flex",
							alignItems: "center",
							gap: 6,
							whiteSpace: "nowrap",
							transition: "all 0.15s ease",
							boxSizing: "border-box",
						}}
					>
						{currentCatObj ? (
							<>
								<span
									style={{
										width: 7,
										height: 7,
										borderRadius: "50%",
										background: currentCatObj.color,
										boxShadow: `0 0 6px ${currentCatObj.color}`,
										flexShrink: 0,
									}}
								/>
								<span>{currentCatObj.name}</span>
							</>
						) : (
							<span>Kategori</span>
						)}
						<ChevronDown size={12} style={{ opacity: 0.6 }} />
					</button>

					{/* Dark Menu Options Popup */}
					{isCategoryOpen && (
						<div
							style={{
								position: "absolute",
								bottom: "calc(100% + 6px)",
								right: 0,
								zIndex: 60,
								background: "#121214",
								backdropFilter: "blur(16px)",
								border: "1px solid rgba(255, 255, 255, 0.14)",
								borderRadius: 14,
								boxShadow: "0 12px 32px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(255, 255, 255, 0.06)",
								padding: "4px",
								minWidth: 150,
								display: "flex",
								flexDirection: "column",
								gap: 2,
							}}
						>
							<div
								style={{
									fontSize: 10,
									fontWeight: 600,
									color: "rgba(255, 255, 255, 0.4)",
									padding: "4px 8px 2px",
									textTransform: "uppercase",
									letterSpacing: 0.5,
								}}
							>
								Kategori
							</div>

							{/* Clear / All Categories */}
							<button
								type="button"
								onClick={() => {
									setSelectedCategory("");
									setIsCategoryOpen(false);
								}}
								style={{
									display: "flex",
									alignItems: "center",
									gap: 8,
									width: "100%",
									textAlign: "left",
									background: selectedCategory === "" ? "rgba(255, 255, 255, 0.12)" : "transparent",
									border: "none",
									borderRadius: 9999,
									padding: "6px 10px",
									fontSize: 12,
									color: selectedCategory === "" ? "#fff" : "rgba(255, 255, 255, 0.8)",
									cursor: "pointer",
									transition: "background 150ms ease",
								}}
								onMouseEnter={(e) => {
									if (selectedCategory !== "") e.currentTarget.style.background = "rgba(255, 255, 255, 0.08)";
								}}
								onMouseLeave={(e) => {
									if (selectedCategory !== "") e.currentTarget.style.background = "transparent";
								}}
							>
								<span
									style={{
										width: 7,
										height: 7,
										borderRadius: "50%",
										border: "1px dashed rgba(255, 255, 255, 0.4)",
										flexShrink: 0,
									}}
								/>
								<span style={{ flex: 1 }}>Tanpa Kategori</span>
								{selectedCategory === "" && <Check size={12} style={{ color: "#10b981" }} />}
							</button>

							{/* Category options */}
							{CATEGORIES.map((c) => {
								const isSelected = selectedCategory === c.id;
								return (
									<button
										key={c.id}
										type="button"
										onClick={() => {
											setSelectedCategory(c.id);
											setIsCategoryOpen(false);
										}}
										style={{
											display: "flex",
											alignItems: "center",
											gap: 8,
											width: "100%",
											textAlign: "left",
											background: isSelected ? "rgba(255, 255, 255, 0.12)" : "transparent",
											border: "none",
											borderRadius: 9999,
											padding: "6px 10px",
											fontSize: 12,
											color: isSelected ? "#fff" : "rgba(255, 255, 255, 0.8)",
											cursor: "pointer",
											transition: "background 150ms ease",
										}}
										onMouseEnter={(e) => {
											if (!isSelected) e.currentTarget.style.background = "rgba(255, 255, 255, 0.08)";
										}}
										onMouseLeave={(e) => {
											if (!isSelected) e.currentTarget.style.background = "transparent";
										}}
									>
										<span
											style={{
												width: 7,
												height: 7,
												borderRadius: "50%",
												background: c.color,
												boxShadow: `0 0 6px ${c.color}`,
												flexShrink: 0,
											}}
										/>
										<span style={{ flex: 1 }}>{c.name}</span>
										{isSelected && <Check size={12} style={{ color: "#10b981" }} />}
									</button>
								);
							})}
						</div>
					)}
				</div>

				{/* Unified Pill Add Submit Button */}
				{input.trim().length > 0 && (
					<button
						type="button"
						onClick={handleSubmit}
						disabled={submitting}
						title="Tambah tugas"
						style={{
							background: "linear-gradient(135deg, rgba(255,255,255,0.22), rgba(255,255,255,0.1))",
							border: "1px solid rgba(255,255,255,0.18)",
							borderRadius: 9999,
							padding: "0 12px",
							height: 32,
							fontSize: 12,
							fontWeight: 600,
							color: "#fff",
							cursor: "pointer",
							display: "inline-flex",
							alignItems: "center",
							gap: 4,
							whiteSpace: "nowrap",
							transition: "all 0.15s ease",
							boxSizing: "border-box",
							boxShadow: "0 2px 8px rgba(0,0,0,0.3)",
						}}
					>
						<Plus size={13} strokeWidth={2.5} />
						<span>Tambah</span>
					</button>
				)}

				{/* Accessible / programmatic select fallback */}
				<select
					aria-label="Pilih Kategori"
					value={selectedCategory}
					onChange={(e) => setSelectedCategory(e.target.value)}
					style={{ display: "none" }}
				>
					<option value="">Kategori</option>
					{CATEGORIES.map((c) => (
						<option key={c.id} value={c.id}>{c.name}</option>
					))}
				</select>
			</div>
		</div>
	);
}
