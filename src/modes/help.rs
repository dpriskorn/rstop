pub struct HelpMode {
    pub active: bool,
}

impl HelpMode {
    pub fn new() -> Self {
        HelpMode { active: false }
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

impl Default for HelpMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_mode_creation() {
        let mode = HelpMode::new();
        assert!(!mode.active);
    }

    #[test]
    fn test_toggle() {
        let mut mode = HelpMode::new();
        assert!(!mode.active);
        mode.toggle();
        assert!(mode.active);
        mode.toggle();
        assert!(!mode.active);
    }

    #[test]
    fn test_activate() {
        let mut mode = HelpMode::new();
        mode.activate();
        assert!(mode.active);
    }

    #[test]
    fn test_deactivate() {
        let mut mode = HelpMode::new();
        mode.activate();
        mode.deactivate();
        assert!(!mode.active);
    }
}
