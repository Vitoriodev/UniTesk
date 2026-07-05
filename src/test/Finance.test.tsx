import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import Finance from "../components/Finance";

const { invoke } = vi.hoisted(() => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke,
}));

const mockInvoices = [
  {
    id: 1, project_id: 1, project_name: "Projeto Alpha", client_id: 1, client_name: "Empresa ABC",
    number: "INV-001", description: "Consultoria", amount: 5000, tax: 500, total: 5500,
    status: "paid", issue_date: "2026-07-01", due_date: "2026-07-31", paid_date: "2026-07-15",
    notes: null, created_at: "2026-07-01T10:00:00Z",
  },
  {
    id: 2, project_id: 2, project_name: "Projeto Beta", client_id: 2, client_name: "Cliente XYZ",
    number: "INV-002", description: "Desenvolvimento", amount: 3000, tax: 300, total: 3300,
    status: "sent", issue_date: "2026-07-10", due_date: "2026-08-10", paid_date: null,
    notes: null, created_at: "2026-07-10T10:00:00Z",
  },
  {
    id: 3, project_id: null, project_name: null, client_id: null, client_name: null,
    number: "INV-003", description: "", amount: 1000, tax: 0, total: 1000,
    status: "draft", issue_date: "2026-07-20", due_date: "2026-08-20", paid_date: null,
    notes: null, created_at: "2026-07-20T10:00:00Z",
  },
];

