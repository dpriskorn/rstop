use crate::color::{ColorScheme, Colors};
use crate::health::HealthFactors;
use crate::zram_stats::{ZramReader, ZramStats};
use tabled::{
    settings::{object::Rows, Remove, Style},
    Table, Tabled,
};

#[derive(Tabled)]
pub struct OverviewRow {
    pub label: String,
    pub value: String,
}

pub struct OverviewTable;

impl OverviewTable {
    pub fn new() -> Self {
        OverviewTable
    }

    pub fn print(
        &self,
        cpu: f32,
        mem_percent: f32,
        zram_swap_percent: f32,
        disk_swap_percent: f32,
        load1: f64,
        load5: f64,
        load10: f64,
        cores: usize,
        health: i32,
        health_label: &str,
        health_factors: &HealthFactors,
        zram_stats: Option<&ZramStats>,
        zram_reader: &ZramReader,
    ) {
        let colors = ColorScheme::global();
        let cpu_color = colors.color_for_cpu(cpu);
        let swap_color = colors.color_for_zram_swap(zram_swap_percent);
        let disk_swap_color = colors.color_for_disk_swap(disk_swap_percent);
        let load_color = colors.color_for_load(load1, cores);
        let load5_color = colors.color_for_load(load5, cores);
        let load10_color = colors.color_for_load(load10, cores);
        let health_color = colors.color_for_health(health);

        let mut rows = vec![
            OverviewRow {
                label: "HEALTH".to_string(),
                value: format!(
                    "{}{}/100 [{}]{}",
                    health_color,
                    health,
                    health_label,
                    Colors::RESET
                ),
            },
            OverviewRow {
                label: "PENALTIES".to_string(),
                value: format!(
                    "mem={}{}{} swap={}{}{} load={}{}{} zram={}{}",
                    if health_factors.mem_penalty > 0 {
                        Colors::RED
                    } else {
                        Colors::GREEN
                    },
                    if health_factors.mem_penalty > 0 {
                        format!("-{}", health_factors.mem_penalty)
                    } else {
                        "0".to_string()
                    },
                    Colors::RESET,
                    if health_factors.swap_penalty > 0 {
                        Colors::RED
                    } else {
                        Colors::GREEN
                    },
                    if health_factors.swap_penalty > 0 {
                        format!("-{}", health_factors.swap_penalty)
                    } else {
                        "0".to_string()
                    },
                    Colors::RESET,
                    if health_factors.load_penalty > 0 {
                        Colors::RED
                    } else {
                        Colors::GREEN
                    },
                    if health_factors.load_penalty > 0 {
                        format!("-{}", health_factors.load_penalty)
                    } else {
                        "0".to_string()
                    },
                    Colors::RESET,
                    if health_factors.zram_penalty != 0 {
                        if health_factors.zram_penalty > 0 {
                            Colors::RED
                        } else {
                            Colors::GREEN
                        }
                    } else {
                        Colors::GREEN
                    },
                    format!("{:+}", health_factors.zram_penalty),
                ),
            },
            OverviewRow {
                label: "CPU".to_string(),
                value: format!("{}{:.0}%{}", cpu_color, cpu, Colors::RESET),
            },
            OverviewRow {
                label: "RAM".to_string(),
                value: format!("{}{:.0}%{}", Colors::WHITE, mem_percent, Colors::RESET),
            },
        ];

        if let Some(z) = zram_stats {
            let ratio = zram_reader.compression_ratio(z);
            let ratio_color = colors.color_for_zram(ratio);
            rows.push(OverviewRow {
                label: "ZRAM".to_string(),
                value: format!("{}{:.1}x{}", ratio_color, ratio, Colors::RESET),
            });
        }

        rows.push(OverviewRow {
            label: "SWAP(ZRAM)".to_string(),
            value: format!("{}{:.0}%{}", swap_color, zram_swap_percent, Colors::RESET),
        });
        rows.push(OverviewRow {
            label: "SWAP(SSD)".to_string(),
            value: format!(
                "{}{:.0}%{}",
                disk_swap_color,
                disk_swap_percent,
                Colors::RESET
            ),
        });
        let load_value = format!(
            "{}{:.2}{}  {}{:.2}{}  {}{:.2}{}",
            load_color,
            load1,
            Colors::RESET,
            load5_color,
            load5,
            Colors::RESET,
            load10_color,
            load10,
            Colors::RESET
        );
        rows.push(OverviewRow {
            label: "LOAD".to_string(),
            value: load_value,
        });

        let mut table = Table::new(&rows);
        table.with(Style::empty());
        table.with(Remove::row(Rows::first()));
        println!();
        print!("{}", table);
    }
}

impl Default for OverviewTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overview_table_creation() {
        let _overview = OverviewTable::new();
    }

    #[test]
    fn test_overview_print_no_panic() {
        let overview = OverviewTable::new();
        overview.print(
            50.0,
            60.0,
            0.0,
            0.0,
            1.0,
            1.5,
            2.0,
            4,
            85,
            "EXCELLENT",
            None,
            &ZramReader::new(),
        );
    }
}
