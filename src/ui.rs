use crate::color::Colors;
use crate::overview::OverviewTable;
use crate::process_list::ProcessInfo;
use crate::process_table_render::ProcessTable;

pub struct TerminalUI;

impl TerminalUI {
    pub fn new() -> Self {
        TerminalUI
    }

    pub fn clear_screen(&self) {
        print!("{}\x1b[2J\x1b[H", Colors::RESET);
    }

    pub fn print_header(
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
        zram_stats: Option<&crate::zram_stats::ZramStats>,
        zram_reader: &crate::zram_stats::ZramReader,
    ) {
        let overview = OverviewTable::new();
        overview.print(
            cpu,
            mem_percent,
            swap_percent,
            load1,
            load5,
            load10,
            cores,
            health,
            health_label,
            zram_stats,
            zram_reader,
        );
    }

    pub fn print_advanced_info(&self) {}

    fn build_markers(
        &self,
        advanced: bool,
        help: bool,
        paused: bool,
        renice: bool,
        kill: bool,
    ) -> String {
        let mut markers = String::new();
        if advanced {
            markers.push_str(&format!(" {}ADVANCED{}", Colors::CYAN, Colors::RESET));
        }
        if help {
            markers.push_str(&format!(" {}HELP{}", Colors::CYAN, Colors::RESET));
        }
        if paused {
            markers.push_str(&format!(" {}PAUSED{}", Colors::YELLOW, Colors::RESET));
        }
        if renice {
            markers.push_str(&format!(" {}RENICE{}", Colors::YELLOW, Colors::RESET));
        }
        if kill {
            markers.push_str(&format!(" {}KILL{}", Colors::RED, Colors::RESET));
        }
        markers
    }

    pub fn print_process_list(
        &self,
        processes: &[&ProcessInfo],
        renice_active: bool,
        renice_sel: usize,
        kill_active: bool,
        kill_sel: usize,
    ) {
        let table = ProcessTable::new();
        table.print(processes, renice_active, renice_sel, kill_active, kill_sel);
    }

    pub fn print_renice_status(&self, nice_value: i32) {
        println!("\n{}{}RENICE MODE:{}{} nice={}{}{}  {}up/down=select  left/right=nice value  enter=apply{}",
            Colors::BOLD, Colors::YELLOW, Colors::RESET, Colors::YELLOW, nice_value, Colors::RESET, Colors::CYAN, Colors::RESET, Colors::RESET);
    }

    pub fn print_kill_status(&self, signal: i32) {
        let sig_color = if signal == 9 {
            Colors::RED
        } else {
            Colors::GREEN
        };
        println!("\n{}{}KILL MODE:{}{} signal={}{}{}  {}up/down=select  left/right=toggle signal  enter=send{}",
            Colors::BOLD, Colors::RED, Colors::RESET, sig_color, signal, Colors::RESET, Colors::CYAN, Colors::RESET, Colors::RESET);
    }

    #[allow(dead_code)]
    pub fn print_error_in_footer(&self, message: &str) {
        println!("{}{}", Colors::RED, message);
    }

