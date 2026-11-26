/** Translation result from Core. */
export interface TranslationResult {
	original: string;
	translated: string;
	target_lang: string;
	cached: boolean;
}

export type TranslateCardState = "idle" | "loading" | "result" | "error";
