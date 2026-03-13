pub mod boot;
pub mod buildings;
pub mod combat;
pub mod combat_menu;
pub mod dashboard;
pub mod leaderboard;
pub mod market;
pub mod research;

use misanthropic::combat::PveBattleResult;
use misanthropic::state::GameState;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Boot,
    Dashboard,
    Buildings,
    Research,
    CombatMenu,
    PvE,
    PvP,
    Leaderboard,
    Market,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatPhase {
    SectorSelect,
    LoadoutBuild,
    BattleResult,
}

pub struct App {
    pub state: GameState,
    pub screen: Screen,
    pub status_message: Option<(String, Instant)>,
    pub should_quit: bool,
    pub is_active: bool,
    pub boot_line: usize,
    pub boot_timer: Instant,
    pub selected_index: usize,
    pub notification: Option<(String, Instant)>,
    pub building_tab: u8,
    pub building_selected: usize,
    pub research_selected_branch: u8,
    pub research_selected_level: u8,
    // Combat fields
    pub combat_sector: usize,
    pub combat_loadout: Vec<(usize, u8)>,
    pub combat_selected_attack: usize,
    pub combat_result: Option<PveBattleResult>,
    pub combat_phase: CombatPhase,
    pub combat_menu_selected: usize,
    pub leaderboard_tab: u8,
    // Market fields
    pub market_selected: usize,     // 0=buy data, 1=buy hype
    pub market_amount_idx: usize,   // index into [1, 5, 10, 25]
}

impl App {
    pub fn new(state: GameState) -> Self {
        let screen = if state.boot_sequence_done {
            Screen::Dashboard
        } else {
            Screen::Boot
        };
        Self {
            state,
            screen,
            status_message: None,
            should_quit: false,
            is_active: false,
            boot_line: 0,
            boot_timer: Instant::now(),
            selected_index: 0,
            notification: None,
            building_tab: 0,
            building_selected: 0,
            research_selected_branch: 0,
            research_selected_level: 0,
            combat_sector: 0,
            combat_loadout: vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)],
            combat_selected_attack: 0,
            combat_result: None,
            combat_phase: CombatPhase::SectorSelect,
            combat_menu_selected: 0,
            leaderboard_tab: 0,
            market_selected: 0,
            market_amount_idx: 0,
        }
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }
}
