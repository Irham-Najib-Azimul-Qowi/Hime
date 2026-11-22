import React, { useState, useEffect, useId } from "react";
import "../styles/tokens.css";
import eyesData from "../assets/eyes.json";

export type EyeExpression =
  | "netral"
  | "senang"
  | "semangat"
  | "bangga"
  | "fokus"
  | "berpikir"
  | "dengar"
  | "penasaran"
  | "kaget"
  | "khawatir"
  | "bosan"
  | "ngantuk"
  | "menguap"
  | "tidur"
  | "kesal"
  | "marah"
  | "sedih"
  | "curiga"
  | "nakal"
  | "kedip-sebelah"
  | "pusing"
  | "offline";

export interface HimeEyesProps {
  expression?: EyeExpression;
  size?: number; // base height in px, default 24
  lookX?: number; // -1 to 1 manual cursor gaze override
  lookY?: number; // -1 to 1
  className?: string;
  onClick?: (e?: React.MouseEvent<HTMLDivElement>) => void;
  audioLevel?: number; // 0 to 1 for audio-pulse (e.g. dengar)
  onExpressionComplete?: (expr: EyeExpression) => void;
}

interface EyeParams {
  w: number;
  h: number;
  r: number;
  lidTop: number;
  tilt: number;
  lidBot: number;
  dx: number;
  dy: number;
}

const COLOR_MAP: Record<string, string> = {
  cyan: "var(--accent-cyan, #38D6F5)",
  mint: "var(--accent-mint, #34E3A6)",
  lime: "var(--accent-lime, #A8EE4B)",
  violet: "var(--accent-violet, #A18BFF)",
  amber: "var(--accent-amber, #FFC247)",
  coral: "var(--accent-coral, #FF7A59)",
  red: "var(--accent-red, #FF4D5E)",
  sky: "var(--accent-sky, #5AA9FF)",
  pink: "var(--accent-pink, #FF5C93)",
  "text-3": "var(--text-3, #6F7785)"
};

const SHADOW_MAP: Record<string, string> = {
  cyan: "drop-shadow(0 0 3px rgba(56, 214, 245, 0.45))",
  mint: "drop-shadow(0 0 3px rgba(52, 227, 166, 0.45))",
  lime: "drop-shadow(0 0 3px rgba(168, 238, 75, 0.45))",
  violet: "drop-shadow(0 0 3px rgba(161, 139, 255, 0.45))",
  amber: "drop-shadow(0 0 3px rgba(255, 194, 71, 0.45))",
  coral: "drop-shadow(0 0 3px rgba(255, 122, 89, 0.45))",
  red: "drop-shadow(0 0 3px rgba(255, 77, 94, 0.45))",
  sky: "drop-shadow(0 0 3px rgba(90, 169, 255, 0.45))",
  pink: "drop-shadow(0 0 3px rgba(255, 92, 147, 0.45))",
  "text-3": "none"
};

