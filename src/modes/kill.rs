pub struct KillMode {
    pub active: bool,
    pub selection: usize,
    pub signal: i32,
}

impl KillMode {
    pub fn new() -> Self {
        KillMode {
            active: false,
            selection: 0,
            signal: 9,
        }
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.selection = 0;
        self.signal = 9;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    #[allow(dead_code)]
    pub fn move_up(&mut self) {
        self.selection = self.selection.saturating_sub(1);
    }

    #[allow(dead_code)]
    pub fn move_down(&mut self, max: usize) {
        self.selection = (self.selection + 1).min(max.saturating_sub(1));
    }

    #[allow(dead_code)]
    pub fn toggle_signal(&mut self) {
        self.signal = if self.signal == 9 { 15 } else { 9 };
    }
}

impl Default for KillMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kill_mode_creation() {
        let mode = KillMode::new();
        assert!(!mode.active);
        assert_eq!(mode.selection, 0);
        assert_eq!(mode.signal, 9);
    }

    #[test]
    fn test_activate() {
        let mut mode = KillMode::new();
        mode.activate();
        assert!(mode.active);
        assert_eq!(mode.selection, 0);
        assert_eq!(mode.signal, 9);
    }

    #[test]
    fn test_deactivate() {
        let mut mode = KillMode::new();
        mode.activate();
        mode.deactivate();
        assert!(!mode.active);
    }

    #[test]
    fn test_move_up() {
        let mut mode = KillMode::new();
        mode.selection = 5;
        mode.move_up();
        assert_eq!(mode.selection, 4);
    }

    #[test]
    fn test_move_up_at_zero() {
        let mut mode = KillMode::new();
        mode.move_up();
        assert_eq!(mode.selection, 0);
    }

    #[test]
    fn test_move_down() {
        let mut mode = KillMode::new();
        mode.move_down(10);
        assert_eq!(mode.selection, 1);
    }

    #[test]
    fn test_move_down_max() {
        let mut mode = KillMode::new();
        mode.selection = 9;
        mode.move_down(10);
        assert_eq!(mode.selection, 9);
    }

    #[test]
    fn test_toggle_signal_to_15() {
        let mut mode = KillMode::new();
        assert_eq!(mode.signal, 9);
        mode.toggle_signal();
        assert_eq!(mode.signal, 15);
    }

    #[test]
    fn test_toggle_signal_back_to_9() {
        let mut mode = KillMode::new();
        mode.signal = 15;
        mode.toggle_signal();
        assert_eq!(mode.signal, 9);
    }
}
