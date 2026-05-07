import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import "@fontsource/source-sans-3/latin-400.css";
import "@fontsource/source-sans-3/latin-500.css";
import "@fontsource/source-sans-3/latin-600.css";
import "@fontsource/source-sans-3/latin-700.css";
import "@fontsource/geist-sans/latin-400.css";
import "@fontsource/geist-sans/latin-500.css";
import "@fontsource/geist-sans/latin-600.css";
import "@fontsource/geist-sans/latin-700.css";
import "@fontsource/jetbrains-mono/latin-400.css";
import "@fontsource/jetbrains-mono/latin-600.css";
import "./styles/tokens.css";
import "./styles/themes.css";
import "./styles/window-shell.css";
import "./styles/components.css";
import "./styles/theme-features.css";
import "./ui/tree/tree-view.css";
import "./ui/explorer/explorer-shell.css";
import "./ui/context-dock/context-dock.css";
import "./features/scenes/editor/scene-editor.css";
import "./features/target-context/target-context.css";
import { ThemeServiceProvider } from "./theme/themeService";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeServiceProvider>
      <App />
    </ThemeServiceProvider>
  </React.StrictMode>,
);
