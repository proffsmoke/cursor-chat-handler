# Frontend Leptos - Cursor Chat Control Panel

## Arquitetura

```
cursor-chat-handler/
├── src/                          # Backend CLI existente
├── api/                          # Novo: API REST (Axum)
│   ├── src/
│   │   ├── main.rs              # Server entry
│   │   ├── routes/              # Endpoints REST
│   │   ├── websocket.rs         # Real-time updates
│   │   └── state.rs             # App state compartilhado
│   └── Cargo.toml
├── web/                          # Novo: Frontend Leptos
│   ├── src/
│   │   ├── main.rs              # Hydration entry
│   │   ├── app.rs               # Root component
│   │   ├── components/          # UI components
│   │   ├── pages/               # Page components
│   │   └── api.rs               # HTTP client
│   ├── style/                   # CSS terminal theme
│   └── Cargo.toml
└── Cargo.toml                   # Workspace
```

## Stack Técnica

- **Backend API**: Axum + Tower + tokio
- **Frontend**: Leptos 0.7 + SSR
- **Real-time**: WebSockets nativos
- **Estilo**: CSS puro (terminal aesthetic)
- **Build**: Trunk (WASM) + cargo-leptos

---

## Fase 1: Fundação (40h)

### 1.1 Setup Workspace Cargo (8h)
- Converter projeto para workspace multi-crate
- Extrair domain/infrastructure como crate compartilhada (`core/`)
- Configurar `api/` e `web/` como crates separadas
- Arquivos: `Cargo.toml` (workspace), `core/Cargo.toml`, `api/Cargo.toml`, `web/Cargo.toml`

### 1.2 API REST Base (16h)
- Setup Axum server em `api/src/main.rs`
- Endpoints CRUD para chats:
  - `GET /api/chats` - listar conversas
  - `GET /api/chats/:id` - detalhe conversa
  - `GET /api/workspaces` - listar workspaces
  - `GET /api/stats` - estatísticas
  - `POST /api/sync` - trigger sync manual
  - `POST /api/backup` - trigger backup
  - `POST /api/restore` - trigger restore
  - `POST /api/reset` - trigger cursor reset
- Reusar `SyncService`, `RestoreService` do core
- CORS configurado para localhost

### 1.3 WebSocket Real-time (16h)
- Endpoint `WS /api/ws` para updates em tempo real
- Eventos: `chat_updated`, `sync_progress`, `new_message`
- Broadcast de mudanças quando daemon detecta alterações
- Heartbeat para manter conexão viva

---

## Fase 2: Frontend Base (50h)

### 2.1 Setup Leptos (10h)
- Inicializar projeto Leptos com SSR
- Configurar Trunk para build WASM
- Setup hot-reload para desenvolvimento
- Estrutura de pastas e imports

### 2.2 Sistema de Estilo Terminal (15h)
- CSS variables para tema hacker:
  ```css
  --bg-primary: #0a0a0a;
  --text-primary: #00ff41;
  --text-secondary: #00cc33;
  --accent: #ff0055;
  --border: #1a1a1a;
  ```
- Font: JetBrains Mono ou Fira Code
- Efeitos: scanlines, glow, cursor piscante
- Componentes base: Button, Input, Card, Table
- Animações de "digitação" e fade-in

### 2.3 Layout Principal (10h)
- Sidebar fixa com navegação
- Header com status do daemon
- Área de conteúdo principal
- Footer com stats em tempo real
- Responsive (mas foco desktop)

### 2.4 Routing (15h)
- Setup leptos_router
- Páginas:
  - `/` - Dashboard
  - `/chats` - Lista de chats
  - `/chats/:id` - Visualizar chat
  - `/workspaces` - Lista workspaces
  - `/control` - Painel de controle
  - `/settings` - Configurações

---

## Fase 3: Funcionalidades Core (60h)

### 3.1 Dashboard (15h)
- Cards com métricas:
  - Total de chats
  - Mensagens hoje
  - Storage usado (progress bar)
  - Status do daemon (online/offline)
- Gráfico ASCII de atividade (últimos 7 dias)
- Lista de chats recentes (5 últimos)
- Ações rápidas: Sync, Backup, Restore

### 3.2 Lista de Chats (20h)
- Tabela com colunas: Título, Workspace, Mensagens, Data
- Filtros: por workspace, por data, busca texto
- Ordenação por coluna
- Paginação (50 por página)
- Preview on hover (primeiras 2 mensagens)
- Seleção múltipla para ações em lote

### 3.3 Visualizador de Chat (15h)
- Layout de conversa estilo terminal
- User messages: `> ` prefix, cor diferente
- Assistant messages: `$ ` prefix, syntax highlight
- Metadata: modelo, tokens, timestamp
- Copiar mensagem individual
- Exportar chat completo (JSON/MD)

