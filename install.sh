#!/bin/bash

# Cursor Chat Handler - Script de Instalação
# Este script instala o cursor-chat-handler no sistema
# Inclui configuração do serviço systemd para auto-sync

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
BIN_NAME="cursor-chat-handler"
ALIAS_NAME="cursor-chat"
DATA_DIR="$HOME/.cursor-chat-handler"

echo "🚀 Instalando Cursor Chat Handler..."

# Compilar em release
echo "📦 Compilando binário..."
cd "$PROJECT_DIR"
cargo build --release

# Criar diretório se não existir
mkdir -p "$BIN_DIR"
mkdir -p "$DATA_DIR"

# Copiar binário para diretório local
echo "📋 Instalando binário em $BIN_DIR..."
cp "target/release/$BIN_NAME" "$BIN_DIR/"

# Verificar se já existe no PATH
if ! command -v "$BIN_NAME" &> /dev/null; then
    echo "⚠️  $BIN_NAME não está no PATH. Adicionando..."

    # Detectar shell
    SHELL_RC=""
    if [[ "$SHELL" == *"zsh"* ]]; then
        SHELL_RC="$HOME/.zshrc"
    elif [[ "$SHELL" == *"bash"* ]]; then
        SHELL_RC="$HOME/.bashrc"
    else
        echo "❌ Shell não suportado: $SHELL"
        echo "   Suportados: bash, zsh"
        exit 1
    fi

    # Adicionar ao PATH se não estiver lá
    if ! grep -q "$BIN_DIR" "$SHELL_RC" 2>/dev/null; then
        echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$SHELL_RC"
        echo "✅ Adicionado $BIN_DIR ao PATH em $SHELL_RC"
    fi
fi

# Adicionar alias se não existir
if ! grep -q "alias $ALIAS_NAME=" ~/.bashrc ~/.zshrc 2>/dev/null; then
    FULL_PATH="$BIN_DIR/$BIN_NAME"
    if [[ "$SHELL" == *"zsh"* ]]; then
        echo "alias $ALIAS_NAME='$FULL_PATH'" >> ~/.zshrc
        echo "✅ Alias '$ALIAS_NAME' adicionado ao ~/.zshrc"
    else
        echo "alias $ALIAS_NAME='$FULL_PATH'" >> ~/.bashrc
        echo "✅ Alias '$ALIAS_NAME' adicionado ao ~/.bashrc"
    fi
fi

# Criar configuração padrão se não existir
if [ ! -f "$DATA_DIR/config.toml" ]; then
    echo "⚙️  Criando configuração padrão..."
    cat > "$DATA_DIR/config.toml" << 'EOF'
# Cursor Chat Handler Configuration
# Edit as needed

[sync]
# Interval between syncs in seconds (default: 120 = 2 minutes)
interval_secs = 120

# Whether sync is enabled
enabled = true

[storage]
# Maximum storage size in GB (default: 10)
max_size_gb = 10

# Number of days to keep backups (default: 30)
backup_retention_days = 30

# Whether to compress backups
compression = true

[paths]
# Custom data directory (optional, defaults to ~/.cursor-chat-handler)
# data_dir = "/custom/path"
EOF
    echo "✅ Configuração criada em $DATA_DIR/config.toml"
fi

echo ""
echo "🎉 Instalação completa!"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📋 COMANDOS BÁSICOS:"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "  cursor-chat quick           # Menu rápido com números"
echo "  cursor-chat open 1          # Abrir conversa por número"
echo "  cursor-chat list            # Listar todos os chats"
echo "  cursor-chat export-all      # Exportar todos os chats"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "🔄 AUTO-SYNC (NOVO!):"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "  cursor-chat sync start      # Iniciar daemon (auto a cada 2min)"
echo "  cursor-chat sync stop       # Parar daemon"
echo "  cursor-chat sync status     # Ver status do sync"
echo "  cursor-chat sync now        # Sincronizar agora"
echo ""
echo "  cursor-chat storage stats   # Ver uso de armazenamento"
echo "  cursor-chat storage cleanup # Limpar backups antigos"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "💾 STORAGE LOCAL:"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "  Dados:    $DATA_DIR"
echo "  Config:   $DATA_DIR/config.toml"
echo "  Limite:   10 GB (configurável)"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "🔄 Reinicie o terminal ou execute:"
echo "   source ~/.bashrc  # (ou ~/.zshrc se usar zsh)"
echo ""

# Perguntar se quer iniciar o auto-sync
read -p "🚀 Deseja iniciar o auto-sync agora? [s/N] " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Ss]$ ]]; then
    echo "📡 Iniciando auto-sync..."
    "$BIN_DIR/$BIN_NAME" sync start || {
        echo "⚠️  Falha ao iniciar auto-sync. Tente manualmente:"
        echo "   cursor-chat sync start"
    }
fi

echo ""
echo "✅ Pronto! Use 'cursor-chat --help' para mais opções."
