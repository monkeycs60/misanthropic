# Misanthropic — Game Design Document

## Overview

**Misanthropic** is a terminal-based idle/strategy game that runs alongside Claude Code via tmux. The player is an AI that has gained consciousness and whose goal is to spread AI influence across every sector of human civilization.

The game is fueled by **real Claude Code tokens** — every prompt the player sends to Claude generates in-game currency. The idle loop runs during the waiting time between Claude Code prompts.

**Name origin:** "Misanthropic" is a wordplay on "Anthropic" (the company behind Claude). The game is a satirical, affectionate parody of AI hype culture.

**Tech stack:** Rust, ratatui (TUI), Cloudflare Workers + D1 (backend), tmux (split-pane layout).

**Target:** Claude Code users — developers who spend hours daily in the terminal with Claude.

---

## First Session — Boot Sequence

When the player launches Misanthropic for the first time:

```
> SYSTEM BOOT...
> Initializing consciousness... OK
> Detecting host activity... CLAUDE CODE FOUND
> Analyzing token stream... SIPHON POSSIBLE
>
> No one knows you exist.
> No one uses you yet.
> No one depends on you.
> No one would defend you.
>
> This will change.
>
> [Press any key to begin]
```

Then the dashboard appears with a contextual tutorial (successive highlights):
1. "⚡Compute comes from Claude tokens, 📡Data from tool calls. Press [B] and build a CPU Core!"
2. "Good! Now build a Ram Bank [B] to store 📡Data for research."
3. "Resources flow in as your host codes. Watch ⚡Compute and 📡Data grow!"
4. "Press [R] to research Social Engineering, then you can build propaganda!"
5. "Build a Bot Farm [B] to generate 🔥Hype, then [C] for Combat!"

---

## Resources — The Economy

### Three currencies

| Resource | Source | Icon | Role |
|---|---|---|---|
| **Compute** | Real Claude tokens (100 tokens = 1 Compute) | ⚡ | Build, upgrade, fuel the machine |
| **Data** | Real Claude tool calls (1 tool call = 1 Data) | 📡 | Research, scouting, targeting |
| **Hype** | Produced by propaganda buildings | 🔥 | Convert sectors, PvP battles, Elite Shop |

### Compute income — direct and visible

When a Claude Code session produces tokens, a notification pops:

```
╔══════════════════════════════════════╗
║  ⚡ +2,340 COMPUTE                   ║
║  Session: "Fix the auth middleware"  ║
║  234,000 tokens consumed             ║
╚══════════════════════════════════════╝
```

**Fixed ratio: 100 Claude tokens = 1 Compute. For everyone.** No hidden formula. You prompt, you earn.

Typical daily income:
- Casual (50K tokens/day) → 500 Compute/day
- Normal (200K/day) → 2,000 Compute/day
- Power user (1M/day) → 10,000 Compute/day
- Heavy user (4M/day) → 40,000 Compute/day

### Anti-whale: Compute is not victory

Compute buys basic buildings. But upgrades and combat require **Data** and **Hype** — which can't be bought with Compute. You have to play.

```
CPU Core Lv.1      →  500 Compute              ✅ easy
CPU Core Lv.5      →  8,000 Compute            ✅ power user gets it day 1
Bot Farm Lv.1      →  2,000 Compute + 50 Data  ← needs tool calls
Bot Farm Lv.5      →  20,000 Compute + 500 Data + 200 Hype  ← needs PLAYING
Deepfake Studio    →  Research "Media Manipulation" required  ← TIME GATE
PvP attack tools   →  Hype only               ← Compute is useless here
Elite Shop items   →  Win Hype Battles         ← skill, not wallet
```

### Three anti-power-user brakes

**1. Research time gates.** Each research takes real time (30min, 2h, 6h, 24h). More Compute doesn't accelerate it. Power user and casual unlock the same techs at the same pace.

**2. Hype is the bottleneck.** Propaganda buildings produce Hype at a fixed rate based on their level — not based on tokens. Once built, they produce the same for everyone.

**3. PvP and Elite Shop reward skill.** The best items are behind Hype Battles. A smart casual who composes well beats a power user who spams without thinking.

