use crate::color::Colors;
use crate::process_list::ProcessInfo;
use std::fmt;
use tabled::{
    settings::object::Rows,
    settings::{Format, Modify, Style},
    Table, Tabled,
};

pub struct MaybeEmpty<T>(pub Option<T>);

impl<T: fmt::Display> fmt::Display for MaybeEmpty<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(v) => write!(f, "{}", v),
            None => write!(f, " "),
        }
    }
}

#[derive(Tabled)]
pub struct Row<'a> {
    pub m: &'a str,
    pub pid: MaybeEmpty<u32>,
    pub ni: MaybeEmpty<i32>,
    pub cpu: MaybeEmpty<u64>,
    pub mem: MaybeEmpty<u64>,
    pub time: MaybeEmpty<u64>,
    pub name: String,
}

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
                let mem_mb = (p.mem as f64 / 1024.0 / 1024.0).round() as u64;
                let time_min = p.time / 60;
                let name = if p.name.len() > 20 {
                    p.name.chars().take(20).collect()
                } else {
                    p.name.clone()
                };

                Row {
                    m: if (renice_active && renice_sel == i) || (kill_active && kill_sel == i) {
                        ">"
                    } else {
                        " "
                    },
                    pid: MaybeEmpty(Some(p.pid.as_u32())),
                    ni: MaybeEmpty(Some(p.nice)),
                    cpu: MaybeEmpty(Some(p.cpu.round() as u64)),
                    mem: MaybeEmpty(Some(mem_mb)),
                    time: MaybeEmpty(Some(time_min)),
                    name,
                }
            })
            .collect();

        let selected_row = if renice_active {
            renice_sel
        } else if kill_active {
            kill_sel
        } else {
            usize::MAX
        };

        let empty_rows = MAX_ROWS.saturating_sub(rows.len());
        let mut all_rows = rows;
        for _ in 0..empty_rows {
            all_rows.push(Row {
                m: " ",
                pid: MaybeEmpty(None),
                ni: MaybeEmpty(None),
                cpu: MaybeEmpty(None),
                mem: MaybeEmpty(None),
                time: MaybeEmpty(None),
                name: String::new(),
            });
        }

        if all_rows.is_empty() {
            return;
        }

        let mut table = Table::new(&all_rows);
        table.with(Style::empty());

        if selected_row < all_rows.len() {
            let sel = selected_row + 1;
            table.with(Modify::new(Rows::new(sel..=sel)).with(Format::content(|s| {
                format!("{}{}{}", Colors::BOLD, s, Colors::RESET)
            })));
        }

        println!("\n{}", table);
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
