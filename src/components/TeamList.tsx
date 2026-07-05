import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Team {
  id: number;
  name: string;
  description: string | null;
  created_at: string;
}

interface User {
  id: number;
  name: string;
  email: string;
  role: string;
}

interface TeamMember {
  id: number;
  team_id: number;
  user_id: number;
  user_name: string | null;
  user_email: string | null;
  role: string;
  created_at: string;
}

function TeamList() {
  const [teams, setTeams] = useState<Team[]>([]);
  const [users, setUsers] = useState<User[]>([]);
  const [showTeamModal, setShowTeamModal] = useState(false);
  const [editingTeam, setEditingTeam] = useState<Team | null>(null);
  const [teamForm, setTeamForm] = useState({ name: "", description: "" });
  const [deleteTeamConfirm, setDeleteTeamConfirm] = useState<number | null>(null);
  const [expandedTeam, setExpandedTeam] = useState<number | null>(null);
  const [teamMembers, setTeamMembers] = useState<Record<number, TeamMember[]>>({});
  const [showAddMember, setShowAddMember] = useState(false);
  const [selectedTeamId, setSelectedTeamId] = useState<number | null>(null);
  const [memberRole, setMemberRole] = useState("member");

  useEffect(() => {
    loadTeams();
    loadUsers();
  }, []);

  async function loadTeams() {
    try {
      const data = await invoke<Team[]>("get_teams");
      setTeams(data);
    } catch {
      setTeams([]);
    }
  }

  async function loadUsers() {
    try {
      const data = await invoke<User[]>("get_users");
      setUsers(data);
    } catch {
      setUsers([]);
    }
  }

  async function loadTeamMembers(teamId: number) {
    try {
      const data = await invoke<TeamMember[]>("get_team_members", { teamId });
      setTeamMembers((prev) => ({ ...prev, [teamId]: data }));
    } catch {
      setTeamMembers((prev) => ({ ...prev, [teamId]: [] }));
    }
  }

  function toggleExpand(teamId: number) {
    if (expandedTeam === teamId) {
      setExpandedTeam(null);
    } else {
      setExpandedTeam(teamId);
      if (!teamMembers[teamId]) {
        loadTeamMembers(teamId);
      }
    }
  }

  function openCreateTeam() {
    setEditingTeam(null);
    setTeamForm({ name: "", description: "" });
    setShowTeamModal(true);
  }

  function openEditTeam(team: Team) {
    setEditingTeam(team);
    setTeamForm({ name: team.name, description: team.description || "" });
    setShowTeamModal(true);
  }

  async function saveTeam() {
    if (!teamForm.name.trim()) return;
    try {
      if (editingTeam) {
        await invoke("update_team", {
          id: editingTeam.id,
          name: teamForm.name,
          description: teamForm.description || null,
        });
      } else {
        await invoke("create_team", {
          name: teamForm.name,
          description: teamForm.description || null,
        });
      }
      setShowTeamModal(false);
      loadTeams();
    } catch (err) {
      alert("Erro ao salvar equipe: " + String(err));
    }
  }

  async function confirmDeleteTeam(id: number) {
    try {
      await invoke("delete_team", { id });
      setDeleteTeamConfirm(null);
      loadTeams();
    } catch {
      setTeams(teams.filter((t) => t.id !== id));
      setDeleteTeamConfirm(null);
    }
  }

  function openAddMember(teamId: number) {
    setSelectedTeamId(teamId);
    setMemberRole("member");
    setShowAddMember(true);
  }

  async function confirmAddMember() {
    if (selectedTeamId === null) return;
    const selectEl = document.getElementById("member-select") as HTMLSelectElement;
    if (!selectEl) return;
    const userId = parseInt(selectEl.value);
    if (!userId) return;

    try {
      await invoke("add_team_member", {
        teamId: selectedTeamId,
        userId,
        role: memberRole,
      });
      setShowAddMember(false);
      loadTeamMembers(selectedTeamId);
    } catch (err) {
      alert("Erro ao adicionar membro: " + String(err));
    }
  }

  async function removeMember(memberId: number, teamId: number) {
    try {
      await invoke("remove_team_member", { id: memberId });
      loadTeamMembers(teamId);
    } catch {
      setTeamMembers((prev) => ({
        ...prev,
        [teamId]: (prev[teamId] || []).filter((m) => m.id !== memberId),
      }));
    }
  }

  async function toggleMemberRole(member: TeamMember) {
    const newRole = member.role === "leader" ? "member" : "leader";
    try {
      await invoke("update_team_member_role", { id: member.id, role: newRole });
      loadTeamMembers(member.team_id);
    } catch {
      // fallback local
    }
  }

  const usersNotInTeam = (teamId: number) => {
    const memberIds = new Set((teamMembers[teamId] || []).map((m) => m.user_id));
    return users.filter((u) => !memberIds.has(u.id));
  };

  return (
    <div>
      <div className="flex-between" style={{ marginBottom: 20 }}>
        <h2 className="section-title">👥 Equipes</h2>
        <div className="flex gap-6">
          <button className="btn btn-secondary" onClick={() => loadUsers()}>
            👤 Ver Usuários
          </button>
          <button className="btn btn-primary" onClick={openCreateTeam}>
            ➕ Nova Equipe
          </button>
        </div>
      </div>

      {/* Seção de Usuários */}
      {users.length > 0 && (
        <div className="card" style={{ marginBottom: 20 }}>
          <div className="card-header">
            <h3 className="card-title">👤 Usuários Cadastrados</h3>
          </div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
            {users.map((user) => (
              <span
                key={user.id}
                className={`badge ${
                  user.role === "admin"
                    ? "badge-overdue"
                    : user.role === "manager"
                    ? "badge-progress"
                    : "badge-pending"
                }`}
              >
                {user.name}{" "}
                <span style={{ opacity: 0.7, fontSize: "0.7rem" }}>
                  ({user.role})
                </span>
              </span>
            ))}
          </div>
        </div>
      )}

      {teams.length === 0 ? (
        <div className="card text-center" style={{ padding: 48 }}>
          <p style={{ fontSize: "3rem", marginBottom: 12 }}>👥</p>
          <p className="text-secondary" style={{ fontSize: "1.1rem" }}>
            Nenhuma equipe ainda.
          </p>
          <p className="text-secondary" style={{ marginBottom: 16 }}>
            Crie equipes e adicione membros para organizar seus projetos!
          </p>
          <button className="btn btn-primary" onClick={openCreateTeam}>
            ➕ Nova Equipe
          </button>
        </div>
      ) : (
        <div className="grid-2">
          {teams.map((team) => {
            const members = teamMembers[team.id] || [];
            const isExpanded = expandedTeam === team.id;
            const availableUsers = usersNotInTeam(team.id);

            return (
              <div className="card" key={team.id}>
                <div className="flex-between" style={{ marginBottom: 12 }}>
                  <h3 className="project-name">{team.name}</h3>
                  <div className="flex gap-6">
                    <button
                      className="btn btn-secondary btn-sm"
                      onClick={() => openEditTeam(team)}
                      title="Editar"
                    >
                      ✏️
                    </button>
                    {deleteTeamConfirm === team.id ? (
                      <div className="flex gap-6">
                        <button
                          className="btn btn-danger btn-sm"
                          onClick={() => confirmDeleteTeam(team.id)}
                        >
                          Confirmar
                        </button>
                        <button
                          className="btn btn-secondary btn-sm"
                          onClick={() => setDeleteTeamConfirm(null)}
                        >
                          Cancelar
                        </button>
                      </div>
                    ) : (
                      <button
                        className="btn btn-danger btn-sm"
                        onClick={() => setDeleteTeamConfirm(team.id)}
                        title="Excluir"
                      >
                        🗑️
                      </button>
                    )}
                  </div>
                </div>

                {team.description && (
                  <p className="text-secondary text-sm" style={{ marginBottom: 12 }}>
                    {team.description}
                  </p>
                )}

                <div className="meta-info">
                  <span>👥 {members.length} membro(s)</span>
                  <span>•</span>
                  <span>
                    Criado em {new Date(team.created_at).toLocaleDateString("pt-BR")}
                  </span>
                </div>

                <div className="flex gap-6" style={{ marginTop: 12 }}>
                  <button
                    className="btn btn-secondary btn-sm"
                    onClick={() => toggleExpand(team.id)}
                  >
                    {isExpanded ? "👥 Ocultar Membros" : "👥 Ver Membros"}
                  </button>
                  {isExpanded && availableUsers.length > 0 && (
                    <button
                      className="btn btn-primary btn-sm"
                      onClick={() => openAddMember(team.id)}
                    >
                      ➕ Adicionar Membro
                    </button>
                  )}
                </div>

                {isExpanded && (
                  <div className="files-section">
                    <p className="files-section-title">👤 Membros da Equipe</p>
                    {members.length === 0 ? (
                      <p className="text-secondary text-italic text-xs">
                        Nenhum membro adicionado ainda.
                      </p>
                    ) : (
                      <div className="flex-col gap-6">
                        {members.map((member) => (
                          <div key={member.id} className="file-row">
                            <div className="file-info">
                              <span>{member.role === "leader" ? "👑" : "👤"}</span>
                              <span className="file-name">{member.user_name}</span>
                              <span className="file-size">{member.user_email}</span>
                              <span
                                className={`badge ${
                                  member.role === "leader"
                                    ? "badge-overdue"
                                    : "badge-pending"
                                }`}
                                style={{ fontSize: "0.65rem", padding: "2px 6px" }}
                              >
                                {member.role === "leader" ? "Líder" : "Membro"}
                              </span>
                            </div>
                            <div className="flex gap-4">
                              <button
                                className="btn btn-secondary btn-xs"
                                onClick={() => toggleMemberRole(member)}
                                title="Alternar cargo"
                              >
                                {member.role === "leader" ? "⬇️" : "👑"}
                              </button>
                              <button
                                className="btn btn-danger btn-xs"
                                onClick={() => removeMember(member.id, team.id)}
                                title="Remover"
                              >
                                ✕
                              </button>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {showTeamModal && (
        <div className="modal-overlay" onClick={() => setShowTeamModal(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>{editingTeam ? "✏️ Editar Equipe" : "👥 Nova Equipe"}</h2>

            <div className="form-group">
              <label>Nome da Equipe *</label>
              <input
                className="form-input"
                placeholder="Ex: Equipe de Desenvolvimento"
                value={teamForm.name}
                onChange={(e) => setTeamForm({ ...teamForm, name: e.target.value })}
              />
            </div>

            <div className="form-group">
              <label>Descrição</label>
              <textarea
                className="form-textarea"
                placeholder="Descreva o propósito da equipe..."
                value={teamForm.description}
                onChange={(e) => setTeamForm({ ...teamForm, description: e.target.value })}
              />
            </div>

            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={() => setShowTeamModal(false)}>
                Cancelar
              </button>
              <button className="btn btn-primary" onClick={saveTeam}>
                {editingTeam ? "Salvar" : "Criar"}
              </button>
            </div>
          </div>
        </div>
      )}

      {showAddMember && selectedTeamId && (
        <div className="modal-overlay" onClick={() => setShowAddMember(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>➕ Adicionar Membro</h2>

            <div className="form-group">
              <label>Usuário</label>
              <select id="member-select" className="form-input">
                <option value="">Selecione um usuário...</option>
                {usersNotInTeam(selectedTeamId).map((user) => (
                  <option key={user.id} value={user.id}>
                    {user.name} ({user.email})
                  </option>
                ))}
              </select>
            </div>

            <div className="form-group">
              <label>Cargo</label>
              <select
                className="form-input"
                value={memberRole}
                onChange={(e) => setMemberRole(e.target.value)}
              >
                <option value="member">Membro</option>
                <option value="leader">Líder</option>
              </select>
            </div>

            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={() => setShowAddMember(false)}>
                Cancelar
              </button>
              <button className="btn btn-primary" onClick={confirmAddMember}>
                Adicionar
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default TeamList;
