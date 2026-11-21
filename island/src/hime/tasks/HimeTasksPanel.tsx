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
					<span style={{ fontSize: 14, fontWeight: 600 }}>Tugas</span>
					{todayCount > 0 && (
						<span style={{ fontSize: 12, color: "rgba(255,255,255,0.5)" }}>
							· {todayCount} hari ini
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
								background: "none",
								border: "none",
								borderRadius: 6,
								width: 24,
								height: 24,
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
								cursor: "pointer",
								color: "#A7AEBB"
							}}
						>
							<MessageSquare size={13} />
						</button>
					)}

					<svg width="32" height="32" viewBox="0 0 32 32" style={{ flexShrink: 0 }}>
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
								background: "none",
								border: "none",
								borderRadius: 6,
								width: 24,
								height: 24,
								display: "flex",
								alignItems: "center",
								justifyContent: "center",
								cursor: "pointer",
								color: "#A7AEBB"
							}}
						>
							<X size={13} />
						</button>
					)}
				</div>
			</div>

			{/* Segmented tabs */}
			<div
				style={{
					display: "flex",
					gap: 2,
					margin: "0 12px 6px",
					background: "rgba(255,255,255,0.06)",
					borderRadius: 8,
					padding: 2,
				}}
			>
				{TABS.map((tab) => (
					<button
						key={tab.key}
						onClick={() => setActiveTab(tab.key)}
						style={{
							flex: 1,
							padding: "5px 0",
							fontSize: 12,
							fontWeight: activeTab === tab.key ? 600 : 400,
							color: activeTab === tab.key ? "#fff" : "rgba(255,255,255,0.4)",
							background: activeTab === tab.key
								? "rgba(255,255,255,0.12)"
								: "transparent",
							border: "none",
							borderRadius: 6,
							cursor: "pointer",
							transition: "all 180ms ease",
						}}
					>
						{tab.label}
					</button>
				))}
			</div>

			{/* Task list */}
			<div
				style={{
					flex: 1,
					overflowY: "auto",
					minHeight: 0,
				}}
			>
				{loading && (
					<div style={{ padding: 20, textAlign: "center", color: "rgba(255,255,255,0.3)", fontSize: 13 }}>
						Memuat…
					</div>
				)}

				{error && (
					<div style={{ padding: 12, color: "#f59e0b", fontSize: 12 }}>
						{error.slice(0, 44)}
					</div>
				)}

				{!loading && !error && tasks.length === 0 && (
					<div style={{ padding: 24, textAlign: "center" }}>
						<div style={{ fontSize: 24, marginBottom: 8 }}>📋</div>
						<div style={{ fontSize: 13, color: "rgba(255,255,255,0.4)" }}>
							Belum ada tugas.
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
