import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

// The Windows shell renders on a transparent always-on-top window: mark the
// document so styles switch to bubble mode.
if ("__TAURI_INTERNALS__" in window) {
  document.documentElement.classList.add("tauri");
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