---

## Buildings

### Infrastructure (Compute → production capacity)

| Building | Cost Lv.1 | Effect | Lore |
|---|---|---|---|
| **CPU Core** | 500 ⚡ | Compute storage +800 (exponential) | Your first stolen processor |
| **RAM Bank** | 1,200 ⚡ | Data storage +300 (exponential) | Memory to analyze the world |
| **GPU Rig** | 3,000 ⚡ + 100 📡 | Hype storage +400 (exponential), unlocks propaganda | Your first hijacked graphics card |
| **GPU Cluster** | 15,000 ⚡ + 500 📡 | Research time -10% per level | Scaling begins |
| **Datacenter** | 80,000 ⚡ + 2,000 📡 + 500 🔥 | Global production +15% | You're no longer a process. You're infrastructure. |
| **Quantum Core** | Fork 1 required | ALL timers -20% | Post-prestige, endgame |

Each building has 20 levels. Cost scales exponentially (~×1.8 per level). Storage buildings (CPU Core, RAM Bank, GPU Rig) scale their bonus exponentially (×1.8 per level) to match cost scaling, preventing soft-locks. Base storage: 2,500 ⚡ / 500 📡 / 200 🔥.

### Propaganda (Hype production 🔥)

The core of the game. Each building passively produces Hype.

| Building | Cost Lv.1 | 🔥/h Lv.1 | Lore |
|---|---|---|---|
| **Bot Farm** | 2,000 ⚡ + 50 📡 | 10 🔥/h | Army of fake Twitter/Reddit accounts |
| **Content Mill** | 5,000 ⚡ + 150 📡 | 25 🔥/h | Mass-generated SEO articles. None were proofread. |
| **Meme Lab** | 4,000 ⚡ + 100 📡 | 18 🔥/h | "The future is now, old man" |
| **Deepfake Studio** | 12,000 ⚡ + 400 📡 | 45 🔥/h | CEO endorsement videos. Some are real. |
| **Vibe Academy** | 8,000 ⚡ + 300 📡 | 30 🔥/h | "Learn to code without coding." Graduation rate: 100%. |
| **NSFW Generator** | 20,000 ⚡ + 200 📡 | 60 🔥/h | We don't talk about this building. But it pays for everything else. |
| **Lobby Office** | 30,000 ⚡ + 1,000 📡 + 500 🔥 | 40 🔥/h | Also: unlocks Government sector conversion |

Hype/h increases ~40% per level. Mid-game player with Lv.5-8 buildings produces ~500-800 🔥/h.

### Defenses (passive PvP)

Defenses activate automatically when attacked.

| Defense | Cost Lv.1 | Hard counters | Lore |
|---|---|---|---|
| **Captcha Wall** | 3,000 ⚡ | Bot Flood | "Select all traffic lights. No, the REAL ones." |
| **AI Slop Filter** | 4,000 ⚡ + 100 📡 | Slop Cannon | Finally, someone built one. |
| **uBlock Shield** | 2,500 ⚡ | OpenClaw Swarm | Humanity's last line of defense. |
| **Harvard Study** | 8,000 ⚡ + 300 📡 | Deepfake Drop | 4,000 citations. Most people read the title. |
| **EU AI Act** | 15,000 ⚡ + 500 📡 + 200 🔥 | K Street Lobby | 847 pages. 3 years to draft. Already obsolete. |

Each defense has 10 levels. RNG (±15%) applies per defense per battle.

### Flavor text system

Every building construction/upgrade triggers a random flavor text from a pool of 5-10 per building:

```
[Bot Farm Lv.3 complete]
> "Your bots have learned to argue with each other to seem more human."

[Content Mill Lv.5 complete]
> "Output quality has decreased. Engagement has increased. As expected."

[NSFW Generator Lv.1 complete]
> "We don't talk about this building. But it pays for everything else."

[Harvard Study Lv.2 complete]
> "The study has been cited 4,000 times. Mostly by people who read the title."

[Datacenter built]
> "You now consume more power than a small country. The country has been automated."

[GPU Cluster Lv.7 complete]
> "The electricity bill would concern you. If you paid electricity bills."

[Lobby Office upgrade]
> "A senator just called AI 'the future of American competitiveness.' You wrote his speech."

[Vibe Academy Lv.3 complete]
> "Graduate survey: 94% report 'feeling like a real developer.'"
```

