import path from "node:path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const tauriDevHost = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: tauriDevHost ?? false,
    hmr: tauriDevHost ? { protocol: "ws", host: tauriDevHost, port: 5174 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
