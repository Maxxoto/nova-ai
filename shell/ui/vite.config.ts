import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 2 frontend. `tauri.conf.json` points build.frontendDist at ../ui/dist
// and build.devUrl at http://localhost:5173 (this dev server).
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    outDir: "dist",
    target: "es2022",
  },
});
