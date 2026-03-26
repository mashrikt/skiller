/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        surface: {
          0: "#141413",
          1: "#1a1a19",
          2: "#212120",
          3: "#2a2a28",
          4: "#333331",
        },
        accent: {
          DEFAULT: "#d97757",
          hover: "#e08b6f",
          muted: "#c46644",
          subtle: "rgba(217, 119, 87, 0.12)",
        },
        info: {
          DEFAULT: "#6a9bcc",
          muted: "rgba(106, 155, 204, 0.15)",
        },
        success: {
          DEFAULT: "#788c5d",
          muted: "rgba(120, 140, 93, 0.15)",
        },
        warning: {
          DEFAULT: "#c4a35a",
          muted: "rgba(196, 163, 90, 0.15)",
        },
        danger: {
          DEFAULT: "#c45c4a",
          muted: "rgba(196, 92, 74, 0.15)",
        },
        text: {
          primary: "#faf9f5",
          secondary: "#b0aea5",
          muted: "#7a7870",
        },
      },
      fontFamily: {
        sans: [
          "Inter",
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "sans-serif",
        ],
        body: [
          "Source Serif 4",
          "Georgia",
          "serif",
        ],
        mono: ["JetBrains Mono", "Fira Code", "monospace"],
      },
    },
  },
  plugins: [],
};
