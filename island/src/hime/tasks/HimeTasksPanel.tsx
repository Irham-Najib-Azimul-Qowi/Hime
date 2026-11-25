import { MessageSquare, X } from "lucide-react";
import type { TaskTab } from "./types";
import { useTasks } from "./useTasks";
import { TaskCard } from "./TaskCard";
import { QuickAddBar } from "./QuickAddBar";

const TABS: { key: TaskTab; label: string }[] = [
	{ key: "today", label: "Hari ini" },
	{ key: "upcoming", label: "Mendatang" },
	{ key: "done", label: "Selesai" },
];

export interface HimeTasksPanelProps {
	onClose?: () => void;
	onOpenChat?: () => void;
}

export function HimeTasksPanel({ onClose, onOpenChat }: HimeTasksPanelProps = {}) {
	const {
		tasks,
		loading,
		error,
		activeTab,
		setActiveTab,
		createTask,
		deleteTask,
		toggleComplete,
		refresh,
		todayCount,
		doneCount,
		totalActive,
	} = useTasks();

	const total = totalActive + doneCount;
	const progressPct = total > 0 ? Math.round((doneCount / total) * 100) : 0;
	const circumference = 2 * Math.PI * 14;
	const strokeDashoffset = circumference - (progressPct / 100) * circumference;

	return (
		<div
			style={{
				display: "flex",
				flexDirection: "column",
				height: "100%",
				minHeight: 200,
				color: "#fff",
			}}
		>
			{/* Header */}
			<div
				style={{
					display: "flex",
					alignItems: "center",
					justifyContent: "space-between",
					padding: "10px 14px 6px",
				}}
			>
				<div style={{ display: "flex", alignItems: "center", gap: 8 }}>
					<span style={{ fontSize: 14, fontWeight: 700, letterSpacing: "-0.01em" }}>Tugas</span>
					{todayCount > 0 ? (
						<span
							style={{
								fontSize: 11,
								fontWeight: 600,
								padding: "1px 7px",
								borderRadius: 9999,
								background: "rgba(56, 214, 245, 0.15)",
								color: "#38D6F5",
								border: "1px solid rgba(56, 214, 245, 0.3)",
								boxShadow: "0 0 8px rgba(56, 214, 245, 0.2)",
							}}
						>
							{todayCount} hari ini
						</span>
					) : (
						<span style={{ fontSize: 12, color: "rgba(255,255,255,0.4)" }}>
							· Produktivitas
						</span>
					)}
				</div>

				{/* Actions & Progress ring */}
				<div style={{ display: "flex", alignItems: "center", gap: 6 }}>
					{onOpenChat && (
						<button
							type="button"
							onClick={onOpenChat}
							title="Chat"
							style={{
								background: "rgba(255,255,255,0.06)",
								border: "1px solid rgba(255,255,255,0.08)",
								borderRadius: 9999,
								width: 26,
								height: 26,
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
								cursor: "pointer",
								color: "#A7AEBB",
								transition: "all 0.15s ease",
							}}
						>
							<MessageSquare size={13} />
						</button>
					)}

					<svg width="32" height="32" viewBox="0 0 32 32" style={{ flexShrink: 0, filter: "drop-shadow(0 0 6px rgba(16, 185, 129, 0.3))" }}>
						<circle
							cx="16" cy="16" r="14"
							fill="none"
							stroke="rgba(255,255,255,0.08)"
							strokeWidth="2.5"
						/>
						<circle
							cx="16" cy="16" r="14"
							fill="none"
							stroke="#10b981"
							strokeWidth="2.5"
							strokeLinecap="round"
							strokeDasharray={circumference}
							strokeDashoffset={strokeDashoffset}
							transform="rotate(-90 16 16)"
							style={{ transition: "stroke-dashoffset 400ms ease" }}
						/>
						<text
							x="16" y="16"
							textAnchor="middle"
							dominantBaseline="central"
							fill="#fff"
							fontSize="9"
							fontWeight="600"
						>
							{doneCount}/{total}
						</text>
					</svg>

					{onClose && (
						<button
							type="button"
							onClick={onClose}
							title="Tutup"
							style={{
								background: "rgba(255,255,255,0.06)",
								border: "1px solid rgba(255,255,255,0.08)",
								borderRadius: 9999,
								width: 26,
								height: 26,
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
								cursor: "pointer",
								color: "#A7AEBB",
								transition: "all 0.15s ease",
							}}
						>
							<X size={13} />
						</button>
					)}
				</div>
			</div>

			{/* Beautiful Segmented tab track */}
			<div
				style={{
					display: "flex",
					gap: 4,
					margin: "2px 12px 8px",
					background: "rgba(255,255,255,0.04)",
					borderRadius: 9999,
					padding: 3,
					border: "1px solid rgba(255,255,255,0.08)",
					boxShadow: "inset 0 1px 3px rgba(0,0,0,0.35)",
				}}
			>
				{TABS.map((tab) => {
					const isActive = activeTab === tab.key;
					return (
						<button
							key={tab.key}
							onClick={() => setActiveTab(tab.key)}
							style={{
								flex: 1,
								padding: "5px 0",
								fontSize: 12,
								fontWeight: isActive ? 600 : 500,
								color: isActive ? "#fff" : "rgba(255,255,255,0.5)",
								background: isActive
									? "linear-gradient(135deg, rgba(255,255,255,0.16), rgba(255,255,255,0.08))"
									: "transparent",
								border: isActive ? "1px solid rgba(255,255,255,0.14)" : "1px solid transparent",
								borderRadius: 9999,
								cursor: "pointer",
								transition: "all 180ms ease",
								boxShadow: isActive ? "0 2px 8px rgba(0,0,0,0.35)" : "none",
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
								gap: 5,
							}}
						>
							<span>{tab.label}</span>
							{tab.key === "today" && todayCount > 0 && (
								<span
									style={{
										fontSize: 9,
										fontWeight: 700,
										padding: "0 5px",
										borderRadius: 9999,
										background: isActive ? "rgba(255,255,255,0.25)" : "rgba(255,255,255,0.1)",
										color: "#fff",
									}}
								>
									{todayCount}
								</span>
							)}
							{tab.key === "done" && doneCount > 0 && (
								<span
									style={{
										fontSize: 9,
										fontWeight: 700,
										padding: "0 5px",
										borderRadius: 9999,
										background: isActive ? "rgba(16, 185, 129, 0.35)" : "rgba(16, 185, 129, 0.15)",
										color: "#10b981",
									}}
								>
									{doneCount}
								</span>
							)}
						</button>
					);
				})}
			</div>

			{/* Task list */}
			<div
				style={{
					flex: 1,
					overflowY: "auto",
					overflowX: "hidden",
					scrollbarWidth: "none",
					msOverflowStyle: "none",
					minHeight: 0,
					padding: "0 2px",
				}}
			>
				{loading && (
					<div style={{ padding: 20, textAlign: "center", color: "rgba(255,255,255,0.3)", fontSize: 13 }}>
						Memuat…
					</div>
				)}

				{error && (
					<div
						style={{
							padding: "8px 12px",
							margin: "4px 10px 8px",
							borderRadius: 10,
							background: "rgba(245, 158, 11, 0.08)",
							border: "1px solid rgba(245, 158, 11, 0.25)",
							color: "#f59e0b",
							fontSize: 12,
							display: "flex",
							alignItems: "center",
							justifyContent: "space-between",
							gap: 8,
						}}
					>
						<span
							style={{
								overflow: "hidden",
								textOverflow: "ellipsis",
								whiteSpace: "nowrap",
								flex: 1,
								minWidth: 0,
								fontWeight: 500,
							}}
						>
							{error}
						</span>
						<button
							type="button"
							onClick={() => refresh()}
							style={{
								padding: "3px 10px",
								borderRadius: 9999,
								fontSize: 11,
								fontWeight: 600,
								background: "rgba(245, 158, 11, 0.2)",
								border: "1px solid rgba(245, 158, 11, 0.4)",
								color: "#f59e0b",
								cursor: "pointer",
								flexShrink: 0,
								transition: "all 0.15s ease",
							}}
						>
							Coba lagi
						</button>
					</div>
				)}

				{!loading && !error && tasks.length === 0 && (
					<div style={{ padding: 28, textAlign: "center" }}>
						<div style={{ fontSize: 28, marginBottom: 8, filter: "drop-shadow(0 0 8px rgba(255,255,255,0.2))" }}>
							{activeTab === "done" ? "✨" : activeTab === "upcoming" ? "📅" : "📋"}
						</div>
						<div style={{ fontSize: 13, color: "rgba(255,255,255,0.5)", fontWeight: 500 }}>
							{activeTab === "done"
								? "Belum ada tugas selesai."
								: activeTab === "upcoming"
									? "Tidak ada tugas mendatang."
									: "Semua tugas hari ini beres!"}
						</div>
					</div>
				)}

				{tasks.map((task) => (
					<TaskCard
						key={task.id}
						task={task}
						onToggle={toggleComplete}
						onDelete={deleteTask}
					/>
				))}
			</div>

			{/* Quick add bar */}
			<QuickAddBar onAdd={createTask} />
		</div>
	);
}
