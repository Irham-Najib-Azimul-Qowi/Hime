/**
 * Synthesizes a soft, pleasant, rising notification chime for "Penting"
 * proactive alerts using Web Audio API. Zero external audio asset dependencies.
 */
let proactiveAudioCtx: AudioContext | null = null;

export function getAudioContext(): AudioContext | null {
	try {
		if (typeof window === "undefined") return null;
		const AudioContextClass = window.AudioContext || (window as any).webkitAudioContext;
		if (!AudioContextClass) return null;
		if (!proactiveAudioCtx) {
			proactiveAudioCtx = new AudioContextClass();
		}
		if (proactiveAudioCtx.state === "suspended") {
			proactiveAudioCtx.resume().catch(() => {});
		}
		return proactiveAudioCtx;
	} catch {
		return null;
	}
}

export function playProactiveChime(): void {
	const ctx = getAudioContext();
	if (!ctx) return;

	try {
		const now = ctx.currentTime + 0.01;
		const master = ctx.createGain();
		master.gain.setValueAtTime(0.3, now);
		master.connect(ctx.destination);

		// Two-tone rising harmonic chime: C5 (523.25 Hz) -> G5 (783.99 Hz)
		const tones = [
			{ freq: 523.25, start: 0, dur: 0.22 },
			{ freq: 783.99, start: 0.12, dur: 0.35 }
		];

		for (const tone of tones) {
			const osc = ctx.createOscillator();
			const gain = ctx.createGain();

			osc.type = "sine";
			osc.frequency.setValueAtTime(tone.freq, now + tone.start);

			gain.gain.setValueAtTime(0.001, now + tone.start);
			gain.gain.exponentialRampToValueAtTime(0.4, now + tone.start + 0.03);
			gain.gain.exponentialRampToValueAtTime(0.001, now + tone.start + tone.dur);

			osc.connect(gain);
			gain.connect(master);

			osc.start(now + tone.start);
			osc.stop(now + tone.start + tone.dur + 0.05);
		}
	} catch {
		// Ignore audio errors gracefully
	}
}
