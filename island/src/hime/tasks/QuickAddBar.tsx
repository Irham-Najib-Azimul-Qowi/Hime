import { useCallback, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
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
	const [submitting, setSubmitting] = useState(false);
	const inputRef = useRef<HTMLInputElement>(null);
	const debounceRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

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

	return (
		<div style={{ padding: "8px 12px", borderTop: "1px solid rgba(255,255,255,0.08)" }}>
			{/* Live pills */}
			{parsed && input.trim().length > 2 && (
				<div style={{ display: "flex", gap: 4, marginBottom: 4, flexWrap: "wrap" }}>
					{parsed.priority !== "medium" && (
						<span style={{
							fontSize: 10, padding: "1px 6px", borderRadius: 8,
							background: parsed.priority === "critical" ? "#ef444433" : parsed.priority === "high" ? "#ec489933" : "#6b728033",
							color: parsed.priority === "critical" ? "#ef4444" : parsed.priority === "high" ? "#ec4899" : "#6b7280",
						}}>
							{parsed.priority}
						</span>
					)}
					{parsed.due_date && (
						<span style={{
							fontSize: 10, padding: "1px 6px", borderRadius: 8,
							background: "#0ea5e933", color: "#0ea5e9",
						}}>
							{new Date(parsed.due_date * 1000).toLocaleDateString("id-ID", { weekday: "short", day: "numeric", month: "short" })}
						</span>
					)}
				</div>
			)}

			<div style={{ display: "flex", gap: 6, alignItems: "center" }}>
				<input
					ref={inputRef}
					type="text"
					value={input}
					onChange={(e) => handleInputChange(e.target.value)}
					onKeyDown={handleKeyDown}
					placeholder="+ Tambah tugas…"
					style={{
						flex: 1,
						background: "rgba(255,255,255,0.06)",
						border: "1px solid rgba(255,255,255,0.1)",
						borderRadius: 8,
						padding: "6px 10px",
						fontSize: 13,
						color: "#fff",
						outline: "none",
					}}
				/>

				{/* Category select */}
				<select
					value={selectedCategory}
					onChange={(e) => setSelectedCategory(e.target.value)}
					style={{
						background: "rgba(255,255,255,0.06)",
						border: "1px solid rgba(255,255,255,0.1)",
						borderRadius: 6,
						padding: "4px 6px",
						fontSize: 11,
						color: "rgba(255,255,255,0.6)",
						outline: "none",
						cursor: "pointer",
					}}
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