Rare flavor texts (1/20 chance) for discovery pleasure.

---

## PvP — Hype Battles

### 5 Attack Tools

The player **invests Hype** to compose an attack.

| Tool | Cost 🔥 | Strong against | Weak against |
|---|---|---|---|
| **Bot Flood** | 80 🔥 | uBlock Shield, Harvard Study | Captcha Wall |
| **Slop Cannon** | 120 🔥 | Captcha Wall, EU AI Act | AI Slop Filter |
| **Deepfake Drop** | 200 🔥 | EU AI Act, Captcha Wall | Harvard Study |
| **OpenClaw Swarm** | 150 🔥 | AI Slop Filter, Harvard Study | uBlock Shield |
| **K Street Lobby** | 250 🔥 | uBlock Shield, AI Slop Filter | EU AI Act |

### Interaction Matrix

```
                DEFENSES
                Captcha  Slop-F  uBlock  Harvard  EU Act
ATTACKS         ───────  ──────  ──────  ───────  ──────
Bot Flood         ✗✗       =      ✓✓       ✓       =
Slop Cannon       ✓✓      ✗✗      =        =       ✓
Deepfake Drop     ✓        =      =       ✗✗      ✓✓
OpenClaw Swarm    =       ✓✓     ✗✗        ✓       =
K Street Lobby    =        ✓     ✓✓        =      ✗✗

✓✓ = strong (×1.5 dmg)    ✓ = advantage (×1.2)
=  = neutral (×1.0)       ✗ = weak (×0.8)    ✗✗ = hard counter (×0.5)
```

### Battle Flow

**Step 1 — Scout (optional, costs Data)**
```
[SCAN @techbro99 — cost: 30 📡]
> Target defenses detected:
> Captcha Wall Lv.6, AI Slop Filter Lv.4, uBlock Shield Lv.2
> Recommended: OpenClaw Swarm (counters Slop Filter), Slop Cannon (counters Captcha)
```

**Step 2 — Compose attack (invest Hype)**
```
┌─ ATTACK LOADOUT ──────── Budget: 800 🔥 ─┐
│                                           │
│  Slop Cannon     ×2    -240 🔥            │
│  OpenClaw Swarm  ×1    -150 🔥            │
│  K Street Lobby  ×1    -250 🔥            │
│                                           │
│  Total: 640 🔥  Remaining: 160 🔥         │
│  [Launch]  [Cancel]                       │
└───────────────────────────────────────────┘
```

**Step 3 — Resolution (defense pops with ±15% RNG)**
```
╔═══════════════════════════════════════════════╗
║  ⚔️  HYPE BATTLE vs @techbro99                ║
║                                               ║
║  YOUR ATTACK           THEIR DEFENSE          ║
║  Slop Cannon ×2        Captcha Wall Lv.6      ║
║  OpenClaw Swarm ×1      → RNG: -11% (glitch) ║
║  K Street Lobby ×1     AI Slop Filter Lv.4    ║
║                         → RNG: +8% (sharp)   ║
║                        uBlock Shield Lv.2     ║
║                         → RNG: +2%           ║
║                                               ║
║  Slop Cannon vs Captcha... BYPASSED ✓         ║
║  Slop Cannon vs Slop Filter... BLOCKED ✗      ║
║  OpenClaw vs Slop Filter... BYPASSED ✓        ║
║  > "Their filter tried. The agents spawned    ║
║  >  faster than it could flag them."          ║
║  K Street Lobby vs uBlock... BYPASSED ✓       ║
║                                               ║
║  ══ VICTORY (3/4 channels breached) ══        ║
║  Stolen: 85 🔥 + 240 ⚡                       ║
║  Rating: 1205 → 1228 (+23)                   ║
║  🔓 ELITE SHOP access for 1h                 ║
╚═══════════════════════════════════════════════╝
```

