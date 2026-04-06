use clap::Parser;
use std::time::Instant;

mod color;
mod config;
mod filter;
mod health;
mod keys;
mod logger;
mod process_list;
mod swap;
mod system_monitor;
mod tui;
mod zram_stats;

use config::Config;
use filter::ProcessFilter;
use health::{HealthCalculator, HealthFactors};
use keys::{KeyAction, Keys};
use logger::Logger;
use process_list::{ProcessInfo, ProcessList};
use system_monitor::SystemMonitor;
use tui::{AppState, TuiRenderer};
use zram_stats::ZramReader;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "2.0")]
    interval: f64,
    #[arg(short, long)]
    min_cpu: Option<f64>,
    #[arg(short, long, value_delimiter = ',')]
    exclude: Vec<String>,
}

struct App {
    monitor: SystemMonitor,
    process_list: ProcessList,
    process_filter: ProcessFilter,
    zram_reader: ZramReader,
    keys: Keys,
    logger: Logger,
    state: AppState,
    cpu: f32,
    zram_swap_percent: f32,
    disk_swap_percent: f32,
    cores: usize,
    load1: f64,
    load5: f64,
    load10: f64,
    health: i32,
    health_label: &'static str,
    health_factors: HealthFactors,
    zram_stats: Option<zram_stats::ZramStats>,
    frozen_procs: Vec<ProcessInfo>,
}

impl App {
    fn new(min_cpu: f32, exclude_names: Vec<String>, logger: Logger) -> Self {
        App {
            monitor: SystemMonitor::new(),
            process_list: ProcessList::new(),
            process_filter: ProcessFilter::new(min_cpu, exclude_names),
            zram_reader: ZramReader::new(),
            keys: Keys::new(),
            logger,
            state: AppState::new(),
            cpu: 0.0,
            zram_swap_percent: 0.0,
            disk_swap_percent: 0.0,
            cores: 1,
            load1: 0.0,
            load5: 0.0,
            load10: 0.0,
            health: 100,
            health_label: "EXCELLENT",
            health_factors: HealthFactors {
                mem_penalty: 0,
                swap_penalty: 0,
                load_penalty: 0,
                zram_penalty: 0,
            },
            zram_stats: None,
            frozen_procs: Vec::new(),
        }
    }

    fn handle_key(&mut self, key: Option<u8>) {
        let action = self.keys.handle_key(
            key,
            self.state.renice_active,
            self.state.kill_active,
            self.state.help_active,
            self.state.pause_active,
            self.frozen_procs.len(),
            &self.logger,
        );

        match action {
            KeyAction::Quit => panic!("Quit requested"),
            KeyAction::ExitMode => {
                self.state.renice_active = false;
                self.state.kill_active = false;
                self.state.help_active = false;
                self.state.pause_active = false;
                self.logger.info("Mode deactivated");
            }
            KeyAction::TogglePause => {
                self.state.pause_active = !self.state.pause_active;
            }
            KeyAction::ToggleAdvanced => {
                self.state.advanced = !self.state.advanced;
            }
            KeyAction::ToggleHelp => {
                self.state.help_active = !self.state.help_active;
            }
            KeyAction::ToggleSort => {
                self.state.sort_by_mem = !self.state.sort_by_mem;
            }
            KeyAction::ActivateRenice => {
                self.state.renice_active = true;
                self.state.kill_active = false;
                self.frozen_procs = if self.state.sort_by_mem {
                    self.process_list.top_by_mem(30)
                } else {
                    self.process_list.top_by_cpu(30)
                }
                .into_iter()
                .cloned()
                .collect();
                self.logger.info(&format!(
                    "Renice mode with {} procs",
                    self.frozen_procs.len()
                ));
            }
            KeyAction::ActivateKill => {
                self.state.kill_active = true;
                self.state.renice_active = false;
                self.frozen_procs = if self.state.sort_by_mem {
                    self.process_list.top_by_mem(30)
                } else {
                    self.process_list.top_by_cpu(30)
                }
                .into_iter()
                .cloned()
                .collect();
                self.logger.info("Kill mode activated");
            }
            KeyAction::ExecuteAction => self.handle_execute_action(),
            KeyAction::NavigateUp => {
                if self.state.renice_active && self.state.renice_selection > 0 {
                    self.state.renice_selection -= 1;
                }
                if self.state.kill_active && self.state.kill_selection > 0 {
                    self.state.kill_selection -= 1;
                }
            }
            KeyAction::NavigateDown => {
                if self.state.renice_active
                    && self.state.renice_selection < self.frozen_procs.len().saturating_sub(1)
                {
                    self.state.renice_selection += 1;
                }
                if self.state.kill_active
                    && self.state.kill_selection < self.frozen_procs.len().saturating_sub(1)
                {
                    self.state.kill_selection += 1;
                }
            }
            KeyAction::NiceValueUp => {
                if self.state.renice_active {
                    self.state.renice_nice_value = (self.state.renice_nice_value + 1).min(19);
                }
            }
            KeyAction::NiceValueDown => {
                if self.state.renice_active {
                    self.state.renice_nice_value = (self.state.renice_nice_value - 1).max(-20);
                }
            }
            KeyAction::Signal9 => {
                if self.state.kill_active {
                    self.state.kill_signal = 9;
                }
            }
            KeyAction::Signal15 => {
                if self.state.kill_active {
                    self.state.kill_signal = 15;
                }
            }
            KeyAction::None => {}
        }
    }

