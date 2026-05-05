# 🤖 MyBot - Production-Grade Microservices Workspace

A high-performance Discord + LINE bot system refactored into a Rust microservices architecture with gRPC inter-service communication.

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Discord / LINE                         │
│  (Users interact via Discord servers & LINE messaging)     │
└──────────────────────┬──────────────────────────────────┘
                           │
              ┌────────────┴────────────┐
              │                       │
    ┌─────────▼─────────┐    ┌───▼──────────────┐
    │  gateway-discord  │    │  gateway-line    │
    │  (Discord API)   │    │  (LINE Webhook)  │
    └─────────┬─────────┘    └───┬──────────────┘
              │                       │
              └────────────┬────────────┘
                           │ gRPC (bot_messaging.proto)
              ┌────────────▼────────────┐
              │      ai-core (port 50052)   │
              │  - Groq API integration     │
              │  - Chat/Analyze/Summarize  │
              └────────────┬────────────┘
                           │ gRPC (data_service.proto)
              ┌────────────▼────────────┐
              │   db-manager (port 50051)  │
              │  - MongoDB operations        │
              │  - User history, stats, etc. │
              └────────────────────────────┘
```

## 📁 Directory Structure

```
mybot/
├── Cargo.toml              # Workspace definition
├── README.md
├── deploy_to_screen.sh     # Deploy script for screen session
├── .gitignore
├── proto/
│   ├── bot_messaging.proto  # gRPC: Gateways <-> AI-Core
│   └── data_service.proto   # gRPC: AI-Core <-> DB-Manager
├── services/
│   ├── db-manager/          # Port 50051 — All MongoDB logic
│   ├── ai-core/            # Port 50052 — All Groq API logic
│   ├── gateway-discord/     # Discord bot handlers
│   └── gateway-line/       # LINE bot webhook
└── target/                 # Build artifacts (gitignored)
```

## 🚀 Services

### `db-manager` (Port 50051)
All MongoDB operations. Handles:
- User chat history (read/write/delete)
- User gossip summaries
- User statistics (message count, rude/lewd scores)
- Economy system (wallet, earnings)
- RPG system (player classes, stats)
- Config storage

### `ai-core` (Port 50052)
All AI/Groq logic. Handles:
- `Chat` — Generate AI replies with history context
- `Analyze` — Analyze text for rudeness/lewdness (0-10 scale)
- `SummarizeGossip` — Update user personality summaries
- Proxies DB operations to `db-manager`

### `gateway-discord`
Discord bot handlers (no AI logic, no DB logic):
- **ต๊ะ** — Teenager with a sharp tongue (Tah)
- **อาวัง** — Horny older man (Wang)
- **เจ๊มุ่ง** — Horny woman (Mung)
- **เสี่ยหนู** — Rich politician (Nutin)
- **โยฮัน** — Dark boss (Johan)
- **ผู้คุมกฎ** — Judge (stat tracking)
- **สารวัตรแจ๊ะ** — Inspector (economy/RPG + slash commands)
- **WelcomeBot** — The Coolest Welcome Experience in the World (4-message cinematic)

### `gateway-line`
LINE messaging bot (webhook-based):
- Listens on `http://0.0.0.0:8080/webhook`
- Forwards to `ai-core` via gRPC

## 🛠️ Prerequisites

- **Rust** (edition 2021) — `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **MongoDB** — running on `localhost:27017`
- **Groq API Key** — get from [console.groq.com](https://console.groq.com)
- **Discord Bot Tokens** — create at [discord.com/developers](https://discord.com/developers)
- **LINE Channel Token** — from [developers.line.me](https://developers.line.me)
- **screen** — `apt install screen`

## 🔧 Environment Variables

Each service uses its own `.env` file:

### `services/db-manager/.env`
```env
MONGODB_URI=mongodb://localhost:27017
DATA_SERVICE_ADDR=0.0.0.0:50051
```

### `services/ai-core/.env`
```env
GROQ_API_KEY=your_groq_api_key_here
DATA_SERVICE_ADDR=http://0.0.0.0:50051
BOT_MESSAGING_ADDR=0.0.0.0:50052
```

### `services/gateway-discord/.env`
```env
DISCORD_BOT_TOKEN=your_token_here
DISCORD_BOT_TOKEN_2=your_token_2_here
DISCORD_BOT_TOKEN_3=your_token_3_here
DISCORD_BOT_TOKEN_4=your_token_4_here
DISCORD_BOT_TOKEN_5=your_token_5_here
DISCORD_BOT_TOKEN_6=your_token_6_here
DISCORD_BOT_TOKEN_7=your_token_7_here
DISCORD_BOT_TOKEN_8=your_token_8_here
DISCORD_BOT_TOKEN_9=your_token_9_here
BOT_MESSAGING_ADDR=http://0.0.0.0:50052
```

### `services/gateway-line/.env`
```env
LINE_TOKEN=your_line_token_here
LINE_SECRET=your_line_secret_here
LINE_PORT=8080
BOT_MESSAGING_ADDR=http://0.0.0.0:50052
```

> **⚠️ IMPORTANT:** All `.env` files are gitignored. Never commit secrets!

## 🏗️ Building

```bash
# Build entire workspace in release mode
cargo build --release