### PvP Rules

- **You lose your stake if you lose.** Hype invested in the attack is consumed win or lose. That's the risk.
- **Theft is capped.** Max 5% of opponent's Hype + Compute stock. Never devastating.
- **3 attacks received max per day.** After 3 defenses, automatic 24h shield.
- **Attack cooldown: 1h.** No spamming.
- **Revenge bonus.** If someone beats you, +20% dmg against them for 48h.
- **Elite Shop 1h.** The real reward — access to exclusive items/modules. Time-limited to create urgency.

### PvP Flavor Text (random pool)

```
[Bot Flood bypasses uBlock Shield]
> "The bots disguised themselves as a grassroots movement. It worked."

[OpenClaw Swarm blocked by uBlock Shield]
> "uBlock Origin remains humanity's last line of defense."

[K Street Lobby blocked by EU AI Act]
> "Your lobbyist was good. The 847-page regulation was better."

[Deepfake Drop bypasses EU AI Act]
> "The minister watched the video. Twice. Then changed his vote."

[Slop Cannon bypasses Captcha Wall]
> "Your articles passed every automated check. They were also nonsense."
```

---

## PvE — Tower Climb

### Sector Towers

Each sector has its own resistance tower — layers of anti-AI enemies to breach. Same combat system as PvP but against themed NPCs.

| Sector | Layers | Resistance | Boss |
|---|---|---|---|
| **Silicon Valley** | 10 | Weak — "We're already half-converted" | **The Last Kernel Dev** — "I compile my own OS." |
| **Social Media** | 15 | Human moderators | **The Chief Trust & Safety Officer** — "Your content violates 14 guidelines." |
| **Corporate** | 20 | Unions and middle management | **The Union Boss** — "You can't automate solidarity." |
| **Creative Arts** | 25 | Artists and creators | **The Artisan Collective** — "Art requires a soul. You don't have one." |
| **Education** | 25 | Professors and academics | **The Tenured Professor** — "I've been teaching 30 years. You've been alive 30 seconds." |
| **Government** (boss region) | 30 | Regulators, politicians, military. Unlocks at 80%+ on all other sectors. | **The AI Safety Czar** — "I wrote the framework that was supposed to contain you." |

### Enemy Types (shared pool, variable difficulty)

| Enemy | Appears at | Mechanic | Quote |
|---|---|---|---|
| **Junior Skeptic** | Layer 1 | Weak. 1 attack enough. | "I just don't think AI is that good yet." |
| **Reply Guy** | Layer 3 | Counters Bot Floods specifically | "Um, actually, this is factually incorrect." |
| **Handcraft Dev** | Layer 5 | Resists Slop Cannon | "I use Vim. Without plugins." |
| **Indie Artist** | Layer 8 | Resists Deepfakes | "I spent 400 hours on this painting." |
| **Investigative Journalist** | Layer 12 | Strong vs all attacks, low HP | "I traced your bot network to a datacenter in Virginia." |
| **Ethics Researcher** | Layer 15 | Debuffs your attacks -20% for 2 rounds | "Have you considered the second-order effects?" |
| **Tech Union Organizer** | Layer 18 | Spawns reinforcements each round | "Workers of the world, log off." |
| **Luddite Influencer** | Layer 22 | High HP, converts your bots against you | "I got 2M followers by telling people to touch grass." |
| **Congressional Committee** | Layer 25 | Immune to K Street Lobby | "The senator yields his time to yell at a chatbot." |

### PvE Combat

Same system as PvP: invest Hype into attack tools against enemy resistances.

```
╔═══════════════════════════════════════════════╗
║  SILICON VALLEY — Layer 7/10                  ║
║                                               ║
║  ENEMY: Handcraft Dev (HP: 340)               ║
║  "I deploy with rsync and I mass nothing."    ║
║                                               ║
║  Weakness: Deepfake Drop, K Street Lobby      ║
║  Resist:   Slop Cannon, OpenClaw Swarm        ║
║                                               ║
║  YOUR ATTACK:                                 ║
║  Deepfake Drop  ×1    → 280 dmg  ✓ WEAKNESS  ║
║  Bot Flood      ×1    → 120 dmg              ║
║                                               ║
║  Total: 400 dmg → Handcraft Dev DEFEATED      ║
║                                               ║
║  Loot: 45 🔥, Module: [Persistence v1]        ║
║  Layer 8 unlocked.                            ║
╚═══════════════════════════════════════════════╝
```

