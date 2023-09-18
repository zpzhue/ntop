use std::error;
use ratatui::widgets::TableState;

use crate::system::AppSystemInfo;

/// Application result type.
pub type AppResult<T> = Result<T, Box<dyn error::Error>>;


/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub counter: u8,

    pub state: TableState,

    pub app_sys_info: AppSystemInfo

}


impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            counter: 0,
            state: TableState::default(),
            app_sys_info: AppSystemInfo::default()
        }
    }
}


impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {

    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn increment_counter(&mut self) {
        if let Some(res) = self.counter.checked_add(1) {
            self.counter = res;
        }
    }

    pub fn decrement_counter(&mut self) {
        if let Some(res) = self.counter.checked_sub(1) {
            self.counter = res;
        }
    }

}