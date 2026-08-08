import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup } from "@testing-library/svelte";
import ThemeToggle from "../lib/components/ThemeToggle.svelte";

describe("ThemeToggle", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove("dark");
  });

  afterEach(() => {
    cleanup();
  });

  it("renders a button", () => {
    const { getByRole } = render(ThemeToggle, { props: { theme: "system" } });
    expect(getByRole("button")).toBeInTheDocument();
  });

  it("cycles from light to dark on click", () => {
    let theme = "light" as "light" | "dark" | "system";
    const { getByRole, component } = render(ThemeToggle, {
      props: { get theme() { return theme; }, set theme(v) { theme = v; } },
    });

    const button = getByRole("button");
    fireEvent.click(button);
    expect(theme).toBe("dark");
  });

  it("cycles from dark to system on click", () => {
    let theme = "dark" as "light" | "dark" | "system";
    const { getByRole } = render(ThemeToggle, {
      props: { get theme() { return theme; }, set theme(v) { theme = v; } },
    });

    const button = getByRole("button");
    fireEvent.click(button);
    expect(theme).toBe("system");
  });

  it("cycles from system back to light", () => {
    let theme = "system" as "light" | "dark" | "system";
    const { getByRole } = render(ThemeToggle, {
      props: { get theme() { return theme; }, set theme(v) { theme = v; } },
    });

    const button = getByRole("button");
    fireEvent.click(button);
    expect(theme).toBe("light");
  });
});
