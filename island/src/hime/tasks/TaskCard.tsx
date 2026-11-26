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
	const [isHovered, setIsHovered] = useState(false);
	const urgency = getUrgency(task.due_date);
	const chipColor = urgencyColor(urgency);
	const dueText = formatDue(task.due_date);
	const isDone = task.status === "done";

	const category = CATEGORIES.find(
		(c) => task.labels?.toLowerCase().includes(c.id)
	);

	const handleDelete = () => {
		setDeleting(true);
		onDelete(task.id);
	};

	const hasPriority = task.priority === "high" || task.priority === "critical";
	const priorityColor = task.priority === "critical" ? "#ef4444" : "#ec4899";

	return (
		<div
			onMouseEnter={() => setIsHovered(true)}
			onMouseLeave={() => setIsHovered(false)}
			style={{
				display: "flex",
				alignItems: "center",
				gap: 10,
				margin: "3px 10px 4px 10px",
				padding: "8px 12px",
				minHeight: 44,
				borderRadius: 12,
				background: isDone
					? "rgba(255, 255, 255, 0.02)"
					: isHovered
						? "rgba(255, 255, 255, 0.07)"
						: "rgba(255, 255, 255, 0.04)",
				border: isDone
					? "1px solid rgba(255, 255, 255, 0.04)"
					: isHovered
						? "1px solid rgba(255, 255, 255, 0.14)"
						: "1px solid rgba(255, 255, 255, 0.07)",
				boxShadow: isHovered && !isDone
					? "0 4px 16px rgba(0, 0, 0, 0.3), 0 0 0 1px rgba(255, 255, 255, 0.05)"
					: "none",
				opacity: deleting ? 0.3 : isDone ? 0.5 : 1,
				transition: "all 180ms cubic-bezier(0.16, 1, 0.3, 1)",
				position: "relative",
				overflow: "hidden",
			}}
		>
			{/* Priority left accent strip */}
			{hasPriority && (
				<div
					style={{
						position: "absolute",
						left: 0,
						top: 6,
						bottom: 6,
						width: 3,
						borderRadius: "0 9999px 9999px 0",
						background: priorityColor,
						boxShadow: `0 0 8px ${priorityColor}88`,
					}}
				/>
			)}

			{/* Pill-shaped Checkbox */}
			<button
				onClick={() => onToggle(task)}
				style={{
					width: 20,
					height: 20,
					borderRadius: 9999,
					border: isDone
						? "2px solid #10b981"
						: isHovered
							? "2px solid rgba(255, 255, 255, 0.5)"
							: "2px solid rgba(255, 255, 255, 0.25)",
					background: isDone
						? "linear-gradient(135deg, #10b981, #059669)"
						: "rgba(255, 255, 255, 0.04)",
					boxShadow: isDone ? "0 0 10px rgba(16, 185, 129, 0.45)" : "none",
					cursor: "pointer",
					flexShrink: 0,
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					transition: "all 180ms ease",
					padding: 0,
				}}
				aria-label={isDone ? "Batal selesai" : "Selesai"}
			>
				{isDone && (
					<svg width="10" height="10" viewBox="0 0 10 10" fill="none">
						<path d="M2 5L4.5 7.5L8 3" stroke="#fff" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
					</svg>
				)}
			</button>

			{/* Title with neat typography */}
			<span
				style={{
					flex: 1,
					minWidth: 0,
					fontSize: 13,
					fontWeight: isDone ? 400 : 500,
					letterSpacing: "-0.01em",
					color: isDone ? "rgba(255,255,255,0.4)" : "#fff",
					textDecoration: isDone ? "line-through" : "none",
					overflow: "hidden",
					textOverflow: "ellipsis",
					whiteSpace: "nowrap",
					transition: "color 150ms ease",
				}}
			>
				{task.title}
			</span>

			{/* Vibrant Category Tag (Rounded-Pill Badge with Luminous Accent) */}
			{category && (
				<span
					title={`Kategori: ${category.name}`}
					style={{
						fontSize: 10,
						fontWeight: 600,
						padding: "2px 8px",
						borderRadius: 9999,
						background: `${category.color}22`,
						color: category.color,
						border: `1px solid ${category.color}55`,
						boxShadow: `0 0 10px ${category.color}22`,
						display: "inline-flex",
						alignItems: "center",
						gap: 4,
						whiteSpace: "nowrap",
						flexShrink: 0,
					}}
				>
					<span
						style={{
							width: 5,
							height: 5,
							borderRadius: "50%",
							background: category.color,
							boxShadow: `0 0 6px ${category.color}`,
						}}
					/>
					{category.name}
				</span>
			)}

			{/* Due chip */}
			{dueText && !isDone && (
				<span
					style={{
						fontSize: 10,
						fontWeight: 600,
						padding: "2px 8px",
						borderRadius: 9999,
						background: `${chipColor}22`,
						color: chipColor,
						whiteSpace: "nowrap",
						flexShrink: 0,
						border: `1px solid ${chipColor}44`,
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
					borderRadius: 9999,
					color: "rgba(255,255,255,0.35)",
					cursor: "pointer",
					fontSize: 14,
					width: 22,
					height: 22,
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					padding: 0,
					flexShrink: 0,
					opacity: isHovered ? 0.8 : 0,
					transition: "all 150ms ease",
				}}
				onMouseEnter={(e) => {
					(e.currentTarget as HTMLElement).style.opacity = "1";
					(e.currentTarget as HTMLElement).style.background = "rgba(255,255,255,0.12)";
					(e.currentTarget as HTMLElement).style.color = "#ff4d4f";
				}}
				onMouseLeave={(e) => {
					(e.currentTarget as HTMLElement).style.opacity = "0.8";
					(e.currentTarget as HTMLElement).style.background = "transparent";
					(e.currentTarget as HTMLElement).style.color = "rgba(255,255,255,0.35)";
				}}
				aria-label="Hapus"
			>
				×
			</button>
		</div>
	);
}
