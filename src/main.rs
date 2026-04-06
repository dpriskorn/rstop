use clap::Parser;
use std::time::Instant;

mod color;
mod config;
mod filter;
mod health;
mod input;
mod keyboard_commands;
mod keys;
mod logger;
mod modes;
mod overview;
mod process_list;
mod process_table_render;
mod swap;
mod system_monitor;
mod ui;
mod zram_stats;

use config::Config;
use filter::ProcessFilter;
use health::{HealthCalculator, HealthFactors};
use input::InputHandler;
use keyboard_commands::KeyboardCommands;
use keys::{KeyAction, Keys};
use logger::Logger;
use modes::help::HelpMode;
use modes::kill::KillMode;
use modes::pause::PauseMode;
use modes::renice::ReniceMode;
use modes::sort::SortMode;
use process_list::{ProcessInfo, ProcessList};
use system_monitor::SystemMonitor;
use ui::TerminalUI;
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
    let config = Config::load();

    let (min_cpu, exclude_names, interval) =
        config.merge_with_args(args.min_cpu, args.exclude, args.interval);
    let refresh_interval = interval;

    logger.info("Starting RTOP");

    let original_termios = enable_raw_mode();

    let mut input = InputHandler::new();
    let keys = Keys::new();
    let mut monitor = SystemMonitor::new();
    let mut process_list = ProcessList::new();
    let process_filter = ProcessFilter::new(min_cpu, exclude_names);
    let zram_reader = ZramReader::new();
    let ui = TerminalUI::new();

    let mut renice_mode = ReniceMode::new();
    let mut kill_mode = KillMode::new();
    let mut pause_mode = PauseMode::new();
    let mut help_mode = HelpMode::new();
    let mut sort_mode = SortMode::new();

    let mut advanced = false;

    let mut cpu = 0.0;
    let mut mem_percent = 0.0;
    let mut zram_swap_percent = 0.0;
    let mut disk_swap_percent = 0.0;
    let mut load1 = 0.0;
    let mut load5 = 0.0;
    let mut load10 = 0.0;
    let mut cores = 1;
    let mut health = 100;
    let mut health_label = "EXCELLENT";
    let mut health_factors = HealthFactors {
        mem_penalty: 0,
        swap_penalty: 0,
        load_penalty: 0,
        zram_penalty: 0,
    };

    let mut zram_stats: Option<zram_stats::ZramStats> = None;
    let mut frozen_procs: Vec<process_list::ProcessInfo> = Vec::new();
    let mut error_msg: Option<String> = None;
    let mut error_until: Option<std::time::Instant> = None;

    loop {
        let key = input.read_key(&logger);

        let skip_render = (renice_mode.active || kill_mode.active) && key.is_none();

        if let Some(_k) = key {
            let action = keys.handle_key(
                key,
                renice_mode.active,
                kill_mode.active,
                help_mode.active,
                pause_mode.active,
                frozen_procs.len(),
                &logger,
            );

            match action {
                KeyAction::Quit => break,
                KeyAction::ExitMode => {
                    renice_mode.deactivate();
                    kill_mode.deactivate();
                    help_mode.deactivate();
                    pause_mode.deactivate();
                    logger.info("Mode deactivated");
                }
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
                KeyAction::ToggleSort => {
                    sort_mode.toggle();
                    logger.info(&format!("Sort by MEM: {}", sort_mode.sort_by_mem));
                }
                KeyAction::ActivateRenice => {
                    renice_mode.activate();
                    kill_mode.deactivate();
                    let all = if sort_mode.sort_by_mem {
                        process_list.top_by_mem(30)
                    } else {
                        process_list.top_by_cpu(30)
                    };
                    let owned: Vec<ProcessInfo> = all.into_iter().cloned().collect();
                    frozen_procs = process_filter.filter_owned(owned);
                    logger.info("Renice mode activated");
                }
                KeyAction::ActivateKill => {
                    kill_mode.activate();
                    renice_mode.deactivate();
                    let all = if sort_mode.sort_by_mem {
                        process_list.top_by_mem(30)
                    } else {
                        process_list.top_by_cpu(30)
                    };
                    let owned: Vec<ProcessInfo> = all.into_iter().cloned().collect();
                    frozen_procs = process_filter.filter_owned(owned);
                    logger.info("Kill mode activated");
                }
                KeyAction::ExecuteAction => {
                    if renice_mode.active && renice_mode.selection < frozen_procs.len() {
                        let proc = &frozen_procs[renice_mode.selection];
                        let is_root = unsafe { libc::geteuid() } == 0;
                        if renice_mode.nice_value < 1 && !is_root {
                            error_msg = Some(
                                "Cannot lower nice value - run as root to increase priority"
                                    .to_string(),
                            );
                            error_until =
                                Some(std::time::Instant::now() + std::time::Duration::from_secs(5));
                            logger.error(&format!(
                                "Failed renice: PID {} name='{}' current_nice={} target_nice={} - cannot lower nice without root",
                                proc.pid, proc.name, proc.nice, renice_mode.nice_value
                            ));
                            renice_mode.deactivate();
                        } else {
                            let result = unsafe {
                                libc::setpriority(
                                    libc::PRIO_PROCESS,
                                    proc.pid.as_u32() as libc::id_t,
                                    renice_mode.nice_value,
                                )
                            };
                            if result != 0 {
                                error_msg = Some(
                                    "Cannot lower nice value - run as root to increase priority"
                                        .to_string(),
                                );
                                error_until = Some(
                                    std::time::Instant::now() + std::time::Duration::from_secs(5),
                                );
                                let errno_val = unsafe { *libc::__errno_location() };
                                logger.error(&format!(
                                    "Failed renice: PID {} name='{}' current_nice={} target_nice={} errno={} - cannot lower nice without root",
                                    proc.pid, proc.name, proc.nice, renice_mode.nice_value, errno_val
                                ));
                            } else {
                                logger.info(&format!(
                                    "Reniced PID {} name='{}' from {} to {}",
                                    proc.pid, proc.name, proc.nice, renice_mode.nice_value
                                ));
                            }
                            renice_mode.deactivate();
                        }
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
                    if renice_mode.active && !frozen_procs.is_empty() {
                        renice_mode.selection =
                            (renice_mode.selection + 1).min(frozen_procs.len() - 1);
                    } else if kill_mode.active && !frozen_procs.is_empty() {
                        kill_mode.selection = (kill_mode.selection + 1).min(frozen_procs.len() - 1);
                    }
                }
                KeyAction::NiceValueUp => {
                    if renice_mode.active {
                        renice_mode.nice_value = (renice_mode.nice_value + 1).min(19);
                        logger.debug(&format!(
                            "NiceValueUp: new value = {}",
                            renice_mode.nice_value
                        ));
                    }
                }
                KeyAction::NiceValueDown => {
                    if renice_mode.active {
                        renice_mode.nice_value = (renice_mode.nice_value - 1).max(-20);
                        logger.debug(&format!(
                            "NiceValueDown: new value = {}",
                            renice_mode.nice_value
                        ));
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

        let should_refresh =
            !(pause_mode.active || renice_mode.active || kill_mode.active || help_mode.active);

        if should_refresh {
            let start = Instant::now();
            monitor.refresh();
            logger.log_timed("monitor.refresh", start);

            let start = Instant::now();
            cpu = monitor.get_stats().cpu;
            mem_percent = monitor.get_stats().mem_percent;
            zram_swap_percent = monitor.get_stats().zram_swap_percent;
            disk_swap_percent = monitor.get_stats().disk_swap_percent;
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

            let (h, label) = HealthCalculator::calculate(
                zram_swap_percent,
                disk_swap_percent,
                load1,
                zram_ratio,
                cores,
            );
            health = h;
            health_label = label;
            health_factors = HealthFactors::calculate(
                zram_swap_percent,
                disk_swap_percent,
                load1,
                zram_ratio,
                cores,
            );
        }

        if help_mode.active {
            ui.clear_screen();
            ui.print_help(refresh_interval, advanced, pause_mode.active);
            std::thread::sleep(std::time::Duration::from_millis(50));
            continue;
        }

        if pause_mode.active {
            std::thread::sleep(std::time::Duration::from_millis(50));
            continue;
        }

        ui.clear_screen();

        KeyboardCommands::new().print_line(refresh_interval);

        let start = Instant::now();
        ui.print_header(
            cpu,
            zram_swap_percent,
            disk_swap_percent,
            load1,
            load5,
            load10,
            cores,
            health,
            health_label,
            &health_factors,
            zram_stats.as_ref(),
            &zram_reader,
        );

        if advanced {
            ui.print_advanced_info();
        }

        let all_procs = if sort_mode.sort_by_mem {
            process_list.top_by_mem(30)
        } else {
            process_list.top_by_cpu(30)
        };
        let filtered_procs = process_filter.filter(&all_procs);

        let procs: Vec<&ProcessInfo> = if renice_mode.active || kill_mode.active {
            frozen_procs.iter().collect()
        } else {
            filtered_procs.iter().collect()
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

        if let Some(err) = &error_msg {
            if let Some(until) = error_until {
                if std::time::Instant::now() < until {
                    ui.print_error_in_footer(err);
                } else {
                    error_msg = None;
                    error_until = None;
                }
            }
        }

        logger.log_timed("UI render", start);

        if skip_render {
            std::thread::sleep(std::time::Duration::from_millis(20));
            continue;
        }

        if pause_mode.active || renice_mode.active || kill_mode.active {
            std::thread::sleep(std::time::Duration::from_millis(100));
        } else {
            std::thread::sleep(std::time::Duration::from_secs_f64(refresh_interval));
        }
    }

    disable_raw_mode(&original_termios);
    logger.info("Exiting RTOP");
}