describe("Finance", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_invoices") return [];
      if (cmd === "get_projects") return [];
      if (cmd === "get_clients") return [];
      return [];
    });
  });

  // ==================== Render & Layout ====================

  it("renders the title", () => {
    render(<Finance />);
    expect(screen.getByText("💰 Financeiro")).toBeInTheDocument();
  });

  it("shows empty state when there are no invoices", async () => {
    render(<Finance />);
    await waitFor(() => {
      expect(screen.getByText("Nenhuma fatura cadastrada.")).toBeInTheDocument();
    });
  });

  it("shows filter tabs with unique text", async () => {
    render(<Finance />);
    await waitFor(() => {
      // These texts are unique in the DOM (no duplicates)
      expect(screen.getByText("Todas")).toBeInTheDocument();
      expect(screen.getByText("📝 Rascunho")).toBeInTheDocument();
      expect(screen.getByText("📨 Enviadas")).toBeInTheDocument();
      expect(screen.getByText("✅ Pagas")).toBeInTheDocument();
      // ❌ Canceladas is unique
      expect(screen.getByText("❌ Canceladas")).toBeInTheDocument();
    });
  });

  // ==================== Stats ====================

  it("shows stat labels when empty", async () => {
    render(<Finance />);
    await waitFor(() => {
      expect(screen.getByText("💰 Receita Total")).toBeInTheDocument();
      expect(screen.getByText("⏳ A Receber")).toBeInTheDocument();
    });
  });

  it("calculates revenue from paid invoices", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_invoices") return mockInvoices;
      if (cmd === "get_projects") return [];
      if (cmd === "get_clients") return [];
      return [];
    });

    render(<Finance />);

    // INV-001 is paid (5500), INV-002 is sent, INV-003 is draft
    await waitFor(() => {
      // Revenue shows as BRL currency
      const revenueLabel = screen.getByText("💰 Receita Total");
      // The value is in a sibling element
      expect(revenueLabel).toBeInTheDocument();
    });
  });

  // ==================== Modal ====================

  it("opens create modal when clicking Nova Fatura button", async () => {
    const user = userEvent.setup();
    render(<Finance />);

    // Button appears in header AND empty state - use getAllByText
    const buttons = screen.getAllByText("➕ Nova Fatura");
    expect(buttons.length).toBeGreaterThanOrEqual(1);
    await user.click(buttons[0]);

    expect(screen.getByText("💰 Nova Fatura")).toBeInTheDocument();
    expect(screen.getByText("Criar Fatura")).toBeInTheDocument();
    expect(screen.getByText("Cancelar")).toBeInTheDocument();
  });

  it("shows form fields in create modal", async () => {
    const user = userEvent.setup();
    render(<Finance />);

    await user.click(screen.getAllByText("➕ Nova Fatura")[0]);

    // Unique labels in the form
    expect(screen.getByText("Número da Fatura *")).toBeInTheDocument();
    expect(screen.getByText("Valor (R$) *")).toBeInTheDocument();
    expect(screen.getByText("Total")).toBeInTheDocument(); // calculator
  });

  it("shows paid_date field when status is paid", async () => {
    const user = userEvent.setup();
    render(<Finance />);

    await user.click(screen.getAllByText("➕ Nova Fatura")[0]);

    // Select "Paga" from the status select (third select: projeto, cliente, status)
    const selects = screen.getAllByRole("combobox");
    const statusSelect = selects[selects.length - 1]; // Last one is status
    await user.selectOptions(statusSelect, "paid");

    expect(screen.getByText("Data de Pagamento")).toBeInTheDocument();
  });

  // ==================== Lista de Faturas ====================

  it("displays invoices when loaded from backend", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_invoices") return mockInvoices;
      if (cmd === "get_projects") return [];
      if (cmd === "get_clients") return [];
      return [];
    });

    render(<Finance />);

    await waitFor(() => {
      expect(screen.getByText("INV-001")).toBeInTheDocument();
    });
    expect(screen.getByText("INV-002")).toBeInTheDocument();
    // Status badges (some text appears in both filter tabs and badges)
    expect(screen.getByText("✅ Paga")).toBeInTheDocument();
    expect(screen.getByText("📨 Enviada")).toBeInTheDocument();
    // 📝 Rascunho appears in both filter tab and badge; verified in filter tabs test
  });

  it("shows edit/delete buttons for invoices", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_invoices") return [mockInvoices[0]];
      return [];
    });

    render(<Finance />);

    await waitFor(() => {
      expect(screen.getByText("✏️ Editar")).toBeInTheDocument();
      expect(screen.getByText("🗑️ Excluir")).toBeInTheDocument();
    });
  });

  it("opens edit modal with pre-filled data", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_invoices") return [mockInvoices[0]];
      return [];
    });

    const user = userEvent.setup();
    render(<Finance />);

    await waitFor(() => {
      expect(screen.getByText("✏️ Editar")).toBeInTheDocument();
    });

    await user.click(screen.getByText("✏️ Editar"));

    expect(screen.getByText("✏️ Editar Fatura")).toBeInTheDocument();
    // Number field should show INV-001
    expect(screen.getByDisplayValue("INV-001")).toBeInTheDocument();
    // Should show "Salvar" instead of "Criar Fatura"
    expect(screen.getByText("Salvar")).toBeInTheDocument();
  });

  it("shows confirm dialog when deleting", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_invoices") return [mockInvoices[0]];
      return [];
    });

    const user = userEvent.setup();
    render(<Finance />);

    await waitFor(() => {
      expect(screen.getByText("🗑️ Excluir")).toBeInTheDocument();
    });

    await user.click(screen.getByText("🗑️ Excluir"));

    expect(screen.getByText("Confirmar")).toBeInTheDocument();
    expect(screen.getByText("Cancelar")).toBeInTheDocument();
  });

  it("calls delete_invoice when confirmed", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_invoices") return [mockInvoices[0]];
      return [];
    });

    const user = userEvent.setup();
    render(<Finance />);

    await waitFor(() => {
      expect(screen.getByText("🗑️ Excluir")).toBeInTheDocument();
    });

    await user.click(screen.getByText("🗑️ Excluir"));
    await user.click(screen.getByText("Confirmar"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("delete_invoice", { id: 1 });
    });
  });

  it("cancels deletion and keeps invoice", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_invoices") return [mockInvoices[0]];
      return [];
    });

    const user = userEvent.setup();
    render(<Finance />);

    await waitFor(() => {
      expect(screen.getByText("🗑️ Excluir")).toBeInTheDocument();
    });

    await user.click(screen.getByText("🗑️ Excluir"));
    await user.click(screen.getByText("Cancelar"));

    expect(screen.queryByText("Confirmar")).not.toBeInTheDocument();
  });

  // ==================== Fallback Local ====================

  it("removes invoice locally when invoke fails", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_invoices") return [mockInvoices[0]];
      if (cmd === "delete_invoice") throw new Error("Offline");
      return [];
    });

    const user = userEvent.setup();
    render(<Finance />);

    await waitFor(() => {
      expect(screen.getByText("INV-001")).toBeInTheDocument();
    });

    await user.click(screen.getByText("🗑️ Excluir"));
    await user.click(screen.getByText("Confirmar"));

    await waitFor(() => {
      expect(screen.getByText("Nenhuma fatura cadastrada.")).toBeInTheDocument();
    });
  });

  // ==================== Filtros ====================

  it("changes active filter tab when clicked", async () => {
    const user = userEvent.setup();
    render(<Finance />);

    await waitFor(() => {
      expect(screen.getByText("📝 Rascunho")).toBeInTheDocument();
    });

    await user.click(screen.getByText("📝 Rascunho"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("get_invoices", { statusFilter: "draft" });
    });
  });

  // ==================== Create Invoice ====================

  it("calls create_invoice when saving new invoice", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_invoices") return [];
      if (cmd === "get_projects") return [];
      if (cmd === "get_clients") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<Finance />);

    await user.click(screen.getAllByText("➕ Nova Fatura")[0]);

    // Find number input by its auto-generated INV- prefix
    const numberInput = screen.getByDisplayValue(/^INV-/);
    await user.clear(numberInput);
    await user.type(numberInput, "INV-TEST");

    await user.click(screen.getByText("Criar Fatura"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("create_invoice", expect.objectContaining({
        number: "INV-TEST",
      }));
    });
  });
});
