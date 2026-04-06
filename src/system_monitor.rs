use crate::swap::SwapStats;
use sysinfo::System;

pub struct SystemStats {
    pub cpu: f32,
    #[allow(dead_code)]
    pub mem_percent: f32,
    pub zram_swap_percent: f32,
    pub disk_swap_percent: f32,
    #[allow(dead_code)]
    pub load1: f64,
    #[allow(dead_code)]
    pub load5: f64,
    #[allow(dead_code)]
    pub load10: f64,
    pub cores: usize,
}

pub struct SystemMonitor {
    pub sys: System,
}

impl SystemMonitor {
    pub fn new() -> Self {
        SystemMonitor {
            sys: System::new_all(),
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_all();
    }

    pub fn get_stats(&self) -> SystemStats {
        let swap = SwapStats::read();
        SystemStats {
            cpu: self.sys.global_cpu_usage(),
            mem_percent: if self.sys.total_memory() > 0 {
                (self.sys.used_memory() as f32 / self.sys.total_memory() as f32) * 100.0
            } else {
                0.0
            },
            zram_swap_percent: swap.zram.percent,
            disk_swap_percent: swap.disk.percent,
            load1: 0.0,
            load5: 0.0,
            load10: 0.0,
            cores: self.sys.cpus().len(),
        }
    }

    pub fn load_average(&self) -> (f64, f64, f64) {
        let content = std::fs::read_to_string("/proc/loadavg")
            .ok()
            .map(|s| {
                s.split_whitespace()
                    .take(3)
                    .filter_map(|w| w.parse().ok())
                    .collect::<Vec<f64>>()
            })
            .unwrap_or_else(|| vec![0.0, 0.0, 0.0]);

        (
            content.first().copied().unwrap_or(0.0),
            content.get(1).copied().unwrap_or(0.0),
            content.get(2).copied().unwrap_or(0.0),
        )
    }

    #[allow(dead_code)]
    pub fn total_memory(&self) -> u64 {
        self.sys.total_memory()
    }

    #[allow(dead_code)]
    pub fn used_memory(&self) -> u64 {
        self.sys.used_memory()
    }

    #[allow(dead_code)]
    pub fn total_swap(&self) -> u64 {
        self.sys.total_swap()
    }

    #[allow(dead_code)]
    pub fn used_swap(&self) -> u64 {
        self.sys.used_swap()
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_monitor_creation() {
        let monitor = SystemMonitor::new();
        let stats = monitor.get_stats();
        assert!(stats.cores >= 1);
    }

    #[test]
    fn test_load_average() {
        let monitor = SystemMonitor::new();
        let (l1, l5, l10) = monitor.load_average();
        assert!(l1 >= 0.0);
        assert!(l5 >= 0.0);
        assert!(l10 >= 0.0);
    }
}
