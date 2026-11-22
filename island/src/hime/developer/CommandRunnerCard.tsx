import React, { useState } from "react";
import { Terminal, Play } from "lucide-react";
import { ALLOWED_DEV_BINARIES, CommandRunResult } from "./types";

export const CommandRunnerCard: React.FC = () => {
	const [selectedBinary, setSelectedBinary] = useState<string>("cargo");
	const [argsText, setArgsText] = useState<string>("check -p hime-core");
	const [running, setRunning] = useState<boolean>(false);
	const [lastResult, setLastResult] = useState<CommandRunResult | null>(null);

	const handleExecute = () => {
		setRunning(true);
		setTimeout(() => {
			setLastResult({
				command: selectedBinary,
				args: argsText.split(" ").filter(Boolean),
				exit_code: 0,
				stdout: "Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.42s",
				stderr: "",
				duration_ms: 420,
				is_success: true
			});
			setRunning(false);
		}, 600);
	};

	return (
		<div
			style={{
				background: "#161922",
				border: "1px solid #2B3142",
				borderRadius: "14px",
				padding: "14px",
				color: "#F3F4F6"
			}}
		>
			<div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "8px" }}>
				<Terminal size={16} color="#38BDF8" />
				<div>
					<div style={{ fontSize: "13px", fontWeight: 700, color: "#38BDF8" }}>
						{"Jalankan CLI".slice(0, 16)}
					</div>
					<div style={{ fontSize: "11px", color: "#9CA3AF" }}>
						{"Eksekusi binary terverifikasi".slice(0, 36)}
					</div>
				</div>
			</div>

			<div style={{ display: "flex", gap: "6px", marginTop: "10px" }}>
				<select
					value={selectedBinary}
					onChange={(e) => setSelectedBinary(e.target.value)}
					style={{
						background: "#0D0F14",
						border: "1px solid #2B3142",
						borderRadius: "6px",
						padding: "5px 8px",
						color: "#38BDF8",
						fontSize: "11px",
						fontFamily: "monospace",
						outline: "none"
					}}
				>
					{ALLOWED_DEV_BINARIES.map((bin) => (
						<option key={bin} value={bin}>
							{bin}
						</option>
					))}
				</select>

				<input
					type="text"
					value={argsText}
					onChange={(e) => setArgsText(e.target.value)}
					placeholder="Argumen..."
					style={{
						flex: 1,
						background: "#0D0F14",
						border: "1px solid #2B3142",
						borderRadius: "6px",
						padding: "5px 8px",
						color: "#F3F4F6",
						fontSize: "11px",
						fontFamily: "monospace",
						outline: "none"
					}}
				/>

				<button
					type="button"
					disabled={running}
					onClick={handleExecute}
					style={{
						background: "#38BDF8",
						color: "#0F172A",
						border: "none",
						borderRadius: "6px",
						padding: "5px 12px",
						fontSize: "11px",
						fontWeight: 700,
						cursor: running ? "not-allowed" : "pointer",
						display: "flex",
						alignItems: "center",
						gap: "4px"
					}}
				>
					<Play size={11} />
					<span>{"Jalankan".slice(0, 10)}</span>
				</button>
			</div>

			{lastResult && (
				<div
					style={{
						marginTop: "10px",
						background: "#0D0F14",
						borderRadius: "6px",
						padding: "8px",
						fontSize: "11px",
						fontFamily: "monospace",
						color: lastResult.is_success ? "#34D399" : "#F87171"
					}}
				>
					<div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
						<span>Exit: {lastResult.exit_code}</span>
						<span>{lastResult.duration_ms} ms</span>
					</div>
					<div style={{ color: "#E2E8F0" }}>{lastResult.stdout}</div>
				</div>
			)}
		</div>
	);
};
