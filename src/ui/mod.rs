pub mod boot;
pub mod buildings;
pub mod dashboard;

use misanthropic::state::GameState;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Boot,
    Dashboard,
    Buildings,
    Research,
    Combat,
    Leaderboard,
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
        }
    }

    pub fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }
}