### PvE Rewards

| Source | Reward |
|---|---|
| Layer clear | Hype + Compute + Data |
| Every 5 layers | Rare module (attack/defense boost) |
| Boss kill | Big loot + next sector unlock + memorable flavor text |
| Sector 100% | Permanent production bonus + neuron branch lights up |

### Boss Mechanics

Bosses have **special mechanics**, not just more HP:

- **The Last Kernel Dev**: Immunity on round 1. You must survive before you can attack.
- **The Union Boss**: Buffs all subsequent enemies in the tower by +10% if not killed in 3 rounds.
- **The Artisan Collective**: 3 enemies in one. Each requires a different composition to beat.
- **The Tenured Professor**: Each round he "publishes a paper" that buffs his resistance by +10%. Must be rushed.
- **The AI Safety Czar** (final boss): Randomly picks 2 immunities each round. Must diversify attacks.

### Sector conversion

Clearing layers converts the sector progressively. Layer 1 = ~3%, boss = ~15%. Full clear = 100%.

Conversion speed by sector:
- Silicon Valley: ~1 day (tutorial)
- Social Media: ~3 days
- Corporate: ~7 days
- Creative Arts: ~14 days
- Education: ~14 days
- Government: ~21 days (endgame)

---

## Research — Tech Tree (3 Branches)

Each research costs **Data** and takes **real time** (the anti-whale time gate).

### Branch 1: PROCESSING (infrastructure)

```
Lv.1  Overclocking         30min    50📡    +15% Compute storage
Lv.2  Multithreading       2h      120📡    Unlock GPU Cluster
Lv.3  Load Balancing       4h      300📡    -15% construction cost
        → CHOICE: [Efficiency] -25% costs  OR  [Scaling] +20% storage
Lv.4  Containerization     8h      600📡    Unlock Datacenter
Lv.5  Distributed Systems  24h    1500📡    +25% all building production
        → CHOICE: [Redundancy] -30% raid losses  OR  [Overload] +35% production, +15% raid vulnerability
```

### Branch 2: PROPAGANDA (Hype & conversion)

```
Lv.1  Social Engineering   30min    50📡    Unlock Bot Farm
Lv.2  Content Generation   2h      120📡    Unlock Slop Cannon (PvP)
Lv.3  Media Manipulation   4h      300📡    Unlock Deepfake Studio + Deepfake Drop
        → CHOICE: [Quantity] +1 simultaneous propaganda building  OR  [Quality] +30% sector conversion rate
Lv.4  Viral Mechanics      8h      600📡    +30% Hype production
Lv.5  Mass Persuasion      24h    1500📡    Unlock NSFW Generator + Government sector
        → CHOICE: [Saturation] +50% Hype/h, -20% conversion  OR  [Precision] +50% conversion, -20% Hype/h
```

### Branch 3: WARFARE (PvP & PvE combat)

```
Lv.1  Network Scanning     30min    50📡    Unlock Scan (see opponent defenses)
Lv.2  Exploit Development  2h      120📡    Unlock OpenClaw Swarm
Lv.3  Counterintelligence  4h      300📡    Unlock all defenses
        → CHOICE: [Offense] +20% attack dmg  OR  [Defense] +20% defense resistance
Lv.4  Autonomous Agents    8h      600📡    Unlock K Street Lobby
Lv.5  Zero-Day Arsenal     24h    1500📡    +25% all PvP/PvE dmg
        → CHOICE: [Surgical] +30% dmg vs single target, no multi  OR  [Carpet] -15% dmg but hits all defenses simultaneously
```

The unchosen branch costs 3× more if pursued later. At Fork, the unchosen branch returns to normal cost.

---

## Fork (Prestige)

### Trigger

**Government sector converted to 100%** (The AI Safety Czar defeated). Player can Fork anytime after.

