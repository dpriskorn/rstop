use clap::Parser;
use std::fs;
use sysinfo::System;
use tabled::settings::Style;
use tabled::Table;

const RED: &str = "\x1b[91m";
const GREEN: &str = "\x1b[92m";
const YELLOW: &str = "\x1b[93m";
const BLUE: &str = "\x1b[94m";
const MAGENTA: &str = "\x1b[95m";
const CYAN: &str = "\x1b[96m";
const WHITE: &str = "\x1b[97m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "2.0")]
    interval: f64,
}

struct ZramStats {
    orig: u64,
    compr: u64,
    mem: u64,
}

fn get_zram() -> Option<ZramStats> {
    let content = fs::read_to_string("/sys/block/zram0/mm_stat").ok()?;
    let vals: Vec<u64> = content
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    Some(ZramStats {
        orig: vals.first().copied().unwrap_or(0),
        compr: vals.get(1).copied().unwrap_or(0),
        mem: vals.get(2).copied().unwrap_or(0),
    })
}

fn system_health(
    mem_percent: f32,
    swap_percent: f32,
    load1: f64,
    zram: &Option<ZramStats>,
    cores: usize,
) -> i32 {
    let mut score = 100;

    if mem_percent > 85.0 {
        score -= 30;
    } else if mem_percent > 70.0 {
        score -= 15;
    }

    if swap_percent > 50.0 {
        score -= 25;
    } else if swap_percent > 20.0 {
        score -= 10;
    }

    let cores = cores as f64;
    if load1 > cores * 1.5 {
        score -= 25;
    } else if load1 > cores {
        score -= 10;
    }

    if let Some(z) = zram {
        if z.compr > 0 {
            let ratio = z.orig as f64 / z.compr as f64;
            if ratio < 1.5 {
                score -= 10;
            } else if ratio > 3.0 {
                score += 5;
            }
        }
    }

    score.max(0).min(100)
}

fn health_color(score: i32) -> &'static str {
    if score >= 85 {
        GREEN
    } else if score >= 70 {
        CYAN
    } else if score >= 50 {
        YELLOW
    } else {
        RED
    }
}

fn health_label(score: i32) -> &'static str {
    if score >= 85 {
        "EXCELLENT"
    } else if score >= 70 {
        "GOOD"
    } else if score >= 50 {
        "OK"
    } else {
        "STRESSED"
    }
}

fn mem_color(percent: f32) -> &'static str {
    if percent > 90.0 {
        RED
    } else {
        GREEN
    }
}

fn enable_raw_mode() -> libc::termios {
    unsafe {
        let mut termios = std::mem::zeroed();
        libc::tcgetattr(0, &mut termios);
        let original = termios;
        termios.c_lflag = original.c_lflag & !(libc::ICANON | libc::ECHO);
        libc::tcsetattr(0, libc::TCSANOW, &termios);
        original
    }
}

fn disable_raw_mode(termios: &libc::termios) {
    unsafe {
        libc::tcsetattr(0, libc::TCSANOW, termios);
    }
}

fn kbhit() -> Option<u8> {
    unsafe {
        let mut buf: [u8; 1] = [0; 1];
        let flags = libc::fcntl(0, libc::F_GETFL);
        libc::fcntl(0, libc::F_SETFL, flags | libc::O_NONBLOCK);
        let res = libc::read(0, buf.as_mut_ptr() as *mut libc::c_void, 1);
        if res == 1 {
            return Some(buf[0]);
        }
        None
    }
}

