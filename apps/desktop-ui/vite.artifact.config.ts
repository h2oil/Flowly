import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Single-file build for the hosted artifact demo: every chunk and asset
// (WASM engine, docx/pdf importers) inlined so the page works standalone
// under a strict CSP with no external requests.
export default defineConfig({
  plugins: [react()],
  base: "./",
  assetsInclude: ["**/*.wasm"],
  build: {
    outDir: "dist-artifact",
    assetsInlineLimit: 100_000_000,
    chunkSizeWarningLimit: 5_000,
    rollupOptions: { output: { inlineDynamicImports: true } },
  },
});
