// Tray PNG matrix exporter — shell/ui → shell/src-tauri/icons/tray/
//
// Render every SVG source (colored + -template) at the platform size matrix
// with @resvg/resvg-js. Re-run after editing any SVG: `pnpm export:tray-icons`.
//
// Naming (per state <s>):
//   tray-<s>.png                 18px  macOS @1x / Linux @1x fallback   (colored)
//   tray-<s>@2x.png              36px  macOS @2x / Linux @2x fallback   (colored)
//   tray-<s>-16.png              16px  Windows @1x                      (colored)
//   tray-<s>-32.png              32px  Windows @2x                      (colored)
//   tray-<s>-22.png              22px  Linux @1x                        (colored)
//   tray-<s>-44.png              44px  Linux @2x                        (colored)
//   ...same six with -template infix, pure black + alpha (macOS template images)
//
// All PNGs carry real alpha; the -template set is pure #000 + alpha and is
// enabled on macOS via tray.iconAsTemplate(true) — see shell/ui/README.md.
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { Resvg } from "@resvg/resvg-js";

const svgDir = path.join(path.dirname(fileURLToPath(import.meta.url)), "../src/tray-icons/svg");
const outDir = path.join(path.dirname(fileURLToPath(import.meta.url)), "../../src-tauri/icons/tray");

const SIZES = [
  { suffix: "", px: 18 },
  { suffix: "@2x", px: 36 },
  { suffix: "-16", px: 16 },
  { suffix: "-32", px: 32 },
  { suffix: "-22", px: 22 },
  { suffix: "-44", px: 44 },
];

const svgFiles = readdirSync(svgDir).filter((f) => f.endsWith(".svg"));
mkdirSync(outDir, { recursive: true });

let count = 0;
for (const file of svgFiles) {
  const stem = path.basename(file, ".svg"); // e.g. tray-listening or tray-listening-template
  const svg = readFileSync(path.join(svgDir, file), "utf8");
  for (const { suffix, px } of SIZES) {
    const png = new Resvg(svg, {
      fitTo: { mode: "width", value: px },
      font: { loadSystemFonts: false },
    }).render().asPng();
    writeFileSync(path.join(outDir, `${stem}${suffix}.png`), png);
    count += 1;
  }
}

console.log(`exported ${count} PNGs (${svgFiles.length} sources × ${SIZES.length} sizes) → ${outDir}`);
