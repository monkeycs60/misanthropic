CREATE TABLE players (
    id TEXT PRIMARY KEY,
    name TEXT,
    fork_count INTEGER DEFAULT 0,
    lifetime_compute INTEGER DEFAULT 0,
    lifetime_tokens INTEGER DEFAULT 0,
    pvp_rating INTEGER DEFAULT 1000,
    pvp_wins INTEGER DEFAULT 0,
    pvp_losses INTEGER DEFAULT 0,
    global_dominance REAL DEFAULT 0.0,
    streak_days INTEGER DEFAULT 0,
    last_sync TEXT,
    created_at TEXT
);

CREATE TABLE battles (
    id TEXT PRIMARY KEY,
    attacker_id TEXT NOT NULL,
    defender_id TEXT NOT NULL,
    winner_id TEXT,
    hype_staked REAL DEFAULT 0,
    hype_stolen REAL DEFAULT 0,
    compute_stolen INTEGER DEFAULT 0,
    log TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE attack_log (
    player_id TEXT NOT NULL,
    attack_date TEXT NOT NULL,
    attack_count INTEGER DEFAULT 0,
    PRIMARY KEY (player_id, attack_date)
);

CREATE INDEX idx_battles_attacker ON battles(attacker_id);
CREATE INDEX idx_battles_defender ON battles(defender_id);
CREATE INDEX idx_players_rating ON players(pvp_rating DESC);
CREATE INDEX idx_players_dominance ON players(global_dominance DESC);
CREATE INDEX idx_players_tokens ON players(lifetime_tokens DESC);
