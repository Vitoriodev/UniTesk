# 📡 API de Comandos Tauri — Unitesk v2.0

## 📁 Projetos

### `get_projects`
Retorna todos os projetos (com nome do cliente, se houver).
```typescript
const projects = await invoke("get_projects");
// Retorno: Project[] com client_id e client_name
```

### `create_project`
Cria um novo projeto.
```typescript
await invoke("create_project", {
  name: "Sistema de Gestão",
  description: "ERP para empresa ABC",
  clientId: 1  // opcional
});
```

### `update_project`
Edita um projeto existente.
```typescript
await invoke("update_project", {
  id: 1,
  name: "Novo Nome",
  description: "Nova descrição",
  clientId: 1  // opcional
});
```

## 🤝 Clientes

### `get_clients`
Retorna todos os clientes.
```typescript
const clients = await invoke("get_clients");
// Retorno: Client[]
```

### `create_client`
Cria um novo cliente.
```typescript
await invoke("create_client", {
  name: "Empresa ABC",
  email: "contato@abc.com",
  phone: "11999999999",
  company: "ABC Ltda",
  notes: "Cliente premium"
});
```

### `update_client`
Edita um cliente existente.
```typescript
await invoke("update_client", {
  id: 1,
  name: "Empresa XYZ",
  email: "contato@xyz.com",
  phone: null,
  company: null,
  notes: null
});
```

### `delete_client`
Remove um cliente (projetos vinculados ficam sem cliente, não são deletados).
```typescript
await invoke("delete_client", { id: 1 });
```

## 👥 Equipes

### `get_teams`
Retorna todas as equipes.
```typescript
const teams = await invoke("get_teams");
// Retorno: Team[]
```

### `create_team`
Cria uma nova equipe.
```typescript
await invoke("create_team", {
  name: "Equipe Dev",
  description: "Time de desenvolvimento"
});
```

### `update_team`
Edita uma equipe.
```typescript
await invoke("update_team", {
  id: 1,
  name: "Equipe Dev++",
  description: "Time atualizado"
});
```

### `delete_team`
Remove equipe (membros são removidos em cascata).
```typescript
await invoke("delete_team", { id: 1 });
```

## 👤 Membros da Equipe

### `get_team_members`
Lista membros de uma equipe.
```typescript
const members = await invoke("get_team_members", { teamId: 1 });
// Retorno: TeamMember[] com user_name e user_email
```

### `add_team_member`
Adiciona um usuário a uma equipe.
```typescript
await invoke("add_team_member", {
  teamId: 1,
  userId: 2,
  role: "leader"  // ou "member"
});
```

### `update_team_member_role`
Altera o cargo de um membro.
```typescript
await invoke("update_team_member_role", {
  id: 1,
  role: "leader"
});
```

### `remove_team_member`
Remove um membro da equipe.
```typescript
await invoke("remove_team_member", { id: 1 });
```

## 👤 Usuários

### `get_users`
Retorna todos os usuários.
```typescript
const users = await invoke("get_users");
// Retorno: User[] (name, email, role)
```

### `create_user`
Cria um novo usuário (email único).
```typescript
await invoke("create_user", {
  name: "João Silva",
  email: "joao@email.com",
  role: "member"  // admin | manager | member
});
```

### `delete_user`
Remove um usuário (também o remove de equipes).
```typescript
await invoke("delete_user", { id: 1 });
```

## 📅 Atividades (com Prioridade)

### `get_assignments`
Retorna todas as atividades (incluindo prioridade).
```typescript
const assignments = await invoke("get_assignments");
// Retorno: Assignment[] com priority
```

### `create_assignment`
Cria uma nova atividade com prioridade.
```typescript
await invoke("create_assignment", {
  title: "Entregar relatório",
  description: "Relatório mensal",
  dueDate: "2026-07-15",
  dueTime: "14:00",
  projectName: "Projeto X",
  priority: "high"  // low | medium | high | urgent
});
```

## 💰 Faturas / Financeiro

### `get_invoices`
Retorna todas as faturas com filtro opcional de status.
```typescript
const invoices = await invoke("get_invoices");
// Todas as faturas

const draft = await invoke("get_invoices", { statusFilter: "draft" });
// Faturas em rascunho
// statusFilter: undefined | "draft" | "sent" | "paid" | "overdue" | "cancelled"
// Retorno: Invoice[]
```

