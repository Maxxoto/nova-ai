import { defineConfig } from "@playwright/test";

// Motion-test harness for the Ruòxī UI. Every run boots the *production*
// build through `vite preview`, then reads real computed animation styles, so
// the assertions in `e2e/` match the artifact the shell actually ships.
export default defineConfig({
  testDir: "./e2e",
  outputDir: "e2e/.artifacts",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  reporter: "list",
  use: {
    baseURL: "http://localhost:4173",
    reducedMotion: "no-preference",
    viewport: { width: 1280, height: 800 },
  },
  webServer: {
    command: "pnpm build && pnpm preview --port 4173",
    url: "http://localhost:4173",
    reuseExistingServer: true,
    timeout: 180_000,
  },
  projects: [
    {
      name: "chromium",
      use: { browserName: "chromium" },
    },
  ],
});
