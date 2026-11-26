import React from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";

window.addEventListener("error", (e) => {
	invoke("log_frontend_event", {
		level: "error",
		message: `${e.message} at ${e.filename}:${e.lineno}:${e.colno}`
	}).catch(() => {});
});

window.addEventListener("unhandledrejection", (e) => {
	invoke("log_frontend_event", {
		level: "unhandledrejection",
		message: String(e.reason)
	}).catch(() => {});
});

invoke("log_frontend_event", {
	level: "info",
	message: "main.tsx bootstrapping"
}).catch(() => {});

const rootEl = document.getElementById("root");
if (rootEl) {
	ReactDOM.createRoot(rootEl).render(
		<React.StrictMode>
			<App />
		</React.StrictMode>
	);
	const bootNotch = document.getElementById("boot-notch");
	if (bootNotch) bootNotch.remove();
}
