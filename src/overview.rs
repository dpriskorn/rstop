use crate::color::{ColorScheme, Colors};
use crate::zram_stats::{ZramReader, ZramStats};

pub struct OverviewTable;

impl OverviewTable {
    pub fn new() -> Self {
        OverviewTable
    }

    pub fn print(
        &self,
        cpu: f32,
        mem_percent: f32,
        swap_percent: f32,
        load1: f64,
        load5: f64,
        load10: f64,
        cores: usize,
        health: i32,
        health_label: &str,
        zram_stats: Option<&ZramStats>,
        zram_reader: &ZramReader,
    ) {
        let colors = ColorScheme::global();
        let cpu_color = colors.color_for_cpu(cpu);
        let swap_color = colors.color_for_swap(swap_percent);
        let load_color = colors.color_for_load(load1, cores);
        let load5_color = colors.color_for_load(load5, cores);
        let load10_color = colors.color_for_load(load10, cores);
        let health_color = colors.color_for_health(health);

        println!("\n");
        println!(
            "{}HEALTH{}  {}{}/100 [{}]{}",
            Colors::BOLD,
            Colors::RESET,
            health_color,
            health,
            health_label,
            Colors::RESET
        );
        println!(
            "{}CPU{}     {}{:.1}%{}",
            Colors::BOLD,
            Colors::RESET,
            cpu_color,
            cpu,
            Colors::RESET
        );
        println!(
            "{}RAM{}     {}{:.1}%{}",
            Colors::BOLD,
            Colors::RESET,
            Colors::WHITE,
            mem_percent,
            Colors::RESET
        );

        if let Some(z) = zram_stats {
            let ratio = zram_reader.compression_ratio(z);
            let ratio_color = colors.color_for_zram(ratio);
            println!(
                "{}ZRAM{}    {}{:.2}x{}",
                Colors::BOLD,
                Colors::RESET,
                ratio_color,
                ratio,
                Colors::RESET
            );
        }

        println!(
            "{}SWAP{}    {}{:.1}%{}",
            Colors::BOLD,
            Colors::RESET,
            swap_color,
            swap_percent,
            Colors::RESET
        );
        println!(
            "{}LOAD{}    {}{:.2}  {}{:.2}  {}{:.2}{}",
            Colors::BOLD,
            Colors::RESET,
            load_color,
            load1,
            load5_color,
            load5,
            load10_color,
            load10,
            Colors::RESET
        );
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
            10.0,
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
