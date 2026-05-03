import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        "amigo-bg": "#07111f",
        "amigo-window": "#0a1626",
        "amigo-surface": "#0d1b2e",
        "amigo-accent": "#3b82f6",
      },
      fontFamily: {
        sans: [
          "Inter",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "sans-serif",
        ],
      },
      borderRadius: {
        amigo: "10px",
        "amigo-window": "16px",
      },
    },
  },
  plugins: [],
} satisfies Config;
