import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import ProjectList from "../components/ProjectList";

describe("ProjectList", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("renders the projects title", () => {
    render(<ProjectList />);
    expect(screen.getByText("📁 Projetos Acadêmicos")).toBeInTheDocument();
  });

  it("shows empty state when there are no projects", () => {
    render(<ProjectList />);
    expect(screen.getByText("Nenhum projeto ainda.")).toBeInTheDocument();
    expect(
      screen.getByText("Crie seu primeiro projeto acadêmico!")
    ).toBeInTheDocument();
  });

  it("shows the 'Novo Projeto' button", () => {
    render(<ProjectList />);
    expect(screen.getByText("➕ Novo Projeto")).toBeInTheDocument();
  });

  it("opens the new project modal when clicking 'Novo Projeto'", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);
    await user.click(screen.getByText("➕ Novo Projeto"));
    expect(screen.getByText("📁 Novo Projeto")).toBeInTheDocument();
    expect(screen.getByText("Criar")).toBeInTheDocument();
    expect(screen.getByText("Cancelar")).toBeInTheDocument();
  });

  it("can create and display a project (via fallback)", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    // Open modal and fill form using userEvent.type (handles state properly)
    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "Test Project"
    );
    await user.type(
      screen.getByPlaceholderText("Descreva o projeto..."),
      "Test Description"
    );
    await user.click(screen.getByText("Criar"));

    // Should show the project card
    expect(screen.getByText("Test Project")).toBeInTheDocument();
    expect(screen.getByText("Test Description")).toBeInTheDocument();
  });

  it("closes the modal when clicking Cancelar", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    await user.click(screen.getByText("➕ Novo Projeto"));
    expect(screen.getByText("📁 Novo Projeto")).toBeInTheDocument();

    await user.click(screen.getByText("Cancelar"));

    expect(screen.queryByText("📁 Novo Projeto")).not.toBeInTheDocument();
  });

  it("opens edit modal when clicking the edit button (✏️)", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    // Create a project first
    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "Editable Project"
    );
    await user.click(screen.getByText("Criar"));

    // Click edit
    const editButton = screen.getByRole("button", { name: /✏️/ });
    await user.click(editButton);

    // Edit modal should open
    expect(screen.getByText("✏️ Editar Projeto")).toBeInTheDocument();
  });

  it("can edit a project's name", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    // Create project
    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "Old Name"
    );
    await user.click(screen.getByText("Criar"));

    // Open edit modal
    await user.click(screen.getByRole("button", { name: /✏️/ }));

    // Change name - clear and type
    const nameInput = screen.getByDisplayValue("Old Name");
    await user.clear(nameInput);
    await user.type(nameInput, "New Name");
    await user.click(screen.getByText("Salvar"));

    // Should show updated name
    expect(screen.getByText("New Name")).toBeInTheDocument();
    expect(screen.queryByText("Old Name")).not.toBeInTheDocument();
  });

  it("shows delete confirmation when clicking 🗑️", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    // Create project
    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "To Delete"
    );
    await user.click(screen.getByText("Criar"));

    // Click delete
    await user.click(screen.getByRole("button", { name: /🗑️/ }));

    // Should show confirm/cancel buttons
    expect(screen.getByText("Confirmar")).toBeInTheDocument();
    expect(screen.getByText("Cancelar")).toBeInTheDocument();
  });

  it("can delete a project after confirming", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    // Create project
    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "To Delete"
    );
    await user.click(screen.getByText("Criar"));

    // Click delete then confirm
    await user.click(screen.getByRole("button", { name: /🗑️/ }));
    await user.click(screen.getByText("Confirmar"));

    expect(screen.getByText("Nenhum projeto ainda.")).toBeInTheDocument();
  });

  it("can cancel deletion and keep the project", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    // Create project
    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "Keep Me"
    );
    await user.click(screen.getByText("Criar"));

    // Click delete then cancel
    await user.click(screen.getByRole("button", { name: /🗑️/ }));
    await user.click(screen.getByText("Cancelar"));

    expect(screen.getByText("Keep Me")).toBeInTheDocument();
  });

  it("has the 'Adicionar Artigo' button per project", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    // Create project
    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "Project With Articles"
    );
    await user.click(screen.getByText("Criar"));

    // There should be a button 📄 +Artigo
    expect(screen.getByText(/📄.*Artigo/)).toBeInTheDocument();
  });

  it("opens the article creation modal for a project", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    // Create project
    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "Project For Article"
    );
    await user.click(screen.getByText("Criar"));

    // Click "📄 +Artigo" button
    await user.click(screen.getByText(/📄.*Artigo/));
    expect(screen.getByText("📄 Novo Artigo")).toBeInTheDocument();
    expect(screen.getByText("Salvar Artigo")).toBeInTheDocument();
  });

  it("has a button to anexar arquivos per project", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "Project With Files"
    );
    await user.click(screen.getByText("Criar"));

    expect(screen.getByText("📎 Anexar Arquivo")).toBeInTheDocument();
  });

  it("has a button to ver arquivos per project", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "Project Files"
    );
    await user.click(screen.getByText("Criar"));

    expect(screen.getByText("📂 Ver Arquivos")).toBeInTheDocument();
  });

  it("has a button to exportar ZIP per project", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "Project ZIP"
    );
    await user.click(screen.getByText("Criar"));

    expect(screen.getByText("📦 Exportar ZIP")).toBeInTheDocument();
  });

  it("shows article count for each project", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "Counted Project"
    );
    await user.click(screen.getByText("Criar"));

    expect(screen.getByText(/0 artigo/)).toBeInTheDocument();
  });

  it("shows the creation date for each project", async () => {
    const user = userEvent.setup();
    render(<ProjectList />);

    await user.click(screen.getByText("➕ Novo Projeto"));
    await user.type(
      screen.getByPlaceholderText("Ex: Trabalho de Redes"),
      "Dated Project"
    );
    await user.click(screen.getByText("Criar"));

    expect(screen.getByText(/Criado em/)).toBeInTheDocument();
  });
});
