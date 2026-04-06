use crate::process_list::ProcessInfo;

pub struct ProcessTable;

impl ProcessTable {
    pub fn new() -> Self {
        ProcessTable
    }

    pub fn print(
        &self,
        processes: &[&ProcessInfo],
        renice_active: bool,
        renice_sel: usize,
        kill_active: bool,
        kill_sel: usize,
    ) {
        const MAX_ROWS: usize = 10;

        println!();
        for i in 0..MAX_ROWS {
            if let Some(p) = processes.get(i) {
                let selected = (renice_active && i == renice_sel) || (kill_active && i == kill_sel);
                let mem_mb = (p.mem as f64 / 1024.0 / 1024.0).round() as u64;
                let time_min = p.time / 60;
                let name = if p.name.len() > 20 {
                    p.name.chars().take(20).collect::<String>()
                } else {
                    p.name.clone()
                };
                println!(
                    "  {:>5} {:>3} {:>3} {:>4} {:>5} {:>6} {}",
                    if selected { ">" } else { " " },
                    p.pid.as_u32(),
                    p.nice,
                    p.cpu.round() as i32,
                    mem_mb,
                    time_min,
                    name
                );
            } else {
                println!(
                    "  {:>5} {:>3} {:>3} {:>4} {:>5} {:>6} {}",
                    " ", 0, 0, 0, 0, 0isize, " "
                );
            }
        }
    }
}

impl Default for ProcessTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_table_creation() {
        let _table = ProcessTable::new();
    }
}