# Or build individual services
cargo build --release -p db-manager
cargo build --release -p ai-core
cargo build --release -p gateway-discord
cargo build --release -p gateway-line
```

Binaries will be in `target/release/`.

## 🚀 Deploying with Screen

The `deploy_to_screen.sh` script automates launching all 4 services in a screen session.

```bash
# First, create a screen session named '27088'
screen -S 27088

# (Inside screen, detach with Ctrl+A then d)

# Run the deploy script
chmod +x deploy_to_screen.sh
./deploy_to_screen.sh
```

This will:
1. `cargo build --release` (ensure latest binaries)
2. Create 4 screen windows: `db-manager`, `ai-core`, `gateway-discord`, `gateway-line`
3. Start each service in order (with 3s delays between them)

### Managing the Screen Session

```bash
# Attach to the session
screen -r 27088

# Inside screen:
#   Switch windows:  Ctrl+A, then 0-3
#   Detach:         Ctrl+A, then d
#   Kill window:     Ctrl+A, then k

# List all windows
screen -S 27088 -Q windows

# Kill entire session
screen -S 27088 -X quit
```

## 🔌 gRPC Proto Files

### `proto/bot_messaging.proto`
Defines communication between **Gateways** and **AI-Core**:
- `Chat` — Send user message, get AI reply + updated history
- `Analyze` — Analyze text for rudeness/lewdness scores
- `SummarizeGossip` — Update user gossip summary

### `proto/data_service.proto`
Defines communication between **AI-Core** and **DB-Manager**:
- `GetHistory` / `UpdateHistory` / `DeleteHistory`
- `GetGossip` / `UpdateGossip`
- `GetUserStat` / `UpdateUserStat`
- `GetEconomy` / `UpsertEconomy`
- `GetRpg` / `UpsertRpg`
- `GetConfig` / `SetConfig`
- `FindAll`

## 🎮 Bot Characters

| Bot | Personality | Trigger |
|-----|-------------|---------|
| **ต๊ะ** (Tah) | Teenager with sharp tongue, rude, disrespectful | Channel-based |
| **อาวัง** (Wang) | Horny older man, seductive, womanizer | Channel-based |
| **เจ๊มุ่ง** (Mung) | Horny woman, aggressive, loves men | Channel-based |
| **เสี่ยหนู** (Nutin) | Rich politician, brags about wealth | Channel-based |
| **โยฮัน** (Johan) | Dark boss, mysterious, intimidating | Channel-based |
| **ผู้คุมกฎ** (Judge) | Tracks stats, analyzes messages | All channels |
| **สารวัตรแจ๊ะ** (Inspector) | Economy/RPG, `/balance`, `/work` | Slash commands |

## 🎊 Welcome Bot — "The Coolest in the World"

When a new member joins, they get a **4-message cinematic experience**:
1. **The Grand Entrance** — Dramatic intro with member count, banner, timestamp
2. **The Law & Legends** — Server rules + all bot characters introduction
3. **The Ultimate Challenge** — 4 sub-quests + slash command guide
4. **The Final Cinematic** — "Beginning of a Legend" with RNG status

## 📝 Adding a New Bot

1. Add a new handler struct in `services/gateway-discord/src/main.rs`
2. Add the bot's token to `services/gateway-discord/.env`
3. Add a `spawn_bot()` call in `main()`
4. Rebuild: `cargo build --release -p gateway-discord`

## 🔧 Development Tips

```bash
# Check compilation (fast, no binary)
cargo check

# Run a single service manually
cd target/release
./db-manager
./ai-core
./gateway-discord
./gateway-line

# View logs (if attached to screen)
# Just look at the screen windows

# Restart a single service
# Kill its screen window (Ctrl+A, k) and re-run the binary
```

## 📊 License

MIT (or your chosen license)

---

**Built with:** Rust 🦀 + gRPC 🔌 + MongoDB 🍃 + Groq AI 🤖 + Serenity (Discord) + Axum (LINE)
