use std::time::Instant;

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
    pub info_time: Option<Instant>,
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
            info_time: None,
        }
    }

    pub fn set_info(&mut self, msg: &str) {
        self.info = msg.to_string();
        self.info_time = Some(Instant::now());
    }

    pub fn clear_info_if_old(&mut self, duration_secs: u64) {
        if let Some(time) = self.info_time {
            if time.elapsed().as_secs() >= duration_secs {
                self.info.clear();
                self.info_time = None;
            }
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
