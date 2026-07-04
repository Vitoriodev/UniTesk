import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "../App";

describe("App", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("renders the app title and subtitle", () => {
    render(<App />);
    expect(screen.getByText("Unitesk")).toBeInTheDocument();
    expect(
      screen.getByText("Gerenciador de Projetos Acadêmicos")
    ).toBeInTheDocument();
  });

  it("shows all navigation tabs", () => {
    render(<App />);
    const nav = screen.getAllByRole("button");
    const tabLabels = nav.map((btn) => btn.textContent?.trim());
    expect(tabLabels).toEqual(
      expect.arrayContaining(["📊Dashboard", "📁Projetos", "📅Calendário", "📄Artigos"])
    );
  });

  it("shows Dashboard as the default tab", () => {
    render(<App />);
    expect(screen.getByText("📊 Dashboard")).toBeInTheDocument();
  });

  it("navigates to Projetos tab when clicked", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: /Projetos/ }));
    expect(screen.getByText("📁 Projetos Acadêmicos")).toBeInTheDocument();
  });

  it("navigates to Calendário tab when clicked", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: /Calendário/ }));
    expect(screen.getByText("📅 Calendário de Atividades")).toBeInTheDocument();
  });

  it("navigates to Artigos tab when clicked", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: /Artigos/ }));
    expect(screen.getByText("📄 Documentos Acadêmicos")).toBeInTheDocument();
  });

  it("highlights the active tab", async () => {
    const user = userEvent.setup();
    render(<App />);
    const dashboardBtn = screen.getByRole("button", { name: /Dashboard/ });
    expect(dashboardBtn).toHaveClass("active");

    await user.click(screen.getByRole("button", { name: /Projetos/ }));
    expect(dashboardBtn).not.toHaveClass("active");
    expect(screen.getByRole("button", { name: /Projetos/ })).toHaveClass("active");
  });

  it("renders the footer", () => {
    render(<App />);
    expect(screen.getByText(/Unitesk v1.3/)).toBeInTheDocument();
  });

  it("renders theme toggle button", () => {
    render(<App />);
    expect(screen.getByTitle("Tema Dracula")).toBeInTheDocument();
  });

  it("theme defaults to light when no localStorage value", () => {
    localStorage.removeItem("unitesk_theme");
    render(<App />);
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
  });

  it("loads theme from localStorage", () => {
    localStorage.setItem("unitesk_theme", "dracula");
    render(<App />);
    expect(document.documentElement.getAttribute("data-theme")).toBe("dracula");
  });

  it("toggles theme when clicking the theme button", async () => {
    localStorage.setItem("unitesk_theme", "light");
    const user = userEvent.setup();
    render(<App />);

    // Initially light theme
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");

    // Click theme toggle
    await user.click(screen.getByTitle("Tema Dracula"));

    // Should switch to dracula
    expect(document.documentElement.getAttribute("data-theme")).toBe("dracula");
    expect(localStorage.getItem("unitesk_theme")).toBe("dracula");

    // Button title should change
    expect(screen.getByTitle("Tema Claro")).toBeInTheDocument();
  });

  it("toggles back to light when clicking again", async () => {
    localStorage.setItem("unitesk_theme", "dracula");
    const user = userEvent.setup();
    render(<App />);

    // Initially dracula
    expect(document.documentElement.getAttribute("data-theme")).toBe("dracula");

    // Click theme toggle
    await user.click(screen.getByTitle("Tema Claro"));

    // Should switch back to light
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
    expect(localStorage.getItem("unitesk_theme")).toBe("light");
  });

  it("shows moon icon when theme is light, sun icon when dracula", () => {
    localStorage.setItem("unitesk_theme", "light");
    render(<App />);
    expect(screen.getByText("🌙")).toBeInTheDocument();

    // Sun icon should not be present
    expect(screen.queryByText("☀️")).not.toBeInTheDocument();
  });
});
