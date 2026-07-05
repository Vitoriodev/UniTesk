import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import TimeTracking from "../components/TimeTracking";

const { invoke } = vi.hoisted(() => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke,
}));

const mockProjects = [
  { id: 1, name: "Projeto Alpha" },
  { id: 2, name: "Projeto Beta" },
];

const mockEntries = [
  {
    id: 1, project_id: 1, project_name: "Projeto Alpha", user_id: null, user_name: null,
    description: "Desenvolvendo feature", start_time: "2026-07-04T09:00:00Z",
    end_time: "2026-07-04T11:30:00Z", duration_minutes: 150,
    billable: true, hourly_rate: 100, created_at: "2026-07-04T09:00:00Z",
  },
  {
    id: 2, project_id: 2, project_name: "Projeto Beta", user_id: null, user_name: null,
    description: "Reunião", start_time: "2026-07-04T14:00:00Z",
    end_time: "2026-07-04T15:00:00Z", duration_minutes: 60,
    billable: true, hourly_rate: 150, created_at: "2026-07-04T14:00:00Z",
  },
  {
    id: 3, project_id: 1, project_name: "Projeto Alpha", user_id: null, user_name: null,
    description: "Tarefa não faturável", start_time: "2026-07-03T10:00:00Z",
    end_time: "2026-07-03T10:30:00Z", duration_minutes: 30,
    billable: false, hourly_rate: null, created_at: "2026-07-03T10:00:00Z",
  },
];

