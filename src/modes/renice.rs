pub struct ReniceMode {
    pub active: bool,
    pub selection: usize,
    pub nice_value: i32,
}

impl ReniceMode {
    pub fn new() -> Self {
        ReniceMode {
            active: false,
            selection: 0,
            nice_value: 19,
        }
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.selection = 0;
        self.nice_value = 19;
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
    pub fn decrease_nice(&mut self) {
        self.nice_value = (self.nice_value - 1).max(-20);
    }

    #[allow(dead_code)]
    pub fn increase_nice(&mut self) {
        self.nice_value = (self.nice_value + 1).min(19);
    }
}

impl Default for ReniceMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renice_mode_creation() {
        let mode = ReniceMode::new();
        assert!(!mode.active);
        assert_eq!(mode.selection, 0);
        assert_eq!(mode.nice_value, 19);
    }

    #[test]
    fn test_activate() {
        let mut mode = ReniceMode::new();
        mode.activate();
        assert!(mode.active);
        assert_eq!(mode.selection, 0);
        assert_eq!(mode.nice_value, 19);
    }

    #[test]
    fn test_deactivate() {
        let mut mode = ReniceMode::new();
        mode.activate();
        mode.deactivate();
        assert!(!mode.active);
    }

    #[test]
    fn test_move_up() {
        let mut mode = ReniceMode::new();
        mode.selection = 5;
        mode.move_up();
        assert_eq!(mode.selection, 4);
    }

    #[test]
    fn test_move_up_at_zero() {
        let mut mode = ReniceMode::new();
        mode.move_up();
        assert_eq!(mode.selection, 0);
    }

    #[test]
    fn test_move_down() {
        let mut mode = ReniceMode::new();
        mode.move_down(10);
        assert_eq!(mode.selection, 1);
    }

    #[test]
    fn test_move_down_max() {
        let mut mode = ReniceMode::new();
        mode.selection = 9;
        mode.move_down(10);
        assert_eq!(mode.selection, 9);
    }

    #[test]
    fn test_decrease_nice() {
        let mut mode = ReniceMode::new();
        mode.nice_value = 10;
        mode.decrease_nice();
        assert_eq!(mode.nice_value, 9);
    }

    #[test]
    fn test_decrease_nice_min() {
        let mut mode = ReniceMode::new();
        mode.nice_value = -20;
        mode.decrease_nice();
        assert_eq!(mode.nice_value, -20);
    }

    #[test]
    fn test_increase_nice() {
        let mut mode = ReniceMode::new();
        mode.nice_value = 10;
        mode.increase_nice();
        assert_eq!(mode.nice_value, 11);
    }

    #[test]
    fn test_increase_nice_max() {
        let mut mode = ReniceMode::new();
        mode.nice_value = 19;
        mode.increase_nice();
        assert_eq!(mode.nice_value, 19);
    }
}
