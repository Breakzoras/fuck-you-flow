// Browser build of the real dashboard for the site's interface pictures.
// The four Tauri modules are swapped for the stand-ins in this folder, which
// answer with the sample content in data.ts. Nothing under src/ changes.
//
//   node press/app-shots/capture.mjs <output folder>   (starts Vite itself, port 1432)
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "node:path";

const here = (f: string) => resolve(import.meta.dirname, f);

export default defineConfig({
  root: import.meta.dirname,
  publicDir: resolve(import.meta.dirname, "..", "..", "public"),
  plugins: [react()],
  clearScreen: false,
  resolve: {
    alias: [
      { find: /^@tauri-apps\/api\/core$/, replacement: here("mock-core.ts") },
      { find: /^@tauri-apps\/api\/event$/, replacement: here("mock-event.ts") },
      { find: /^@tauri-apps\/api\/window$/, replacement: here("mock-window.ts") },
      { find: /^@tauri-apps\/plugin-opener$/, replacement: here("mock-opener.ts") },
    ],
  },
  optimizeDeps: {
    entries: [here("index.html")],
    exclude: ["@tauri-apps/api", "@tauri-apps/plugin-opener"],
  },
  server: {
    port: 1432,
    strictPort: true,
    host: "127.0.0.1",
    fs: { allow: [resolve(import.meta.dirname, "..", "..")] },
    watch: { ignored: ["**/src-tauri/**", "**/models/**", "**/site/**"] },
  },
});
