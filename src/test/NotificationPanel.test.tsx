import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import NotificationPanel from "../components/NotificationPanel";

const { invoke } = vi.hoisted(() => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke,
}));

const mockNotifications = [
  {
    id: 1,
    type: "assignment_due",
    title: "📅 Prazo Hoje!",
    message: "A atividade 'Entrega TCC' vence hoje!",
    is_read: false,
    created_at: new Date(Date.now() - 300000).toISOString(),
  },
  {
    id: 2,
    type: "assignment_overdue",
    title: "🔴 Atividade Atrasada!",
    message: "A atividade 'Relatório' do projeto 'Projeto X' está atrasada!",
    is_read: false,
    created_at: new Date(Date.now() - 7200000).toISOString(),
  },
  {
    id: 3,
    type: "invoice_due",
    title: "💰 Fatura Próxima do Vencimento!",
    message: "A fatura INV-001 de R$ 1000.00 vence em breve!",
    is_read: true,
    created_at: new Date(Date.now() - 86400000).toISOString(),
  },
  {
    id: 4,
    type: "unknown_type",
    title: "Notificação Genérica",
    message: "Mensagem de teste.",
    is_read: false,
    created_at: new Date(Date.now() - 5000).toISOString(),
  },
];

describe("NotificationPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [];
      if (cmd === "get_unread_notifications_count") return 0;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });
  });

  // ==================== Render & Layout ====================

  it("renders the bell button", () => {
    render(<NotificationPanel />);
    expect(screen.getByTitle("Notificações")).toBeInTheDocument();
  });

  it("shows bell icon inside the button", () => {
    render(<NotificationPanel />);
    const bellBtn = screen.getByTitle("Notificações");
    expect(bellBtn.textContent).toContain("🔔");
  });

  it("does not show badge when unread count is 0", async () => {
    render(<NotificationPanel />);
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("get_unread_notifications_count");
    });
    // Badge is not rendered when unread count is 0
    expect(screen.queryByText("0")).not.toBeInTheDocument();
  });

  it("shows badge when unread count > 0", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [];
      if (cmd === "get_unread_notifications_count") return 3;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    render(<NotificationPanel />);
    await waitFor(() => {
      const badge = screen.getByText("3");
      expect(badge.className).toContain("notification-badge");
    });
  });

  it("shows 99+ badge when unread count > 99", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [];
      if (cmd === "get_unread_notifications_count") return 150;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    render(<NotificationPanel />);
    await waitFor(() => {
      expect(screen.getByText("99+")).toBeInTheDocument();
    });
  });

  // ==================== Panel ====================

  it("opens panel when bell is clicked", async () => {
    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    expect(screen.getByText("🔔 Notificações")).toBeInTheDocument();
  });

  it("closes panel when bell is clicked again", async () => {
    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));
    expect(screen.getByText("🔔 Notificações")).toBeInTheDocument();

    await user.click(screen.getByTitle("Notificações"));
    expect(screen.queryByText("🔔 Notificações")).not.toBeInTheDocument();
  });

  it("shows empty state when there are no notifications", async () => {
    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    expect(screen.getByText("Nenhuma notificação.")).toBeInTheDocument();
  });

  it("does not show mark all read button when there are no unread", async () => {
    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    expect(screen.queryByText("✅ Marcar todas como lidas")).not.toBeInTheDocument();
  });

  it("shows mark all read button when there are unread notifications", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[0]];
      if (cmd === "get_unread_notifications_count") return 1;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    // Open panel first to show notifications
    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      expect(screen.getByText("📅 Prazo Hoje!")).toBeInTheDocument();
    });

    expect(screen.getByText("✅ Marcar todas como lidas")).toBeInTheDocument();
  });

  // ==================== Notifications List ====================

  it("displays notifications when loaded from backend", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return mockNotifications;
      if (cmd === "get_unread_notifications_count") return 3;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      expect(screen.getByText("📅 Prazo Hoje!")).toBeInTheDocument();
    });
    expect(screen.getByText("🔴 Atividade Atrasada!")).toBeInTheDocument();
    expect(screen.getByText("💰 Fatura Próxima do Vencimento!")).toBeInTheDocument();
    expect(screen.getByText("Notificação Genérica")).toBeInTheDocument();
  });

  it("shows correct icons for different notification types", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[0], mockNotifications[1], mockNotifications[2], mockNotifications[3]];
      if (cmd === "get_unread_notifications_count") return 3;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    // Each notification has an icon div with the emoji
    const icons = screen.getAllByText("📅");
    expect(icons.length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("🔴").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("💰").length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByText("🔔").length).toBeGreaterThanOrEqual(1);
  });

  it("shows ✅ read button only for unread notifications", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return mockNotifications;
      if (cmd === "get_unread_notifications_count") return 3;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    // There should be 3 ✅ buttons (one for each unread: ids 1, 2, 4)
    await waitFor(() => {
      const markReadButtons = screen.getAllByTitle("Marcar como lida");
      expect(markReadButtons.length).toBe(3);
    });
  });

  it("shows 🗑️ delete button for all notifications", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[0], mockNotifications[2]];
      if (cmd === "get_unread_notifications_count") return 1;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      const deleteButtons = screen.getAllByTitle("Excluir");
      expect(deleteButtons.length).toBe(2);
    });
  });

  it("displays unread notification with special styling", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[0]];
      if (cmd === "get_unread_notifications_count") return 1;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      const items = document.querySelectorAll(".notification-item--unread");
      expect(items.length).toBe(1);
    });
  });

  // ==================== Time Ago ====================

  it("shows 'agora' for very recent notifications", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[3]]; // 5 seconds ago
      if (cmd === "get_unread_notifications_count") return 1;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      expect(screen.getByText("agora")).toBeInTheDocument();
    });
  });

  it("shows minutes for recent notifications", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[0]]; // 5 min ago
      if (cmd === "get_unread_notifications_count") return 1;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      expect(screen.getByText("5min")).toBeInTheDocument();
    });
  });

  it("shows hours for older notifications", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[1]]; // 2h ago
      if (cmd === "get_unread_notifications_count") return 1;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      expect(screen.getByText("2h")).toBeInTheDocument();
    });
  });

  it("shows days for old notifications", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[2]]; // 1 day ago
      if (cmd === "get_unread_notifications_count") return 0;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      expect(screen.getByText("1d")).toBeInTheDocument();
    });
  });

  // ==================== Mark as Read ====================

  it("calls mark_notification_read when mark read button is clicked", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[0]];
      if (cmd === "get_unread_notifications_count") return 1;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      expect(screen.getByTitle("Marcar como lida")).toBeInTheDocument();
    });

    await user.click(screen.getByTitle("Marcar como lida"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("mark_notification_read", { id: 1 });
    });
  });

  it("calls mark_all_notifications_read when mark all is clicked", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[0]];
      if (cmd === "get_unread_notifications_count") return 1;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      expect(screen.getByText("✅ Marcar todas como lidas")).toBeInTheDocument();
    });

    await user.click(screen.getByText("✅ Marcar todas como lidas"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("mark_all_notifications_read");
    });
  });

  // ==================== Delete ====================

  it("calls delete_notification when delete button is clicked", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[0]];
      if (cmd === "get_unread_notifications_count") return 1;
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      expect(screen.getByTitle("Excluir")).toBeInTheDocument();
    });

    await user.click(screen.getByTitle("Excluir"));

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("delete_notification", { id: 1 });
    });
  });

  // ==================== Fallback Local ====================

  it("marks notification locally when invoke fails on mark read", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[0]];
      if (cmd === "get_unread_notifications_count") return 1;
      if (cmd === "mark_notification_read") throw new Error("Offline");
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      expect(screen.getByTitle("Marcar como lida")).toBeInTheDocument();
    });

    await user.click(screen.getByTitle("Marcar como lida"));

    await waitFor(() => {
      // Mark read button should disappear (notification marked as read locally)
      expect(screen.queryByTitle("Marcar como lida")).not.toBeInTheDocument();
    });
  });

  it("removes notification locally when invoke fails on delete", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[0]];
      if (cmd === "get_unread_notifications_count") return 1;
      if (cmd === "delete_notification") throw new Error("Offline");
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      expect(screen.getByText("📅 Prazo Hoje!")).toBeInTheDocument();
    });

    await user.click(screen.getByTitle("Excluir"));

    await waitFor(() => {
      expect(screen.queryByText("📅 Prazo Hoje!")).not.toBeInTheDocument();
    });
  });

  it("shows empty state after locally deleting last notification", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_notifications") return [mockNotifications[0]];
      if (cmd === "get_unread_notifications_count") return 1;
      if (cmd === "delete_notification") throw new Error("Offline");
      if (cmd === "auto_generate_notifications") return [];
      return [];
    });

    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));

    await waitFor(() => {
      expect(screen.getByText("📅 Prazo Hoje!")).toBeInTheDocument();
    });

    await user.click(screen.getByTitle("Excluir"));

    await waitFor(() => {
      expect(screen.getByText("Nenhuma notificação.")).toBeInTheDocument();
    });
  });

  // ==================== Auto-generate ====================

  it("calls auto_generate_notifications on mount", async () => {
    render(<NotificationPanel />);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("auto_generate_notifications");
    });
  });

  // ==================== Click Outside ====================

  it("closes panel when clicking outside", async () => {
    const user = userEvent.setup();
    render(<NotificationPanel />);

    await user.click(screen.getByTitle("Notificações"));
    expect(screen.getByText("🔔 Notificações")).toBeInTheDocument();

    // Click outside the panel (on document body)
    await user.click(document.body);

    // Panel should close
    await waitFor(() => {
      expect(screen.queryByText("🔔 Notificações")).not.toBeInTheDocument();
    });
  });
});
