import React, { useState } from "react";
import { HimeEyes, EyeExpression } from "../ui/HimeEyes";
import eyesData from "../assets/eyes.json";

const EXPRESSIONS_LIST: EyeExpression[] = [
	"netral",
	"senang",
	"semangat",
	"bangga",
	"fokus",
	"berpikir",
	"dengar",
	"penasaran",
	"kaget",
	"khawatir",
	"bosan",
	"ngantuk",
	"menguap",
	"tidur",
	"kesal",
	"marah",
	"sedih",
	"curiga",
	"nakal",
	"kedip-sebelah",
	"pusing",
	"offline"
];

export const ExpressionTester: React.FC = () => {
	const [activeExpr, setActiveExpr] = useState<EyeExpression>("netral");
	const [testAudioLevel, setTestAudioLevel] = useState<number>(0);

	const exprData = (eyesData.expressions as Record<string, any>)[activeExpr] ?? eyesData.expressions.netral;
	const color = exprData.color ?? "cyan";
	const motion = exprData.motion ?? "none";

	return (
		<div
			style={{
				display: "flex",
				flexDirection: "column",
				gap: "12px",
				padding: "12px",
				background: "rgba(255, 255, 255, 0.04)",
				border: "1px solid rgba(255, 255, 255, 0.08)",
				borderRadius: "12px"
			}}
		>
			{/* Top Bar: Preview and Details */}
			<div
				style={{
					display: "flex",
					alignItems: "center",
					justifyContent: "space-between",
					gap: "16px",
					padding: "8px 12px",
					background: "rgba(0, 0, 0, 0.2)",
					borderRadius: "8px"
				}}
			>
				<div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
					<HimeEyes
						size={36}
						expression={activeExpr}
						audioLevel={activeExpr === "dengar" ? testAudioLevel : undefined}
					/>
					<div>
						<div
							style={{
								fontSize: "13px",
								fontWeight: 600,
								color: "var(--text-1, #F4F6FA)"
							}}
						>
							{activeExpr}
						</div>
						<div
							style={{
								fontSize: "11px",
								color: "var(--text-3, #6F7785)",
								lineHeight: "14px"
							}}
						>
							Warna: {color} · Gerak: {motion}
						</div>
					</div>
				</div>

				<div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
					{activeExpr === "dengar" && (
						<button
							type="button"
							onClick={() => setTestAudioLevel((prev) => (prev > 0 ? 0 : 0.85))}
							style={{
								background: testAudioLevel > 0 ? "rgba(52, 227, 166, 0.2)" : "rgba(255, 255, 255, 0.08)",
								border: "1px solid rgba(255, 255, 255, 0.12)",
								borderRadius: "6px",
								padding: "4px 8px",
								color: testAudioLevel > 0 ? "#34E3A6" : "#e5e5e5",
								fontSize: "11px",
								fontWeight: 500,
								cursor: "pointer"
							}}
						>
							{testAudioLevel > 0 ? "Mati Pulsa" : "Tes Pulsa"}
						</button>
					)}
					<button
						type="button"
						onClick={() => {
							setActiveExpr("netral");
							setTestAudioLevel(0);
						}}
						style={{
							background: "rgba(255, 255, 255, 0.08)",
							border: "1px solid rgba(255, 255, 255, 0.12)",
							borderRadius: "6px",
							padding: "4px 8px",
							color: "#e5e5e5",
							fontSize: "11px",
							cursor: "pointer"
						}}
					>
						Reset
					</button>
				</div>
			</div>

			{/* 22 Expression Selector Grid */}
			<div
				style={{
					display: "grid",
					gridTemplateColumns: "repeat(auto-fill, minmax(84px, 1fr))",
					gap: "6px"
				}}
			>
				{EXPRESSIONS_LIST.map((expr) => {
					const isSelected = activeExpr === expr;
					return (
						<button
							key={expr}
							type="button"
							onClick={() => setActiveExpr(expr)}
							style={{
								display: "flex",
								flexDirection: "column",
								alignItems: "center",
								gap: "4px",
								padding: "6px 4px",
								background: isSelected
									? "rgba(56, 214, 245, 0.15)"
									: "rgba(255, 255, 255, 0.03)",
								border: isSelected
									? "1px solid rgba(56, 214, 245, 0.5)"
									: "1px solid rgba(255, 255, 255, 0.06)",
								borderRadius: "6px",
								cursor: "pointer",
								transition: "all 0.15s ease"
							}}
						>
							<HimeEyes size={16} expression={expr} />
							<span
								style={{
									fontSize: "10px",
									color: isSelected ? "#38D6F5" : "var(--text-2, #A7AEBB)",
									fontWeight: isSelected ? 600 : 400,
									whiteSpace: "nowrap",
									overflow: "hidden",
									textOverflow: "ellipsis",
									maxWidth: "76px"
								}}
							>
								{expr}
							</span>
						</button>
					);
				})}
			</div>
		</div>
	);
};
