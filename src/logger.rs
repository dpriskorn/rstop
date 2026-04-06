use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

pub struct Logger {
    log_path: PathBuf,
    start_time: Instant,
}

impl Logger {
    pub fn new() -> Self {
        let log_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("debug.log");

        let start_time = Instant::now();

        // Clear log file on start
        let _ = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_path);

        Logger {
            log_path,
            start_time,
        }
    }

    fn timestamp(&self) -> String {
        let now = Local::now();
        let datetime = now.format("%Y-%m-%d %H:%M:%S%.3f");
        let elapsed = self.start_time.elapsed();
        let run_secs = elapsed.as_secs();
        format!("[{}] [{:>5}.{:03}]", datetime, run_secs, run_secs)
    }

    pub fn log(&self, level: &str, message: &str) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = writeln!(file, "{} [{}] {}", self.timestamp(), level, message);
        }
    }

    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    pub fn debug(&self, message: &str) {
        self.log("DEBUG", message);
    }

    #[allow(dead_code)]
    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }

    pub fn log_timed(&self, label: &str, start: Instant) {
        self.debug(&format!("{}: {:?}", label, start.elapsed()));
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new();
        assert!(logger.log_path.ends_with("debug.log"));
    }

    #[test]
    fn test_log_message() {
        let logger = Logger::new();
        logger.log("INFO", "test message");
    }

    #[test]
    fn test_info() {
        let logger = Logger::new();
        logger.info("info message");
    }

    #[test]
    fn test_debug() {
        let logger = Logger::new();
        logger.debug("debug message");
    }

    #[test]
    fn test_error() {
        let logger = Logger::new();
        logger.error("error message");
    }
}
