use crate::process_list::ProcessInfo;
use tabled::{settings::Style, Table, Tabled};

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

        let rows: Vec<Row> = processes
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let selected = (renice_active && i == renice_sel) || (kill_active && i == kill_sel);
                let mem_mb = (p.mem as f64 / 1024.0 / 1024.0).round() as u64;
                let time_min = p.time / 60;
                Row {
                    m: if selected { ">" } else { " " },
                    pid: p.pid.as_u32(),
                    ni: p.nice,
                    cpu: p.cpu.round() as u64,
                    mem: mem_mb,
                    time: time_min,
                    name: if p.name.len() > 20 {
                        p.name.chars().take(20).collect()
                    } else {
                        p.name.clone()
                    },
                }
            })
            .collect();

        let empty_rows = MAX_ROWS.saturating_sub(rows.len());
        let mut all_rows = rows;
        for _ in 0..empty_rows {
            all_rows.push(Row {
                m: " ",
                pid: 0,
                ni: 0,
                cpu: 0,
                mem: 0,
                time: 0,
                name: "".to_string(),
            });
        }

        if !all_rows.is_empty() {
            let mut table = Table::new(&all_rows);
            table.with(Style::empty());
            println!("\n{}", table);
        }
    }
}

#[derive(Tabled)]
pub struct Row<'a> {
    pub m: &'a str,
    pub pid: u32,
    pub ni: i32,
    pub cpu: u64,
    pub mem: u64,
    pub time: u64,
    pub name: String,
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
