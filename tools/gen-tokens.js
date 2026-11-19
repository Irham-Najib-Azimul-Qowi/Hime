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
}
`;

fs.mkdirSync(path.dirname(outCssPath), { recursive: true });
fs.writeFileSync(outCssPath, css, "utf-8");

console.log(`[OK] Generated tokens.css successfully at ${outCssPath}`);
