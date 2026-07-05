import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Notification {
  id: number;
  type: string;
  title: string;
  message: string;
  is_read: boolean;
  created_at: string;
}

function NotificationPanel() {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);
  const [isOpen, setIsOpen] = useState(false);
  const [cleaningUp, setCleaningUp] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadNotifications();
    loadUnreadCount();
    // Auto-generate notifications on mount
    invoke("auto_generate_notifications").catch(() => {});
  }, []);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    }
    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isOpen]);

  async function loadNotifications() {
    try {
      const data = await invoke<Notification[]>("get_notifications", {
        unreadOnly: false,
        limit: 20,
      });
      setNotifications(data);
    } catch {
      setNotifications([]);
    }
  }

  async function loadUnreadCount() {
    try {
      const count = await invoke<number>("get_unread_notifications_count");
      setUnreadCount(count);
    } catch {}
  }

  async function handleMarkRead(id: number) {
    try {
      await invoke("mark_notification_read", { id });
      loadNotifications();
      loadUnreadCount();
    } catch {
      setNotifications(
        notifications.map((n) =>
          n.id === id ? { ...n, is_read: true } : n
        )
      );
      setUnreadCount(Math.max(0, unreadCount - 1));
    }
  }

  async function handleMarkAllRead() {
    try {
      await invoke("mark_all_notifications_read");
      loadNotifications();
      setUnreadCount(0);
    } catch {
      setNotifications(notifications.map((n) => ({ ...n, is_read: true })));
      setUnreadCount(0);
    }
  }

  async function handleDelete(id: number) {
    try {
      await invoke("delete_notification", { id });
      const wasUnread = notifications.find((n) => n.id === id)?.is_read === false;
      setNotifications(notifications.filter((n) => n.id !== id));
      if (wasUnread) setUnreadCount(Math.max(0, unreadCount - 1));
    } catch {
      setNotifications(notifications.filter((n) => n.id !== id));
    }
  }

  function getTypeIcon(type: string): string {
    switch (type) {
      case "assignment_due": return "📅";
      case "assignment_overdue": return "🔴";
      case "invoice_due": return "💰";
      default: return "🔔";
    }
  }

  async function handleCleanup() {
    setCleaningUp(true);
    try {
      await invoke("cleanup_old_notifications", { days: 30 });
      loadNotifications();
      loadUnreadCount();
    } catch {}
    setCleaningUp(false);
  }

  function formatTimeAgo(dateStr: string): string {
    const now = Date.now();
    const date = new Date(dateStr).getTime();
    const diff = Math.floor((now - date) / 1000);
    if (diff < 60) return "agora";
    if (diff < 3600) return `${Math.floor(diff / 60)}min`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
    return `${Math.floor(diff / 86400)}d`;
  }

  return (
    <div className="notification-container" ref={panelRef}>
      <button
        className="notification-bell"
        onClick={() => setIsOpen(!isOpen)}
        title="Notificações"
      >
        🔔
        {unreadCount > 0 && (
          <span className="notification-badge">
            {unreadCount > 99 ? "99+" : unreadCount}
          </span>
        )}
      </button>

      {isOpen && (
        <div className="notification-panel">
          <div className="notification-panel-header">
            <h3>🔔 Notificações</h3>
            {unreadCount > 0 && (
              <button
                className="btn btn-secondary btn-xs"
                onClick={handleMarkAllRead}
              >
                ✅ Marcar todas como lidas
              </button>
            )}
          </div>

          {notifications.length === 0 ? (
            <div className="notification-empty">
              <p>Nenhuma notificação.</p>
            </div>
          ) : (
            <div className="notification-list">
              {notifications.map((n) => (
                <div
                  key={n.id}
                  className={`notification-item ${!n.is_read ? "notification-item--unread" : ""}`}
                >
                  <div className="notification-item-icon">
                    {getTypeIcon(n.type)}
                  </div>
                  <div className="notification-item-content">
                    <p className="notification-item-title">{n.title}</p>
                    <p className="notification-item-message">{n.message}</p>
                    <div className="notification-item-meta">
                      <span>{formatTimeAgo(n.created_at)}</span>
                      {!n.is_read && <span className="notification-dot" />}
                    </div>
                  </div>
                  <div className="notification-item-actions">
                    {!n.is_read && (
                      <button
                        className="btn btn-secondary btn-xs"
                        onClick={() => handleMarkRead(n.id)}
                        title="Marcar como lida"
                      >
                        ✅
                      </button>
                    )}
                    <button
                      className="btn btn-danger btn-xs"
                      onClick={() => handleDelete(n.id)}
                      title="Excluir"
                    >
                      🗑️
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}

          {notifications.length > 0 && (
            <div className="notification-panel-footer">
              <button
                className="btn btn-secondary btn-xs"
                onClick={handleCleanup}
                disabled={cleaningUp}
                title="Excluir notificações com mais de 30 dias"
              >
                {cleaningUp ? "⏳" : "🗑️"} Limpar antigas
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default NotificationPanel;
