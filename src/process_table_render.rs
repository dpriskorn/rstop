use crate::color::Colors;
use crate::process_list::ProcessInfo;
use std::fmt;
use tabled::{
    settings::object::Rows,
    settings::{Format, Margin, Modify, Style},
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
    #[tabled(rename = "M")]
    pub m: &'a str,
    #[tabled(rename = "PID")]
    pub pid: MaybeEmpty<u32>,
    #[tabled(rename = "USER")]
    pub user: &'a str,
    #[tabled(rename = "NI")]
    pub ni: MaybeEmpty<i32>,
    #[tabled(rename = "CPU")]
    pub cpu: MaybeEmpty<u64>,
    #[tabled(rename = "MEM")]
    pub mem: MaybeEmpty<u64>,
    #[tabled(rename = "TIME")]
    pub time: MaybeEmpty<u64>,
    #[tabled(rename = "NAME")]
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
            .take(MAX_ROWS)
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
                    user: &p.user,
                    ni: MaybeEmpty(Some(p.nice)),
                    cpu: MaybeEmpty(Some(p.cpu.round() as u64)),
                    mem: MaybeEmpty(Some(mem_mb)),
                    time: MaybeEmpty(Some(time_min)),
                    name,
                }
            })
            .collect();

        let selected_row = if renice_active {
            renice_sel.min(MAX_ROWS - 1)
        } else if kill_active {
            kill_sel.min(MAX_ROWS - 1)
        } else {
            usize::MAX
        };

        let empty_rows = MAX_ROWS.saturating_sub(rows.len());
        let mut all_rows = rows;
        for _ in 0..empty_rows {
            all_rows.push(Row {
                m: " ",
                pid: MaybeEmpty(None),
                user: "",
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
        table.with(Margin::new(0, 0, 2, 0));
        table.with(Modify::new(Rows::new(1..=1)).with(Format::content(|s: &str| s.to_uppercase())));

        if selected_row < all_rows.len() && selected_row != usize::MAX {
            let sel = selected_row + 1;
            table.with(Modify::new(Rows::new(sel..=sel)).with(Format::content(|s| {
                format!("{}{}{}", Colors::BOLD, s, Colors::RESET)
            })));
        }

        println!("{}", table);
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
