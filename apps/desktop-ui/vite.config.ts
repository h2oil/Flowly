import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The same bundle later loads inside the Tauri WebView2 shell; keep base
// relative so file:// and tauri:// origins both work.
export default defineConfig({
  plugins: [react()],
  base: "./",
});