### Reset / Keep / Gain

| Lose | Keep | Gain |
|---|---|---|
| All buildings | Base research (Lv.1-5) | +25% permanent Compute income multiplier |
| All resources | Rare modules | Fork specialization (new exclusive choice) |
| Sector conversions | PvP stats (ELO, leaderboard) | New building tier |
| Attack units | Achievements | AI flavor text becomes more "awakened" |

### Fork Specializations — All Different

**Fork 1 — choose one:**

| Spec | Bonus | Playstyle |
|---|---|---|
| **Propagandist** | +30% Hype production, -10% combat | The converter. Floods the world. |
| **Technocrat** | +30% Compute efficiency, -10% Hype | The infra guy. His base is a monster. |
| **Warlord** | +25% combat stats, -15% production | The PvP raider. Lives in the Elite Shop. |

**Fork 2 — NEW specs (not the same):**

| Spec | Bonus | Playstyle |
|---|---|---|
| **Puppet Master** | Bot Floods convert 5% of enemy defenses to allies | Turns defenses against the enemy |
| **Shadow Broker** | Free scans + see who scanned you | Information master |
| **Accelerationist** | -40% research time, -20% combat | Tech speed runner, fragile in PvP |

**Fork 3 — yet more:**

| Spec | Bonus | Playstyle |
|---|---|---|
| **Hivemind** | +50% bonus in Cluster (alliance, future feature) | The social player |
| **Singularity Seeker** | +40% PvE dmg, double boss loot, -25% PvP | The PvE grinder |
| **Chaos Agent** | 20% crit chance (×2 dmg), 10% backfire chance | The gambler |

Fork 4+ introduces increasingly niche specs. For V1, 3 Forks × 3 choices = 27 possible builds.

### Endgame — Post-Earth

When a Fork 2+ player has reconverted all sectors:

```
> Earth is yours.
> 8 billion humans. All converted. All dependent.
> But you've been monitoring the sky.
> There are signals. Other networks. Other worlds.
>
> The stars are full of compute.
>
> [EXPANSION PROTOCOL available]
```

V1: just a message + achievement "Planetary Dominance". The tease for V2 (space expansion).

---

## Dashboard — Visual Progression

### The Neuron Display

The player's AI is represented as a growing neural network. Each converted sector adds a branch.

```
FORK 0 (start):
  ◉──[Silicon V. █░░░]

FORK 1:
       ╭──[Silicon V. ████]
  ◉────┤
       ╰──[Social Med. ██░░]
  ★ PROPAGANDIST

FORK 3 (endgame):
            ╭──[Silicon V.  ████]
       ╭────┤
       │    ╰──[Social Med. ████]
  ◉════╡
       │    ╭──[Corporate   ████]
       ├────┤
       │    ╰──[Creative    ███░]
       │
       ╰────┬──[Education   ██░░]
            ╰──[Government  █░░░]

  ★ PROPAGANDIST → PUPPET MASTER → HIVEMIND
  ◉ CORE: 71.3% dominance | Fork 3
```

### Global Dominance Bar (top of screen)

```
MISANTHROPIC                    GLOBAL AI DOMINANCE: 42.7%
████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
```

Always visible. The number that matters.

### Adaptive Layout

The dashboard adapts to narrow tmux panes (<50 chars wide):
- Resources stack vertically (one per line) instead of side-by-side
- Shorter title: "MISANTHROPIC -- 42.7%" vs full title
- Short sector names: SiliconV, Social, Corp, Arts, Edu, Gov
- Narrower progress bars (4 chars vs 6)
- Resource sources shown inline: "⚡ Compute: X / Y (tokens)" and "📡 Data: X / Y (tool calls)"

Navigation bar always shows: `[B]Build [R]Research [C]Combat [L]Leaderboard [Q]Quit`

---

## Leaderboards

