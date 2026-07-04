import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import Dashboard from "../components/Dashboard";

// Create a mutable invoke mock that can be configured per-test
const { invoke } = vi.hoisted(() => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke,
}));

describe("Dashboard", () => {
  const onNavigate = vi.fn();

  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
    // Set default mock return to prevent crashes in tests that don't configure it
    invoke.mockResolvedValue({
      totalProjects: 0,
      totalArticles: 0,
      pendingAssignments: 0,
      overdueAssignments: 0,
      nextDeadline: null,
      nextDeadlineName: null,
    });
  });

  it("renders the dashboard title", () => {
    render(<Dashboard />);
    expect(screen.getByText("📊 Dashboard")).toBeInTheDocument();
  });

  it("shows all stat cards with default values (0)", async () => {
    render(<Dashboard />);

    // Wait for useEffect to complete
    await waitFor(() => {
      expect(screen.getAllByText("Projetos").length).toBeGreaterThanOrEqual(1);
    });
    await waitFor(() => {
      expect(screen.getAllByText("Artigos").length).toBeGreaterThanOrEqual(1);
    });
    await waitFor(() => {
      expect(screen.getAllByText("Pendentes").length).toBeGreaterThanOrEqual(1);
    });
    await waitFor(() => {
      expect(screen.getAllByText("Atrasados").length).toBeGreaterThanOrEqual(1);
    });
  });

  it("shows 'Nenhum prazo próximo' when there is no next deadline", async () => {
    render(<Dashboard />);

    await waitFor(() => {
      expect(screen.getByText(/Nenhum prazo próximo/)).toBeInTheDocument();
    });
  });

  it("renders the quick actions section", async () => {
    render(<Dashboard />);

    await waitFor(() => {
      expect(screen.getByText("⚡ Ações Rápidas")).toBeInTheDocument();
    });
    expect(screen.getByText("➕ Novo Projeto")).toBeInTheDocument();
    expect(screen.getByText("📄 Novo Artigo")).toBeInTheDocument();
    expect(screen.getByText("📅 Nova Atividade")).toBeInTheDocument();
  });

  it("calls onNavigate with 'projects' when Novo Projeto is clicked", async () => {
    const user = userEvent.setup();
    render(<Dashboard onNavigate={onNavigate} />);
    await user.click(screen.getByText("➕ Novo Projeto"));
    expect(onNavigate).toHaveBeenCalledWith("projects");
  });

  it("calls onNavigate with 'articles' when Adicionar Artigo is clicked", async () => {
    const user = userEvent.setup();
    render(<Dashboard onNavigate={onNavigate} />);
    await user.click(screen.getByText("📄 Novo Artigo"));
    expect(onNavigate).toHaveBeenCalledWith("articles");
  });

  it("calls onNavigate with 'calendar' when Nova Atividade is clicked", async () => {
    const user = userEvent.setup();
    render(<Dashboard onNavigate={onNavigate} />);
    await user.click(screen.getByText("📅 Nova Atividade"));
    expect(onNavigate).toHaveBeenCalledWith("calendar");
  });

  it("renders the Próximo Prazo card", () => {
    render(<Dashboard />);
    expect(screen.getByText("📅 Próximo Prazo")).toBeInTheDocument();
  });

  it("renders stats from the backend when available", async () => {
    invoke.mockResolvedValue({
      totalProjects: 5,
      totalArticles: 12,
      pendingAssignments: 3,
      overdueAssignments: 1,
      nextDeadline: "2026-07-15",
      nextDeadlineName: "Entrega Artigo Redes",
    });

    render(<Dashboard onNavigate={onNavigate} />);

    await waitFor(() => {
      expect(screen.getByText("5")).toBeInTheDocument();
    });
    await waitFor(() => {
      expect(screen.getByText("12")).toBeInTheDocument();
    });
    await waitFor(() => {
      expect(screen.getByText("3")).toBeInTheDocument();
    });
    expect(screen.getAllByText("1").length).toBeGreaterThanOrEqual(1);
    await waitFor(() => {
      expect(screen.getByText("Entrega Artigo Redes")).toBeInTheDocument();
    });
    // A data 2026-07-15 é formatada pelo toLocaleDateString no componente
    await waitFor(() => {
      expect(screen.getByText(/2026/)).toBeInTheDocument();
    });
  });

  it("shows export button", async () => {
    render(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText("📤 Exportar")).toBeInTheDocument();
    });
  });

  it("shows import button", async () => {
    render(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText("📥 Importar Dados")).toBeInTheDocument();
    });
  });

  it("shows export and import buttons", async () => {
    render(<Dashboard />);
    await waitFor(() => {
      expect(screen.getByText("📤 Exportar")).toBeInTheDocument();
      expect(screen.getByText("📥 Importar Dados")).toBeInTheDocument();
    });
  });

  it("calls export_all_data when export button is clicked", async () => {
    invoke.mockResolvedValue({ version: "1.0.0", projects: [], articles: [], assignments: [], project_files: [], assignment_files: [] });

    vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:mock");
    vi.spyOn(URL, "revokeObjectURL").mockImplementation(() => {});

    const user = userEvent.setup();
    render(<Dashboard />);

    await waitFor(() => {
      expect(screen.getByText("📤 Exportar")).toBeInTheDocument();
    });

    await user.click(screen.getByText("📤 Exportar"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("export_all_data");
    });

    vi.restoreAllMocks();
  });

  it("shows success message after successful export", async () => {
    invoke.mockResolvedValue({ version: "1.0.0", projects: [], articles: [], assignments: [], project_files: [], assignment_files: [] });

    vi.spyOn(URL, "createObjectURL").mockReturnValue("blob:mock");
    vi.spyOn(URL, "revokeObjectURL").mockImplementation(() => {});

    const user = userEvent.setup();
    render(<Dashboard />);

    await waitFor(() => {
      expect(screen.getByText("📤 Exportar")).toBeInTheDocument();
    });

    await user.click(screen.getByText("📤 Exportar"));

    await waitFor(() => {
      expect(screen.getByText("✅ Dados exportados com sucesso!")).toBeInTheDocument();
    });

    vi.restoreAllMocks();
  });

  it("shows error message when export fails", async () => {
    invoke.mockRejectedValue(new Error("Falha na conexão"));

    const user = userEvent.setup();
    render(<Dashboard />);

    await waitFor(() => {
      expect(screen.getByText("📤 Exportar")).toBeInTheDocument();
    });

    await user.click(screen.getByText("📤 Exportar"));

    await waitFor(() => {
      expect(screen.getByText(/Erro ao exportar dados/)).toBeInTheDocument();
    });
  });

  describe("Update Check", () => {
    const originalFetch = globalThis.fetch;

    afterEach(() => {
      globalThis.fetch = originalFetch;
    });

    it("shows update button", async () => {
      render(<Dashboard />);
      await waitFor(() => {
        expect(screen.getByText("📥 Verificar Atualizações")).toBeInTheDocument();
      });
    });

    it("shows up-to-date message when no update available", async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ tag_name: "v1.1.0", assets: [{ browser_download_url: "" }] }),
      });

      const user = userEvent.setup();
      render(<Dashboard />);

      await waitFor(() => {
        expect(screen.getByText("📥 Verificar Atualizações")).toBeInTheDocument();
      });

      await user.click(screen.getByText("📥 Verificar Atualizações"));

      await waitFor(() => {
        expect(screen.getByText(/versão mais recente/)).toBeInTheDocument();
      });
    });

    it("shows update available message when newer version exists", async () => {
      globalThis.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({
          tag_name: "v2.0.0",
          assets: [{ browser_download_url: "https://example.com/unitesk.deb" }],
          body: "Novas funcionalidades incríveis!",
        }),
      });

      const user = userEvent.setup();
      render(<Dashboard />);

      await waitFor(() => {
        expect(screen.getByText("📥 Verificar Atualizações")).toBeInTheDocument();
      });

      await user.click(screen.getByText("📥 Verificar Atualizações"));

      await waitFor(() => {
        expect(screen.getByText(/Atualização Disponível/)).toBeInTheDocument();
      });
      expect(screen.getByText(/Baixar Atualização/)).toBeInTheDocument();
      expect(screen.getByText(/Novas funcionalidades incríveis/)).toBeInTheDocument();
    });

    it("shows error message when fetch fails", async () => {
      globalThis.fetch = vi.fn().mockRejectedValue(new Error("Network error"));

      const user = userEvent.setup();
      render(<Dashboard />);

      await waitFor(() => {
        expect(screen.getByText("📥 Verificar Atualizações")).toBeInTheDocument();
      });

      await user.click(screen.getByText("📥 Verificar Atualizações"));

      await waitFor(() => {
        expect(screen.getByText(/Erro ao verificar atualizações/)).toBeInTheDocument();
      });
    });
  });
});
