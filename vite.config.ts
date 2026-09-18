import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Tauri expects a fixed dev server port unless started with --strictPort.
const host = process.env.TAURI_DEV_HOST;

/** Resolves a path relative to this config file (ESM-safe, no `__dirname`). */
const resolvePath = (relative: string) =>
  fileURLToPath(new URL(relative, import.meta.url));

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],

  resolve: {
    alias: {
      "@": resolvePath("./src"),
    },
  },

  // Vite options tailored for Tauri development.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Ignore Rust source changes while polishing the frontend.
      ignored: ["**/src-tauri/**"],
    },
  },

  // Multi-page build: the operator app + the dedicated presentation window.
  // The presentation window loads `presentation.html` from its own webview so
  // it never needs routing logic shared with the operator UI.
  build: {
    rollupOptions: {
      input: {
        main: resolvePath("./index.html"),
        presentation: resolvePath("./presentation.html"),
      },
    },
    target: "es2021",
    minify: process.env.TAURI_ENV_DEBUG ? false : "esbuild",
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