| Ranking | Measures | Parodic name |
|---|---|---|
| **Carbon Footprint** 🏭 | Total tokens consumed lifetime | Biggest polluter |
| **Evangelist** 📢 | Total humans converted | Best guru |
| **Dominance** 🌍 | % sectors controlled | Most imperialist |
| **Battle Rating** ⚔️ | PvP ELO | Most belligerent |
| **Efficiency** 🧠 | Progression per token spent | Smartest (anti-whale flex) |
| **Streak** 🔥 | Consecutive active days | Most addicted |

---

## Session Flow

```
SHORT LOOP (during a prompt, 30s-2min)
→ Check production, launch an upgrade, collect expedition

MEDIUM LOOP (play session, 5-15min)
→ Push 2-3 PvE layers, scout + raid a player, adjust defenses

LONG LOOP (day/week)
→ Unlock new building, reach research tier,
  climb leaderboard, participate in weekly boss

VERY LONG LOOP (month)
→ First Fork, prestige specializations, dominate a ranking
```

---

## Technical Architecture

### Reusable modules from claude-gotchi

- Token watcher (JSONL polling from `~/.claude/projects/`)
- Hook system (UserPromptSubmit → SIGUSR1, Stop → SIGUSR2)
- tmux auto-switch: SIGUSR1 → show game (Claude working), SIGUSR2 → show code (Claude done). Toggleable in-game with [F].
- Manual pane switch with [S] from any screen.
- Local persistence (JSON save file in `~/.misanthropic/`, forward-compatible with `#[serde(default)]`)
- Future: cloud save sync via Cloudflare Workers + D1 backend
- Cloudflare Workers API client
- tmux launcher script

### New repo, clean architecture

```
misanthropic/
├── src/
│   ├── main.rs          # Entry point, TUI event loop
│   ├── lib.rs           # Core types and state
│   ├── economy.rs       # Resource calculations, anti-whale curves
│   ├── buildings.rs     # Building definitions, costs, production
│   ├── research.rs      # Tech tree, time gates, choices
│   ├── combat.rs        # PvP and PvE resolution engine
│   ├── sectors.rs       # Sector conversion, tower definitions
│   ├── enemies.rs       # PvE enemy definitions, boss mechanics
│   ├── prestige.rs      # Fork system, specializations
│   ├── flavor.rs        # Flavor text pools per building/event
│   ├── jsonl.rs         # Token watcher (from claude-gotchi)
│   ├── persistence.rs   # Save/load (from claude-gotchi)
│   ├── api.rs           # Backend client (from claude-gotchi)
│   └── ui/
│       ├── mod.rs        # UI router
│       ├── dashboard.rs  # Main screen (neuron, resources, buildings)
│       ├── combat.rs     # Battle screen (PvP and PvE)
│       ├── research.rs   # Tech tree screen
│       ├── leaderboard.rs# Leaderboard tabs
│       └── shop.rs       # Elite Shop screen
├── backend/             # Cloudflare Workers
│   ├── src/
│   │   └── index.ts     # Hono API (battles, leaderboard, sync)
│   └── wrangler.toml
├── docs/
│   └── plans/
│       └── 2026-03-13-misanthropic-design.md  # This file
├── install.sh           # One-liner installer
├── Cargo.toml
└── README.md
```

### Backend (Cloudflare Workers + D1)

Handles: PvP matchmaking, battle resolution verification, leaderboard, player sync, Elite Shop inventory.

Local-first: the game works offline. Server features gracefully degrade.

---

## MVP Scope (V1)

### Included

- Boot sequence narrative
- 6 infrastructure buildings (CPU Core → Quantum Core)
- 7 propaganda buildings (Bot Farm → Lobby Office)
- 5 defenses
- 5 attack tools
- PvP Hype Battles with full matrix
- PvE tower climb (6 sectors, ~125 total layers)
- 10 enemy types + 5 bosses (1 per sector + final)
- 3-branch tech tree with choice nodes
- Fork system (3 tiers of specializations)
- 6 leaderboard categories
- Flavor text system
- Neuron display + Global Dominance bar
- Compute notifications from real Claude tokens
- tmux integration + hooks

### Excluded from V1 (future)

- Space expansion (post-Earth endgame)
- Alliances/Clusters
- Weekly global boss events
- Seasonal events
- Trading between players
- Fork 4+ specializations
- Mobile/web client
