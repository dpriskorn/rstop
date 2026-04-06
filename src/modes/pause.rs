pub struct PauseMode {
    pub active: bool,
}

impl PauseMode {
    pub fn new() -> Self {
        PauseMode { active: false }
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    #[allow(dead_code)]
    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

impl Default for PauseMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pause_mode_creation() {
        let mode = PauseMode::new();
        assert!(!mode.active);
    }

    #[test]
    fn test_toggle() {
        let mut mode = PauseMode::new();
        assert!(!mode.active);
        mode.toggle();
        assert!(mode.active);
        mode.toggle();
        assert!(!mode.active);
    }

    #[test]
    fn test_activate() {
        let mut mode = PauseMode::new();
        mode.activate();
        assert!(mode.active);
    }

    #[test]
    fn test_deactivate() {
        let mut mode = PauseMode::new();
        mode.activate();
        mode.deactivate();
        assert!(!mode.active);
    }
}
