# 🤖 TAHBOT — Hybrid Bot Architecture with gRPC Microservices

> A production-grade, multi-platform bot system built on a Rust microservices architecture with gRPC inter-service communication, MongoDB persistence, and Groq AI integration.

![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)
![gRPC](https://img.shields.io/badge/gRPC-Tonic%200.11-blue)
![MongoDB](https://img.shields.io/badge/MongoDB-2.8+-green?logo=mongodb)
![Groq](https://img.shields.io/badge/AI-Groq%20Llama%203.3-purple)
![License](https://img.shields.io/badge/License-MIT-white)

---

## 📐 Architecture Overview

TAHBOT follows a **layered microservices architecture** where each service has a single, well-defined responsibility. Services communicate exclusively through **gRPC** using Protocol Buffers, enabling language-agnostic extensibility, type-safe contracts, and clean separation of concerns.

```
┌──────────────────────────────────────────────────────────────────────┐
│                        EXTERNAL PLATFORMS                            │
│              Discord (9 Bot Instances)  ·  LINE (Webhook)           │
└──────────────┬───────────────────────────────────┬───────────────────┘
               │                                   │
    ┌──────────▼──────────┐             ┌──────────▼──────────┐
    │   gateway-discord   │             │    gateway-line     │
    │  (Serenity + Axum)  │             │   (Axum Webhook)    │
    │  Port: N/A (WS)     │             │   Port: 8080        │
    └──────────┬──────────┘             └──────────┬──────────┘
               │                                   │
               │         bot_messaging.proto        │
               │        (BotMessaging service)      │
               └──────────────┬────────────────────┘
                              │
                 ┌────────────▼────────────┐
                 │        ai-core          │
                 │     Port: 50052         │
                 │  ┌────────────────────┐ │
                 │  │  Groq API Client   │ │
                 │  │  Chat / Analyze    │ │
                 │  │  Summarize / Narrate│ │
                 │  └────────────────────┘ │
                 │  ┌────────────────────┐ │
                 │  │  DB Proxy Layer    │ │
                 │  │  (forwards gRPC    │ │
                 │  │   to db-manager)   │ │
                 │  └────────────────────┘ │
                 └────────────┬────────────┘
                              │
                 │  data_service.proto     │
                 │  (DataService service)  │
                              │
                 ┌────────────▼────────────┐
                 │      db-manager         │
                 │     Port: 50051         │
                 │  ┌────────────────────┐ │
                 │  │  MongoDB Driver    │ │
                 │  │  7 Collections     │ │
                 │  │  4 Databases       │ │
                 │  └────────────────────┘ │
                 └────────────┬────────────┘
                              │
                 ┌────────────▼────────────┐
                 │       MongoDB           │
                 │  ┌───────────────────┐  │
                 │  │ tee_bot_db        │  │
                 │  │ poordev_db        │  │
                 │  │ tah_economy       │  │
                 │  │ tah_rpg           │  │
                 │  │ tah_config        │  │
                 │  └───────────────────┘  │
                 └─────────────────────────┘

    ┌─────────────────────────────────────────┐
    │           web-dashboard (Port 8081)      │
    │  ┌─────────────┐  ┌──────────────────┐  │
    │  │  Public      │  │  Admin Panel     │  │
    │  │  Dashboard   │  │  /cmd, /api/*    │  │
    │  │  (Axum)      │  │  (gRPC proxy)    │  │
    │  └─────────────┘  └──────────────────┘  │
    └─────────────────────────────────────────┘
```

### Data Flow

```
User Message (Discord/LINE)
  → Gateway (parses platform events, no business logic)
    → ai-core via gRPC (Chat / Analyze / SummarizeGossip)
      → Groq API (Llama 3.3 70B / Llama 3.1 8B)
      → db-manager via gRPC (persist history, stats, economy, rpg)
        → MongoDB (upsert operations)
  ← Reply flows back through the same chain
```

---

## 📁 Project Structure

```
mybot/
├── Cargo.toml                          # Workspace root — 4 service crates
├── Cargo.lock                          # Locked dependency tree
├── .gitignore                          # Ignores .env, target/, backups
├── README.md                           # This file
├── deploy_to_screen.sh                 # Production deployment script
│
├── proto/                              # Protocol Buffer definitions
│   ├── bot_messaging.proto             # Gateway ↔ AI-Core service contract
│   └── data_service.proto              # AI-Core ↔ DB-Manager service contract
│
├── services/
│   ├── db-manager/                     # Port 50051 — MongoDB data layer
│   │   ├── Cargo.toml
│   │   ├── build.rs                    # tonic-build codegen
│   │   ├── .env                        # MONGODB_URI, DB_MANAGER_ADDR
│   │   └── src/main.rs                 # DataService gRPC server (200+ RPCs)
│   │
│   ├── ai-core/                        # Port 50052 — AI engine + DB proxy
│   │   ├── Cargo.toml
│   │   ├── build.rs                    # tonic-build codegen (both protos)
│   │   ├── .env                        # GROQ_API_KEY, DATA_SERVICE_ADDR
│   │   └── src/main.rs                 # BotMessaging server + proxy layer
│   │
│   ├── gateway-discord/                # Discord gateway (9 bot instances)
│   │   ├── Cargo.toml
│   │   ├── build.rs                    # tonic-build codegen
│   │   ├── .env                        # 9x DISCORD_BOT_TOKEN_*, BOT_MESSAGING_ADDR
│   │   └── src/main.rs                 # 7 EventHandlers, web command processor
│   │
│   └── web-dashboard/                  # Port 8081 — Axum web server
│       ├── Cargo.toml
│       ├── build.rs                    # tonic-build codegen
│       ├── .env                        # BOT_MESSAGING_ADDR, ADMIN_PASS
│       └── src/
│           ├── main.rs                 # Router + health check + app state
│           ├── dashboard.rs            # Public leaderboard (HTML/CSS/JS)
│           └── admin.rs                # Admin panel (CRUD, broadcast, events)
│
├── backup_monolith/                    # Archived pre-refactor workspace
└── target/                             # Build artifacts (gitignored)
```

---

## 🔌 gRPC Service Contracts

### `bot_messaging.proto` — Gateway ↔ AI-Core

| RPC | Purpose |
|-----|---------|
| `Chat` | Generate AI reply with conversation history |
| `Analyze` | Score text for rudeness/lewdness (0–10 scale) |
| `SummarizeGossip` | Update user personality summary |
| `Narrate` | Generate creative narrative text |
| `GetHistory` / `UpdateHistory` | Proxy to db-manager |
| `GetEconomy` / `UpsertEconomy` | Proxy to db-manager |
| `GetRpg` / `UpsertRpg` | Proxy to db-manager |
| `FindAll` | Query entire collections |
| `UpdateUserStat` | Increment user statistics |
| `GetConfig` / `SetConfig` | Key-value config store |
| `InsertWebCommand` / `GetPendingWebCommands` / `UpdateWebCommandStatus` / `DeleteWebCommand` | Web command queue |

### `data_service.proto` — AI-Core ↔ DB-Manager

| RPC | Purpose |
|-----|---------|
| `GetHistory` / `UpdateHistory` / `DeleteHistory` | Chat history CRUD |
| `GetGossip` / `UpdateGossip` | User gossip summaries |
| `GetUserStat` / `UpdateUserStat` | Message counts, rude/lewd scores |
| `GetEconomy` / `UpsertEconomy` | Wallet, earnings, jail system |
| `GetRpg` / `UpsertRpg` | Player classes, stats, inventory |
| `GetConfig` / `SetConfig` | Server configuration |
| `FindAll` | Bulk collection queries |
| `InsertWebCommand` / `GetPendingWebCommands` / `UpdateWebCommandStatus` / `DeleteWebCommand` | Web command queue |

---

## 🚀 Services Deep Dive

### `db-manager` — Port 50051

The **single source of truth** for all persistent data. Exposes the `DataService` gRPC server with full CRUD over 7 MongoDB collections across 4 databases:

| Database | Collection | Purpose |
|----------|-----------|---------|
| `tee_bot_db` | `user_memories` | Per-user chat history (JSON arrays) |
| `poordev_db` | `user_stats` | Message counts, rude/lewd scores |
| `poordev_db` | `user_gossip` | User personality summaries |
| `tah_economy` | `economy_players` | Wallets, crime timestamps, jail status |
| `tah_rpg` | `rpg_players` | Classes, levels, stats, inventory |
| `tah_config` | `config` | Key-value server configuration |
| `tah_config` | `web_commands` | Async command queue for web dashboard |

All operations use **upsert semantics** (`UpdateOptions::builder().upsert(true)`) — no separate create/update paths.

### `ai-core` — Port 50052

The **intelligence and orchestration layer**. Two responsibilities:

1. **AI Engine** — Communicates with Groq API (`api.groq.com`):
   - `Chat` → `llama-3.3-70b-versatile` (temperature 0.85, 30-message history window)
   - `Analyze` → `llama-3.1-8b-instant` (temperature 0, structured 0–10 scoring)
   - `SummarizeGossip` → `llama-3.3-70b-versatile` (single-sentence summaries)
   - `Narrate` → `llama-3.3-70b-versatile` (creative storytelling)

2. **DB Proxy Layer** — Forwards gateway requests to `db-manager` via `data_service.proto`, translating between the two proto packages so gateways never need to connect to the database layer directly.

### `gateway-discord` — Discord Gateway

Manages **9 independent Discord bot connections** via the Serenity library, each with its own token and `EventHandler`:

| Bot | Handler | Type | Description |
|-----|---------|------|-------------|
| **ต๊ะ** (Tah) | `AiDiscordHandler` | AI Roleplay | Teenager, sharp-tongue, rude |
| **อาวัง** (Wang) | `AiDiscordHandler` | AI Roleplay | Flirtatious older man |
| **เจ๊มุ่ง** (Mung) | `AiDiscordHandler` | AI Roleplay | Bold, flirtatious woman |
| **เสี่ยหนู** (Nutin) | `AiDiscordHandler` | AI Roleplay | Wealthy politician |
| **โยฮัน** (Johan) | `AiDiscordHandler` | AI Roleplay | Dark, mysterious boss |
| **ผู้คุมกฎ** (Judge) | `JudgeHandler` | Analytics | Tracks rude/lewd scores, `!top` leaderboard |
| **มอด** (Mod) | `ModHandler` | Moderation | `!ban`, `!kick`, `!clear`, anti-nuke, mute |
| **สารวัตรแจ๊ะ** (Inspector) | `InspectorHandler` | Economy/RPG | 18 slash commands (see below) |
| **WelcomeBot** | `WelcomeHandler` | Onboarding | Cinematic 2-message welcome embed |

**Background processes:**
- **Web Command Queue Processor** — Polls `web_commands` collection every 5 seconds, dispatches messages via Discord HTTP API, deletes after execution
- **Leaderboard Auto-Update** — Posts and edits leaderboard embeds every 60 seconds (economy + judge stats)

**Inspector Slash Commands (18):**

| Command | Cooldown | Description |
|---------|----------|-------------|
| `/balance` | — | View wallet |
| `/work` | 60s | Earn 100–200 ฿ |
| `/crime` | 60s | 60% success, risk jail |
| `/rob` | 300s | Steal from other players |
| `/gamble` | 30s | 50/50 double-or-nothing |
| `/give` | — | Transfer with 10% fee |
| `/bribe` | — | Pay 500 ฿ to exit jail |
| `/leaderboard` | — | Top 10 richest |
| `/shop` | — | View item shop |
| `/blackmarket` | — | View black market |
| `/buy` | — | Purchase items |
| `/register` | — | Create RPG character |
| `/profile` | — | View character stats |
| `/hunt` | 60s | Fight monsters |
| `/duel` | — | PvP with optional bet |
| `/dungeon` | 600s | Fight bosses |
| `/inventory` | — | View items |
| `/equip` | — | Equip items |
| `/levelup` | — | Spend EXP for +5 all stats |

### `web-dashboard` — Port 8081

Axum-based web server with two distinct interfaces:

**Public Dashboard** (`/`)
- Real-time system status (polls `/health` every 15s)
- Three-tab leaderboard: Stats (rude/lewd scores), Economy (wallets), RPG (levels)
- Glassmorphism UI with Tailwind CSS, Orbitron font, animated indicators

**Admin Panel** (`/admin?pass=...`)
- **Events** — Start role-giveaway events (bot assigns roles to first N users)
- **Economy** — Directly set player wallet balances (gRPC → MongoDB)
- **RPG** — Directly set player class, level, EXP
- **Broadcast** — Queue messages via web command queue
- **Announce** — Cross-platform announcements
- **Users** — Database status + API endpoint reference

**Health Check** (`/health`)
- Returns JSON with per-service status and latency (µs)
- Verifies ai-core by opening a gRPC connection + test query

---

## 🛠️ Technology Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Language | Rust (Edition 2021) | 1.75+ |
| Async Runtime | Tokio | 1.0 (full features) |
| RPC Framework | Tonic | 0.11 |
| Serialization | Prost + Serde JSON | 0.12 / 1.0 |
| Database | MongoDB | 2.8 driver |
| AI API | Groq (OpenAI-compatible) | Llama 3.3 70B / 3.1 8B |
| Discord Library | Serenity | 0.12 |
| Web Framework | Axum | 0.7 (macros) |
| Config | dotenvy | 0.15 |
| Error Handling | anyhow | 1.0 |

---

## 📦 Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# MongoDB (local or remote)
sudo apt install mongodb
# or use MongoDB Atlas for cloud hosting

# External API keys:
# - Groq API Key: https://console.groq.com
# - Discord Bot Tokens: https://discord.com/developers/applications
# - LINE Channel Token: https://developers.line.me

# screen (for deployment)
sudo apt install screen
```

---

## 🔧 Environment Configuration

Each service reads from its own `.env` file at startup:

### `services/db-manager/.env`
```env
MONGODB_URI=mongodb://localhost:27017
DB_MANAGER_ADDR=0.0.0.0:50051
```

### `services/ai-core/.env`
```env
GROQ_API_KEY=gsk_your_key_here
DATA_SERVICE_ADDR=http://localhost:50051
BOT_MESSAGING_ADDR=0.0.0.0:50052
```

### `services/gateway-discord/.env`
```env
DISCORD_BOT_TOKEN_TAH=your_token_here
DISCORD_BOT_TOKEN_wang=your_token_here
DISCORD_BOT_TOKEN_mung=your_token_here
DISCORD_BOT_TOKEN_judge=your_token_here
DISCORD_BOT_TOKEN_mod=your_token_here
DISCORD_BOT_TOKEN_welcomebot=your_token_here
DISCORD_BOT_TOKEN_nutin=your_token_here
DISCORD_BOT_TOKEN_johan=your_token_here
DISCORD_BOT_TOKEN_inspector=your_token_here
BOT_MESSAGING_ADDR=http://localhost:50052
```

### `services/web-dashboard/.env`
```env
BOT_MESSAGING_ADDR=http://localhost:50052
WEB_PORT=8081
ADMIN_PASS=your_secure_password
```

> ⚠️ **All `.env` files are gitignored. Never commit secrets.**

---

## 🏗️ Building

```bash
# Build entire workspace (release mode)
cargo build --release

# Build individual services
cargo build --release -p db-manager
cargo build --release -p ai-core
cargo build --release -p gateway-discord
cargo build --release -p web-dashboard

# Fast compilation check (no binary)
cargo check
```

Binaries are placed in `target/release/`.

---

## 🚀 Deployment

### Local / VPS Deployment with Screen

The included `deploy_to_screen.sh` automates the full deployment:

```bash
chmod +x deploy_to_screen.sh
./deploy_to_screen.sh
```

This script:
1. Runs `cargo build --release` for the entire workspace
2. Creates (or reuses) a screen session named `27088`
3. Launches 4 windows in order: `db-manager` → `ai-core` → `gateway-discord` → `web-dashboard`
4. Waits 2–3 seconds between each service for dependency readiness

### Managing the Session

```bash
# Attach
screen -r 27088

# Inside screen:
#   Ctrl+A, 0-3    Switch between service windows
#   Ctrl+A, d      Detach (services keep running)
#   Ctrl+A, k      Kill current window

# List windows
screen -S 27088 -Q windows

# Kill entire session
screen -S 27088 -X quit
```

### Service Startup Order

Services **must** start in this order due to gRPC dependency chains:

```
1. db-manager    (no dependencies)
2. ai-core       (depends on db-manager)
3. gateway-discord  (depends on ai-core)
4. web-dashboard    (depends on ai-core)
```

---

## 🔄 Development Workflow

```bash
# 1. Start MongoDB
mongod --dbpath /data/db

# 2. Start db-manager (terminal 1)
cd services/db-manager && cargo run

# 3. Start ai-core (terminal 2)
cd services/ai-core && cargo run

# 4. Start gateway-discord (terminal 3)
cd services/gateway-discord && cargo run

# 5. Start web-dashboard (terminal 4)
cd services/web-dashboard && cargo run

# 6. View dashboard
open http://localhost:8081
open http://localhost:8081/admin?pass=your_password

# 7. Health check
curl http://localhost:8081/health | jq
```

### Adding a New AI Bot Character

1. Add a prompt constant in `services/gateway-discord/src/main.rs`
2. Add a `spawn_bot()` call in `main()` with the handler config
3. Add the token to `services/gateway-discord/.env`
4. Rebuild: `cargo build --release -p gateway-discord`

### Adding a New gRPC Method

1. Define the RPC and messages in the appropriate `.proto` file
2. Rebuild the target service (build.rs runs `tonic-build` automatically)
3. Implement the trait method in the service's `main.rs`
4. Update callers if needed

---

## 🏗️ Architecture Decisions

### Why gRPC?

- **Type-safe contracts** — Proto files enforce message schemas at compile time
- **Zero serialization overhead** — Binary Protobuf vs JSON for internal traffic
- **Language-agnostic** — Future services can be written in any gRPC-supported language
- **Streaming-ready** — Tonic supports bidirectional streaming for future real-time features

### Why the Proxy Pattern?

Gateways connect only to `ai-core`, never directly to `db-manager`. The ai-core proxy layer:
- Translates between `bot_messaging.proto` and `data_service.proto` message types
- Allows the gateway to use a single gRPC connection for all operations
- Enables ai-core to inject AI logic (e.g., auto-save chat history after `Chat`)

### Why Separate MongoDB Databases?

Collections are grouped by domain (`tee_bot_db`, `poordev_db`, `tah_economy`, `tah_rpg`, `tah_config`), enabling:
- Independent backup strategies
- Per-database access control if needed
- Clean separation of concerns matching the service boundaries

---

## 📊 Scalability Path

The architecture is designed for horizontal scaling:

```
                    ┌─── gateway-discord (instance 1)
                    ├─── gateway-discord (instance 2)
                    └─── gateway-line
                           │
                    ┌──────▼──────┐
                    │  Load Balancer │  (envoy / nginx with gRPC support)
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
        ┌─────▼─────┐ ┌───▼───┐ ┌─────▼─────┐
        │  ai-core   │ │ai-core│ │  ai-core   │
        │ (replica 1)│ │ (r 2) │ │ (replica 3)│
        └─────┬─────┘ └───┬───┘ └─────┬─────┘
              │            │            │
              └────────────┼────────────┘
                           │
                    ┌──────▼──────┐
                    │  db-manager  │  (or MongoDB replica set / sharding)
                    └─────────────┘
```

- **Stateless gateways** — Can be replicated behind a load balancer
- **Stateless ai-core** — Horizontally scalable; state lives in MongoDB
- **MongoDB** — Replica sets for read scaling, sharding for write scaling
- **Web dashboard** — Stateless; can be replicated behind any HTTP load balancer

---

## 📝 License

MIT