describe("TimeTracking", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [];
      if (cmd === "get_projects") return mockProjects;
      if (cmd === "get_hours_summary") return [0, 0];
      if (cmd === "get_active_time_entry") return null;
      return [];
    });
  });

  // ==================== Render & Layout ====================

  it("renders the title", () => {
    render(<TimeTracking />);
    expect(screen.getByText("⏱️ Controle de Horas")).toBeInTheDocument();
  });

  it("shows the timer section with Iniciar Timer title", () => {
    render(<TimeTracking />);
    expect(screen.getByText("▶️ Iniciar Timer")).toBeInTheDocument();
  });

  it("shows the start button disabled initially", () => {
    render(<TimeTracking />);
    const startBtn = screen.getByText("▶️ Iniciar");
    expect(startBtn).toBeDisabled();
  });

  it("shows empty state when no entries", async () => {
    render(<TimeTracking />);
    await waitFor(() => {
      expect(screen.getByText("Nenhum registro de horas encontrado.")).toBeInTheDocument();
    });
  });

  // ==================== Summary Cards ====================

  it("shows summary cards with labels", async () => {
    render(<TimeTracking />);
    await waitFor(() => {
      expect(screen.getByText("⏱️ Hoje")).toBeInTheDocument();
      expect(screen.getByText("📅 Esta Semana")).toBeInTheDocument();
      expect(screen.getByText("📊 Total Registrado")).toBeInTheDocument();
    });
  });

  it("shows hours from backend summary", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [];
      if (cmd === "get_projects") return mockProjects;
      if (cmd === "get_hours_summary") return [120, 480];
      if (cmd === "get_active_time_entry") return null;
      return [];
    });

    render(<TimeTracking />);

    await waitFor(() => {
      // Today: 120 min = 2h 0m
      expect(screen.getByText("2h 0m")).toBeInTheDocument();
    });
    // Week: 480 min = 8h 0m
    expect(screen.getByText("8h 0m")).toBeInTheDocument();
  });

  it("shows entry count in total card", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return mockEntries;
      if (cmd === "get_projects") return mockProjects;
      if (cmd === "get_hours_summary") return [0, 0];
      if (cmd === "get_active_time_entry") return null;
      return [];
    });

    render(<TimeTracking />);

    await waitFor(() => {
      expect(screen.getByText("3 entrada(s)")).toBeInTheDocument();
    });
  });

  // ==================== Lista de Entradas ====================

  it("displays entries from backend", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return mockEntries;
      if (cmd === "get_projects") return mockProjects;
      if (cmd === "get_hours_summary") return [0, 0];
      if (cmd === "get_active_time_entry") return null;
      return [];
    });

    render(<TimeTracking />);

    await waitFor(() => {
      // Use getAllByText because "Projeto Alpha" appears in the entry cards AND select options
      const projectMatches = screen.getAllByText(/Projeto Alpha/);
      expect(projectMatches.length).toBeGreaterThanOrEqual(1);
    });

    // Unique descriptions
    expect(screen.getByText("Desenvolvendo feature")).toBeInTheDocument();
    expect(screen.getByText("Tarefa não faturável")).toBeInTheDocument();
  });

  it("shows duration badge for entries", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [mockEntries[0]];
      return [];
    });

    render(<TimeTracking />);

    await waitFor(() => {
      expect(screen.getByText("⏱️ 2h 30m")).toBeInTheDocument();
    });
  });

  it("shows non-billable label", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [mockEntries[2]];
      return [];
    });

    render(<TimeTracking />);

    await waitFor(() => {
      expect(screen.getByText("Não faturável")).toBeInTheDocument();
    });
  });

  it("shows delete button for entries", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [mockEntries[0]];
      return [];
    });

    render(<TimeTracking />);

    await waitFor(() => {
      expect(screen.getByTitle("Excluir")).toBeInTheDocument();
    });
  });

  it("shows confirm/cancel on delete click", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [mockEntries[0]];
      return [];
    });

    const user = userEvent.setup();
    render(<TimeTracking />);

    await waitFor(() => {
      expect(screen.getByTitle("Excluir")).toBeInTheDocument();
    });

    await user.click(screen.getByTitle("Excluir"));

    expect(screen.getByText("Confirmar")).toBeInTheDocument();
    expect(screen.getByText("Cancelar")).toBeInTheDocument();
  });

  it("calls delete_time_entry on confirm", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [mockEntries[0]];
      return [];
    });

    const user = userEvent.setup();
    render(<TimeTracking />);

    await waitFor(() => {
      expect(screen.getByTitle("Excluir")).toBeInTheDocument();
    });

    await user.click(screen.getByTitle("Excluir"));
    await user.click(screen.getByText("Confirmar"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("delete_time_entry", { id: 1 });
    });
  });

  it("removes entry locally when delete fails", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [mockEntries[0]];
      if (cmd === "delete_time_entry") throw new Error("Offline");
      return [];
    });

    const user = userEvent.setup();
    render(<TimeTracking />);

    await waitFor(() => {
      expect(screen.getByTitle("Excluir")).toBeInTheDocument();
    });

    await user.click(screen.getByTitle("Excluir"));
    await user.click(screen.getByText("Confirmar"));

    await waitFor(() => {
      expect(screen.getByText("Nenhum registro de horas encontrado.")).toBeInTheDocument();
    });
  });

  // ==================== Timer ====================

  it("calls start_time_entry when timer is started", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [];
      if (cmd === "get_projects") return mockProjects;
      if (cmd === "get_hours_summary") return [0, 0];
      if (cmd === "get_active_time_entry") return null;
      if (cmd === "start_time_entry") {
        return { id: 1, project_id: 1, project_name: "Projeto Alpha", description: "Working",
          start_time: new Date().toISOString(), end_time: null, duration_minutes: null,
          billable: true, hourly_rate: null, created_at: new Date().toISOString(),
          user_id: null, user_name: null };
      }
      return [];
    });

    const user = userEvent.setup();
    render(<TimeTracking />);

    // Wait for projects to load in the select options
    await waitFor(() => {
      const opts = screen.getAllByText("Projeto Alpha");
      expect(opts.length).toBeGreaterThan(0);
    });

    const timerOption = screen.getByText("Selecione um projeto...");
    const timerSelect = timerOption.closest("select")!;
    await user.selectOptions(timerSelect, "1");

    await user.click(screen.getByText("▶️ Iniciar"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("start_time_entry", {
        projectId: 1, userId: null, description: null,
      });
    });
  });

  it("shows running timer UI when active entry exists", async () => {
    const activeStart = new Date(Date.now() - 3600000).toISOString();
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [];
      if (cmd === "get_projects") return mockProjects;
      if (cmd === "get_hours_summary") return [0, 0];
      if (cmd === "get_active_time_entry") {
        return { id: 5, project_id: 1, project_name: "Projeto Alpha", description: "Running",
          start_time: activeStart, end_time: null, duration_minutes: null,
          billable: true, hourly_rate: null, created_at: activeStart,
          user_id: null, user_name: null };
      }
      return [];
    });

    render(<TimeTracking />);

    await waitFor(() => {
      expect(screen.getByText("⏱️ Timer em andamento")).toBeInTheDocument();
    });
    expect(screen.getByText("⏹️ Parar Timer")).toBeInTheDocument();
  });

  it("calls stop_time_entry when timer stops", async () => {
    const activeStart = new Date(Date.now() - 600000).toISOString();
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [];
      if (cmd === "get_projects") return mockProjects;
      if (cmd === "get_hours_summary") return [0, 0];
      if (cmd === "get_active_time_entry") {
        return { id: 5, project_id: 1, project_name: "Projeto Alpha", description: "Running",
          start_time: activeStart, end_time: null, duration_minutes: null,
          billable: true, hourly_rate: null, created_at: activeStart,
          user_id: null, user_name: null };
      }
      if (cmd === "stop_time_entry") return { id: 5, end_time: new Date().toISOString(), duration_minutes: 10 };
      return [];
    });

    const user = userEvent.setup();
    render(<TimeTracking />);

    await waitFor(() => {
      expect(screen.getByText("⏹️ Parar Timer")).toBeInTheDocument();
    });

    await user.click(screen.getByText("⏹️ Parar Timer"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("stop_time_entry", { id: 5 });
    });
  });

  // ==================== Fallback Local Timer ====================

  it("starts timer locally when invoke fails", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [];
      if (cmd === "get_projects") return mockProjects;
      if (cmd === "get_hours_summary") return [0, 0];
      if (cmd === "get_active_time_entry") return null;
      if (cmd === "start_time_entry") throw new Error("Offline");
      return [];
    });

    const user = userEvent.setup();
    render(<TimeTracking />);

    await waitFor(() => {
      const opts = screen.getAllByText("Projeto Alpha");
      expect(opts.length).toBeGreaterThan(0);
    });

    const timerOption = screen.getByText("Selecione um projeto...");
    const timerSelect = timerOption.closest("select")!;
    await user.selectOptions(timerSelect, "1");

    await user.click(screen.getByText("▶️ Iniciar"));

    await waitFor(() => {
      expect(screen.getByText("⏱️ Timer em andamento")).toBeInTheDocument();
    });
  });

  it("stops timer locally when invoke fails", async () => {
    const activeStart = new Date(Date.now() - 600000).toISOString();
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [];
      if (cmd === "get_projects") return mockProjects;
      if (cmd === "get_hours_summary") return [0, 0];
      if (cmd === "get_active_time_entry") {
        return { id: 5, project_id: 1, project_name: "Projeto Alpha", description: "Running",
          start_time: activeStart, end_time: null, duration_minutes: null,
          billable: true, hourly_rate: null, created_at: activeStart,
          user_id: null, user_name: null };
      }
      if (cmd === "stop_time_entry") throw new Error("Offline");
      return [];
    });

    const user = userEvent.setup();
    render(<TimeTracking />);

    await waitFor(() => {
      expect(screen.getByText("⏹️ Parar Timer")).toBeInTheDocument();
    });

    await user.click(screen.getByText("⏹️ Parar Timer"));

    await waitFor(() => {
      expect(screen.queryByText("⏱️ Timer em andamento")).not.toBeInTheDocument();
    });
  });

  // ==================== Modal Manual ====================

  it("opens manual entry modal", async () => {
    const user = userEvent.setup();
    render(<TimeTracking />);

    // Button appears in header AND empty state
    await user.click(screen.getAllByText("➕ Registrar Horas")[0]);

    expect(screen.getByText("Salvar")).toBeInTheDocument();
    expect(screen.getByText("Cancelar")).toBeInTheDocument();
  });

  it("shows form fields in manual modal", async () => {
    const user = userEvent.setup();
    render(<TimeTracking />);

    await user.click(screen.getAllByText("➕ Registrar Horas")[0]);

    expect(screen.getByText("Descrição")).toBeInTheDocument();
    expect(screen.getByText("Data")).toBeInTheDocument();
    expect(screen.getByText("Duração (minutos) *")).toBeInTheDocument();
    expect(screen.getByText("Valor Hora (R$)")).toBeInTheDocument();
    expect(screen.getByText("Faturável")).toBeInTheDocument();
  });

  it("submits manual entry", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_time_entries") return [];
      if (cmd === "get_projects") return mockProjects;
      if (cmd === "get_hours_summary") return [0, 0];
      if (cmd === "get_active_time_entry") return null;
      return [];
    });

    const user = userEvent.setup();
    render(<TimeTracking />);

    await user.click(screen.getAllByText("➕ Registrar Horas")[0]);

    // Find the modal's project select (the one with "Selecione...")
    const modalOption = screen.getAllByText("Selecione...")[0];
    const modalSelect = modalOption.closest("select")!;
    await user.selectOptions(modalSelect, "1");

    await user.click(screen.getByText("Salvar"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("add_manual_time_entry", expect.objectContaining({
        projectId: 1, durationMinutes: 60, billable: true,
      }));
    });
  });

  // ==================== Filtros ====================

  it("shows filter section", () => {
    render(<TimeTracking />);
    expect(screen.getByText("🔍 Filtrar")).toBeInTheDocument();
  });

  it("calls get_time_entries with filter when Filtrar is clicked", async () => {
    const user = userEvent.setup();
    render(<TimeTracking />);

    // Wait for projects to load in the select options
    await waitFor(() => {
      const opts = screen.getAllByText("Projeto Alpha");
      expect(opts.length).toBeGreaterThan(0);
    });

    // Find filter project select by "Todos" option text
    const todosOption = screen.getByText("Todos");
    const filterSelect = todosOption.closest("select")!;
    await user.selectOptions(filterSelect, "1");

    await user.click(screen.getByText("🔍 Filtrar"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("get_time_entries", expect.objectContaining({
        projectId: 1,
      }));
    });
  });
});