fn main() {
    let args = Args::parse();

    let mut sys = System::new_all();
    let _original_termios = enable_raw_mode();
    let mut paused = false;
    let mut advanced = false;
    let mut help = false;
    let mut renice_mode = false;
    let mut renice_sel: usize = 0;
    let mut renice_value: i32 = 19;
    let mut kill_mode = false;
    let mut kill_sel: usize = 0;
    let mut kill_signal: i32 = 9;

    let mut cpu = 0.0;
    let mut mem_percent = 0.0;
    let mut swap_percent = 0.0;
    let mut load1 = 0.0;
    let mut cores = 1;
    let mut health = 100;
    let mut label = "EXCELLENT";
    let mut hcolor = GREEN;
    let mut mcolor = GREEN;
    let mut zram: Option<ZramStats> = None;
    let mut procs: Vec<(sysinfo::Pid, String, f32, u64, u64)> = Vec::new();
    let mut load5 = 0.0;
    let mut load10 = 0.0;

    loop {
        let key = kbhit();

        if let Some(k) = key {
            match k {
                b'q' | b'Q' => break,
                0x1b => {
                    if renice_mode || kill_mode {
                        if kbhit() == Some(b'[') {
                            match kbhit() {
                                Some(b'A') => {
                                    if renice_mode {
                                        renice_sel = renice_sel.saturating_sub(1);
                                    } else {
                                        kill_sel = kill_sel.saturating_sub(1);
                                    }
                                }
                                Some(b'B') => {
                                    if renice_mode {
                                        renice_sel = (renice_sel + 1).min(9);
                                    } else {
                                        kill_sel = (kill_sel + 1).min(9);
                                    }
                                }
                                Some(b'C') => {
                                    if kill_mode {
                                        kill_signal = if kill_signal == 9 { 15 } else { 9 };
                                    } else if renice_mode {
                                        renice_value = (renice_value - 1).max(-20);
                                    }
                                }
                                Some(b'D') => {
                                    if kill_mode {
                                        kill_signal = if kill_signal == 9 { 15 } else { 9 };
                                    } else if renice_mode {
                                        renice_value = (renice_value + 1).min(19);
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            renice_mode = false;
                            kill_mode = false;
                        }
                    } else {
                        break;
                    }
                }
                b'p' | b'P' => paused = !paused,
                b'a' | b'A' => advanced = !advanced,
                b'h' | b'H' => help = !help,
                b'r' | b'R' => {
                    renice_mode = !renice_mode;
                    kill_mode = false;
                    renice_sel = 0;
                    renice_value = 19;
                    paused = false;
                    if renice_mode {
                        sys.refresh_all();
                        cpu = sys.global_cpu_usage();
                        let total_mem = sys.total_memory();
                        let used_mem = sys.used_memory();
                        let total_swap = sys.total_swap();
                        let used_swap = sys.used_swap();
                        cores = sys.cpus().len();
                        let loadvals: Vec<f64> = fs::read_to_string("/proc/loadavg")
                            .ok()
                            .map(|s| {
                                s.split_whitespace()
                                    .take(3)
                                    .filter_map(|w| w.parse().ok())
                                    .collect()
                            })
                            .unwrap_or_else(|| vec![0.0, 0.0, 0.0]);
                        load1 = loadvals.get(0).copied().unwrap_or(0.0);
                        load5 = loadvals.get(1).copied().unwrap_or(0.0);
                        load10 = loadvals.get(2).copied().unwrap_or(0.0);
                        mem_percent = if total_mem > 0 {
                            (used_mem as f32 / total_mem as f32) * 100.0
                        } else {
                            0.0
                        };
                        swap_percent = if total_swap > 0 {
                            (used_swap as f32 / total_swap as f32) * 100.0
                        } else {
                            0.0
                        };
                        zram = get_zram();
                        health = system_health(mem_percent, swap_percent, load1, &zram, cores);
                        label = health_label(health);
                        hcolor = health_color(health);
                        mcolor = mem_color(mem_percent);
                        procs = sys
                            .processes()
                            .iter()
                            .map(|(pid, p)| {
                                (
                                    *pid,
                                    p.name().to_string_lossy().into_owned(),
                                    p.cpu_usage(),
                                    p.memory(),
                                    p.start_time(),
                                )
                            })
                            .collect();
                        procs.sort_by(|a, b| {
                            b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)
                        });
                    }
                }
                b'k' | b'K' => {
                    kill_mode = !kill_mode;
                    renice_mode = false;
                    kill_sel = 0;
                    kill_signal = 9;
                    paused = false;
                    if kill_mode {
                        sys.refresh_all();
                        cpu = sys.global_cpu_usage();
                        let total_mem = sys.total_memory();
                        let used_mem = sys.used_memory();
                        let total_swap = sys.total_swap();
                        let used_swap = sys.used_swap();
                        cores = sys.cpus().len();
                        let loadvals: Vec<f64> = fs::read_to_string("/proc/loadavg")
                            .ok()
                            .map(|s| {
                                s.split_whitespace()
                                    .take(3)
                                    .filter_map(|w| w.parse().ok())
                                    .collect()
                            })
                            .unwrap_or_else(|| vec![0.0, 0.0, 0.0]);
                        load1 = loadvals.get(0).copied().unwrap_or(0.0);
                        load5 = loadvals.get(1).copied().unwrap_or(0.0);
                        load10 = loadvals.get(2).copied().unwrap_or(0.0);
                        mem_percent = if total_mem > 0 {
                            (used_mem as f32 / total_mem as f32) * 100.0
                        } else {
                            0.0
                        };
                        swap_percent = if total_swap > 0 {
                            (used_swap as f32 / total_swap as f32) * 100.0
                        } else {
                            0.0
                        };
                        zram = get_zram();
                        health = system_health(mem_percent, swap_percent, load1, &zram, cores);
                        label = health_label(health);
                        hcolor = health_color(health);
                        mcolor = mem_color(mem_percent);
                        procs = sys
                            .processes()
                            .iter()
                            .map(|(pid, p)| {
                                (
                                    *pid,
                                    p.name().to_string_lossy().into_owned(),
                                    p.cpu_usage(),
                                    p.memory(),
                                    p.start_time(),
                                )
                            })
                            .collect();
                        procs.sort_by(|a, b| {
                            b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)
                        });
                    }
                }
                b'\n' | b'\r' => {
                    if renice_mode && renice_sel < 10 {
                        let (pid, _, _, _, _) = &procs[renice_sel];
                        unsafe {
                            libc::setpriority(
                                libc::PRIO_PROCESS,
                                pid.as_u32() as libc::id_t,
                                renice_value,
                            );
                        }
                    } else if kill_mode && kill_sel < 10 {
                        let (pid, _, _, _, _) = &procs[kill_sel];
                        let sig = if kill_signal == 9 {
                            libc::SIGKILL
                        } else {
                            libc::SIGTERM
                        };
                        unsafe { libc::kill(pid.as_u32() as i32, sig) };
                    }
                }
                _ => {}
            }
        }

        let key_pressed = key.is_some();

        let should_refresh = if paused {
            false
        } else if renice_mode || kill_mode {
            false
        } else {
            true
        };

        if should_refresh || (key_pressed && !paused) {
            sys.refresh_all();
            cpu = sys.global_cpu_usage();
            let total_mem = sys.total_memory();
            let used_mem = sys.used_memory();
            let total_swap = sys.total_swap();
            let used_swap = sys.used_swap();
            cores = sys.cpus().len();
            let loadvals: Vec<f64> = fs::read_to_string("/proc/loadavg")
                .ok()
                .map(|s| {
                    s.split_whitespace()
                        .take(3)
                        .filter_map(|w| w.parse().ok())
                        .collect()
                })
                .unwrap_or_else(|| vec![0.0, 0.0, 0.0]);
            load1 = loadvals.get(0).copied().unwrap_or(0.0);
            load5 = loadvals.get(1).copied().unwrap_or(0.0);
            load10 = loadvals.get(2).copied().unwrap_or(0.0);

            mem_percent = if total_mem > 0 {
                (used_mem as f32 / total_mem as f32) * 100.0
            } else {
                0.0
            };
            swap_percent = if total_swap > 0 {
                (used_swap as f32 / total_swap as f32) * 100.0
            } else {
                0.0
            };

            zram = get_zram();

            health = system_health(mem_percent, swap_percent, load1, &zram, cores);
            label = health_label(health);
            hcolor = health_color(health);
            mcolor = mem_color(mem_percent);

            procs = sys
                .processes()
                .iter()
                .map(|(pid, p)| {
                    (
                        *pid,
                        p.name().to_string_lossy().into_owned(),
                        p.cpu_usage(),
                        p.memory(),
                        p.start_time(),
                    )
                })
                .collect();
            procs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        }

        if renice_mode && !key_pressed {
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        if kill_mode && !key_pressed {
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        if paused && !key_pressed {
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        let pause_marker = if paused {
            format!(" {YELLOW}[PAUSED]{RESET}")
        } else {
            String::new()
        };

        if help {
            println!("\x1b[2J\x1b[H");
            println!("{BOLD}{BLUE}HELP{RESET}");
            println!("\n{BOLD}{WHITE}ZRAM RATIO:{RESET}");
            println!("  orig    = original data size before compression");
            println!("  compr   = compressed size in zram");
            println!("  ratio   = orig / compr (higher = better compression)");
            println!("  saved   = orig - compr (actual RAM saved)");
            println!("  {CYAN}Example:{RESET} ratio 4.0x means 1000MB compresses to 250MB");
            println!("\n{BOLD}{WHITE}HEALTH SCORE:{RESET}");
            let excellent = format!("{}EXCELLENT{}", GREEN, RESET);
            let good = format!("{}GOOD{}", CYAN, RESET);
            let ok = format!("{}OK{}", YELLOW, RESET);
            let stressed = format!("{}STRESSED{}", RED, RESET);
            let data = [
                ("85+", excellent.as_str(), "all normal"),
                ("70-84", good.as_str(), "slight load"),
                ("50-69", ok.as_str(), "elevated load"),
                ("0-49", stressed.as_str(), "high load"),
            ];
            let mut table = Table::new(&data);
            table.with(Style::empty());
            println!("{}", table);
            println!("\n{BOLD}{WHITE}KEYS:{RESET}");
            println!("  {WHITE}q/ESC = quit{RESET}");
            println!("  {WHITE}p      = pause display{RESET}");
            println!("  {WHITE}a      = advanced mode (shows health score, full zram){RESET}");
            println!("  {WHITE}h      = toggle this help{RESET}");
            println!("  {WHITE}r      = renice mode (select process, set nice -20 to 19){RESET}");
            println!("  {WHITE}k      = kill mode (select process, send signal 9 or 15){RESET}");
            println!("\n{BOLD}{CYAN}interval={}{}s{RESET}", CYAN, args.interval);
            let help_marker = format!(" {CYAN}[HELP]{RESET}");
            let advanced_marker = if advanced {
                format!(" {CYAN}[ADVANCED]{RESET}")
            } else {
                String::new()
            };
            println!("\n{BOLD}{WHITE}q=quit{RESET} | {BOLD}{CYAN}p=pause{RESET} | {BOLD}{CYAN}a=advanced{RESET} | {BOLD}{CYAN}h=help{RESET} | {BOLD}{CYAN}r=renice{RESET} | {BOLD}{CYAN}k=kill{RESET} | {BOLD}{CYAN}interval={}{}s{RESET}{}{}{}", CYAN, args.interval, advanced_marker, help_marker, pause_marker);
            if !key_pressed {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            continue;
        }

        println!("\x1b[2J\x1b[H");
        println!(
            "{BOLD}{CYAN}CPU:{RESET}   {}{:.0}%{RESET}",
            if cpu > 80.0 { RED } else { WHITE },
            cpu
        );
        println!(
            "{BOLD}{CYAN}RAM:{RESET}   {}{:.0}%{RESET}",
            mcolor, mem_percent
        );
        println!(
            "{BOLD}{CYAN}SWAP:{RESET}  {}{:.0}%{RESET}",
            if swap_percent > 50.0 { RED } else { WHITE },
            swap_percent
        );
        println!(
            "{BOLD}{CYAN}AVG. LOAD:{RESET}  {}{:.2}{RESET}",
            if load1 > cores as f64 * 1.5 {
                RED
            } else if load1 > cores as f64 {
                YELLOW
            } else {
                WHITE
            },
            load1
        );
        if advanced {
            let load5_color = if load5 > cores as f64 * 1.5 {
                RED
            } else if load5 > cores as f64 {
                YELLOW
            } else {
                CYAN
            };
            let load10_color = if load10 > cores as f64 * 1.5 {
                RED
            } else if load10 > cores as f64 {
                YELLOW
            } else {
                CYAN
            };
            println!(
                "{BOLD}{CYAN}     5m:{RESET}  {}{:.2}{RESET}  {BOLD}{CYAN}10m:{RESET} {}{:.2}{RESET}",
                load5_color, load5, load10_color, load10
            );
        }
        if advanced {
            print!(
                "{BOLD}{0}HEALTH:{RESET} {1}{2}/100 [{3}{4}{RESET}]\n",
                hcolor, hcolor, health, hcolor, label
            );
        } else {
            print!(
                "{BOLD}{0}HEALTH:{RESET} [{1}{2}{RESET}]\n",
                hcolor, hcolor, label
            );
        }
        if let Some(z) = &zram {
            let ratio = if z.compr > 0 {
                z.orig as f64 / z.compr as f64
            } else {
                0.0
            };
            let rcolor = if ratio > 2.0 { CYAN } else { GREEN };
            if advanced {
                let saved = z.orig.saturating_sub(z.compr);
                println!("{BOLD}{0}ZRAM:{RESET} {1}{2:.1}MB compr={3:.1}MB mem={4:.1}MB ratio={5}{6:.2}x saved={7:.1}MB", MAGENTA, MAGENTA, z.orig as f64 / 1024.0 / 1024.0, z.compr as f64 / 1024.0 / 1024.0, z.mem as f64 / 1024.0 / 1024.0, rcolor, ratio, saved as f64 / 1024.0 / 1024.0);
            } else {
                println!(
                    "{BOLD}{0}ZRAM:{RESET} ratio={1}{2:.2}x{RESET}",
                    MAGENTA, rcolor, ratio
                );
            }
        }
        println!("\n{BOLD}{WHITE}  PID  CPU(%)  MEM(MB)  TIME   COMMAND{RESET}");
        let mut rows = Vec::new();
        for (i, (tid, name, cpu, mem, time)) in procs.iter().take(10).enumerate() {
            let selected = (renice_mode && i == renice_sel) || (kill_mode && i == kill_sel);
            let marker = if selected { ">" } else { " " };
            let tid_str = format!("{:>5}", tid);
            let cpu_str = format!("{:>6}", cpu.round() as u64);
            let mem_str = format!("{:>7}", (*mem as f64 / 1024.0 / 1024.0).round() as u64);
            let time_str = format!("{:>5}", time / 60);
            let display_name: String = if name.len() > 25 {
                name.chars().take(25).collect()
            } else {
                name.clone()
            };
            if selected {
                rows.push((
                    marker,
                    tid_str,
                    cpu_str,
                    mem_str,
                    time_str,
                    format!("{BOLD}{}{RESET}", display_name),
                ));
            } else {
                rows.push((marker, tid_str, cpu_str, mem_str, time_str, display_name));
            }
        }
        let mut table = Table::new(&rows);
        table.with(Style::empty());
        println!("{}", table);
        if renice_mode {
            println!("\n{BOLD}{YELLOW}RENICE MODE:{RESET} nice={}{}{}  {CYAN}up/down=select  left/right=nice value  enter=apply{RESET}", YELLOW, renice_value, RESET);
        }
        if kill_mode {
            let sig_display = if kill_signal == 9 {
                format!("{RED}9=kill{RESET}")
            } else {
                format!("{GREEN}15=term{RESET}")
            };
            println!("\n{BOLD}{RED}KILL MODE:{RESET} signal={}  {CYAN}up/down=select  left/right=toggle signal  enter=send{RESET}", sig_display);
        }
        let help_marker = if help {
            format!(" {CYAN}[HELP]{RESET}")
        } else {
            String::new()
        };
        let advanced_marker = if advanced {
            format!(" {CYAN}[ADVANCED]{RESET}")
        } else {
            String::new()
        };
        let renice_marker = if renice_mode {
            format!(" {YELLOW}[RENICE]{RESET}")
        } else {
            String::new()
        };
        let kill_marker = if kill_mode {
            format!(" {RED}[KILL]{RESET}")
        } else {
            String::new()
        };
        println!("\n{BOLD}{WHITE}q=quit{RESET} | {BOLD}{CYAN}p=pause{RESET} | {BOLD}{CYAN}a=advanced{RESET} | {BOLD}{CYAN}h=help{RESET} | {BOLD}{CYAN}r=renice{RESET} | {BOLD}{CYAN}k=kill{RESET} | {BOLD}{CYAN}interval={}{}s{RESET}{}{}{}{}{}", CYAN, args.interval, advanced_marker, help_marker, renice_marker, kill_marker, pause_marker);

        if paused || renice_mode || kill_mode {
            if !key.is_some() {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        } else if !key.is_some() {
            std::thread::sleep(std::time::Duration::from_secs_f64(args.interval));
        }
    }

    disable_raw_mode(&_original_termios);
}