### 3.4 Painel de Controle (10h)
- Botões grandes com ícones ASCII:
  ```
  [▶ SYNC NOW]  [↻ BACKUP]  [↩ RESTORE]  [⚠ RESET]
  ```
- Confirmação para ações destrutivas (reset)
- Log de output em tempo real (estilo terminal)
- Progress bar para operações longas

---

## Fase 4: Real-time e Polish (30h)

### 4.1 WebSocket Integration (15h)
- Hook `use_websocket()` para conexão
- Auto-reconnect com backoff exponencial
- Atualizar UI automaticamente quando:
  - Novo chat aparece
  - Sync completa
  - Mensagem nova em chat aberto
- Indicador de conexão no header

### 4.2 Notificações e Feedback (10h)
- Toast notifications para ações
- Loading states com spinners ASCII
- Error handling com mensagens claras
- Confirmação visual de sucesso

### 4.3 Performance e UX (5h)
- Lazy loading de chats longos
- Debounce em buscas
- Cache local com localStorage
- Skeleton loaders

---

## Fase 5: Features Avançadas (20h)

### 5.1 Busca Full-text (8h)
- Endpoint `GET /api/search?q=termo`
- Highlight de matches no resultado
- Filtros combinados (workspace + texto)
- Histórico de buscas recentes

### 5.2 Export em Lote (6h)
- Selecionar múltiplos chats
- Export como ZIP com JSONs
- Export como Markdown único
- Download progress

### 5.3 Configurações (6h)
- Editar config.toml via UI
- Ajustar intervalo de sync
- Definir limite de storage
- Tema: variantes do terminal (green/amber/blue)

---

## Endpoints API Completos

| Método | Endpoint | Descrição |
|--------|----------|-----------|
| GET | `/api/health` | Health check |
| GET | `/api/stats` | Estatísticas gerais |
| GET | `/api/chats` | Listar chats (paginado) |
| GET | `/api/chats/:id` | Detalhe de um chat |
| GET | `/api/workspaces` | Listar workspaces |
| GET | `/api/search` | Busca full-text |
| POST | `/api/sync` | Executar sync |
| POST | `/api/backup` | Executar backup |
| POST | `/api/restore` | Restaurar chats |
| POST | `/api/reset` | Reset Cursor |
| WS | `/api/ws` | WebSocket real-time |

---

## Comandos de Desenvolvimento

```bash
# Rodar API (porta 3000)
cargo run -p cursor-chat-api

# Rodar frontend dev (porta 8080)
cd web && trunk serve

# Build produção
cargo leptos build --release

# Rodar tudo junto
cargo run -p cursor-chat-api &
cd web && trunk serve
```

---

## Distribuição de Horas

| Fase | Horas | % |
|------|-------|---|
| 1. Fundação | 40h | 20% |
| 2. Frontend Base | 50h | 25% |
| 3. Core Features | 60h | 30% |
| 4. Real-time | 30h | 15% |
| 5. Avançado | 20h | 10% |
| **Total** | **200h** | 100% |

---

## Dependências Novas

```toml
# api/Cargo.toml
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "fs"] }
tokio-tungstenite = "0.26"

# web/Cargo.toml
leptos = { version = "0.7", features = ["csr"] }
leptos_router = "0.7"
leptos_meta = "0.7"
gloo-net = "0.6"  # HTTP client WASM
wasm-bindgen = "0.2"
```

---

## Checklist de Implementação

- [ ] Setup Workspace Cargo e extrair core crate
- [ ] Implementar API REST Axum com endpoints CRUD
- [ ] Adicionar WebSocket para updates real-time
- [ ] Setup projeto Leptos com SSR e Trunk
- [ ] Criar sistema de estilo terminal/hacker
- [ ] Implementar layout principal com sidebar e routing
- [ ] Criar página Dashboard com métricas e ações rápidas
- [ ] Implementar lista de chats com filtros e paginação
- [ ] Criar visualizador de chat estilo terminal
- [ ] Painel de controle com botões Sync/Backup/Restore/Reset
- [ ] Integrar WebSocket no frontend para updates automáticos
- [ ] Implementar busca full-text com highlights
- [ ] Export em lote (ZIP/Markdown)
- [ ] Página de configurações editáveis

---

## Notas de Design

### Terminal Aesthetic
- Cores: Verde neon (#00ff41) em fundo preto (#0a0a0a)
- Fontes monoespaçadas obrigatórias
- Efeitos visuais: scanlines, glow, cursor piscante
- Animações suaves mas discretas
- Feedback visual imediato para todas as ações

### UX Principles
- Zero configuração inicial (só abrir e usar)
- Feedback em tempo real de todas as operações
- Confirmação clara para ações destrutivas
- Logs visíveis de operações em andamento
- Performance: carregamento rápido, scroll suave

---

**Criado em**: 2025-01-XX  
**Estimativa Total**: 200 horas  
**Status**: Planejado (não iniciado)

