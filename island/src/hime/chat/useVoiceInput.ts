import { useState, useRef, useCallback, useEffect } from "react";
import { PriorityLevel } from "../ncm/types";

export interface UseVoiceInputOptions {
  onTranscript?: (text: string) => void;
  submitPresentation?: (p: any) => void;
  retractPresentation?: (id: string) => void;
}

/**
 * Voice Push-to-Talk Hook with Web Audio API RMS meter (throttled <= 30Hz)
 * and zero audio persistence.
 */
export function useVoiceInput(options: UseVoiceInputOptions = {}) {
  const { onTranscript, submitPresentation, retractPresentation } = options;

  const [isRecording, setIsRecording] = useState(false);
  const [audioLevel, setAudioLevel] = useState(0);
  const [transcript, setTranscript] = useState("");
  const [error, setError] = useState<string | null>(null);

  const mediaStreamRef = useRef<MediaStream | null>(null);
  const audioContextRef = useRef<AudioContext | null>(null);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const animationFrameRef = useRef<number | null>(null);
  const lastUpdateTimeRef = useRef<number>(0);
  const recognitionRef = useRef<any>(null);

  const stopAudioCapture = useCallback(() => {
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }

    if (mediaStreamRef.current) {
      // Zero audio persistence: immediately stop and destroy all tracks
      mediaStreamRef.current.getTracks().forEach((track) => track.stop());
      mediaStreamRef.current = null;
    }

    if (audioContextRef.current && audioContextRef.current.state !== "closed") {
      audioContextRef.current.close().catch(() => {});
      audioContextRef.current = null;
    }

    if (recognitionRef.current) {
      try {
        recognitionRef.current.stop();
      } catch {
        // Ignored
      }
      recognitionRef.current = null;
    }

    setAudioLevel(0);
    setIsRecording(false);

    // Retract voice listening presentation from NCM
    if (retractPresentation) {
      retractPresentation("hime-voice-input");
    } else if (typeof window !== "undefined") {
      window.dispatchEvent(
        new CustomEvent("hime:ncm-retract", { detail: { id: "hime-voice-input" } })
      );
    }
  }, [retractPresentation]);

  const startRecording = useCallback(async () => {
    setError(null);
    setTranscript("");

    try {
      if (typeof navigator === "undefined" || !navigator.mediaDevices?.getUserMedia) {
        throw new Error("Mikrofon tidak didukung");
      }

      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true
        }
      });
      mediaStreamRef.current = stream;

      const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
      if (AudioCtx) {
        const audioCtx = new AudioCtx();
        audioContextRef.current = audioCtx;

        const source = audioCtx.createMediaStreamSource(stream);
        const analyser = audioCtx.createAnalyser();
        analyser.fftSize = 256;
        source.connect(analyser);
        analyserRef.current = analyser;

        const dataArray = new Uint8Array(analyser.frequencyBinCount);

        const sampleRms = (timestamp: number) => {
          // Throttle updates to <= 30Hz (~33.3ms between updates)
          if (timestamp - lastUpdateTimeRef.current >= 33) {
            lastUpdateTimeRef.current = timestamp;
            if (analyserRef.current) {
              analyserRef.current.getByteTimeDomainData(dataArray);

              // Calculate Root-Mean-Square (RMS) amplitude
              let sum = 0;
              for (let i = 0; i < dataArray.length; i++) {
                const normalized = (dataArray[i] - 128) / 128;
                sum += normalized * normalized;
              }
              const rms = Math.sqrt(sum / dataArray.length);
              // Scale to 0.0 .. 1.0 with soft dampening
              const level = Math.min(1, Math.max(0, rms * 3.5));
              setAudioLevel(level);

              // Sync audio pulse to NCM
              const ncmItem = {
                id: "hime-voice-input",
                module: "voice",
                kind: "panel-hint",
                priority: PriorityLevel.P2_ACTION,
                userInitiated: true,
                expression: "dengar",
                payload: {
                  title: "Mendengarkan...",
                  audioLevel: level
                }
              };
              if (submitPresentation) {
                submitPresentation(ncmItem);
              } else if (typeof window !== "undefined") {
                window.dispatchEvent(
                  new CustomEvent("hime:ncm-submit", { detail: ncmItem })
                );
              }
            }
          }
          animationFrameRef.current = requestAnimationFrame(sampleRms);
        };

        animationFrameRef.current = requestAnimationFrame(sampleRms);
      }

      // Initialize Web Speech API if supported
      const SpeechRecognition =
        (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
      if (SpeechRecognition) {
        const reco = new SpeechRecognition();
        reco.continuous = true;
        reco.interimResults = true;
        reco.lang = "id-ID";

        reco.onresult = (event: any) => {
          let currentText = "";
          for (let i = event.resultIndex; i < event.results.length; ++i) {
            currentText += event.results[i][0].transcript;
          }
          if (currentText) {
            setTranscript(currentText);
            onTranscript?.(currentText);
          }
        };

        reco.onerror = (e: any) => {
          if (e.error !== "no-speech") {
            setError("Gagal mengenali suara".slice(0, 44));
          }
        };

        reco.start();
        recognitionRef.current = reco;
      }

      setIsRecording(true);
    } catch (err: any) {
      const msg = err?.message || String(err);
      const isDenied = msg.toLowerCase().includes("denied") || msg.toLowerCase().includes("permission");
      const shortErr = isDenied ? "Izin mikrofon ditolak" : "Gagal akses mikrofon";
      setError(shortErr.slice(0, 44));
      stopAudioCapture();
    }
  }, [onTranscript, submitPresentation, stopAudioCapture]);

  const toggleRecording = useCallback(() => {
    if (isRecording) {
      stopAudioCapture();
    } else {
      startRecording();
    }
  }, [isRecording, startRecording, stopAudioCapture]);

  // Global Push-to-Talk shortcut listener: Ctrl+Alt+M
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.altKey && (e.key === "m" || e.key === "M")) {
        e.preventDefault();
        toggleRecording();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      stopAudioCapture();
    };
  }, [toggleRecording, stopAudioCapture]);

  return {
    isRecording,
    audioLevel,
    transcript,
    error,
    startRecording,
    stopRecording: stopAudioCapture,
    toggleRecording
  };
}
