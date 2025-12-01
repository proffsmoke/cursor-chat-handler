#!/bin/bash

# Cursor Chat Handler - Script de Instalação
# Este script instala o cursor-chat-handler no sistema

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="$HOME/.local/bin"
BIN_NAME="cursor-chat-handler"
ALIAS_NAME="cursor-chat"

echo "🚀 Instalando Cursor Chat Handler..."

# Compilar em release
echo "📦 Compilando binário..."
cd "$PROJECT_DIR"
cargo build --release

# Criar diretório se não existir
mkdir -p "$BIN_DIR"

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
    FULL_PATH="$PROJECT_DIR/target/release/$BIN_NAME"
    if [[ "$SHELL" == *"zsh"* ]]; then
        echo "alias $ALIAS_NAME='$FULL_PATH'" >> ~/.zshrc
        echo "✅ Alias '$ALIAS_NAME' adicionado ao ~/.zshrc"
    else
        echo "alias $ALIAS_NAME='$FULL_PATH'" >> ~/.bashrc
        echo "✅ Alias '$ALIAS_NAME' adicionado ao ~/.bashrc"
    fi
fi

echo ""
echo "🎉 Instalação completa!"
echo ""
echo "📋 Para usar:"
echo "   cursor-chat --help          # Ver ajuda completa"
echo "   cursor-chat quick           # Menu profissional rápido"
echo "   cursor-chat open 1          # Abrir primeira conversa"
echo "   cursor-chat list            # Listar chats"
echo "   cursor-chat export-all      # Exportar todos os chats"
echo ""
echo "🔄 Reinicie o terminal ou execute:"
echo "   source ~/.bashrc  # (ou ~/.zshrc se usar zsh)"
echo ""
echo "📖 Guia rápido para IA (copie e cole):"
echo "========================================"
echo ""
echo "# 🚀 ACESSO ULTRA-RÁPIDO:"
echo "cursor-chat quick          # Menu numerado profissional"
echo "cursor-chat open 1         # Abrir conversa por número"
echo ""
echo "# 💾 SALVAR/CONTINUAR:"
echo "cursor-chat export -c <ID> -o contexto.md    # Salvar específico"
echo "cursor-chat export-all --limit 3 --dir ./backup # Backup automático"
echo ""
echo "# 📋 VISUALIZAR:"
echo "cursor-chat list                              # Lista completa"
echo "cursor-chat show <ID> --last 5              # Ver últimas mensagens"
echo ""
echo "========================================"
