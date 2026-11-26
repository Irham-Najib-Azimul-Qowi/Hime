import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { TranslateCardState, TranslationResult } from "./types";

export function useTranslation() {
	const [state, setState] = useState<TranslateCardState>("idle");
	const [result, setResult] = useState<TranslationResult | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [optedIn, setOptedIn] = useState<boolean | null>(null);

	const translateText = useCallback(async (text: string, targetLang = "id") => {
		setState("loading");
		setError(null);
		try {
			const res = await invoke<TranslationResult>("hime_translate_text", {
				text,
				targetLang,
			});
			setResult(res);
			setState("result");
			return res;
		} catch (e) {
			setError(String(e));
			setState("error");
			return null;
		}
	}, []);

	const translateSelection = useCallback(async (targetLang = "id") => {
		setState("loading");
		setError(null);
		try {
			const res = await invoke<TranslationResult>("hime_translate_selection", {
				targetLang,
			});
			setResult(res);
			setState("result");
			return res;
		} catch (e) {
			setError(String(e));
			setState("error");
			return null;
		}
	}, []);

	const fetchOptIn = useCallback(async () => {
		try {
			const res = await invoke<{ opted_in: boolean }>("hime_get_translate_opt_in");
			setOptedIn(res.opted_in);
			return res.opted_in;
		} catch {
			return null;
		}
	}, []);

	const setOptIn = useCallback(async (enabled: boolean) => {
		try {
			await invoke("hime_set_translate_opt_in", { enabled });
			setOptedIn(enabled);
		} catch (e) {
			setError(String(e));
		}
	}, []);

	const reset = useCallback(() => {
		setState("idle");
		setResult(null);
		setError(null);
	}, []);

	return {
		state,
		result,
		error,
		optedIn,
		translateText,
		translateSelection,
		fetchOptIn,
		setOptIn,
		reset,
	};
}
