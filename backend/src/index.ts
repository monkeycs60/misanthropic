import { Hono } from 'hono';
import { cors } from 'hono/cors';

type Env = {
  DB: D1Database;
};

const app = new Hono<{ Bindings: Env }>();

app.use('/*', cors());

// Register player
app.post('/register', async (c) => {
  const { id, name } = await c.req.json();
  if (!id) return c.json({ error: 'id required' }, 400);

  await c.env.DB.prepare(
    'INSERT OR IGNORE INTO players (id, name, created_at) VALUES (?, ?, datetime("now"))'
  ).bind(id, name || 'Anonymous').run();

  return c.json({ ok: true });
});

// Sync game state
app.post('/sync', async (c) => {
  const body = await c.req.json();
  const { id, fork_count, lifetime_compute, lifetime_tokens, pvp_rating, pvp_wins, pvp_losses, global_dominance, streak_days } = body;

  if (!id) return c.json({ error: 'id required' }, 400);

  await c.env.DB.prepare(`
    UPDATE players SET
      fork_count = ?, lifetime_compute = ?, lifetime_tokens = ?,
      pvp_rating = ?, pvp_wins = ?, pvp_losses = ?,
      global_dominance = ?, streak_days = ?, last_sync = datetime('now')
    WHERE id = ?
  `).bind(fork_count, lifetime_compute, lifetime_tokens, pvp_rating, pvp_wins, pvp_losses, global_dominance, streak_days, id).run();

  return c.json({ ok: true });
});

// Leaderboards
app.get('/leaderboard/:type', async (c) => {
  const type_ = c.req.param('type');
  let query: string;

  switch (type_) {
    case 'carbon':
      query = 'SELECT id, name, lifetime_tokens as score FROM players ORDER BY lifetime_tokens DESC LIMIT 50';
      break;
    case 'dominance':
      query = 'SELECT id, name, global_dominance as score FROM players ORDER BY global_dominance DESC LIMIT 50';
      break;
    case 'battle':
      query = 'SELECT id, name, pvp_rating as score FROM players ORDER BY pvp_rating DESC LIMIT 50';
      break;
    case 'efficiency':
      query = 'SELECT id, name, CASE WHEN lifetime_tokens > 0 THEN (global_dominance / lifetime_tokens * 1000000) ELSE 0 END as score FROM players ORDER BY score DESC LIMIT 50';
      break;
    case 'streak':
      query = 'SELECT id, name, streak_days as score FROM players ORDER BY streak_days DESC LIMIT 50';
      break;
    case 'evangelist':
      query = 'SELECT id, name, global_dominance * 6 as score FROM players ORDER BY global_dominance DESC LIMIT 50';
      break;
    default:
      return c.json({ error: 'invalid type' }, 400);
  }

  const results = await c.env.DB.prepare(query).all();
  return c.json({ entries: results.results });
});

// Battle history
app.get('/battle/history/:player_id', async (c) => {
  const playerId = c.req.param('player_id');
  const results = await c.env.DB.prepare(
    'SELECT * FROM battles WHERE attacker_id = ? OR defender_id = ? ORDER BY created_at DESC LIMIT 20'
  ).bind(playerId, playerId).all();

  return c.json({ battles: results.results });
});

// Attack cooldown check
app.get('/battle/can-attack/:player_id', async (c) => {
  const playerId = c.req.param('player_id');
  const today = new Date().toISOString().split('T')[0];

  const result = await c.env.DB.prepare(
    'SELECT attack_count FROM attack_log WHERE player_id = ? AND attack_date = ?'
  ).bind(playerId, today).first();

  const attacksToday = (result as any)?.attack_count || 0;
  return c.json({ can_attack: attacksToday < 3, attacks_today: attacksToday });
});

// Health check
app.get('/health', (c) => c.json({ status: 'ok', game: 'misanthropic' }));

export default app;