### `get_invoice`
Retorna uma fatura pelo ID.
```typescript
const invoice = await invoke("get_invoice", { id: 1 });
// Retorno: Invoice
```

### `create_invoice`
Cria uma nova fatura.
```typescript
await invoke("create_invoice", {
  projectId: 1,            // opcional
  clientId: 1,              // opcional
  number: "INV-2026-001",   // único
  description: "Consultoria mensal",
  amount: 5000.00,
  tax: 500.00,
  total: 5500.00,
  status: "draft",          // draft | sent | paid | overdue | cancelled
  issueDate: "2026-07-01",
  dueDate: "2026-07-31",    // opcional
  notes: "Pagamento via PIX"  // opcional
});
```

### `update_invoice`
Atualiza uma fatura existente.
```typescript
await invoke("update_invoice", {
  id: 1,
  projectId: 1,
  clientId: 1,
  number: "INV-2026-001",
  description: "Consultoria mensal",
  amount: 5000.00,
  tax: 500.00,
  total: 5500.00,
  status: "paid",
  issueDate: "2026-07-01",
  dueDate: "2026-07-31",
  paidDate: "2026-07-15",  // opcional, preenchido ao marcar como paga
  notes: null
});
```

### `delete_invoice`
Remove uma fatura.
```typescript
await invoke("delete_invoice", { id: 1 });
```

## ⏱️ Controle de Horas

### `get_time_entries`
Retorna registros de horas com filtros opcionais.
```typescript
const entries = await invoke("get_time_entries", {
  projectId: 1,        // opcional
  dateFrom: "2026-07-01", // opcional
  dateTo: "2026-07-15"    // opcional
});
// Retorno: TimeEntry[]
```

### `start_time_entry`
Inicia o timer para um projeto.
```typescript
const entry = await invoke("start_time_entry", {
  projectId: 1,
  description: "Desenvolvendo feature X"
});
// Retorno: TimeEntry (end_time = null, duration_minutes = null)
```

### `stop_time_entry`
Para o timer ativo e calcula a duração automaticamente.
```typescript
const entry = await invoke("stop_time_entry", { id: 1 });
// Retorno: TimeEntry (end_time e duration_minutes preenchidos)
```

### `add_manual_time_entry`
Adiciona horas manualmente (sem timer).
```typescript
await invoke("add_manual_time_entry", {
  projectId: 1,
  description: "Reunião com cliente",
  durationMinutes: 90,
  entryDate: "2026-07-10",
  billable: true,
  hourlyRate: 150.00  // opcional
});
```

### `get_active_time_entry`
Verifica se há um timer ativo.
```typescript
const active = await invoke("get_active_time_entry");
// Retorno: TimeEntry | null
```

### `get_hours_summary`
Retorna resumo de horas (hoje e semana).
```typescript
const [todayMinutes, weekMinutes] = await invoke("get_hours_summary");
// Retorno: [number, number]
```

### `delete_time_entry`
Remove um registro de horas.
```typescript
await invoke("delete_time_entry", { id: 1 });
```

## 🔔 Notificações

### `get_notifications`
Retorna notificações com filtro opcional de não lidas.
```typescript
const notifications = await invoke("get_notifications", {
  unreadOnly: false,  // true para apenas não lidas
  limit: 20           // opcional (padrão 50)
});
// Retorno: Notification[]
```

### `get_unread_notifications_count`
Retorna a quantidade de notificações não lidas.
```typescript
const count = await invoke("get_unread_notifications_count");
// Retorno: number
```

### `mark_notification_read`
Marca uma notificação como lida.
```typescript
await invoke("mark_notification_read", { id: 1 });
```

### `mark_all_notifications_read`
Marca todas as notificações como lidas.
```typescript
await invoke("mark_all_notifications_read");
```

### `delete_notification`
Remove uma notificação.
```typescript
await invoke("delete_notification", { id: 1 });
```

