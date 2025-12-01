# 🎯 Cursor Chat Handler

> **Extract, view, and manage your Cursor IDE chat history with style**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-orange.svg?style=for-the-badge)]()

A powerful CLI tool to extract, browse, and export your Cursor IDE conversations with an elegant interface designed for productivity and AI-assisted development workflows.

## 🚀 Quick Start

Get up and running in 3 steps:

```bash
# 1. Clone & Install
git clone <repo-url>
cd cursor-chat-handler
./install.sh

# 2. You're ready!
cursor-chat quick

# 3. Start exploring
cursor-chat open 1
```

## 🏗️ Architecture

### Clean Architecture
```
┌─────────────────────────────────────────┐
│           🎯 CLI Layer                   │
│   (Commands, Args, User Interaction)    │
├─────────────────────────────────────────┤
│           📋 Application Layer          │
│   (Use Cases, Export, Formatting)       │
├─────────────────────────────────────────┤
│           🏛️ Domain Layer               │
│   (Models, Business Logic, Types)       │
├─────────────────────────────────────────┤
│           🗄️ Infrastructure Layer       │
│   (SQLite, File System, External APIs)  │
└─────────────────────────────────────────┘
```

### Key Components
- **Domain**: Pure business logic, conversation models, error types
- **Application**: Use cases for extraction, formatting, export workflows
- **Infrastructure**: SQLite readers, path discovery, file operations
- **CLI**: Command parsing, user interaction, output formatting

## 📦 Installation

### Option 1: Automated Install (Recommended)

```bash
git clone <repo-url>
cd cursor-chat-handler
./install.sh
```

This sets up:
- ✅ Compiles the binary
- ✅ Adds to system PATH
- ✅ Creates `cursor-chat` alias
- ✅ Ready to use globally

### Option 2: Manual Install

```bash
# Clone and build
git clone <repo-url>
cd cursor-chat-handler
cargo build --release

# Add to PATH (add to your ~/.bashrc or ~/.zshrc)
export PATH="$PWD/target/release:$PATH"
alias cursor-chat="$PWD/target/release/cursor-chat-handler"
```

## 🎯 Essential Commands

### 🚀 Professional Workflow
```bash
# 1. Quick menu access
cursor-chat quick

# 2. Open by number
cursor-chat open 1

# 3. Export for AI context
cursor-chat export -c abc123 -o context.md
```

### 💾 Export & Backup
```bash
# Export specific conversation
cursor-chat export -c <id> -o chat.md

# Batch export recent chats
cursor-chat export-all --limit 5 --dir ./chats

# Export in JSON format
cursor-chat -f json export -c <id> -o chat.json
```

### 📋 Browse & Search
```bash
# List all conversations with titles
cursor-chat list

# Show last N messages
cursor-chat show <id> --last 10

# Open directly (partial ID works)
cursor-chat open abc    # matches abc123, abc456, etc.
```

## ✨ Features

### 🚀 Core Features
- 🎯 **Quick Access Menu** - Interactive numbered menu for instant chat selection
- 🔍 **Direct Chat Opening** - Open conversations by number or partial ID
- 📋 **Smart Listing** - Auto-generated titles with conversation statistics
- 📤 **Intelligent Export** - Export individual chats or batch with descriptive names
- ⏪ **Context Limiting** - View last N messages for focused context
- 🎨 **Multiple Formats** - Markdown, JSON, and table outputs

### 🔧 Technical Features
- 🗂️ **Multi-Database Support** - Reads both global storage and workspace databases
- 🎯 **Smart Filtering** - Partial ID matching and conversation search
- 📊 **Rich Statistics** - Detailed analytics of your chat history
- ⚡ **Optimized Performance** - Fast SQLite queries with read-only access
- 🔒 **Safe Operations** - Read-only database access, no modifications

## 📸 Preview

### Quick Access Menu
```bash
🚀 Quick Access Menu
==================

   1. abc12345 | claude-4.5-opus | 15 msgs | como_criar_uma_api_rest_em_rust
   2. def67890 | grok-code-fast- | 8 msgs  | implementando_testes_unitarios
   3. ghi54321 | claude-4.5-opus | 23 msgs | melhores_praticas_de_arquitetura

💡 Quick commands:
   cursor-chat open 1        # Open conversation by number
   cursor-chat show abc12345 # Show full conversation
   cursor-chat export -c abc12345 # Export conversation
```

### Direct Chat Access
```bash
cursor-chat open 1
# Opens conversation with last 10 messages + export tips
```

## Uso

### 🚀 Acesso Rápido (Mais Usado)

```bash
# Menu rápido com números para seleção
cursor-chat quick

# Abrir conversa diretamente (últimas 10 mensagens)
cursor-chat open 1    # por número
cursor-chat open abc  # por ID parcial

# Exportar tudo automaticamente
cursor-chat export-all --limit 3 --dir ./chats
```

### Listar conversas

```bash
# Lista as 20 conversas mais recentes
cursor-chat-handler list

# Lista com limite customizado
cursor-chat-handler list --limit 10

# Apenas conversas com pelo menos 5 mensagens
cursor-chat-handler list --min-messages 5
```

### Ver uma conversa

```bash
# Por ID completo ou parcial
cursor-chat-handler show abc123

# Formato JSON
cursor-chat-handler -f json show abc123

# Incluir mensagens vazias
cursor-chat-handler show abc123 --include-empty
```

### Exportar

