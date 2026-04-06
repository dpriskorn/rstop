use crate::logger::Logger;

pub enum KeyAction {
    Quit,
    TogglePause,
    ToggleAdvanced,
    ToggleHelp,
    ToggleSort,
    ActivateRenice,
    ActivateKill,
    ExecuteAction,
    NavigateUp,
    NavigateDown,
    NiceValueUp,
    NiceValueDown,
    Signal9,
    Signal15,
    ExitMode,
    None,
}

pub struct Keys;

impl Keys {
    pub fn new() -> Self {
        Keys
    }

    pub fn handle_key(
        &self,
        key: Option<u8>,
        renice_active: bool,
        kill_active: bool,
        help_active: bool,
        pause_active: bool,
        frozen_procs_len: usize,
        logger: &Logger,
    ) -> KeyAction {
        logger.debug(&format!(
            "handle_key: key={:?}, renice={}, kill={}",
            key, renice_active, kill_active
        ));
        match key {
            Some(b'q' | b'Q') => {
                logger.debug("Key: Quit");
                KeyAction::Quit
            }
            Some(b'p' | b'P') => {
                if help_active || renice_active || kill_active {
                    KeyAction::None
                } else {
                    logger.debug("Key: TogglePause");
                    KeyAction::TogglePause
                }
            }
            Some(b'a' | b'A') => {
                if renice_active || kill_active {
                    KeyAction::None
                } else {
                    logger.debug("Key: ToggleAdvanced");
                    KeyAction::ToggleAdvanced
                }
            }
            Some(b'h' | b'H') => {
                if renice_active || kill_active {
                    KeyAction::None
                } else {
                    logger.debug("Key: ToggleHelp");
                    KeyAction::ToggleHelp
                }
            }
            Some(b'm' | b'M') => {
                if help_active || renice_active || kill_active {
                    KeyAction::None
                } else {
                    logger.debug("Key: ToggleSort");
                    KeyAction::ToggleSort
                }
            }
            Some(b'r' | b'R') => {
                if help_active || renice_active || kill_active {
                    KeyAction::None
                } else {
                    logger.debug("Key: ActivateRenice");
                    KeyAction::ActivateRenice
                }
            }
            Some(b'k' | b'K') => {
                if help_active || renice_active || kill_active {
                    KeyAction::None
                } else {
                    logger.debug("Key: ActivateKill");
                    KeyAction::ActivateKill
                }
            }
            Some(b'\n' | b'\r') => {
                if renice_active || kill_active {
                    logger.debug("Key: ExecuteAction");
                    KeyAction::ExecuteAction
                } else {
                    KeyAction::None
                }
            }
            Some(0xF0) => {
                logger.debug("Key: NavigateUp");
                KeyAction::NavigateUp
            }
            Some(0xF1) => {
                logger.debug("Key: NavigateDown");
                if renice_active && frozen_procs_len > 0 {
                    KeyAction::NavigateDown
                } else if kill_active && frozen_procs_len > 0 {
                    KeyAction::NavigateDown
                } else {
                    KeyAction::None
                }
            }
            Some(0xF3) => {
                if renice_active {
                    logger.debug("Key: NiceValueUp");
                    KeyAction::NiceValueUp
                } else if kill_active {
                    logger.debug("Key: Signal9");
                    KeyAction::Signal9
                } else {
                    KeyAction::None
                }
            }
            Some(0xF2) => {
                if renice_active {
                    logger.debug("Key: NiceValueDown");
                    KeyAction::NiceValueDown
                } else if kill_active {
                    logger.debug("Key: Signal15");
                    KeyAction::Signal15
                } else {
                    KeyAction::None
                }
            }
            Some(0x1b) => {
                if help_active || pause_active || renice_active || kill_active {
                    logger.debug("Key: ExitMode");
                    KeyAction::ExitMode
                } else {
                    KeyAction::None
                }
            }
            _ => KeyAction::None,
        }
    }
}

impl Default for Keys {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::Logger;

    #[test]
    fn test_keys_creation() {
        let _keys = Keys::new();
    }

    #[test]
    fn test_handle_key_quit() {
        let keys = Keys::new();
        let logger = Logger::new();
        assert!(matches!(
            keys.handle_key(Some(b'q'), false, false, false, false, 0, &logger),
            KeyAction::Quit
        ));
    }

    #[test]
    fn test_handle_key_renice() {
        let keys = Keys::new();
        let logger = Logger::new();
        assert!(matches!(
            keys.handle_key(Some(b'r'), false, false, false, false, 0, &logger),
            KeyAction::ActivateRenice
        ));
    }

    #[test]
    fn test_handle_key_kill() {
        let keys = Keys::new();
        let logger = Logger::new();
        assert!(matches!(
            keys.handle_key(Some(b'k'), false, false, false, false, 0, &logger),
            KeyAction::ActivateKill
        ));
    }

    #[test]
    fn test_handle_key_escape_exits_mode() {
        let keys = Keys::new();
        let logger = Logger::new();
        assert!(matches!(
            keys.handle_key(Some(0x1b), true, false, false, false, 5, &logger),
            KeyAction::ExitMode
        ));
    }

    #[test]
    fn test_handle_key_escape_does_nothing_when_no_mode() {
        let keys = Keys::new();
        let logger = Logger::new();
        assert!(matches!(
            keys.handle_key(Some(0x1b), false, false, false, false, 0, &logger),
            KeyAction::None
        ));
    }

    #[test]
    fn test_handle_key_escape_exits_help_mode() {
        let keys = Keys::new();
        let logger = Logger::new();
        assert!(matches!(
            keys.handle_key(Some(0x1b), false, false, true, false, 0, &logger),
            KeyAction::ExitMode
        ));
    }

    #[test]
    fn test_handle_key_escape_exits_pause_mode() {
        let keys = Keys::new();
        let logger = Logger::new();
        assert!(matches!(
            keys.handle_key(Some(0x1b), false, false, false, true, 0, &logger),
            KeyAction::ExitMode
        ));
    }
}
