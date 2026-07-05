import { describe, it, expect, beforeEach, vi } from "vitest";
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
      totalClients: 0,
      totalTeams: 0,
      totalUsers: 0,
      pendingAssignments: 0,
      overdueAssignments: 0,
      nextDeadline: null,
      nextDeadlineName: null,
      hoursToday: 0,
      hoursWeek: 0,
      totalRevenue: 0,
      pendingInvoices: 0,
      pendingAmount: 0,
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
      expect(screen.getAllByText("Documentos").length).toBeGreaterThanOrEqual(1);
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
      totalClients: 3,
      totalTeams: 2,
      totalUsers: 8,
      pendingAssignments: 3,
      overdueAssignments: 1,
      nextDeadline: "2026-07-15",
      nextDeadlineName: "Entrega Artigo Redes",
      hoursToday: 120,
      hoursWeek: 600,
      totalRevenue: 15000,
      pendingInvoices: 2,
      pendingAmount: 8500,
    });

    render(<Dashboard onNavigate={onNavigate} />);

    await waitFor(() => {
      expect(screen.getByText("5")).toBeInTheDocument();
    });
    await waitFor(() => {
      expect(screen.getByText("12")).toBeInTheDocument();
    });
    await waitFor(() => {
      expect(screen.getAllByText("3").length).toBeGreaterThanOrEqual(1);
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

  function mockStatsForExport() {
    // Return dashboard stats for initial load, export data for export call
    invoke.mockImplementation(async (_cmd: string) => {
      if (_cmd === "export_all_data") {
        return { version: "1.0.0", projects: [], articles: [], assignments: [],
                 project_files: [], assignment_files: [] };
      }
      return { totalProjects: 0, totalArticles: 0, totalClients: 0, totalTeams: 0,
               totalUsers: 0, pendingAssignments: 0, overdueAssignments: 0,
               nextDeadline: null, nextDeadlineName: null,
               hoursToday: 0, hoursWeek: 0, totalRevenue: 0, pendingInvoices: 0,
               pendingAmount: 0 };
    });
  }

  it("calls export_all_data when export button is clicked", async () => {
    mockStatsForExport();

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
    mockStatsForExport();

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
    // Mock stats normally, but reject on any invoke call
    invoke.mockImplementation(async (_cmd: string) => {
      throw new Error("Falha na conexão");
    });

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

  // NOTA: Testes de verificação de atualizações removidos —
  // o botão "Verificar Atualizações" foi removido do Dashboard.
});