```bash
# Exportar todas para arquivo
cursor-chat-handler export -o chats.md

# Exportar em JSON
cursor-chat-handler -f json export -o chats.json

# Exportar conversa específica
cursor-chat-handler export -c abc123 -o conversa.md
```

### Estatísticas

```bash
cursor-chat-handler stats
```

### Ver caminhos dos bancos

```bash
cursor-chat-handler paths
```

## Estrutura do Projeto

```
src/
├── main.rs              # Entrypoint e comandos
├── cli/                 # Interface de linha de comando
│   └── mod.rs
├── domain/              # Tipos e erros de domínio
│   ├── mod.rs
│   ├── error.rs
│   └── models.rs
├── application/         # Lógica de negócio
│   ├── mod.rs
│   ├── extractor.rs     # Extração de conversas
│   ├── formatter.rs     # Formatação de saída
│   └── parser.rs        # Parsing JSON
└── infrastructure/      # Acesso a dados
    ├── mod.rs
    ├── cursor_paths.rs  # Descoberta de caminhos
    └── sqlite_reader.rs # Leitura SQLite
```

## Formatos de saída

- **markdown** (padrão): Formatação rica com thinking blocks colapsáveis
- **json**: Dados estruturados para processamento
- **table**: Visão resumida em tabela

## Requisitos

- Rust 1.70+
- Cursor IDE instalado (os arquivos de dados são lidos diretamente)

## 🤖 AI Integration Workflows

### 🎯 For Cursor/ChatGPT Users

#### Quick Context Sharing
```bash
# 1. Find your conversation
cursor-chat quick

# 2. Open and view recent context
cursor-chat open 1

# 3. Export for AI
cursor-chat export -c <id> -o current_context.md
```

#### Continuing Previous Work
```bash
# Get the last 10 messages for context
cursor-chat show <previous_id> --last 10

# Export entire conversation
cursor-chat export -c <previous_id> -o full_context.md
```

### 📋 Copy-Paste Commands for AI

### 🎯 Comandos Essenciais (Copie e Cole)

```bash
# 📋 VER TODOS OS CHATS DISPONÍVEIS
cursor-chat list

# 🔍 VER UM CHAT ESPECÍFICO (últimas 5 mensagens)
cursor-chat show <ID_DO_CHAT> --last 5

# 💾 SALVAR CHAT ATUAL PARA CONTINUAR DEPOIS
cursor-chat export -c <ID_DO_CHAT> -o chat_atual.md

# 📤 EXPORTAR TODOS OS CHATS RECENTES
cursor-chat export-all --limit 3 --dir ./chats-salvos
```

### 📝 Como Usar com Cursor

1. **Para salvar o contexto atual:**
   ```
   cursor-chat export -c <cole_o_id_aqui> -o contexto_atual.md
   ```

2. **Para recuperar um chat anterior:**
   ```
   cursor-chat show <id_do_chat> --last 10
   ```

3. **Para ver todos os chats disponíveis:**
   ```
   cursor-chat list
   ```

### 💡 Dicas para IA

- **Sempre use `--last N`** para não sobrecarregar com mensagens antigas
- **Exporte em Markdown** (`-f md`) para melhor legibilidade
- **Use IDs parciais** - funciona com os primeiros 8 caracteres
- **Títulos são auto-gerados** do primeiro texto do usuário

### 🔄 Workflow Recomendado

```
1. cursor-chat list                                    # Ver chats disponíveis
2. cursor-chat show <id> --last 5                      # Ver últimas mensagens
3. cursor-chat export -c <id> -o contexto.md           # Salvar contexto
4. [Continue seu trabalho no Cursor]
```

## 🔧 Requirements

- **Rust**: 1.70+ (for latest features and performance)
- **Cursor IDE**: Any version (reads from SQLite databases)
- **Platform**: Linux, macOS, Windows (with SQLite support)

### Dependencies
- `rusqlite` - SQLite database access
- `clap` - Command line argument parsing
- `serde` - JSON serialization
- `chrono` - Date/time handling
- `colored` - Terminal colors
- `comfy-table` - Beautiful table formatting

## 🛠️ Development

### Building from Source
```bash
# Clone the repository
git clone <repo-url>
cd cursor-chat-handler

# Build in debug mode
cargo build

# Build optimized release
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt --check
```

### Project Structure
```
src/
├── main.rs              # CLI entry point and command routing
├── cli/                 # Command-line interface definitions
│   └── mod.rs
├── domain/              # Business logic and data models
│   ├── mod.rs
│   ├── error.rs         # Error types and handling
│   └── models.rs        # Conversation and bubble models
├── application/         # Use cases and business workflows
│   ├── mod.rs
│   ├── extractor.rs     # Chat extraction logic
│   ├── formatter.rs     # Output formatting
│   └── parser.rs        # JSON parsing utilities
└── infrastructure/      # External system adapters
    ├── mod.rs
    ├── cursor_paths.rs  # Cursor installation detection
    └── sqlite_reader.rs # Database reading
```

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Quality
- **Clippy**: `cargo clippy` (all warnings must pass)
- **Format**: `cargo fmt` (code must be formatted)
- **Tests**: `cargo test` (all tests must pass)
- **Unsafe**: Forbidden (memory safety is critical)

## 📄 License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- **Cursor IDE** - The amazing AI-powered code editor
- **Rust Community** - For the incredible ecosystem
- **SQLite** - Reliable embedded database
- **Clap** - Excellent CLI framework

---

**Made with ❤️ for developers who live in their terminals**

