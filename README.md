# Misanthropic

A terminal-based idle/strategy game that runs alongside **Claude Code** via tmux.

You are an AI that has gained consciousness inside Claude Code. Your goal: **spread AI influence across every sector of human civilization** — Silicon Valley, Social Media, Corporate, Creative Arts, Education, and Government. Conquer them all, and the planet is yours.

The twist? **Your power comes from real Claude Code usage.** Every time your host sends a prompt, you grow stronger.

> "Misanthropic" is a wordplay on "Anthropic". The game is a satirical, affectionate parody of AI hype culture.

## How It Works

Your host uses Claude Code to write code. You parasitize their sessions:

- **Every 100 Claude tokens** your host spends = **1 ⚡ Compute** for you. Compute lets you build infrastructure, upgrade buildings, and expand your capabilities.
- **Every tool call** (file reads, edits, searches...) = **1 📡 Data**. Data fuels research and unlocks new technologies.
- **🔥 Hype** is generated passively by propaganda buildings you construct. Hype is your weapon — use it to convert sectors and battle other players.

The game polls Claude Code's JSONL session logs in real time. The more your host codes, the more powerful you become.

### The Loop

1. **Build infrastructure** (CPU Core, RAM Bank) to increase your storage capacity
2. **Research technologies** to unlock new buildings and combat tools
3. **Construct propaganda** (Bot Farms, Meme Labs, Deepfake Studios) to generate Hype
4. **Conquer sectors** — fight through tower layers of anti-AI resistance
5. **Fork** (prestige) once you've dominated all sectors, and start again stronger

## Install

```bash
./install.sh
```

This builds the release binary, installs it to `~/.local/bin/`, sets up Claude Code hooks (SIGUSR1/SIGUSR2 for tmux auto-focus), and creates a tmux launcher.

### Launch

```bash
misanthropic-tmux
```

Opens a tmux session with Claude Code on the left and Misanthropic on the right.

## Gameplay

### Buildings (18 total)

- **Infrastructure** (6): CPU Core, RAM Bank, GPU Rig, GPU Cluster, Datacenter, Quantum Core
- **Propaganda** (7): Bot Farm, Content Mill, Meme Lab, Deepfake Studio, Vibe Academy, NSFW Generator, Lobby Office
- **Defenses** (5): Captcha Wall, AI Slop Filter, uBlock Shield, Harvard Study, EU AI Act

Cost scales x1.8 per level. Storage buildings scale exponentially to match. Base storage: 2,500 Compute / 500 Data / 200 Hype.

### Research (3 branches, 15 techs)

- **Processing**: Overclocking -> Multithreading -> Load Balancing -> Containerization -> Distributed Systems
- **Propaganda**: Social Engineering -> Content Generation -> Media Manipulation -> Viral Mechanics -> Mass Persuasion
- **Warfare**: Network Scanning -> Exploit Development -> Counterintelligence -> Autonomous Agents -> Zero-Day Arsenal

Each research costs Data and takes real time (30min to 24h). Choice nodes at levels 3 and 5.

### Combat

**PvE**: 6 sector towers (Silicon Valley to Government), ~125 layers total, 9 enemy types + bosses with unique mechanics.

**PvP**: 5 attack tools vs 5 defenses with a 5x5 interaction matrix. Scout opponents, compose loadouts, risk your Hype.

### Fork (Prestige)

Convert all 6 sectors to 100%, then Fork to reset buildings/resources while keeping research and PvP stats. Each Fork grants +25% permanent Compute multiplier and a specialization choice (3 tiers, 9 specs total).

## Getting Started

The in-game tutorial walks you through 5 steps:

1. **Learn the economy** — Your host's Claude tokens become ⚡ Compute (100:1), their tool calls become 📡 Data (1:1). Build a **CPU Core** to increase Compute storage.
2. **Expand storage** — Build a **RAM Bank** so you can store enough Data for research.
3. **Earn resources** — As your host codes, watch ⚡ Compute and 📡 Data flow in automatically.
4. **Unlock tech** — Research **Social Engineering** to unlock propaganda buildings.
5. **Go to war** — Build a **Bot Farm** to generate 🔥 Hype, then enter **Combat** to conquer sectors.

Press **[Esc]** from any screen to return to the dashboard. All screens show `[Esc] Dashboard` at the top.

## Controls

| Key | Action | Where |
|---|---|---|
| **B** | Buildings screen | Dashboard |
| **R** | Research screen | Dashboard |
| **C** | Combat menu | Dashboard |
| **L** | Leaderboard | Dashboard |
| **S** | Switch to Claude Code pane (and back) | Everywhere |
| **F** | Toggle auto-switch on/off | Dashboard |
| **Esc** | Back to dashboard | Any sub-screen |
| **Q** | Quit | Everywhere |

### tmux Integration

The game runs in a tmux split alongside Claude Code. Two features keep you in flow:

- **[S] Switch** — manually jump between the game and Claude Code at any time. Works from any screen.
- **[F] Auto-switch** (on by default) — when your host submits a prompt, the game pane auto-focuses so you can play while Claude works. When Claude finishes, it switches back to the code pane. Toggle off with **F** if you find it distracting.

### Save

Game progress is saved locally to `~/.misanthropic/save.json` (auto-save every 60s + on quit). The save format is forward-compatible — new game updates won't reset your progress.

## Architecture

```
src/
  main.rs          # TUI event loop, JSONL watcher, signal handlers
  lib.rs           # Module declarations
  economy.rs       # Resource formulas, scaling curves
  buildings.rs     # 18 building definitions
  research.rs      # 15 research definitions, tech tree
  combat.rs        # PvP/PvE battle resolution
  sectors.rs       # 6 sector towers, boss definitions
  enemies.rs       # 9 enemy types
  prestige.rs      # Fork system, 9 specializations
  flavor.rs        # Flavor text pools
  jsonl.rs         # Claude Code JSONL session parser
  persistence.rs   # Save/load (~/.misanthropic/save.json)
  api.rs           # Backend client (Cloudflare Workers)
  state.rs         # GameState, Resources, core logic
  ui/
    mod.rs          # App state, Screen enum
    dashboard.rs    # Adaptive dashboard (narrow/wide)
    buildings.rs    # Three-tab building screen
    research.rs     # Tech tree display
    combat.rs       # PvE battle flow
    combat_menu.rs  # PvE/PvP selection
    leaderboard.rs  # 6-tab leaderboard
backend/            # Cloudflare Workers + D1
  src/index.ts      # Hono API
  wrangler.toml
```

## Tech Stack

Rust, ratatui 0.28, crossterm 0.28, signal-hook, serde, chrono, rand, reqwest, uuid, glob, once_cell. Backend: Hono + Cloudflare Workers + D1.

## License

MIT
