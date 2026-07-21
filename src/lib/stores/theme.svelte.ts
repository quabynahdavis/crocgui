import { browser } from "$app/environment";

export type Theme = "light" | "dark" | "system";

function applyTheme(theme: Theme) {
  if (!browser) return;
  const root = document.documentElement;
  if (theme === "dark") {
    root.classList.add("dark");
  } else if (theme === "light") {
    root.classList.remove("dark");
  } else {
    root.classList.toggle("dark", window.matchMedia("(prefers-color-scheme: dark)").matches);
  }
}

function loadTheme(): Theme {
  if (!browser) return "system";
  const stored = localStorage.getItem("theme") as Theme | null;
  if (stored === "light" || stored === "dark" || stored === "system") return stored;
  return "system";
}

export function saveTheme(theme: Theme) {
  if (browser) {
    localStorage.setItem("theme", theme);
  }
  applyTheme(theme);
}

export function initTheme() {
  const theme = loadTheme();
  applyTheme(theme);
  if (theme === "system" && browser) {
    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
      applyTheme("system");
    });
  }
  return theme;
}