export const HimeEyes: React.FC<HimeEyesProps> = ({
  expression = "netral",
  size = 24,
  lookX = 0,
  lookY = 0,
  className = "",
  onClick,
  audioLevel,
  onExpressionComplete
}) => {
  const instanceId = useId().replace(/[:/]/g, "_");
  const [isBlinking, setIsBlinking] = useState(false);
  const [yawnPhase, setYawnPhase] = useState<number | null>(null);
  const [saccade, setSaccade] = useState({ dx: 0, dy: 0 });

  // Parametric expression data from eyes.json catalog
  const expr = (Object.prototype.hasOwnProperty.call(eyesData.expressions, expression) ? (eyesData.expressions as Record<string, any>)[expression] : null) ?? eyesData.expressions.netral;
  const leftParams: EyeParams = expr.l;
  const rightParams: EyeParams = expr.r ?? expr.l;
  const color = COLOR_MAP[expr.color] ?? COLOR_MAP.cyan;
  const opacity = expr.opacity ?? (expression === "offline" ? 0.4 : 1.0);
  const rot = expr.rot ?? 0;
  const shadow = expression === "offline" ? "none" : (SHADOW_MAP[expr.color] ?? SHADOW_MAP.cyan);

  // Expression completion timer (if expression has duration and onExpressionComplete is provided)
  useEffect(() => {
    if (!onExpressionComplete || !expr.duration || !Array.isArray(expr.duration)) return;
    const durationMs = Math.round((expr.duration[0] + expr.duration[1]) / 2);
    const timer = setTimeout(() => {
      onExpressionComplete(expression);
    }, durationMs);

    return () => clearTimeout(timer);
  }, [expression, onExpressionComplete, expr.duration]);

  // Blink timer:
  // - netral: random interval 3000ms - 7000ms, close for 120ms
  // - ngantuk: random interval 2000ms - 4000ms, slow lazy close for 250ms
  useEffect(() => {
    if (expression !== "netral" && expression !== "ngantuk") {
      setIsBlinking(false);
      return;
    }

    let blinkTimeout: ReturnType<typeof setTimeout>;
    let closeTimeout: ReturnType<typeof setTimeout>;

    const isLazy = expression === "ngantuk";
    const minInterval = isLazy ? 2000 : 3000;
    const maxInterval = isLazy ? 4000 : 7000;
    const closeDur = isLazy ? 250 : 120;

    const scheduleNextBlink = () => {
      const delay = Math.floor(Math.random() * (maxInterval - minInterval + 1)) + minInterval;
      blinkTimeout = setTimeout(() => {
        setIsBlinking(true);
        closeTimeout = setTimeout(() => {
          setIsBlinking(false);
          scheduleNextBlink();
        }, closeDur);
      }, delay);
    };

    scheduleNextBlink();

    return () => {
      clearTimeout(blinkTimeout);
      clearTimeout(closeTimeout);
    };
  }, [expression]);

  // 3-Phase Yawn sequence for 'menguap' (1600ms total):
  // Phase 1 (0-300ms): squint (lidTop 0.5)
  // Phase 2 (300-1300ms): tight shut (lidTop 0.95)
  // Phase 3 (1300-1600ms): reopen (lidTop 0.2)
  useEffect(() => {
    if (expression !== "menguap") {
      setYawnPhase(null);
      return;
    }

    setYawnPhase(1);
    const t2 = setTimeout(() => setYawnPhase(2), 300);
    const t3 = setTimeout(() => setYawnPhase(3), 1300);

    return () => {
      clearTimeout(t2);
      clearTimeout(t3);
    };
  }, [expression]);

  // Micro-saccades: subtle shift every 2.5 - 4.5 seconds for netral & bosan state
  useEffect(() => {
    if (expression !== "netral" && expression !== "bosan") {
      setSaccade({ dx: 0, dy: 0 });
      return;
    }

    let saccadeTimer: ReturnType<typeof setTimeout>;

    const scheduleNextSaccade = () => {
      const delay = Math.floor(Math.random() * 2000) + 2500;
      saccadeTimer = setTimeout(() => {
        const nextDx = (Math.random() - 0.5) * (expression === "bosan" ? 0.2 : 0.1);
        const nextDy = (Math.random() - 0.5) * (expression === "bosan" ? 0.08 : 0.06);
        setSaccade({ dx: nextDx, dy: nextDy });
        scheduleNextSaccade();
      }, delay);
    };

    scheduleNextSaccade();

    return () => {
      clearTimeout(saccadeTimer);
    };
  }, [expression]);

  // Determine container motion style from 8 presets
  const containerMotionStyle: React.CSSProperties = {};
  const motionPreset = expr.motion;
  if (motionPreset && motionPreset !== "orbit-small") {
    switch (motionPreset) {
      case "bounce-soft":
        containerMotionStyle.animation = "hime-bounce-soft 1.8s ease-in-out infinite";
        break;
      case "pulse-twice":
        containerMotionStyle.animation = "hime-pulse-twice 0.8s ease-out 1";
        break;
      case "wander-slow":
        containerMotionStyle.animation = "hime-wander-slow 3.2s ease-in-out infinite";
        break;
      case "audio-pulse":
        if (typeof audioLevel === "number" && audioLevel > 0) {
          const dynamicScale = 1 + Math.min(1, Math.max(0, audioLevel)) * 0.25;
          containerMotionStyle.transform = `rotate(${rot}deg) scale(${dynamicScale * (size / 24)})`;
          containerMotionStyle.transition = "transform 0.06s ease-out";
        } else {
          containerMotionStyle.animation = "hime-audio-pulse 0.8s ease-in-out infinite";
        }
        break;
      case "pop":
        containerMotionStyle.animation = "hime-pop 0.35s cubic-bezier(0.16, 1.3, 0.3, 1) 1";
        break;
      case "breathing-slow":
        containerMotionStyle.animation = "hime-breathing-slow 4s ease-in-out infinite";
        break;
      case "jitter-2px":
        containerMotionStyle.animation = "hime-jitter-2px 0.15s linear infinite";
        break;
    }
  }

  // Render an individual eye SVG using SVG <mask id="..."> (white = visible, black = cutout)
  const renderEye = (params: EyeParams, isRight: boolean, idSuffix: string) => {
    const eyeW = 10 * params.w;
    const eyeH = 24 * params.h;
    const rx = eyeW * params.r;

    // Gaze offset combining parametric dx/dy, micro-saccades, and manual cursor gaze
    const offsetX = (params.dx + saccade.dx + lookX * 0.25) * 4;
    const offsetY = (params.dy + saccade.dy + lookY * 0.25) * 4;

    const tiltAngle = isRight ? -params.tilt : params.tilt;
    const maskId = `hime-eye-mask-${instanceId}-${idSuffix}`;

    // Compute effective lid top
    let effectiveLidTop = params.lidTop;
    if (yawnPhase !== null) {
      if (yawnPhase === 1) effectiveLidTop = 0.5;
      else if (yawnPhase === 2) effectiveLidTop = 0.95;
      else if (yawnPhase === 3) effectiveLidTop = 0.2;
    } else if (isBlinking) {
      effectiveLidTop = 0.95;
    }

    // Individual eye motion (e.g. orbit-small for pusing)
    const eyeStyle: React.CSSProperties = {
      overflow: "hidden",
      transition: isBlinking
        ? "transform 0.08s ease-in"
        : "transform 0.22s cubic-bezier(0.16, 1, 0.3, 1)",
      transform: `translate(${offsetX}px, ${offsetY}px)`
    };

    if (motionPreset === "orbit-small") {
      eyeStyle.animation = isRight
        ? "hime-orbit-right 1.4s linear infinite"
        : "hime-orbit-left 1.4s linear infinite";
    }

    return (
      <svg
        width={eyeW}
        height={eyeH}
        viewBox="0 0 20 40"
        preserveAspectRatio="none"
        style={eyeStyle}
      >
        <defs>
          <mask
            id={maskId}
            maskUnits="userSpaceOnUse"
            maskContentUnits="userSpaceOnUse"
            x="-10"
            y="-10"
            width="40"
            height="60"
          >
            {/* Base visible eye window: white means visible */}
            <rect x="-10" y="-10" width="40" height="60" fill="#FFFFFF" />

            {/* Top eyelid cover: black cutout */}
            {effectiveLidTop > 0 && (
              <rect
                x="-10"
                y="-10"
                width="40"
                height={40 * effectiveLidTop + 10}
                transform={`rotate(${tiltAngle} 10 20)`}
                fill="#000000"
              />
            )}

            {/* Bottom eyelid cover: black cutout forming smiling '^ ^' curve */}
            {params.lidBot > 0 && (
              <circle
                cx="10"
                cy={40 + 20 * (1 - params.lidBot)}
                r="18"
                fill="#000000"
              />
            )}
          </mask>
        </defs>

        <rect
          x="2"
          y="2"
          width="16"
          height="36"
          rx={rx * 1.6}
          fill={color}
          mask={`url(#${maskId})`}
          style={{
            filter: shadow,
            transition: "fill 0.25s ease, filter 0.25s ease"
          }}
        />
      </svg>
    );
  };

  const scale = size / 24;
  const baseTransform = `rotate(${rot}deg) scale(${scale})`;

  return (
    <div
      className={`hime-eyes-container ${className}`}
      onClick={(e) => {
        if (onClick) {
          e.stopPropagation();
          onClick(e);
        }
      }}
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        gap: `${6 * scale}px`,
        opacity,
        transform: containerMotionStyle.transform || baseTransform,
        transformOrigin: "center center",
        transition: containerMotionStyle.transition || "transform 0.25s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.2s ease",
        cursor: onClick ? "pointer" : "default",
        userSelect: "none",
        ...containerMotionStyle
      }}
      title={`Hime: ${expression}`}
    >
      {renderEye(leftParams, false, "left")}
      {renderEye(rightParams, true, "right")}
    </div>
  );
};
