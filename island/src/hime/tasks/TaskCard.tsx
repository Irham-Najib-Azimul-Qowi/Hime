import { useState } from "react";
import type { TaskItem } from "./types";
import { CATEGORIES } from "./types";
import { formatDue, getUrgency, urgencyColor } from "./useTasks";

interface TaskCardProps {
	task: TaskItem;
	onToggle: (task: TaskItem) => void;
	onDelete: (taskId: string) => void;
}

export function TaskCard({ task, onToggle, onDelete }: TaskCardProps) {
	const [deleting, setDeleting] = useState(false);
	const urgency = getUrgency(task.due_date);
	const chipColor = urgencyColor(urgency);
	const dueText = formatDue(task.due_date);
	const isDone = task.status === "done";

	const category = CATEGORIES.find(
		(c) => task.labels?.toLowerCase().includes(c.id)
	);

	const handleDelete = () => {
		setDeleting(true);
		// 8-second undo window (visual only; delete fires immediately for responsiveness)
		onDelete(task.id);
	};

	return (
		<div
			style={{
				display: "flex",
				alignItems: "center",
				gap: 8,
				padding: "6px 12px",
				minHeight: 44,
				borderBottom: "1px solid rgba(255,255,255,0.06)",
				opacity: deleting ? 0.3 : isDone ? 0.5 : 1,
				transition: "opacity 220ms ease",
			}}
		>
			{/* Checkbox */}
			<button
				onClick={() => onToggle(task)}
				style={{
					width: 18,
					height: 18,
					borderRadius: 4,
					border: isDone
						? "2px solid #10b981"
						: "2px solid rgba(255,255,255,0.3)",
					background: isDone ? "#10b981" : "transparent",
					cursor: "pointer",
					flexShrink: 0,
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					transition: "all 220ms ease",
					padding: 0,
				}}
				aria-label={isDone ? "Batal selesai" : "Selesai"}
			>
				{isDone && (
					<svg width="10" height="10" viewBox="0 0 10 10" fill="none">
						<path d="M2 5L4.5 7.5L8 3" stroke="#fff" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
					</svg>
				)}
			</button>

			{/* Priority marker */}
			{task.priority === "high" || task.priority === "critical" ? (
				<div
					style={{
						width: 4,
						height: 20,
						borderRadius: 2,
						background: task.priority === "critical" ? "#ef4444" : "#ec4899",
						flexShrink: 0,
					}}
				/>
			) : null}

			{/* Title */}
			<span
				style={{
					flex: 1,
					fontSize: 13,
					color: isDone ? "rgba(255,255,255,0.4)" : "#fff",
					textDecoration: isDone ? "line-through" : "none",
					overflow: "hidden",
					textOverflow: "ellipsis",
					whiteSpace: "nowrap",
				}}
			>
				{task.title}
			</span>

			{/* Category dot */}
			{category && (
				<div
					title={category.name}
					style={{
						width: 8,
						height: 8,
						borderRadius: "50%",
						background: category.color,
						flexShrink: 0,
					}}
				/>
			)}

			{/* Due chip */}
			{dueText && !isDone && (
				<span
					style={{
						fontSize: 11,
						padding: "1px 6px",
						borderRadius: 6,
						background: `${chipColor}22`,
						color: chipColor,
						whiteSpace: "nowrap",
						flexShrink: 0,
					}}
				>
					{dueText}
				</span>
			)}

			{/* Delete on hover */}
			<button
				onClick={handleDelete}
				style={{
					background: "none",
					border: "none",
					color: "rgba(255,255,255,0.2)",
					cursor: "pointer",
					fontSize: 14,
					padding: "0 2px",
					flexShrink: 0,
					opacity: 0.5,
					transition: "opacity 150ms",
				}}
				onMouseEnter={(e) => { (e.target as HTMLElement).style.opacity = "1"; }}
				onMouseLeave={(e) => { (e.target as HTMLElement).style.opacity = "0.5"; }}
				aria-label="Hapus"
			>
				×
			</button>
		</div>
	);
}
