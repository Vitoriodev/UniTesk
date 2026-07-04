import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import CalendarView from "../components/CalendarView";

describe("CalendarView", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("renders the calendar title", () => {
    render(<CalendarView />);
    expect(screen.getByText("📅 Calendário de Atividades")).toBeInTheDocument();
  });

  it("shows 'Nenhuma atividade cadastrada' when there are no assignments", () => {
    render(<CalendarView />);
    expect(
      screen.getByText("Nenhuma atividade cadastrada.")
    ).toBeInTheDocument();
  });

  it("shows all day names (Dom, Seg, Ter, etc.)", () => {
    render(<CalendarView />);
    const dayNames = ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sáb"];
    dayNames.forEach((name) => {
      expect(screen.getByText(name)).toBeInTheDocument();
    });
  });

  it("shows the 'Nova Atividade' button", () => {
    render(<CalendarView />);
    expect(screen.getByText("➕ Nova Atividade")).toBeInTheDocument();
  });

  it("opens the new assignment modal when clicking 'Nova Atividade'", async () => {
    const user = userEvent.setup();
    render(<CalendarView />);
    await user.click(screen.getByText("➕ Nova Atividade"));
    expect(screen.getByText("📅 Nova Atividade")).toBeInTheDocument();
    expect(screen.getByText("Salvar")).toBeInTheDocument();
    expect(screen.getByText("Cancelar")).toBeInTheDocument();
  });

  it("shows the Atividades Próximas section", () => {
    render(<CalendarView />);
    expect(screen.getByText("Atividades Próximas")).toBeInTheDocument();
  });

  it("displays assignments from localStorage fallback on load", async () => {
    const mockAssignments = [
      {
        id: 1,
        title: "Test Assignment",
        description: "Test description",
        due_date: "2026-07-15",
        project_name: "Test Project",
        status: "pending",
      },
    ];
    localStorage.setItem(
      "unitesk_assignments",
      JSON.stringify(mockAssignments)
    );

    render(<CalendarView />);

    await waitFor(() => {
      expect(screen.getByText("Test Assignment")).toBeInTheDocument();
    });
    expect(screen.getByText(/Test Project/)).toBeInTheDocument();
  });

  it("displays overdue badge for overdue assignments", async () => {
    const mockAssignments = [
      {
        id: 1,
        title: "Overdue Task",
        description: "",
        due_date: "2025-01-01",
        project_name: "Old Project",
        status: "overdue",
      },
    ];
    localStorage.setItem(
      "unitesk_assignments",
      JSON.stringify(mockAssignments)
    );

    render(<CalendarView />);

    await waitFor(() => {
      expect(screen.getByText("🔴 Atrasado")).toBeInTheDocument();
    });
  });

  it("displays done badge for completed assignments", async () => {
    const mockAssignments = [
      {
        id: 1,
        title: "Done Task",
        description: "",
        due_date: "2026-06-01",
        project_name: "Done Project",
        status: "done",
      },
    ];
    localStorage.setItem(
      "unitesk_assignments",
      JSON.stringify(mockAssignments)
    );

    render(<CalendarView />);

    await waitFor(() => {
      expect(screen.getByText("✅ Concluído")).toBeInTheDocument();
    });
  });

  it("changes month when clicking next month button", async () => {
    const user = userEvent.setup();
    render(<CalendarView />);
    const now = new Date();

    const nextMonth = now.getMonth() === 11 ? 0 : now.getMonth() + 1;

    const buttons = screen.getAllByRole("button");
    const nextButton = buttons.find((btn) => btn.textContent === "→")!;
    await user.click(nextButton);

    // O mês é exibido em um elemento <select> - verifica se o valor mudou
    const selects = screen.getAllByRole("combobox");
    expect(selects.length).toBeGreaterThanOrEqual(1);
    // O primeiro select (mês) deve mostrar o valor do próximo mês
    expect(selects[0]).toHaveValue(String(nextMonth));
  });

  it("can create an assignment via the modal (localStorage fallback)", async () => {
    const user = userEvent.setup();
    render(<CalendarView />);

    // Open modal
    await user.click(screen.getByText("➕ Nova Atividade"));

    // Fill form using userEvent.type (properly handles state updates)
    await user.type(
      screen.getByPlaceholderText("Ex: Entrega do artigo de Redes"),
      "New Test Task"
    );
    await user.type(
      screen.getByPlaceholderText("Ex: Redes de Computadores"),
      "Test Project"
    );
    await user.type(
      screen.getByPlaceholderText("Descreva a atividade..."),
      "Test description"
    );

    // Save
    await user.click(screen.getByText("Salvar"));

    // The assignment should appear in the list
    expect(screen.getByText("New Test Task")).toBeInTheDocument();
    expect(screen.getByText(/Test Project/)).toBeInTheDocument();
  });

  it("shows delete button (🗑️) for assignments", async () => {
    const mockAssignments = [
      {
        id: 1,
        title: "Deletable Task",
        description: "",
        due_date: "2026-07-15",
        project_name: "Test",
        status: "pending",
      },
    ];
    localStorage.setItem("unitesk_assignments", JSON.stringify(mockAssignments));

    render(<CalendarView />);

    await waitFor(() => {
      expect(screen.getByText("Deletable Task")).toBeInTheDocument();
    });

    // Delete button should be present
    const deleteButton = screen.getByTitle("Excluir atividade");
    expect(deleteButton).toBeInTheDocument();
  });

  it("deletes assignment via localStorage fallback when confirm is accepted", async () => {
    // Mock window.confirm to return true
    const originalConfirm = window.confirm;
    window.confirm = vi.fn().mockReturnValue(true);

    const mockAssignments = [
      {
        id: 1,
        title: "To Be Deleted",
        description: "",
        due_date: "2026-07-15",
        project_name: "Test",
        status: "pending",
      },
    ];
    localStorage.setItem("unitesk_assignments", JSON.stringify(mockAssignments));

    const user = userEvent.setup();
    render(<CalendarView />);

    await waitFor(() => {
      expect(screen.getByText("To Be Deleted")).toBeInTheDocument();
    });

    // Click delete button
    await user.click(screen.getByTitle("Excluir atividade"));

    // Assignment should be removed
    await waitFor(() => {
      expect(screen.queryByText("To Be Deleted")).not.toBeInTheDocument();
    });

    // localStorage should be updated (empty array)
    const saved = JSON.parse(localStorage.getItem("unitesk_assignments") || "[]");
    expect(saved).toHaveLength(0);

    // Restore original confirm
    window.confirm = originalConfirm;
  });

  it("does not delete assignment when confirm is dismissed", async () => {
    // Mock window.confirm to return false (cancel)
    const originalConfirm = window.confirm;
    window.confirm = vi.fn().mockReturnValue(false);

    const mockAssignments = [
      {
        id: 1,
        title: "Keep Me",
        description: "",
        due_date: "2026-07-15",
        project_name: "Test",
        status: "pending",
      },
    ];
    localStorage.setItem("unitesk_assignments", JSON.stringify(mockAssignments));

    const user = userEvent.setup();
    render(<CalendarView />);

    await waitFor(() => {
      expect(screen.getByText("Keep Me")).toBeInTheDocument();
    });

    // Click delete button
    await user.click(screen.getByTitle("Excluir atividade"));

    // Assignment should still be present
    expect(screen.getByText("Keep Me")).toBeInTheDocument();

    // localStorage should still have the assignment
    const saved = JSON.parse(localStorage.getItem("unitesk_assignments") || "[]");
    expect(saved).toHaveLength(1);

    // Restore original confirm
    window.confirm = originalConfirm;
  });
});
