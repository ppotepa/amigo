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
import { ThemeServiceProvider } from "./theme/themeService";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeServiceProvider>
      <App />
    </ThemeServiceProvider>
  </React.StrictMode>,
);
