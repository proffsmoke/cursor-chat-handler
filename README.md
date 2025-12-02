# Cursor Chat Handler

> **Extraia, visualize e faça backup automático dos seus chats do Cursor IDE**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)](LICENSE)

CLI em Rust para extrair e gerenciar histórico de conversas do Cursor IDE, com **auto-sync** que salva seus chats automaticamente a cada 2 minutos.

## Quick Start

```bash
# Instalar
git clone <repo-url>
cd cursor-chat-handler
./install.sh

# Começar a usar
cursor-chat quick           # Menu rápido
cursor-chat open 1          # Abrir última conversa
cursor-chat sync start      # Iniciar auto-backup (a cada 2min)

# LIMPOU O CURSOR? Restaura tudo:
cursor-chat restore
```

## Guia Rápido para IA

```
🤖 GUIA RÁPIDO - cursor-chat --help

📋 RECUPERAR CONTEXTO DE CHAT ANTERIOR:
  cursor-chat quick              # Menu com números
  cursor-chat open 1             # Abrir última conversa
  cursor-chat show <ID> --last 10  # Ver últimas 10 msgs

💾 SALVAR CONTEXTO ATUAL:
  cursor-chat export -c <ID> -o chat.md
  cursor-chat export-all --limit 3

🔄 AUTO-SYNC (salva automaticamente a cada 2min):
  cursor-chat sync start         # Iniciar daemon
  cursor-chat sync status        # Ver status
  cursor-chat sync now           # Forçar sync

📁 VER POR PROJETO/WORKSPACE:
  cursor-chat storage workspaces  # Listar projetos
  cursor-chat list -w <projeto>   # Filtrar por projeto

💡 DICA: Os chats são salvos mesmo se o Cursor resetar!
   Dados em: ~/.cursor-chat-handler/
```

## Auto-Sync + Auto-Restore

O sistema mantém backup dos seus chats e restaura automaticamente após reset:

```bash
cursor-chat sync start      # Iniciar daemon (systemd)
cursor-chat sync stop       # Parar
cursor-chat sync status     # Ver status
cursor-chat restore         # Restaurar após limpar Cursor
```

**Recursos:**
- Sincroniza a cada 2 minutos
- **Auto-restore**: Detecta quando o Cursor foi limpo e restaura automaticamente
- Persiste mesmo após trial reset
- Limite de 10GB configurável
- Organiza por projeto/workspace

## Restore Manual

Limpou os dados do Cursor (trial reset)? Restaure tudo:

```bash
cursor-chat restore              # Restaurar todos os chats
cursor-chat restore --force      # Forçar mesmo se Cursor tiver chats
cursor-chat restore -i abc123    # Restaurar chat específico
```

**Após restaurar:** Reinicie o Cursor para ver os chats de volta.

## Storage Local

```bash
cursor-chat storage stats       # Ver uso de armazenamento
cursor-chat storage cleanup     # Limpar backups antigos
cursor-chat storage workspaces  # Listar projetos detectados
cursor-chat storage config      # Ver configuração
```

**Estrutura:**
```
~/.cursor-chat-handler/
├── storage.db        # SQLite com todos os chats
├── config.toml       # Configuração
├── exports/          # Chats exportados
└── backups/          # Backups incrementais
```

## Comandos Principais

### Visualizar Chats
```bash
cursor-chat quick              # Menu interativo com números
cursor-chat open 1             # Abrir por número
cursor-chat open abc123        # Abrir por ID parcial
cursor-chat list               # Listar todos
cursor-chat show <ID>          # Ver conversa completa
cursor-chat show <ID> --last 5 # Últimas 5 mensagens
```

### Exportar
```bash
cursor-chat export -c <ID> -o chat.md      # Exportar específico
cursor-chat export-all --limit 5           # Exportar últimos 5
cursor-chat export-all --dir ./backup      # Exportar para pasta
```

### Formatos
```bash
cursor-chat -f markdown show <ID>    # Markdown (padrão)
cursor-chat -f json show <ID>        # JSON
cursor-chat -f table list            # Tabela
```

## Configuração

Edite `~/.cursor-chat-handler/config.toml`:

```toml
[sync]
interval_secs = 120          # 2 minutos
enabled = true

[storage]
max_size_gb = 10             # Limite de 10GB
backup_retention_days = 30   # Manter backups por 30 dias
compression = true
```

## Arquitetura

```
┌─────────────────────────────────────────┐
│           CLI Layer                      │
│   (Commands, Args, User Interaction)    │
├─────────────────────────────────────────┤
│           Application Layer              │
│   (Sync, Storage Manager, Formatting)   │
├─────────────────────────────────────────┤
│           Domain Layer                   │
│   (Models, Config, Business Logic)      │
├─────────────────────────────────────────┤
│           Infrastructure Layer           │
│   (SQLite, Systemd, File System)        │
└─────────────────────────────────────────┘
```

## Requisitos

- Rust 1.70+
- Linux com systemd (para auto-sync)
- Cursor IDE instalado

## Desenvolvimento

```bash
cargo build              # Build debug
cargo build --release    # Build release
cargo test               # Rodar testes
cargo clippy             # Linter
```

## License

MIT
