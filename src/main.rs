use clap::Parser;
use std::time::Instant;

mod color;
mod input;
mod keys;
mod logger;
mod modes;
mod overview;
mod process_list;
mod process_table_render;
mod system_monitor;
mod ui;
mod zram_stats;

use input::InputHandler;
use keys::{KeyAction, Keys};
use logger::Logger;
use modes::help::HelpMode;
use modes::kill::KillMode;
use modes::pause::PauseMode;
use modes::renice::ReniceMode;
use process_list::{HealthCalculator, ProcessInfo, ProcessList};
use system_monitor::SystemMonitor;
use ui::TerminalUI;
use zram_stats::ZramReader;

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "2.0")]
    interval: f64,
}

fn enable_raw_mode() -> libc::termios {
    let mut term = unsafe { std::mem::zeroed() };
    unsafe { libc::tcgetattr(0, &mut term) };
    let original = term;
    term.c_lflag &= !(libc::ICANON | libc::ECHO);
    unsafe { libc::tcsetattr(0, libc::TCSANOW, &term) };
    original
}

fn disable_raw_mode(termios: &libc::termios) {
    unsafe { libc::tcsetattr(0, libc::TCSANOW, termios) };
}

fn main() {
    let args = Args::parse();
    let logger = Logger::new();

    logger.info("Starting RTOP");

    let original_termios = enable_raw_mode();

    let mut input = InputHandler::new();
    let keys = Keys::new();
    let mut monitor = SystemMonitor::new();
    let mut process_list = ProcessList::new();
    let zram_reader = ZramReader::new();
    let ui = TerminalUI::new();

    let mut renice_mode = ReniceMode::new();
    let mut kill_mode = KillMode::new();
    let mut pause_mode = PauseMode::new();
    let mut help_mode = HelpMode::new();

    let mut advanced = false;

    let mut cpu = 0.0;
    let mut mem_percent = 0.0;
    let mut swap_percent = 0.0;
    let mut load1 = 0.0;
    let mut load5 = 0.0;
    let mut load10 = 0.0;
    let mut cores = 1;
    let mut health = 100;
    let mut health_label = "EXCELLENT";

    let mut zram_stats: Option<zram_stats::ZramStats> = None;
    let mut frozen_procs: Vec<process_list::ProcessInfo> = Vec::new();

    loop {
        let key = input.read_key();

        if let Some(k) = key {
            let action = keys.handle_key(
                key,
                renice_mode.active,
                kill_mode.active,
                frozen_procs.len(),
                &logger,
            );

            match action {
                KeyAction::Quit => break,
                KeyAction::TogglePause => {
                    pause_mode.toggle();
                    logger.info(&format!("Pause toggled: {}", pause_mode.active));
                }
                KeyAction::ToggleAdvanced => {
                    advanced = !advanced;
                    logger.info(&format!("Advanced toggled: {}", advanced));
                }
                KeyAction::ToggleHelp => {
                    help_mode.toggle();
                    logger.info(&format!("Help toggled: {}", help_mode.active));
                }
                KeyAction::ActivateRenice => {
                    renice_mode.activate();
                    kill_mode.deactivate();
                    frozen_procs = process_list.top_by_cpu(10).into_iter().cloned().collect();
                    logger.info("Renice mode activated");
                }
                KeyAction::ActivateKill => {
                    kill_mode.activate();
                    renice_mode.deactivate();
                    frozen_procs = process_list.top_by_cpu(10).into_iter().cloned().collect();
                    logger.info("Kill mode activated");
                }
                KeyAction::ExecuteAction => {
                    if renice_mode.active && renice_mode.selection < frozen_procs.len() {
                        let proc = &frozen_procs[renice_mode.selection];
                        unsafe {
                            libc::setpriority(
                                libc::PRIO_PROCESS,
                                proc.pid.as_u32() as libc::id_t,
                                renice_mode.nice_value,
                            );
                        }
                        logger.info(&format!(
                            "Reniced PID {} to {}",
                            proc.pid, renice_mode.nice_value
                        ));
                        renice_mode.deactivate();
                    } else if kill_mode.active && kill_mode.selection < frozen_procs.len() {
                        let proc = &frozen_procs[kill_mode.selection];
                        let sig = if kill_mode.signal == 9 {
                            libc::SIGKILL
                        } else {
                            libc::SIGTERM
                        };
                        unsafe { libc::kill(proc.pid.as_u32() as i32, sig) };
                        logger.info(&format!(
                            "Sent signal {} to PID {}",
                            kill_mode.signal, proc.pid
                        ));
                        kill_mode.deactivate();
                    }
                }
                KeyAction::NavigateUp => {
                    if renice_mode.active {
                        renice_mode.selection = renice_mode.selection.saturating_sub(1);
                    } else if kill_mode.active {
                        kill_mode.selection = kill_mode.selection.saturating_sub(1);
                    }
                }
                KeyAction::NavigateDown => {
                    if renice_mode.active {
                        renice_mode.selection =
                            (renice_mode.selection + 1).min(frozen_procs.len().saturating_sub(1));
                    } else if kill_mode.active {
                        kill_mode.selection =
                            (kill_mode.selection + 1).min(frozen_procs.len().saturating_sub(1));
                    }
                }
                KeyAction::NiceValueUp => {
                    if renice_mode.active {
                        renice_mode.nice_value = (renice_mode.nice_value + 1).min(19);
                    }
                }
                KeyAction::NiceValueDown => {
                    if renice_mode.active {
                        renice_mode.nice_value = (renice_mode.nice_value - 1).max(-20);
                    }
                }
                KeyAction::Signal9 => {
                    if kill_mode.active {
                        kill_mode.signal = 9;
                    }
                }
                KeyAction::Signal15 => {
                    if kill_mode.active {
                        kill_mode.signal = 15;
                    }
                }
                KeyAction::None => {}
            }
        }

        let should_refresh = !(pause_mode.active || renice_mode.active || kill_mode.active);

        if should_refresh {
            let start = Instant::now();
            monitor.refresh();
            logger.log_timed("monitor.refresh", start);

            let start = Instant::now();
            cpu = monitor.get_stats().cpu;
            mem_percent = monitor.get_stats().mem_percent;
            swap_percent = monitor.get_stats().swap_percent;
            cores = monitor.get_stats().cores;
            (load1, load5, load10) = monitor.load_average();
            logger.log_timed("get stats", start);

            let start = Instant::now();
            zram_stats = zram_reader.read();
            logger.log_timed("zram read", start);

            let start = Instant::now();
            process_list.refresh(&monitor.sys);
            logger.log_timed("process refresh", start);

            let zram_ratio = zram_stats
                .as_ref()
                .map(|z| zram_reader.compression_ratio(z))
                .unwrap_or(0.0);

            let (h, label) =
                HealthCalculator::calculate(mem_percent, swap_percent, load1, zram_ratio, cores);
            health = h;
            health_label = label;
        }

        if renice_mode.active || kill_mode.active {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if key.is_none() {
                continue;
            }
        }

        if help_mode.active {
            ui.clear_screen();
            ui.print_help(args.interval, advanced, pause_mode.active);
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        ui.clear_screen();

        let start = Instant::now();
        ui.print_header(
            cpu,
            mem_percent,
            swap_percent,
            load1,
            load5,
            load10,
            cores,
            health,
            health_label,
            zram_stats.as_ref(),
            &zram_reader,
        );

        if advanced {
            ui.print_advanced_info();
        }

        let procs: Vec<&ProcessInfo> = if renice_mode.active || kill_mode.active {
            frozen_procs.iter().collect()
        } else {
            process_list.top_by_cpu(10)
        };

        ui.print_process_list(
            &procs,
            renice_mode.active,
            renice_mode.selection,
            kill_mode.active,
            kill_mode.selection,
        );

        if renice_mode.active {
            ui.print_renice_status(renice_mode.nice_value);
        }
        if kill_mode.active {
            ui.print_kill_status(kill_mode.signal);
        }

        ui.print_footer(
            args.interval,
            advanced,
            help_mode.active,
            pause_mode.active,
            renice_mode.active,
            kill_mode.active,
        );

        logger.log_timed("UI render", start);

        if pause_mode.active || renice_mode.active || kill_mode.active {
            std::thread::sleep(std::time::Duration::from_millis(100));
        } else {
            std::thread::sleep(std::time::Duration::from_secs_f64(args.interval));
        }
    }

    disable_raw_mode(&original_termios);
    logger.info("Exiting RTOP");
}
