# 🎯 Guia Rápido: Cursor Chat Handler para IA

## 🚀 Comandos Essenciais (Copie e Cole)

### ⚡ ACESSO ULTRA-RÁPIDO (Profissional):
```bash
cursor-chat quick          # Menu numerado para seleção instantânea
cursor-chat open 1         # Abrir primeira conversa diretamente
cursor-chat open abc123    # Abrir por ID (parcial funciona)
```

### 📋 Ver todos os chats disponíveis:
```bash
cursor-chat list
```

### 🔍 Ver últimas mensagens de um chat específico:
```bash
cursor-chat show <ID_DO_CHAT> --last 5
```

### 💾 Salvar chat atual para continuar depois:
```bash
cursor-chat export -c <ID_DO_CHAT> -o contexto_atual.md
```

### 📤 Exportar múltiplos chats automaticamente:
```bash
cursor-chat export-all --limit 3 --dir ./chats-salvos
```

## 💡 Como usar com Cursor/IA:

### 🚀 MÉTODO ULTRA-RÁPIDO (Profissional):
1. **Menu instantâneo:**
   - Execute: `cursor-chat quick`
   - Veja lista numerada dos chats

2. **Abra diretamente:**
   - Execute: `cursor-chat open 1` (número da conversa)
   - Veja últimas 10 mensagens automaticamente

3. **Salve tudo:**
   - Execute: `cursor-chat export-all --limit 3 --dir ./backup`

### 📝 MÉTODO COMPLETO (Desenvolvimento):
1. **Identifique o chat atual:**
   - Execute: `cursor-chat list`
   - Copie o ID do chat que você quer salvar

2. **Salve o contexto:**
   - Execute: `cursor-chat export -c <ID_AQUI> -o contexto.md`
   - Agora você tem o histórico salvo

3. **Continue de onde parou:**
   - Execute: `cursor-chat show <ID_AQUI> --last 10`
   - Veja as últimas 10 mensagens para relembrar

## 🎯 Workflow Recomendado:

```
# 1. Ver chats disponíveis
cursor-chat list

# 2. Salvar contexto atual
cursor-chat export -c abc123 -o projeto_atual.md

# 3. Continuar trabalhando...
# (faça seu trabalho no Cursor)

# 4. Recuperar contexto quando necessário
cursor-chat show abc123 --last 5
```

## 📝 Notas Importantes:

- **IDs parciais funcionam** - use apenas os primeiros 8 caracteres
- **Títulos são auto-gerados** do conteúdo do chat
- **--last N** limita para últimas N mensagens (evita sobrecarga)
- **Formatos**: markdown (padrão), json, ou table
- **Funciona sempre** - alias configurado permanentemente

## 🤖 Exemplo de uso com IA:

> "Estou trabalhando em um projeto Rust e preciso salvar o contexto atual. Execute: `cursor-chat export -c <ID_DO_CHAT_ATUAL> -o contexto_rust.md`"

---

**Alias configurado:** `cursor-chat` funciona em qualquer diretório após reiniciar o terminal.
