import React from "react";
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
  onClick?: () => void;
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

export const HimeEyes: React.FC<HimeEyesProps> = ({
  expression = "netral",
  size = 24,
  lookX = 0,
  lookY = 0,
  className = "",
  onClick
}) => {
  const expr = (eyesData.expressions as Record<string, any>)[expression] ?? eyesData.expressions.netral;
  const leftParams: EyeParams = expr.l;
  const rightParams: EyeParams = expr.r ?? expr.l;
  const color = COLOR_MAP[expr.color] ?? COLOR_MAP.cyan;
  const opacity = expr.opacity ?? 1.0;
  const rot = expr.rot ?? 0;

  // Render an individual eye SVG using SVG clipping mask
  const renderEye = (params: EyeParams, isRight: boolean, idSuffix: string) => {
    const eyeW = 10 * params.w;
    const eyeH = 24 * params.h;
    const rx = eyeW * params.r;

    // Gaze offset
    const offsetX = (params.dx + lookX * 0.25) * 4;
    const offsetY = (params.dy + lookY * 0.25) * 4;

    const tiltAngle = isRight ? -params.tilt : params.tilt;
    const clipId = `hime-eye-clip-${idSuffix}`;

    return (
      <svg
        width={eyeW}
        height={eyeH}
        viewBox="0 0 20 40"
        style={{
          overflow: "hidden",
          transition: "all 0.22s cubic-bezier(0.16, 1, 0.3, 1)",
          transform: `translate(${offsetX}px, ${offsetY}px)`
        }}
      >
        <defs>
          <clipPath id={clipId}>
            {/* Base visible eye window */}
            <rect x="0" y="0" width="20" height="40" />
            {/* Top eyelid cover */}
            {params.lidTop > 0 && (
              <rect
                x="-10"
                y="-10"
                width="40"
                height={40 * params.lidTop + 10}
                transform={`rotate(${tiltAngle} 10 20)`}
                fill="black"
              />
            )}
            {/* Bottom eyelid cover */}
            {params.lidBot > 0 && (
              <circle
                cx="10"
                cy={40 + 20 * (1 - params.lidBot)}
                r="18"
                fill="black"
              />
            )}
          </clipPath>
        </defs>

        <rect
          x="2"
          y="2"
          width="16"
          height="36"
          rx={rx * 1.6}
          fill={color}
          clipPath={`url(#${clipId})`}
          style={{
            filter: "drop-shadow(0 0 3px rgba(56, 214, 245, 0.4))",
            transition: "fill 0.25s ease"
          }}
        />
      </svg>
    );
  };

  const scale = size / 24;

  return (
    <div
      className={`hime-eyes-container ${className}`}
      onClick={onClick}
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        gap: `${6 * scale}px`,
        opacity,
        transform: `rotate(${rot}deg) scale(${scale})`,
        transformOrigin: "center center",
        transition: "transform 0.25s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.2s ease",
        cursor: onClick ? "pointer" : "default",
        userSelect: "none"
      }}
      title={`Hime: ${expression}`}
    >
      {renderEye(leftParams, false, "left")}
      {renderEye(rightParams, true, "right")}
    </div>
  );
};
