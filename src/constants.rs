pub const MIN_CPU: f32 = 10.0;
pub const SORT_MIN_MEM_DEFAULT: u64 = 50 * 1024 * 1024; // 50MB

pub const PROCESS_LIMIT: usize = 30;

pub const NICE_MIN: i32 = -20;
pub const NICE_MAX: i32 = 19;
pub const SIGNAL_TERM: i32 = 15;
pub const SIGNAL_KILL: i32 = 9;

pub const DEFAULT_CORES: usize = 1;
pub const DEFAULT_HEALTH: i32 = 100;

pub const MSG_NEED_ROOT: &str = "Need root to increase priority";
pub const MSG_INVALID_NICE: &str = "Invalid nice value";
pub const MSG_RENICE_FAILED: &str = "Failed to renice";
pub const MSG_RENICED: &str = "Reniced";
pub const MSG_KILL_FAILED: &str = "Failed to send signal";
pub const MSG_KILL_SENT: &str = "Sent signal";
pub const MSG_FAILED_TO_KILL: &str = "Failed to kill PID";

pub const LOG_EXCELLENT: &str = "EXCELLENT";
pub const LOG_MODE_DEACTIVATED: &str = "Mode deactivated";
pub const LOG_KILL_MODE_ACTIVATED: &str = "Kill mode activated";
pub const LOG_START: &str = "Starting RTOP";
pub const LOG_EXIT: &str = "Exiting RTOP";
pub const LOG_QUIT: &str = "Quit requested";
pub const LOG_TUI_FAIL: &str = "Failed to initialize TUI";

pub const SLEEP_NORMAL: u64 = 50;
pub const SLEEP_FAST: u64 = 10;
pub const MSG_DURATION_SECS: u64 = 5;