    pub fn print_help(&self, interval: f64, advanced: bool, paused: bool) {
        self.clear_screen();
        println!("{}{}HELP{}", Colors::BOLD, Colors::BLUE, Colors::RESET);
        println!(
            "\n{}{}ZRAM RATIO:{}",
            Colors::BOLD,
            Colors::WHITE,
            Colors::RESET
        );
        println!("  orig    = original data size before compression");
        println!("  compr   = compressed size in zram");
        println!("  ratio   = orig / compr (higher = better compression)");
        println!("  saved   = orig - compr (actual RAM saved)");
        println!(
            "  {}Example:{} ratio 4.0x means 1000MB compresses to 250MB",
            Colors::CYAN,
            Colors::RESET
        );
        println!(
            "\n{}{}HEALTH SCORE:{}",
            Colors::BOLD,
            Colors::WHITE,
            Colors::RESET
        );
        println!(
            "  85+  = {}EXCELLENT{} - all normal",
            Colors::GREEN,
            Colors::RESET
        );
        println!(
            "  70-84 = {}GOOD{} - slight load",
            Colors::CYAN,
            Colors::RESET
        );
        println!(
            "  50-69 = {}OK{} - elevated load",
            Colors::YELLOW,
            Colors::RESET
        );
        println!(
            "  0-49  = {}STRESSED{} - high load",
            Colors::RED,
            Colors::RESET
        );
        println!("\n{}{}KEYS:{}", Colors::BOLD, Colors::WHITE, Colors::RESET);
        println!("  q/ESC = quit");
        println!("  p      = pause display");
        println!("  a      = advanced mode");
        println!("  h      = toggle this help");
        println!("  r      = renice mode");
        println!("  k      = kill mode");
        println!(
            "\n{}{}interval={:.1}s{}",
            Colors::BOLD,
            Colors::CYAN,
            interval,
            Colors::RESET
        );

        let markers = self.build_markers(advanced, false, paused, false, false);
        print!(
            "\nq=quit | p=pause | a=advanced | h=help | r=renice | k=kill | interval={:.1}s{}",
            interval,
            Colors::RESET
        );
        print!("{}", markers);
    }

    pub fn print_footer(
        &self,
        interval: f64,
        advanced: bool,
        help: bool,
        paused: bool,
        renice: bool,
        kill: bool,
    ) {
        let markers = self.build_markers(advanced, help, paused, renice, kill);
        println!(
            "\nq=quit | p=pause | a=advanced | h=help | r=renice | k=kill | interval={:.1}s{}",
            interval,
            Colors::RESET
        );
        print!("{}", markers);
        std::io::Write::flush(&mut std::io::stdout()).ok();
    }
}

impl Default for TerminalUI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{ColorScheme, Colors};

    #[test]
    fn test_colors_exist() {
        assert!(!Colors::RED.is_empty());
        assert!(!Colors::GREEN.is_empty());
    }

    #[test]
    fn test_terminal_ui_creation() {
        let _ui = TerminalUI::new();
    }

    #[test]
    fn test_clear_screen() {
        let ui = TerminalUI::new();
        ui.clear_screen();
    }

    #[test]
    fn test_build_markers_empty() {
        let ui = TerminalUI::new();
        let result = ui.build_markers(false, false, false, false, false);
        assert_eq!(result, "");
    }

    #[test]
    fn test_build_markers_advanced() {
        let ui = TerminalUI::new();
        let result = ui.build_markers(true, false, false, false, false);
        assert!(result.contains("ADVANCED"));
    }

    #[test]
    fn test_build_markers_all() {
        let ui = TerminalUI::new();
        let result = ui.build_markers(true, true, true, true, true);
        assert!(result.contains("ADVANCED"));
        assert!(result.contains("HELP"));
        assert!(result.contains("PAUSED"));
        assert!(result.contains("RENICE"));
        assert!(result.contains("KILL"));
    }

    #[test]
    fn test_color_for_load_yellow() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_load(5.0, 4), Colors::YELLOW);
    }

    #[test]
    fn test_color_for_load_red() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_load(10.0, 4), Colors::RED);
    }

    #[test]
    fn test_color_for_percent_white() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_percent(50.0, 80.0), Colors::WHITE);
    }

    #[test]
    fn test_color_for_percent_red() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_percent(90.0, 80.0), Colors::RED);
    }

    #[test]
    fn test_color_for_health_excellent() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_health(90), Colors::GREEN);
    }

    #[test]
    fn test_color_for_health_good() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_health(75), Colors::CYAN);
    }

    #[test]
    fn test_color_for_health_ok() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_health(55), Colors::YELLOW);
    }

    #[test]
    fn test_color_for_health_stressed() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_health(30), Colors::RED);
    }
}
