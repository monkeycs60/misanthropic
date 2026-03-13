use crate::state::GameState;
use std::path::Path;

pub fn save_game(state: &GameState, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_game(path: &Path) -> Result<GameState, String> {
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn save_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| Path::new("/tmp").to_path_buf())
        .join(".misanthropic")
}

pub fn save_path() -> std::path::PathBuf {
    save_dir().join("save.json")
}
