import React, { useEffect, useState, useRef } from "react";
import { motion } from "framer-motion";
import { Sparkles, HelpCircle, AlertCircle, Check, X } from "lucide-react";
import { ProactiveItem } from "./types";
import { HimeEyes } from "../ui/HimeEyes";

export interface ProactiveBubbleProps {
	item: ProactiveItem;
	onAction?: (actionId: string, remember?: boolean) => void;
	onDismiss?: () => void;
	onExpandCard?: () => void;
}

export const ProactiveBubble: React.FC<ProactiveBubbleProps> = ({
	item,
	onAction,
	onDismiss,
	onExpandCard
}) => {
	const [rememberOption, setRememberOption] = useState(false);
	const [remainingSecs, setRemainingSecs] = useState<number>(() => item.autoDismissSecs ?? 6);
	const autoDismissTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);

	// Handle auto-dismiss for "colek"
	useEffect(() => {
		if (item.level !== "colek") return;

		const totalSecs = item.autoDismissSecs ?? 6;
		setRemainingSecs(totalSecs);

		autoDismissTimerRef.current = setInterval(() => {
			setRemainingSecs((prev) => {
				if (prev <= 1) {
					if (autoDismissTimerRef.current) clearInterval(autoDismissTimerRef.current);
					onDismiss?.();
					return 0;
				}
				return prev - 1;
			});
		}, 1000);

		return () => {
			if (autoDismissTimerRef.current) clearInterval(autoDismissTimerRef.current);
		};
	}, [item.id, item.level, item.autoDismissSecs, onDismiss]);

	const getLevelAccent = () => {
		switch (item.level) {
			case "penting":
				return {
					bg: "rgba(239, 68, 68, 0.16)",
					border: "rgba(239, 68, 68, 0.4)",
					color: "#f87171",
					icon: AlertCircle
				};
			case "tanya":
				return {
					bg: "rgba(56, 189, 248, 0.16)",
					border: "rgba(56, 189, 248, 0.35)",
					color: "#38bdf8",
					icon: HelpCircle
				};
			case "colek":
				return {
					bg: "rgba(245, 158, 11, 0.16)",
					border: "rgba(245, 158, 11, 0.35)",
					color: "#fbbf24",
					icon: Sparkles
				};
			case "bisik":
			default:
				return {
					bg: "rgba(168, 85, 247, 0.16)",
					border: "rgba(168, 85, 247, 0.35)",
					color: "#c084fc",
					icon: Sparkles
				};
		}
	};

	const accent = getLevelAccent();

	return (
		<motion.div
			key={`proactive-bubble-${item.id}`}
			className="proactive-bubble"
			role="alert"
			aria-label={`${item.title}: ${item.bubbleText}`}
			style={{
				display: "inline-flex",
				alignItems: "center",
				gap: "8px",
				background: accent.bg,
				border: `1px solid ${accent.border}`,
				color: "#ffffff",
				padding: "4px 12px 4px 8px",
				minHeight: "32px",
				borderRadius: "16px",
				backdropFilter: "blur(14px)",
				boxShadow: `0 4px 14px ${accent.bg}`,
				zIndex: 98,
				fontSize: "12px",
				fontWeight: 500,
				cursor: "default",
				position: "relative"
			}}
			initial={{ opacity: 0, scale: 0.85, y: -6 }}
			animate={{ opacity: 1, scale: 1, y: 0 }}
			exit={{ opacity: 0, scale: 0.85, y: -6 }}
			transition={{ type: "spring", stiffness: 450, damping: 28 }}
		>
			{/* Eye mascot peek or indicator icon */}
			<div
				style={{
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					width: "20px",
					height: "20px",
					borderRadius: "50%",
					background: "rgba(0, 0, 0, 0.35)",
					flexShrink: 0
				}}
			>
				<HimeEyes expression={item.expression} size={18} />
			</div>

			{/* Bubble Text (strictly <= 40 chars) */}
			<span
				style={{
					maxWidth: "260px",
					overflow: "hidden",
					textOverflow: "ellipsis",
					whiteSpace: "nowrap",
					color: "#f3f4f6",
					fontSize: "12px",
					lineHeight: "16px"
				}}
				title={item.bubbleText}
				onClick={onExpandCard}
			>
				{item.bubbleText.slice(0, 40)}
			</span>

			{/* For Colek: timer countdown indicator */}
			{item.level === "colek" && (
				<span
					style={{
						fontSize: "10px",
						opacity: 0.7,
						color: accent.color,
						fontFamily: "monospace",
						flexShrink: 0
					}}
				>
					{remainingSecs}s
				</span>
			)}

			{/* For Tanya: <= 3 Action Buttons */}
			{item.level === "tanya" && item.actions && item.actions.length > 0 && (
				<div
					style={{
						display: "flex",
						alignItems: "center",
						gap: "4px",
						marginLeft: "4px"
					}}
				>
					{item.actions.slice(0, 3).map((act, index) => {
						const isDefaultFirst = index === 0;
						return (
							<button
								key={act.id}
								type="button"
								onClick={(e) => {
									e.stopPropagation();
									onAction?.(act.id, rememberOption);
								}}
								style={{
									border: "none",
									borderRadius: "8px",
									padding: "3px 8px",
									fontSize: "11px",
									fontWeight: 600,
									cursor: "pointer",
									background: isDefaultFirst
										? accent.color
										: "rgba(255, 255, 255, 0.12)",
									color: isDefaultFirst ? "#0f172a" : "#f1f5f9",
									transition: "transform 0.1s ease, filter 0.1s ease"
								}}
								onMouseEnter={(e) => (e.currentTarget.style.filter = "brightness(1.15)")}
								onMouseLeave={(e) => (e.currentTarget.style.filter = "none")}
								title={act.label}
							>
								{act.label.slice(0, 10)}
							</button>
						);
					})}

					{/* Remember Rule Option ("Selalu...") */}
					{item.hasRememberOption && (
						<button
							type="button"
							onClick={(e) => {
								e.stopPropagation();
								setRememberOption(!rememberOption);
							}}
							style={{
								display: "flex",
								alignItems: "center",
								gap: "3px",
								background: rememberOption
									? "rgba(16, 185, 129, 0.25)"
									: "rgba(255, 255, 255, 0.08)",
								border: rememberOption
									? "1px solid #10b981"
									: "1px solid rgba(255, 255, 255, 0.15)",
								borderRadius: "6px",
								padding: "3px 6px",
								fontSize: "10px",
								color: rememberOption ? "#34d399" : "#94a3b8",
								cursor: "pointer"
							}}
							title="Ingat pilihan untuk situasi ini (dapat dicabut di Pengaturan)"
						>
							<Check size={9} strokeWidth={rememberOption ? 3 : 1.5} />
							<span>Selalu</span>
						</button>
					)}
				</div>
			)}

			{/* Dismiss / Close button */}
			{onDismiss && (
				<button
					type="button"
					onClick={(e) => {
						e.stopPropagation();
						onDismiss();
					}}
					aria-label="Tutup"
					style={{
						background: "transparent",
						border: "none",
						color: "rgba(255, 255, 255, 0.5)",
						cursor: "pointer",
						padding: "2px",
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
						marginLeft: "2px"
					}}
					onMouseEnter={(e) => (e.currentTarget.style.color = "#ffffff")}
					onMouseLeave={(e) => (e.currentTarget.style.color = "rgba(255, 255, 255, 0.5)")}
				>
					<X size={12} strokeWidth={2.2} />
				</button>
			)}
		</motion.div>
	);
};
