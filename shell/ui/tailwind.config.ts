// Verbatim from docs/DESIGN.md § "Tailwind + shadcn Implementation" (tailwind.config.ts extract).
// Token source of truth lives in the DESIGN.md front matter — do not edit values here.
import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        border: "hsl(var(--border))", input: "hsl(var(--input))",
        ring: "hsl(var(--ring))", background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: { DEFAULT: "hsl(var(--primary))", foreground: "hsl(var(--primary-foreground))" },
        secondary: { DEFAULT: "hsl(var(--secondary))", foreground: "hsl(var(--secondary-foreground))" },
        destructive: { DEFAULT: "hsl(var(--destructive))", foreground: "hsl(var(--destructive-foreground))" },
        muted: { DEFAULT: "hsl(var(--muted))", foreground: "hsl(var(--muted-foreground))" },
        accent: { DEFAULT: "hsl(var(--accent))", foreground: "hsl(var(--accent-foreground))" },
        success: "hsl(var(--success))",
        warning: "hsl(var(--warning))",
        live: "hsl(var(--live))",
        "primary-soft": "hsl(var(--primary-soft))",
        "primary-active": "hsl(var(--primary-active))",
        "thinking-text": "hsl(var(--thinking-text))",
        "speaking-text": "hsl(var(--speaking-text))",
        card: { DEFAULT: "hsl(var(--card))", foreground: "hsl(var(--card-foreground))" },
      },
      fontFamily: {
        ui: ["Inter", "system-ui", "sans-serif"],
        companion: ["Nunito", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "ui-monospace", "monospace"],
      },
      borderRadius: {
        sm: "6px", DEFAULT: "var(--radius)", lg: "14px", xl: "18px",
        pill: "9999px", sharp: "2px",
      },
      boxShadow: {
        e1: "0 1px 2px rgba(43,36,30,0.06)",
        e2: "0 4px 12px rgba(43,36,30,0.10)",
        e3: "0 8px 32px rgba(43,36,30,0.16), 0 2px 8px rgba(43,36,30,0.08)",
      },
      backdropBlur: { panel: "24px" },
      keyframes: {
        breathe: { "0%,100%": { transform: "scale(1)" }, "50%": { transform: "scale(1.03)" } },
        "pulse-ring": { "0%": { transform: "scale(1)", opacity: "0.5" }, "100%": { transform: "scale(1.35)", opacity: "0" } },
        orbit: { to: { transform: "rotate(360deg)" } },
        shimmer: { "0%": { backgroundPosition: "-200% 0" }, "100%": { backgroundPosition: "200% 0" } },
        "token-caret": { "0%,100%": { opacity: "1" }, "50%": { opacity: "0" } },
      },
      animation: {
        breathe: "breathe 3s ease-in-out infinite",
        "pulse-ring": "pulse-ring 1.2s ease-out infinite",
        orbit: "orbit 1.6s linear infinite",
        shimmer: "shimmer 1.4s linear infinite",
        "token-caret": "token-caret 1s step-end infinite",
      },
    },
  },
} satisfies Config;
