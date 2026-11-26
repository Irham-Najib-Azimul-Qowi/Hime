import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { TaskItem, TaskTab, UrgencyLevel } from "./types";

const DAY_MS = 86_400_000;

/** Derives urgency from due_date (seconds) relative to now. */
export function getUrgency(dueDate: number | null): UrgencyLevel {
	if (dueDate == null) return "none";
	const nowSec = Math.floor(Date.now() / 1000);
	const diff = dueDate - nowSec;
	if (diff < 0) return "overdue";
	if (diff < 86400) return "today";
	if (diff < 3 * 86400) return "soon";
	return "normal";
}

/** Urgency → chip color token. */
export function urgencyColor(u: UrgencyLevel): string {
	switch (u) {
		case "overdue": return "#ef4444";
		case "today": return "#f59e0b";
		case "soon": return "#0ea5e9";
		default: return "#6b7280";
	}
}

/** Format due_date (unix sec) as short Indonesian string. */
export function formatDue(dueDate: number | null): string {
	if (dueDate == null) return "";
	const d = new Date(dueDate * 1000);
	const now = new Date();
	const diffMs = d.getTime() - now.getTime();
	const diffDays = Math.round(diffMs / DAY_MS);

	if (diffDays < 0) return `${Math.abs(diffDays)}h lalu`;
	if (diffDays === 0) return "Hari ini";
	if (diffDays === 1) return "Besok";
	if (diffDays === 2) return "Lusa";
	if (diffDays <= 7) {
		const days = ["Min", "Sen", "Sel", "Rab", "Kam", "Jum", "Sab"];
		const hh = d.getHours().toString().padStart(2, "0");
		const mm = d.getMinutes().toString().padStart(2, "0");
		return `${days[d.getDay()]} ${hh}:${mm}`;
	}
	const dd = d.getDate();
	const months = ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"];
	return `${dd} ${months[d.getMonth()]}`;
}

/** Filter tasks by tab. */
function filterByTab(tasks: TaskItem[], tab: TaskTab): TaskItem[] {
	const nowSec = Math.floor(Date.now() / 1000);
	const endOfToday = nowSec + (86400 - (nowSec % 86400));

	switch (tab) {
		case "today":
			return tasks.filter(
				(t) => t.status !== "done" && (t.due_date == null || t.due_date <= endOfToday)
			);
		case "upcoming":
			return tasks.filter(
				(t) => t.status !== "done" && t.due_date != null && t.due_date > endOfToday
			);
		case "done":
			return tasks.filter((t) => t.status === "done");
	}
}

export function useTasks() {
	const [tasks, setTasks] = useState<TaskItem[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [activeTab, setActiveTab] = useState<TaskTab>("today");
	const mountedRef = useRef(true);
	const retryTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

	const fetchTasks = useCallback(async () => {
		try {
			const result = await invoke<TaskItem[]>("hime_list_tasks", {});
			if (mountedRef.current) {
				setTasks(Array.isArray(result) ? result : []);
				setError(null);
			}
		} catch (e) {
			if (mountedRef.current) {
				const raw = String(e);
				// If Named Pipe or Core is initializing/connecting, show friendly status instead of raw pipe error
				if (
					raw.toLowerCase().includes("named pipe") ||
					raw.toLowerCase().includes("cannot find the file specified") ||
					raw.toLowerCase().includes("connection failed") ||
					raw.toLowerCase().includes("the system can") ||
					raw.toLowerCase().includes("broken pipe") ||
					raw.toLowerCase().includes("os error 232") ||
					raw.toLowerCase().includes("hime core")
				) {
					setError("Menghubungkan ke layanan Hime...");
					// Auto retry after 1.5s
					if (!retryTimerRef.current) {
						retryTimerRef.current = setTimeout(() => {
							retryTimerRef.current = null;
							if (mountedRef.current) void fetchTasks();
						}, 1500);
					}
				} else {
					setError(raw);
				}
			}
		} finally {
			if (mountedRef.current) setLoading(false);
		}
	}, []);

	useEffect(() => {
		mountedRef.current = true;
		fetchTasks();
		return () => {
			mountedRef.current = false;
			if (retryTimerRef.current) {
				clearTimeout(retryTimerRef.current);
				retryTimerRef.current = null;
			}
		};
	}, [fetchTasks]);

	const createTask = useCallback(async (
		title: string,
		priority?: string,
		dueDate?: number,
		labels?: string,
		description?: string,
	) => {
		const result = await invoke<TaskItem>("hime_create_task", {
			title,
			priority: priority ?? "medium",
			dueDate: dueDate ?? null,
			labels: labels ?? null,
			description: description ?? null,
		});
		await fetchTasks();
		return result;
	}, [fetchTasks]);

	const updateTask = useCallback(async (
		taskId: string,
		updates: Partial<Pick<TaskItem, "title" | "priority" | "status" | "due_date" | "labels" | "description">>,
	) => {
		await invoke("hime_update_task", {
			taskId,
			title: updates.title ?? null,
			priority: updates.priority ?? null,
			status: updates.status ?? null,
			dueDate: updates.due_date ?? null,
			labels: updates.labels ?? null,
			description: updates.description ?? null,
		});
		await fetchTasks();
	}, [fetchTasks]);

	const deleteTask = useCallback(async (taskId: string) => {
		await invoke("hime_delete_task", { taskId });
		await fetchTasks();
	}, [fetchTasks]);

	const toggleComplete = useCallback(async (task: TaskItem) => {
		const newStatus = task.status === "done" ? "todo" : "done";
		const newColumn = newStatus === "done" ? "col-done" : "col-todo";
		await invoke("hime_update_task", {
			taskId: task.id,
			status: newStatus,
			columnId: newColumn,
		});
		await fetchTasks();
	}, [fetchTasks]);

	const filteredTasks = filterByTab(tasks, activeTab);
	const todayCount = tasks.filter(
		(t) => t.status !== "done" && (t.due_date != null && t.due_date <= Math.floor(Date.now() / 1000) + 86400)
	).length;

	const doneCount = tasks.filter((t) => t.status === "done").length;
	const totalActive = tasks.filter((t) => t.status !== "done").length;

	return {
		tasks: filteredTasks,
		allTasks: tasks,
		loading,
		error,
		activeTab,
		setActiveTab,
		createTask,
		updateTask,
		deleteTask,
		toggleComplete,
		refresh: fetchTasks,
		todayCount,
		doneCount,
		totalActive,
	};
}
