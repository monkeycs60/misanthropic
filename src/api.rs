// HTTP client for backend API (Cloudflare Workers + D1)

use crate::state::GameState;
use serde::{Deserialize, Serialize};

const REQUEST_TIMEOUT_SECS: u64 = 10;

pub struct ApiClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

// --- Response structs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub id: Option<String>,
    pub name: Option<String>,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleRecord {
    pub id: Option<String>,
    pub attacker_id: Option<String>,
    pub defender_id: Option<String>,
    pub winner_id: Option<String>,
    pub hype_staked: Option<f64>,
    pub hype_stolen: Option<f64>,
    pub compute_stolen: Option<u64>,
    pub log: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CanAttackResponse {
    pub can_attack: bool,
    pub attacks_today: u32,
}

#[derive(Debug, Deserialize)]
struct OkResponse {
    #[allow(dead_code)]
    ok: Option<bool>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LeaderboardResponse {
    entries: Option<Vec<LeaderboardEntry>>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BattleHistoryResponse {
    battles: Option<Vec<BattleRecord>>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: Option<String>,
}

// --- Request bodies ---

#[derive(Debug, Serialize)]
struct RegisterRequest {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct SyncRequest {
    id: String,
    fork_count: u32,
    lifetime_compute: u64,
    lifetime_tokens: u64,
    pvp_rating: u32,
    pvp_wins: u32,
    pvp_losses: u32,
    global_dominance: f64,
    streak_days: u32,
}

impl ApiClient {
    /// Create a new API client with the given base URL.
    /// Pass an empty string to disable networking (offline mode).
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .unwrap_or_default();

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    /// Returns true if the client has a configured backend URL.
    pub fn is_online(&self) -> bool {
        !self.base_url.is_empty()
    }

    /// Register a new player.
    /// POST /register
    pub fn register(&self, id: &str, name: &str) -> Result<(), String> {
        if !self.is_online() {
            return Ok(());
        }

        let url = format!("{}/register", self.base_url);
        let body = RegisterRequest {
            id: id.to_string(),
            name: name.to_string(),
        };

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("Network error: {}", e))?;

        if !resp.status().is_success() {
            let err: ErrorResponse = resp.json().unwrap_or(ErrorResponse { error: None });
            return Err(err
                .error
                .unwrap_or_else(|| "Registration failed".to_string()));
        }

        let data: OkResponse = resp
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(err) = data.error {
            return Err(err);
        }

        Ok(())
    }

    /// Sync local game state to the server.
    /// POST /sync
    pub fn sync(&self, state: &GameState) -> Result<(), String> {
        if !self.is_online() {
            return Ok(());
        }

        let url = format!("{}/sync", self.base_url);
        let body = SyncRequest {
            id: state.player_id.clone(),
            fork_count: state.fork_count,
            lifetime_compute: state.lifetime_compute,
            lifetime_tokens: state.lifetime_tokens,
            pvp_rating: state.pvp_rating,
            pvp_wins: state.pvp_wins,
            pvp_losses: state.pvp_losses,
            global_dominance: state.global_dominance(),
            streak_days: state.streak_days,
        };

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .map_err(|e| format!("Network error: {}", e))?;

        if !resp.status().is_success() {
            let err: ErrorResponse = resp.json().unwrap_or(ErrorResponse { error: None });
            return Err(err
                .error
                .unwrap_or_else(|| "Sync failed".to_string()));
        }

        let data: OkResponse = resp
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if let Some(err) = data.error {
            return Err(err);
        }

        Ok(())
    }

    /// Fetch leaderboard by type: "carbon", "dominance", "battle", "efficiency", "streak", "evangelist".
    /// GET /leaderboard/:type
    pub fn get_leaderboard(&self, board_type: &str) -> Result<Vec<LeaderboardEntry>, String> {
        if !self.is_online() {
            return Ok(Vec::new());
        }

        let url = format!("{}/leaderboard/{}", self.base_url, board_type);

        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| format!("Network error: {}", e))?;

        if !resp.status().is_success() {
            let err: ErrorResponse = resp.json().unwrap_or(ErrorResponse { error: None });
            return Err(err
                .error
                .unwrap_or_else(|| "Failed to fetch leaderboard".to_string()));
        }

        let data: LeaderboardResponse = resp
            .json()
            .map_err(|e| format!("Failed to parse leaderboard: {}", e))?;

        if let Some(err) = data.error {
            return Err(err);
        }

        Ok(data.entries.unwrap_or_default())
    }

    /// Fetch battle history for a player.
    /// GET /battle/history/:player_id
    pub fn get_battle_history(&self, player_id: &str) -> Result<Vec<BattleRecord>, String> {
        if !self.is_online() {
            return Ok(Vec::new());
        }

        let url = format!("{}/battle/history/{}", self.base_url, player_id);

        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| format!("Network error: {}", e))?;

        if !resp.status().is_success() {
            let err: ErrorResponse = resp.json().unwrap_or(ErrorResponse { error: None });
            return Err(err
                .error
                .unwrap_or_else(|| "Failed to fetch battle history".to_string()));
        }

        let data: BattleHistoryResponse = resp
            .json()
            .map_err(|e| format!("Failed to parse battle history: {}", e))?;

        if let Some(err) = data.error {
            return Err(err);
        }

        Ok(data.battles.unwrap_or_default())
    }

    /// Check whether a player can attack today (max 3 per day).
    /// GET /battle/can-attack/:player_id
    pub fn can_attack(&self, player_id: &str) -> Result<CanAttackResponse, String> {
        if !self.is_online() {
            return Ok(CanAttackResponse {
                can_attack: true,
                attacks_today: 0,
            });
        }

        let url = format!("{}/battle/can-attack/{}", self.base_url, player_id);

        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| format!("Network error: {}", e))?;

        if !resp.status().is_success() {
            let err: ErrorResponse = resp.json().unwrap_or(ErrorResponse { error: None });
            return Err(err
                .error
                .unwrap_or_else(|| "Failed to check attack status".to_string()));
        }

        let data: CanAttackResponse = resp
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_new() {
        let client = ApiClient::new("https://example.com/");
        assert_eq!(client.base_url, "https://example.com");
        assert!(client.is_online());
    }

    #[test]
    fn test_api_client_offline() {
        let client = ApiClient::new("");
        assert!(!client.is_online());
    }

    #[test]
    fn test_offline_register_succeeds() {
        let client = ApiClient::new("");
        let result = client.register("test-id", "test-user");
        assert!(result.is_ok());
    }

    #[test]
    fn test_offline_leaderboard_returns_empty() {
        let client = ApiClient::new("");
        let result = client.get_leaderboard("carbon");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_offline_can_attack_returns_true() {
        let client = ApiClient::new("");
        let result = client.can_attack("test-id");
        assert!(result.is_ok());
        assert!(result.unwrap().can_attack);
    }

    #[test]
    fn test_register_network_error() {
        let client = ApiClient::new("http://127.0.0.1:1");
        let result = client.register("test-id", "test-user");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Network error"));
    }

    #[test]
    fn test_leaderboard_entry_deserialize() {
        let json = r#"{"id":"abc","name":"player1","score":1200.0}"#;
        let entry: LeaderboardEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id.as_deref(), Some("abc"));
        assert_eq!(entry.name.as_deref(), Some("player1"));
        assert_eq!(entry.score, Some(1200.0));
    }

    #[test]
    fn test_battle_record_deserialize() {
        let json = r#"{
            "id": "battle-42",
            "attacker_id": "aaa",
            "defender_id": "bbb",
            "winner_id": "aaa",
            "hype_staked": 80.0,
            "hype_stolen": 85.0,
            "compute_stolen": 240,
            "log": "Bot Flood bypassed Captcha Wall",
            "created_at": "2026-03-12T10:00:00Z"
        }"#;
        let record: BattleRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.id.as_deref(), Some("battle-42"));
        assert_eq!(record.attacker_id.as_deref(), Some("aaa"));
        assert_eq!(record.hype_stolen, Some(85.0));
        assert_eq!(record.compute_stolen, Some(240));
    }

    #[test]
    fn test_can_attack_response_deserialize() {
        let json = r#"{"can_attack": true, "attacks_today": 1}"#;
        let resp: CanAttackResponse = serde_json::from_str(json).unwrap();
        assert!(resp.can_attack);
        assert_eq!(resp.attacks_today, 1);
    }
}