    fn handle_execute_action(&mut self) {
        if self.state.renice_active && self.state.renice_selection < self.frozen_procs.len() {
            let proc = &self.frozen_procs[self.state.renice_selection];
            self.logger.info(&format!(
                "Attempting to renice PID {} to {}",
                proc.pid, self.state.renice_nice_value
            ));
            let is_root = unsafe { libc::geteuid() } == 0;

            if self.state.renice_nice_value < 1 && !is_root {
                self.logger
                    .error(&format!("Cannot renice PID {} - run as root", proc.pid));
                self.state.renice_active = false;
                return;
            }

            let result = unsafe {
                libc::setpriority(
                    libc::PRIO_PROCESS,
                    proc.pid.as_u32() as libc::id_t,
                    self.state.renice_nice_value,
                )
            };

            let errno_val = unsafe { *libc::__errno_location() };
            self.logger.info(&format!(
                "setpriority result={}, errno={}",
                result, errno_val
            ));

            if result == 0 {
                self.logger.info(&format!(
                    "Reniced PID {} to {}",
                    proc.pid, self.state.renice_nice_value
                ));
                self.state.renice_active = false;
            } else {
                self.logger.error(&format!(
                    "Failed to renice PID {}, errno={}",
                    proc.pid, errno_val
                ));
                self.state.renice_active = false;
            }
        }
        if self.state.kill_active && self.state.kill_selection < self.frozen_procs.len() {
            let proc = &self.frozen_procs[self.state.kill_selection];
            let result = unsafe {
                libc::kill(
                    proc.pid.as_u32() as libc::pid_t,
                    self.state.kill_signal as libc::c_int,
                )
            };
            if result == 0 {
                self.logger.info(&format!(
                    "Sent signal {} to PID {}",
                    self.state.kill_signal, proc.pid
                ));
            } else {
                self.logger
                    .error(&format!("Failed to kill PID {}", proc.pid));
            }
        }
    }

    fn refresh(&mut self, interval: f64, last_refresh: &mut Instant) {
        if self.state.pause_active
            || self.state.renice_active
            || self.state.kill_active
            || self.state.help_active
        {
            return;
        }

        if last_refresh.elapsed().as_secs_f64() < interval {
            return;
        }

        let start = Instant::now();
        self.monitor.refresh();
        self.logger.log_timed("monitor.refresh", start);

        self.cpu = self.monitor.get_stats().cpu;
        self.zram_swap_percent = self.monitor.get_stats().zram_swap_percent;
        self.disk_swap_percent = self.monitor.get_stats().disk_swap_percent;
        self.cores = self.monitor.get_stats().cores;
        (self.load1, self.load5, self.load10) = self.monitor.load_average();

        self.zram_stats = self.zram_reader.read();

        self.process_list.refresh(&self.monitor.sys);

        let zram_ratio = self
            .zram_stats
            .as_ref()
            .map(|z| self.zram_reader.compression_ratio(z))
            .unwrap_or(0.0);

        let (h, label) =
            HealthCalculator::calculate(self.disk_swap_percent, self.load1, zram_ratio, self.cores);
        self.health = h;
        self.health_label = label;
        self.health_factors =
            HealthFactors::calculate(self.disk_swap_percent, self.load1, zram_ratio, self.cores);

        *last_refresh = Instant::now();
    }

    fn get_processes(&self) -> Vec<ProcessInfo> {
        if self.state.renice_active || self.state.kill_active {
            self.frozen_procs.clone()
        } else {
            let all = if self.state.sort_by_mem {
                self.process_list.top_by_mem(30)
            } else {
                self.process_list.top_by_cpu(30)
            };
            self.process_filter.filter(&all)
        }
    }
}

fn main() {
    let args = Args::parse();
    let logger = Logger::new();
    let config = Config::load();

    let (min_cpu, exclude_names, interval) =
        config.merge_with_args(args.min_cpu, args.exclude, args.interval);

    logger.info("Starting RTOP");

    let mut app = App::new(min_cpu as f32, exclude_names, logger);
    let mut tui = TuiRenderer::new().expect("Failed to initialize TUI");

    let mut last_refresh = Instant::now();

    loop {
        let key = tui.poll_event();

        if key.is_some() {
            app.handle_key(key);
            if !app.state.renice_active
                && !app.state.kill_active
                && !app.state.help_active
                && !app.state.pause_active
            {
                break;
            }
        }

        app.refresh(interval, &mut last_refresh);

        let procs = app.get_processes();

        tui.draw(
            app.cpu,
            app.zram_swap_percent,
            app.disk_swap_percent,
            app.load1,
            app.load5,
            app.load10,
            app.cores,
            app.health,
            app.health_label,
            &app.health_factors,
            app.zram_stats.as_ref(),
            &app.zram_reader,
            &procs,
            &app.state,
        );

        let sleep_time = if app.state.help_active || app.state.pause_active {
            50
        } else if app.state.renice_active || app.state.kill_active {
            if key.is_none() {
                50
            } else {
                10
            }
        } else {
            50
        };
        std::thread::sleep(std::time::Duration::from_millis(sleep_time));
    }

    app.logger.info("Exiting RTOP");
}
