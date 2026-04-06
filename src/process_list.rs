use sysinfo::{Pid, System};

#[derive(Clone)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub cpu: f32,
    pub mem: u64,
    pub time: u64,
    pub nice: i32,
}

pub struct ProcessList {
    processes: Vec<ProcessInfo>,
}

impl ProcessList {
    pub fn new() -> Self {
        ProcessList {
            processes: Vec::new(),
        }
    }

    pub fn refresh(&mut self, sys: &System) {
        self.processes = sys
            .processes()
            .iter()
            .map(|(pid, p)| {
                let nice =
                    unsafe { libc::getpriority(libc::PRIO_PROCESS, pid.as_u32() as libc::id_t) };
                ProcessInfo {
                    pid: *pid,
                    name: p.name().to_string_lossy().into_owned(),
                    cpu: p.cpu_usage(),
                    mem: p.memory(),
                    time: p.start_time(),
                    nice,
                }
            })
            .collect();
    }

    pub fn top_by_cpu(&self, count: usize) -> Vec<&ProcessInfo> {
        let mut sorted = self.processes.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| {
            b.cpu
                .partial_cmp(&a.cpu)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(count).collect()
    }

    #[allow(dead_code)]
    pub fn get(&self, index: usize) -> Option<&ProcessInfo> {
        self.processes.get(index)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.processes.len()
    }
}

impl Default for ProcessList {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HealthCalculator;

impl HealthCalculator {
    pub fn calculate(
        mem_percent: f32,
        swap_percent: f32,
        load1: f64,
        zram_ratio: f64,
        cores: usize,
    ) -> (i32, &'static str) {
        let mut score = 100;

        if mem_percent > 85.0 {
            score -= 30;
        } else if mem_percent > 70.0 {
            score -= 15;
        }

        if swap_percent > 50.0 {
            score -= 25;
        } else if swap_percent > 20.0 {
            score -= 10;
        }

        let cores = cores as f64;
        if load1 > cores * 1.5 {
            score -= 25;
        } else if load1 > cores {
            score -= 10;
        }

        if zram_ratio > 0.0 {
            if zram_ratio < 1.5 {
                score -= 10;
            } else if zram_ratio > 3.0 {
                score += 5;
            }
        }

        let score = score.max(0).min(100);
        let label = if score >= 85 {
            "EXCELLENT"
        } else if score >= 70 {
            "GOOD"
        } else if score >= 50 {
            "OK"
        } else {
            "STRESSED"
        };

        (score, label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_list_creation() {
        let list = ProcessList::new();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_top_by_cpu_empty() {
        let list = ProcessList::new();
        let top = list.top_by_cpu(10);
        assert!(top.is_empty());
    }

    #[test]
    fn test_health_calculator_excellent() {
        let (score, label) = HealthCalculator::calculate(50.0, 5.0, 1.0, 3.5, 4);
        assert!(score >= 85);
        assert_eq!(label, "EXCELLENT");
    }

    #[test]
    fn test_health_calculator_stressed() {
        let (score, label) = HealthCalculator::calculate(90.0, 60.0, 10.0, 1.2, 2);
        assert!(score < 50);
        assert_eq!(label, "STRESSED");
    }

    #[test]
    fn test_health_calculator_good() {
        let (score, label) = HealthCalculator::calculate(73.0, 15.0, 3.5, 1.4, 4);
        assert!(score >= 70, "Score should be >= 70, got {}", score);
        assert!(score <= 84, "Score should be <= 84, got {}", score);
        assert_eq!(label, "GOOD");
    }

    #[test]
    fn test_health_calculator_ok() {
        let (score, label) = HealthCalculator::calculate(80.0, 35.0, 5.0, 1.8, 4);
        assert!(score >= 50, "Score should be >= 50, got {}", score);
        assert!(score < 70, "Score should be < 70, got {}", score);
        assert_eq!(label, "OK");
    }
}
