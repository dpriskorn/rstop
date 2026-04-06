use procfs::process::Process;
use procfs::ticks_per_second;
use sysinfo::{Pid, System};

#[derive(Clone)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub user: String,
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
            .filter(|(_, p)| p.thread_kind().is_none())
            .map(|(pid, p)| {
                let nice =
                    unsafe { libc::getpriority(libc::PRIO_PROCESS, pid.as_u32() as libc::id_t) };
                let user = p
                    .user_id()
                    .map(|uid| {
                        let uid_val: u32 = **uid;
                        let passwd = unsafe {
                            let mut pw: libc::passwd = std::mem::zeroed();
                            let mut buf: Vec<u8> = vec![0; 1024];
                            let mut result: *mut libc::passwd = std::ptr::null_mut();
                            let ret = libc::getpwuid_r(
                                uid_val,
                                &mut pw,
                                buf.as_mut_ptr() as *mut libc::c_char,
                                buf.len(),
                                &mut result,
                            );
                            if ret == 0 && !result.is_null() {
                                std::ffi::CStr::from_ptr(pw.pw_name)
                                    .to_string_lossy()
                                    .into_owned()
                                    .to_lowercase()
                            } else {
                                uid_val.to_string()
                            }
                        };
                        passwd
                    })
                    .unwrap_or_else(|| "?".to_string());

                let cpu_time = Self::get_cpu_time(pid.as_u32() as i32);

                ProcessInfo {
                    pid: *pid,
                    name: p.name().to_string_lossy().into_owned(),
                    user,
                    cpu: p.cpu_usage(),
                    mem: p.memory(),
                    time: cpu_time,
                    nice,
                }
            })
            .collect();
    }

    fn get_cpu_time(pid: i32) -> u64 {
        if let Ok(proc) = Process::new(pid) {
            if let Ok(stat) = proc.stat() {
                let total_ticks =
                    stat.utime + stat.stime + (stat.cutime as u64) + (stat.cstime as u64);
                return total_ticks / ticks_per_second() as u64;
            }
        }
        0
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

    pub fn top_by_mem(&self, count: usize) -> Vec<&ProcessInfo> {
        let mut sorted = self.processes.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| b.mem.cmp(&a.mem));
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
    fn test_top_by_mem_empty() {
        let list = ProcessList::new();
        let top = list.top_by_mem(10);
        assert!(top.is_empty());
    }
}
