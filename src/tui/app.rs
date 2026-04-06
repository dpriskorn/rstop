#[derive(Default, PartialEq, Clone, Copy)]
pub enum Mode {
    #[default]
    Normal,
    Pause,
    Renice,
    Kill,
    Help,
}

pub struct AppState {
    pub sort_by_mem: bool,
    pub mode: Mode,
    pub selection: usize,
    pub nice_value: i32,
    pub kill_signal: i32,
    pub advanced: bool,
    pub info: String,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            sort_by_mem: false,
            mode: Mode::Normal,
            selection: 0,
            nice_value: 19,
            kill_signal: 15,
            advanced: false,
            info: String::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
