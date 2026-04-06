use crate::process_list::ProcessInfo;
use crate::process_table_render::ProcessTable;

pub struct Colors;

impl Colors {
    pub const RED: &'static str = "\x1b[91m";
    pub const GREEN: &'static str = "\x1b[92m";
    pub const YELLOW: &'static str = "\x1b[93m";
    pub const BLUE: &'static str = "\x1b[94m";
    #[allow(dead_code)]
    pub const MAGENTA: &'static str = "\x1b[95m";
    pub const CYAN: &'static str = "\x1b[96m";
    pub const WHITE: &'static str = "\x1b[97m";
    pub const BOLD: &'static str = "\x1b[1m";
    pub const RESET: &'static str = "\x1b[0m";
}

pub struct TerminalUI;

impl TerminalUI {
    pub fn new() -> Self {
        TerminalUI
    }

    pub fn clear_screen(&self) {
        print!("{}\x1b[2J\x1b[H", Colors::RESET);
    }

    fn color_for_load(&self, load: f64, cores: usize) -> &'static str {
        if load > cores as f64 * 1.5 {
            Colors::RED
        } else if load > cores as f64 {
            Colors::YELLOW
        } else {
            Colors::WHITE
        }
    }

    fn color_for_percent(&self, value: f32, threshold: f32) -> &'static str {
        if value > threshold {
            Colors::RED
        } else {
            Colors::WHITE
        }
    }

    fn color_for_health(&self, score: i32) -> &'static str {
        if score >= 85 {
            Colors::GREEN
        } else if score >= 70 {
            Colors::CYAN
        } else if score >= 50 {
            Colors::YELLOW
        } else {
            Colors::RED
        }
    }

    pub fn print_header(
        &self,
        cpu: f32,
        mem_percent: f32,
        swap_percent: f32,
        load1: f64,
        cores: usize,
    ) {
        let cpu_color = self.color_for_percent(cpu, 80.0);
        let swap_color = self.color_for_percent(swap_percent, 50.0);
        let load_color = self.color_for_load(load1, cores);

        print!(
            "{}{}CPU:{}   {}{:.0}%{}",
            Colors::BOLD,
            Colors::CYAN,
            Colors::RESET,
            cpu_color,
            cpu,
            Colors::RESET
        );
        print!(
            "{}{}RAM:{}   {}{:.0}%{}",
            Colors::BOLD,
            Colors::CYAN,
            Colors::RESET,
            Colors::WHITE,
            mem_percent,
            Colors::RESET
        );
        print!(
            "{}{}SWAP:{}  {}{:.0}%{}",
            Colors::BOLD,
            Colors::CYAN,
            Colors::RESET,
            swap_color,
            swap_percent,
            Colors::RESET
        );
        println!(
            "{}{}AVG. LOAD:{}  {}{:.2}{}",
            Colors::BOLD,
            Colors::CYAN,
            Colors::RESET,
            load_color,
            load1,
            Colors::RESET
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn print_advanced_info(
        &self,
        load5: f64,
        load10: f64,
        cores: usize,
        health: i32,
        health_label: &str,
        zram_stats: Option<&crate::zram_stats::ZramStats>,
        zram_reader: &crate::zram_stats::ZramReader,
    ) {
        let load5_color = self.color_for_load(load5, cores);
        let _load10_color = self.color_for_load(load10, cores);
        print!(
            "{}{}5m:{}  {:.2}  {}{}10m:{}  {:.2}{}",
            Colors::CYAN,
            Colors::RESET,
            load5_color,
            load5,
            Colors::CYAN,
            Colors::RESET,
            load10,
            Colors::RESET,
            Colors::RESET
        );

        let health_color = self.color_for_health(health);
        print!(
            "{}{}HEALTH:{} {}/100 [{}{}{}]",
            health_color,
            Colors::RESET,
            health,
            health_color,
            health_label,
            Colors::RESET,
            Colors::RESET
        );

        if let Some(z) = zram_stats {
            let ratio = zram_reader.compression_ratio(z);
            let saved = zram_reader.saved_bytes(z);
            println!(
                "{}{}ZRAM:{} orig={}MB compr={}MB ratio={:.1}x saved={}MB",
                Colors::CYAN,
                Colors::RESET,
                z.orig / 1024 / 1024,
                z.compr / 1024 / 1024,
                ratio,
                saved / 1024 / 1024,
                Colors::RESET
            );
        }
    }

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
    fn test_color_for_load_white() {
        let ui = TerminalUI::new();
        assert_eq!(ui.color_for_load(1.0, 4), Colors::WHITE);
    }

    #[test]
    fn test_color_for_load_yellow() {
        let ui = TerminalUI::new();
        assert_eq!(ui.color_for_load(5.0, 4), Colors::YELLOW);
    }

    #[test]
    fn test_color_for_load_red() {
        let ui = TerminalUI::new();
        assert_eq!(ui.color_for_load(10.0, 4), Colors::RED);
    }

    #[test]
    fn test_color_for_percent_white() {
        let ui = TerminalUI::new();
        assert_eq!(ui.color_for_percent(50.0, 80.0), Colors::WHITE);
    }

    #[test]
    fn test_color_for_percent_red() {
        let ui = TerminalUI::new();
        assert_eq!(ui.color_for_percent(90.0, 80.0), Colors::RED);
    }

    #[test]
    fn test_color_for_health_excellent() {
        let ui = TerminalUI::new();
        assert_eq!(ui.color_for_health(90), Colors::GREEN);
    }

    #[test]
    fn test_color_for_health_good() {
        let ui = TerminalUI::new();
        assert_eq!(ui.color_for_health(75), Colors::CYAN);
    }

    #[test]
    fn test_color_for_health_ok() {
        let ui = TerminalUI::new();
        assert_eq!(ui.color_for_health(55), Colors::YELLOW);
    }

    #[test]
    fn test_color_for_health_stressed() {
        let ui = TerminalUI::new();
        assert_eq!(ui.color_for_health(30), Colors::RED);
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
}
