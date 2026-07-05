import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import ArticleManager from "../components/ArticleManager";

describe("ArticleManager", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("renders the documents title", () => {
    render(<ArticleManager />);
    expect(screen.getByText("📄 Documentos")).toBeInTheDocument();
  });

  it("shows empty state when there are no articles", () => {
    render(<ArticleManager />);
    expect(screen.getByText("Nenhum documento cadastrado.")).toBeInTheDocument();
    expect(
      screen.getByText(/Adicione documentos aos seus projetos!/)
    ).toBeInTheDocument();
  });

  it("shows the search input", () => {
    render(<ArticleManager />);
    expect(
      screen.getByPlaceholderText("🔍 Pesquisar documentos...")
    ).toBeInTheDocument();
  });

  it("shows filter tabs (Todos, Rascunhos, Prontos)", () => {
    render(<ArticleManager />);
    expect(screen.getByText("Todos")).toBeInTheDocument();
    expect(screen.getByText("📝 Rascunhos")).toBeInTheDocument();
    expect(screen.getByText("✅ Prontos")).toBeInTheDocument();
  });

  it("shows the 'Novo Documento' button", () => {
    render(<ArticleManager />);
    expect(screen.getByText("➕ Novo Documento")).toBeInTheDocument();
  });

  it("opens the new document modal when clicking 'Novo Documento'", async () => {
    const user = userEvent.setup();
    render(<ArticleManager />);
    await user.click(screen.getByText("➕ Novo Documento"));
    expect(screen.getByText("📄 Novo Documento")).toBeInTheDocument();
    expect(screen.getByText("Salvar")).toBeInTheDocument();
    expect(screen.getByText("Cancelar")).toBeInTheDocument();
  });

  it("opens the create modal when clicking 'Adicionar Documento' in empty state", async () => {
    const user = userEvent.setup();
    render(<ArticleManager />);
    await user.click(screen.getByText("➕ Adicionar Documento"));
    expect(screen.getByText("📄 Novo Documento")).toBeInTheDocument();
  });

  it("can create and display an article (via fallback)", async () => {
    const user = userEvent.setup();
    render(<ArticleManager />);

    await user.click(screen.getByText("➕ Novo Documento"));
    await user.type(
      screen.getByPlaceholderText("Ex: Introdução às Redes Neurais"),
      "Test Article"
    );
    await user.type(
      screen.getByPlaceholderText("Cole o conteúdo do documento aqui..."),
      "This is the article content."
    );
    await user.click(screen.getByText("Salvar"));

    expect(screen.getByText("Test Article")).toBeInTheDocument();
  });

  it("filters articles by search term", async () => {
    const user = userEvent.setup();
    render(<ArticleManager />);

    // Create first article
    await user.click(screen.getByText("➕ Novo Documento"));
    await user.type(
      screen.getByPlaceholderText("Ex: Introdução às Redes Neurais"),
      "Redes Neurais"
    );
    await user.click(screen.getByText("Salvar"));

    // Create second article
    await user.click(screen.getByText("➕ Novo Documento"));
    await user.type(
      screen.getByPlaceholderText("Ex: Introdução às Redes Neurais"),
      "Banco de Dados"
    );
    await user.click(screen.getByText("Salvar"));

    expect(screen.getByText("Redes Neurais")).toBeInTheDocument();
    expect(screen.getByText("Banco de Dados")).toBeInTheDocument();

    // Search for "Redes"
    const searchInput = screen.getByPlaceholderText("🔍 Pesquisar documentos...");
    await user.clear(searchInput);
    await user.type(searchInput, "Redes");

    expect(screen.getByText("Redes Neurais")).toBeInTheDocument();
    expect(screen.queryByText("Banco de Dados")).not.toBeInTheDocument();
  });

  it("shows empty search result message when search has no matches", async () => {
    const user = userEvent.setup();
    render(<ArticleManager />);

    await user.click(screen.getByText("➕ Novo Documento"));
    await user.type(
      screen.getByPlaceholderText("Ex: Introdução às Redes Neurais"),
      "Redes Neurais"
    );
    await user.click(screen.getByText("Salvar"));

    const searchInput = screen.getByPlaceholderText("🔍 Pesquisar documentos...");
    await user.clear(searchInput);
    await user.type(searchInput, "ZZZZ_NOT_FOUND");

    expect(
      screen.getByText("Nenhum documento encontrado para esta busca.")
    ).toBeInTheDocument();
  });

  it("opens the article viewer when clicking 'Ler mais'", async () => {
    const user = userEvent.setup();
    render(<ArticleManager />);

    await user.click(screen.getByText("➕ Novo Documento"));
    await user.type(
      screen.getByPlaceholderText("Ex: Introdução às Redes Neurais"),
      "Article With Content"
    );
    await user.type(
      screen.getByPlaceholderText("Cole o conteúdo do documento aqui..."),
      "This is the full content of the article."
    );
    await user.click(screen.getByText("Salvar"));

    await user.click(screen.getByText("📖 Ler mais"));

    const titleMatches = screen.getAllByText("Article With Content");
    expect(titleMatches.length).toBe(2);

    const contentMatches = screen.getAllByText(
      "This is the full content of the article."
    );
    expect(contentMatches.length).toBe(2);
  });

  it("can delete an article with confirmation", async () => {
    const originalConfirm = window.confirm;
    window.confirm = vi.fn().mockReturnValue(true);

    const user = userEvent.setup();
    render(<ArticleManager />);

    await user.click(screen.getByText("➕ Novo Documento"));
    await user.type(
      screen.getByPlaceholderText("Ex: Introdução às Redes Neurais"),
      "To Delete"
    );
    await user.click(screen.getByText("Salvar"));

    expect(screen.getByText("To Delete")).toBeInTheDocument();

    const deleteButtons = screen.getAllByTitle("Excluir");
    await user.click(deleteButtons[0]);

    expect(screen.queryByText("To Delete")).not.toBeInTheDocument();
    expect(
      screen.getByText("Nenhum documento cadastrado.")
    ).toBeInTheDocument();

    window.confirm = originalConfirm;
  });

  it("shows status badge for articles", async () => {
    const user = userEvent.setup();
    render(<ArticleManager />);

    await user.click(screen.getByText("➕ Novo Documento"));
    await user.type(
      screen.getByPlaceholderText("Ex: Introdução às Redes Neurais"),
      "Status Test"
    );
    await user.click(screen.getByText("Salvar"));

    expect(screen.getByText("📝 Rascunho")).toBeInTheDocument();
  });

  it("can toggle article status between draft and published", async () => {
    const user = userEvent.setup();
    render(<ArticleManager />);

    await user.click(screen.getByText("➕ Novo Documento"));
    await user.type(
      screen.getByPlaceholderText("Ex: Introdução às Redes Neurais"),
      "Toggle Status"
    );
    await user.click(screen.getByText("Salvar"));

    // Should show as draft initially
    expect(screen.getByText("📝 Rascunho")).toBeInTheDocument();

    // Click the toggle button (✅ - mark as ready/published)
    const toggleBtn = screen.getByTitle("Marcar como pronto");
    await user.click(toggleBtn);

    // Should now show as published
    expect(screen.getByText("✅ Pronto")).toBeInTheDocument();
  });
});
