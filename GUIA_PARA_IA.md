# Guia para IA - Cursor Chat Handler

> Execute `cursor-chat --help` para ver este guia completo

## Comandos Essenciais

### Recuperar Contexto de Chat Anterior
```bash
cursor-chat quick              # Menu com números
cursor-chat open 1             # Abrir última conversa
cursor-chat show <ID> --last 10  # Ver últimas 10 msgs
```

### Salvar Contexto Atual
```bash
cursor-chat export -c <ID> -o chat.md
cursor-chat export-all --limit 3
```

### Auto-Sync + Restore
```bash
cursor-chat sync start         # Iniciar daemon (a cada 2min)
cursor-chat sync restore       # Restaurar após limpar Cursor
```

### Reset Trial Completo (NOVO!)
```bash
cursor-chat reset              # Backup → Reset → Restore automático
cursor-chat reset --no-restore # Apenas reset, sem restaurar
```

**O que faz:**
- Faz backup de todos os chats
- Reseta machine-id e limpa configs do Cursor
- Restaura chats automaticamente
- Pronto para usar após abrir Cursor novamente

### Limpou o Cursor? (trial reset)
```bash
cursor-chat restore            # Restaura TODOS os chats do backup!
cursor-chat reset              # OU: reset completo automático
```

### Ver por Projeto/Workspace
```bash
cursor-chat storage workspaces  # Listar projetos
```

## Dica Principal

Os chats são salvos em `~/.cursor-chat-handler/` e **persistem mesmo após trial reset**.

- `cursor-chat sync start` - Inicia backup automático (roda como serviço)
- `cursor-chat restore` - Restaura chats após limpar dados do Cursor
- `cursor-chat reset` - Reset completo com backup/restore automático

## Workflow Recomendado

```bash
# 1. Ver chats disponíveis
cursor-chat quick

# 2. Abrir chat anterior por número
cursor-chat open 1

# 3. Ver últimas mensagens
cursor-chat show <ID> --last 5

# 4. Exportar se precisar
cursor-chat export -c <ID> -o contexto.md

# 5. Reset trial quando necessário
cursor-chat reset  # Faz tudo automaticamente!
```

## Notas

- IDs parciais funcionam (primeiros 8 caracteres)
- `--last N` limita mensagens (evita sobrecarga)
- Formatos: markdown (padrão), json, table
- Auto-sync: `cursor-chat sync start` (configura systemd)
- Reset: `cursor-chat reset` (backup + reset + restore em um comando)
