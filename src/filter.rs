use crate::constants::{MIN_CPU, SORT_MIN_MEM_DEFAULT};
use crate::process_list::ProcessInfo;

pub struct ProcessFilter {
    pub min_cpu: f32,
    pub min_mem: u64,
    pub exclude_names: Vec<String>,
}

impl ProcessFilter {
    pub fn new(min_cpu: f32, min_mem: u64, exclude_names: Vec<String>) -> Self {
        ProcessFilter {
            min_cpu,
            min_mem,
            exclude_names,
        }
    }

    #[allow(dead_code)]
    pub fn from_config(min_cpu: f32, exclude_names: Vec<String>) -> Self {
        Self::new(min_cpu, SORT_MIN_MEM_DEFAULT, exclude_names)
    }

    #[allow(dead_code)]
    pub fn set_min_cpu(&mut self, min_cpu: f32) {
        self.min_cpu = min_cpu;
    }

    #[allow(dead_code)]
    pub fn set_min_mem(&mut self, min_mem: u64) {
        self.min_mem = min_mem;
    }

    pub fn filter<'a>(&self, processes: &'a [&ProcessInfo], sort_by_mem: bool) -> Vec<ProcessInfo> {
        processes
            .iter()
            .filter(|p| {
                if sort_by_mem {
                    p.mem >= self.min_mem && !self.should_exclude(&p.name)
                } else {
                    p.cpu >= self.min_cpu && !self.should_exclude(&p.name)
                }
            })
            .map(|p| (*p).clone())
            .collect()
    }

    pub fn should_exclude(&self, name: &str) -> bool {
        self.exclude_names.iter().any(|n| name.contains(n))
    }
}

impl Default for ProcessFilter {
    fn default() -> Self {
        Self::new(
            MIN_CPU,
            SORT_MIN_MEM_DEFAULT,
            vec!["HeapHelper".to_string()],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sysinfo::Pid;

    fn make_process(name: &str, cpu: f32) -> ProcessInfo {
        ProcessInfo {
            pid: Pid::from_u32(1),
            name: name.to_string(),
            user: "test".to_string(),
            cpu,
            mem: 100,
            time: 1000,
            nice: 0,
        }
    }

    #[test]
    fn test_filter_creation() {
        let filter = ProcessFilter::new(10.0, SORT_MIN_MEM_DEFAULT, vec!["HeapHelper".to_string()]);
        assert_eq!(filter.min_cpu, 10.0);
        assert!(filter.exclude_names.contains(&"HeapHelper".to_string()));
    }

    #[test]
    fn test_set_min_cpu() {
        let mut filter = ProcessFilter::default();
        filter.set_min_cpu(5.0);
        assert_eq!(filter.min_cpu, 5.0);
    }

    #[test]
    fn test_should_exclude() {
        let filter = ProcessFilter::default();
        assert!(filter.should_exclude("HeapHelper"));
        assert!(filter.should_exclude("HeapHelperXYZ"));
        assert!(!filter.should_exclude("other"));
    }

    #[test]
    fn test_filter_by_cpu_and_name() {
        let filter = ProcessFilter::new(50.0, SORT_MIN_MEM_DEFAULT, vec!["HeapHelper".to_string()]);

        let p1 = make_process("HeapHelper", 100.0);
        let p2 = make_process("other", 60.0);
        let p3 = make_process("test", 40.0);
        let processes: Vec<ProcessInfo> = vec![p1, p2, p3];
        let filtered = filter.filter_owned(processes, false);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "other");
    }
}
