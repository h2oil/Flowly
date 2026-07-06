import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The same bundle later loads inside the Tauri WebView2 shell; keep base
// relative so file:// and tauri:// origins both work.
export default defineConfig({
  plugins: [react()],
  base: "./",
  // The WASM engine is imported as `?inline` (data URI) so the whole app can
  // ship as one self-contained file (artifact/offline builds).
  assetsInclude: ["**/*.wasm"],
});
