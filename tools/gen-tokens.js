/**
 * Hime Island Design Tokens Generator (tools/gen-tokens.js)
 * Compiles design/tokens.json into island/src/hime/styles/tokens.css
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

const tokensPath = path.join(repoRoot, "design", "tokens.json");
const outCssPath = path.join(repoRoot, "island", "src", "hime", "styles", "tokens.css");

if (!fs.existsSync(tokensPath)) {
  console.error(`Tokens file not found at ${tokensPath}`);
  process.exit(1);
}

const tokens = JSON.parse(fs.readFileSync(tokensPath, "utf-8"));

let css = `/**
 * AUTO-GENERATED FILE — DO NOT EDIT DIRECTLY
 * Generated from design/tokens.json via tools/gen-tokens.js
 */

:root {
  /* Surfaces */
  --surface-island: ${tokens.surfaces["surface-island"]};
  --surface-1: ${tokens.surfaces["surface-1"]};
  --surface-2: ${tokens.surfaces["surface-2"]};
  --surface-3: ${tokens.surfaces["surface-3"]};
  --line: ${tokens.surfaces["line"]};

  /* Text */
  --text-1: ${tokens.text["text-1"].value};
  --text-2: ${tokens.text["text-2"].value};
  --text-3: ${tokens.text["text-3"].value};

  /* Accents */
`;

for (const [key, val] of Object.entries(tokens.accents)) {
  css += `  --accent-${key}: ${val.value};\n`;
  css += `  --accent-${key}-tint: ${val.value}2e; /* 18% opacity */\n`;
  css += `  --accent-${key}-hover: ${val.value}3d; /* 24% opacity */\n`;
}

css += `
  /* Spacing */
`;
tokens.geometry.spacing.forEach((s) => {
  css += `  --space-${s}: ${s}px;\n`;
});

css += `
  /* Radii */
  --radius-pill: ${tokens.geometry.radius.pill}px;
  --radius-card: ${tokens.geometry.radius.card}px;
  --radius-icon-tile: ${tokens.geometry.radius.iconTile}px;
  --radius-input: ${tokens.geometry.radius.input}px;

  /* Typography */
  --font-title-size: ${tokens.typography.title.size}px;
  --font-title-weight: ${tokens.typography.title.weight};
  --font-title-lh: ${tokens.typography.title.lineHeight}px;

  --font-body-size: ${tokens.typography.body.size}px;
  --font-body-weight: ${tokens.typography.body.weight};
  --font-body-lh: ${tokens.typography.body.lineHeight}px;

  --font-desc-size: ${tokens.typography.desc.size}px;
  --font-desc-weight: ${tokens.typography.desc.weight};
  --font-desc-lh: ${tokens.typography.desc.lineHeight}px;

  /* Motion & Spring Physics */
  --anim-spring-snappy: cubic-bezier(0.16, 1, 0.3, 1);
  --anim-duration-snappy: 220ms;
  --anim-spring-smooth: cubic-bezier(0.2, 0.9, 0.3, 1);
  --anim-duration-smooth: 300ms;
  --anim-spring-soft: cubic-bezier(0.25, 1, 0.5, 1);
  --anim-duration-soft: 350ms;
  --anim-spring-bouncy: cubic-bezier(0.34, 1.3, 0.64, 1);
  --anim-duration-bouncy: 260ms;

  --spring-snappy-stiffness: ${tokens.motion.snappy.stiffness};
  --spring-snappy-damping: ${tokens.motion.snappy.damping};
  --spring-snappy-mass: ${tokens.motion.snappy.mass};

  --spring-smooth-stiffness: ${tokens.motion.smooth.stiffness};
  --spring-smooth-damping: ${tokens.motion.smooth.damping};
  --spring-smooth-mass: ${tokens.motion.smooth.mass};

  --spring-soft-stiffness: ${tokens.motion.soft.stiffness};
  --spring-soft-damping: ${tokens.motion.soft.damping};
  --spring-soft-mass: ${tokens.motion.soft.mass};

  --spring-bouncy-stiffness: ${tokens.motion.bouncy.stiffness};
  --spring-bouncy-damping: ${tokens.motion.bouncy.damping};
  --spring-bouncy-mass: ${tokens.motion.bouncy.mass};
}

/* 8 Hime Mascot Keyframe Motion Presets */
@keyframes hime-bounce-soft {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-2px); }
}

@keyframes hime-pulse-twice {
  0% { transform: scale(1); }
  25% { transform: scale(1.08); }
  50% { transform: scale(1); }
  75% { transform: scale(1.08); }
  100% { transform: scale(1); }
}

@keyframes hime-wander-slow {
  0%, 100% { transform: translate(0, 0); }
  25% { transform: translate(-1.5px, -1px); }
  50% { transform: translate(-0.5px, -2px); }
  75% { transform: translate(1px, -1px); }
}

@keyframes hime-audio-pulse {
  0%, 100% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.12); opacity: 0.9; }
}

@keyframes hime-pop {
  0% { transform: scale(0.8); }
  60% { transform: scale(1.22); }
  100% { transform: scale(1); }
}

@keyframes hime-breathing-slow {
  0%, 100% { transform: scaleY(0.95); opacity: 0.85; }
  50% { transform: scaleY(1.05); opacity: 1; }
}

@keyframes hime-jitter-2px {
  0%, 100% { transform: translateX(0); }
  20% { transform: translateX(-2px); }
  40% { transform: translateX(2px); }
  60% { transform: translateX(-1px); }
  80% { transform: translateX(1px); }
}

@keyframes hime-orbit-left {
  0% { transform: translate(-1.5px, 0); }
  25% { transform: translate(0, -1.5px); }
  50% { transform: translate(1.5px, 0); }
  75% { transform: translate(0, 1.5px); }
  100% { transform: translate(-1.5px, 0); }
}

@keyframes hime-orbit-right {
  0% { transform: translate(1.5px, 0); }
  25% { transform: translate(0, 1.5px); }
  50% { transform: translate(-1.5px, 0); }
  75% { transform: translate(0, -1.5px); }
  100% { transform: translate(1.5px, 0); }
}

`;

fs.mkdirSync(path.dirname(outCssPath), { recursive: true });
fs.writeFileSync(outCssPath, css, "utf-8");

console.log(`[OK] Generated tokens.css successfully at ${outCssPath}`);