### `auto_generate_notifications`
Gera notificações automáticas para prazos de hoje, atividades atrasadas e faturas próximas do vencimento. Evita duplicatas por dia.
```typescript
const created = await invoke("auto_generate_notifications");
// Retorno: Notification[] (apenas as novas notificações criadas)
```

### `check_today_assignments`
Verifica atividades com notificação agendada para o horário atual e dispara notificações nativas do sistema.
```typescript
const assignments = await invoke("check_today_assignments");
// Retorno: Assignment[] com notificação disparada
```

## 📊 Relatórios

### `get_report_stats`
Retorna dados agregados para os gráficos de relatórios.
```typescript
const stats = await invoke("get_report_stats");
// Retorno: ReportStats com dados de atividades, financeiro, horas e faturas
```

## 📊 Dashboard

### `get_dashboard_stats`
Retorna estatísticas (incluindo horas, financeiro e notificações).
```typescript
const stats = await invoke("get_dashboard_stats");
// Retorno: { totalProjects, totalArticles, totalClients, totalTeams, totalUsers,
//            hoursToday, hoursWeek, totalRevenue, pendingInvoices, pendingAmount,
//            unreadNotifications, ... }
```

---

## 📦 Modelos

```typescript
interface Invoice {
  id: number;
  project_id: number | null;
  project_name: string | null;
  client_id: number | null;
  client_name: string | null;
  number: string;
  description: string | null;
  amount: number;
  tax: number;
  total: number;
  status: "draft" | "sent" | "paid" | "overdue" | "cancelled";
  issue_date: string;
  due_date: string | null;
  paid_date: string | null;
  notes: string | null;
  created_at: string;
}
```

```typescript
interface TimeEntry {
  id: number;
  project_id: number;
  project_name: string | null;
  user_id: number | null;
  user_name: string | null;
  description: string | null;
  start_time: string;
  end_time: string | null;
  duration_minutes: number | null;
  billable: boolean;
  hourly_rate: number | null;
  created_at: string;
}
```

```typescript
interface Project {
  id: number;
  name: string;
  description: string | null;
  client_id: number | null;
  client_name: string | null;
  created_at: string;
}

interface Client {
  id: number;
  name: string;
  email: string | null;
  phone: string | null;
  company: string | null;
  notes: string | null;
  created_at: string;
}

interface User {
  id: number;
  name: string;
  email: string;
  role: string;  // admin | manager | member
  created_at: string;
}

interface Team {
  id: number;
  name: string;
  description: string | null;
  created_at: string;
}

interface TeamMember {
  id: number;
  team_id: number;
  user_id: number;
  user_name: string | null;
  user_email: string | null;
  role: string;  // leader | member
  created_at: string;
}

interface Assignment {
  id: number;
  title: string;
  description: string | null;
  due_date: string;
  due_time: string | null;
  notification_time: string | null;
  project_name: string | null;
  status: "pending" | "done" | "overdue";
  priority: "low" | "medium" | "high" | "urgent";
  created_at: string;
}

interface Notification {
  id: number;
  type: "assignment_due" | "assignment_overdue" | "invoice_due" | string;
  title: string;
  message: string;
  is_read: boolean;
  created_at: string;
}

interface ReportStats {
  totalProjects: number;
  totalArticles: number;
  totalClients: number;
  totalTeams: number;
  totalUsers: number;
  assignmentsByMonth: { month: string; count: number }[];
  assignmentsPending: number;
  assignmentsDone: number;
  assignmentsOverdue: number;
  revenueByMonth: { month: string; amount: number }[];
  totalRevenue: number;
  pendingAmount: number;
  hoursByProject: { project_name: string; hours: number }[];
  totalHours: number;
  invoicesDraft: number;
  invoicesSent: number;
  invoicesPaid: number;
  invoicesOverdue: number;
  invoicesCancelled: number;
}

interface DashboardStats {
  totalProjects: number;
  totalArticles: number;
  totalClients: number;
  totalTeams: number;
  totalUsers: number;
  pendingAssignments: number;
  overdueAssignments: number;
  nextDeadline: string | null;
  nextDeadlineName: string | null;
  hoursToday: number;
  hoursWeek: number;
  totalRevenue: number;
  pendingInvoices: number;
  pendingAmount: number;
  unreadNotifications: number;
}
```
