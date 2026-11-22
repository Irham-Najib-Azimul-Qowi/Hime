import { useEffect, useRef, useState } from "react";
import { useTranslation } from "./useTranslation";

interface TranslationCardProps {
	/** Pre-filled text to translate (e.g. from selection capture). */
	initialText?: string;
	/** Called when the card auto-closes or user dismisses. */
	onClose: () => void;
}

const AUTO_CLOSE_MS = 9000;

export function TranslationCard({ initialText, onClose }: TranslationCardProps) {
	const { state, result, error, translateText } = useTranslation();
	const [hovered, setHovered] = useState(false);
	const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

	// Auto-translate on mount if text provided
	useEffect(() => {
		if (initialText && initialText.trim()) {
			translateText(initialText);
		}
	}, [initialText, translateText]);

	// Auto-close after result, unless hovered
	useEffect(() => {
		if (state === "result" && !hovered) {
			timerRef.current = setTimeout(onClose, AUTO_CLOSE_MS);
		}
		return () => {
			if (timerRef.current) clearTimeout(timerRef.current);
		};
	}, [state, hovered, onClose]);

	const handleCopy = () => {
		if (result?.translated) {
			navigator.clipboard.writeText(result.translated).catch(() => { });
		}
	};

	return (
		<div
			onMouseEnter={() => setHovered(true)}
			onMouseLeave={() => setHovered(false)}
			style={{
				background: "rgba(0,0,0,0.85)",
				borderRadius: 12,
				padding: "10px 14px",
				border: "1px solid rgba(255,255,255,0.1)",
				maxWidth: 340,
				color: "#fff",
				fontSize: 13,
				backdropFilter: "blur(16px)",
			}}
		>
			{state === "loading" && (
				<div style={{ color: "rgba(255,255,255,0.4)" }}>
					Menerjemahkan…
				</div>
			)}

			{state === "error" && (
				<div style={{ color: "#ef4444", fontSize: 12 }}>
					{error?.slice(0, 44) || "Gagal menerjemahkan"}
				</div>
			)}

			{state === "result" && result && (
				<>
					{/* Original (subtle) */}
					<div
						style={{
							fontSize: 11,
							color: "rgba(255,255,255,0.4)",
							marginBottom: 4,
							overflow: "hidden",
							textOverflow: "ellipsis",
							display: "-webkit-box",
							WebkitLineClamp: 2,
							WebkitBoxOrient: "vertical",
						}}
					>
						{result.original}
					</div>

					{/* Translated */}
					<div
						style={{
							fontSize: 14,
							fontWeight: 500,
							marginBottom: 8,
							overflow: "hidden",
							textOverflow: "ellipsis",
							display: "-webkit-box",
							WebkitLineClamp: 3,
							WebkitBoxOrient: "vertical",
						}}
					>
						{result.translated}
					</div>

					{/* Actions */}
					<div style={{ display: "flex", gap: 6 }}>
						<button
							onClick={handleCopy}
							style={{
								fontSize: 11,
								padding: "3px 10px",
								borderRadius: 6,
								border: "1px solid rgba(255,255,255,0.15)",
								background: "rgba(255,255,255,0.08)",
								color: "#fff",
								cursor: "pointer",
							}}
						>
							Salin
						</button>
						<button
							onClick={onClose}
							style={{
								fontSize: 11,
								padding: "3px 10px",
								borderRadius: 6,
								border: "1px solid rgba(255,255,255,0.15)",
								background: "transparent",
								color: "rgba(255,255,255,0.5)",
								cursor: "pointer",
							}}
						>
							Tutup
						</button>
					</div>

					{/* Cache indicator */}
					{result.cached && (
						<div style={{ fontSize: 10, color: "rgba(255,255,255,0.2)", marginTop: 4 }}>
							dari cache
						</div>
					)}
				</>
			)}
		</div>
	);
}
