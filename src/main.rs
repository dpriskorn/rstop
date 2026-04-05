use clap::Parser;
use std::fs;
use sysinfo::System;

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

    loop {
        if let Some(key) = kbhit() {
            match key {
                b'q' | b'Q' | 0x1b => break,
                b'p' | b'P' => paused = !paused,
                b'a' | b'A' => advanced = !advanced,
                b'h' | b'H' => help = !help,
                _ => {}
            }
        }

        let pause_marker = if paused {
            format!(" {YELLOW}[PAUSED]{RESET}")
        } else {
            String::new()
        };

        if paused {
            println!("\x1b[2J\x1b[H");
            println!("{BOLD}{BLUE}HTOP_ZRAM{RESET}");
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
            println!("\n{BOLD}{WHITE}q=quit{RESET} | {BOLD}{CYAN}p=pause{RESET} | {BOLD}{CYAN}a=advanced{RESET} | {BOLD}{CYAN}h=help{RESET} | {BOLD}{CYAN}interval={}{}s{RESET}{}{}{}", CYAN, args.interval, advanced_marker, help_marker, pause_marker);
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        if help {
            println!("\x1b[2J\x1b[H");
            println!("{BOLD}{BLUE}HTOP_ZRAM - HELP{RESET}");
            println!("\n{BOLD}{WHITE}ZRAM RATIO:{RESET}");
            println!("  orig    = original data size before compression");
            println!("  compr   = compressed size in zram");
            println!("  ratio   = orig / compr (higher = better compression)");
            println!("  saved   = orig - compr (actual RAM saved)");
            println!("  {CYAN}Example:{RESET} ratio 4.0x means 1000MB compresses to 250MB");
            println!("\n{BOLD}{WHITE}HEALTH SCORE:{RESET}");
            println!("  Based on: RAM%, SWAP%, LOAD, ZRAM ratio");
            println!("  {GREEN}85+  = EXCELLENT{RESET} - all normal");
            println!("  {CYAN}70-84 = GOOD{RESET} - slight load");
            println!("  {YELLOW}50-69 = OK{RESET} - elevated load");
            println!("  {RED}0-49  = STRESSED{RESET} - high load");
            println!("\n{BOLD}{WHITE}KEYS:{RESET}");
            println!("  {WHITE}q/ESC = quit{RESET}");
            println!("  {WHITE}p      = pause display{RESET}");
            println!("  {WHITE}a      = advanced mode (shows health score, full zram){RESET}");
            println!("  {WHITE}h      = toggle this help{RESET}");
            println!("\n{BOLD}{CYAN}interval={}{}s{RESET}", CYAN, args.interval);
            let help_marker = format!(" {CYAN}[HELP]{RESET}");
            let advanced_marker = if advanced {
                format!(" {CYAN}[ADVANCED]{RESET}")
            } else {
                String::new()
            };
            println!("\n{BOLD}{WHITE}q=quit{RESET} | {BOLD}{CYAN}p=pause{RESET} | {BOLD}{CYAN}a=advanced{RESET} | {BOLD}{CYAN}h=help{RESET} | {BOLD}{CYAN}interval={}{}s{RESET}{}{}{}", CYAN, args.interval, advanced_marker, help_marker, pause_marker);
            std::thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        sys.refresh_all();

        let cpu = sys.global_cpu_usage();
        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let total_swap = sys.total_swap();
        let used_swap = sys.used_swap();
        let cores = sys.cpus().len();
        let load1: f64 = fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|s| s.split_whitespace().next().map(|w| w.to_string()))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let mem_percent = if total_mem > 0 {
            (used_mem as f32 / total_mem as f32) * 100.0
        } else {
            0.0
        };
        let swap_percent = if total_swap > 0 {
            (used_swap as f32 / total_swap as f32) * 100.0
        } else {
            0.0
        };

        let zram = get_zram();

        let health = system_health(mem_percent, swap_percent, load1, &zram, cores);
        let label = health_label(health);
        let hcolor = health_color(health);
        let mcolor = mem_color(mem_percent);

        println!("\x1b[2J\x1b[H");
        println!("{BOLD}{BLUE}HTOP_ZRAM{RESET}");
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
            "{BOLD}{CYAN}LOAD:{RESET}  {}{:.2}{RESET}",
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
                println!("{BOLD}{0}ZRAM:{RESET} {1}{2:.1}MB compr={3:.1}MB mem={4:.1}MB ratio={5}{6:.2}x saved={7:.1}MB",
                    MAGENTA, MAGENTA, z.orig as f64 / 1024.0 / 1024.0, z.compr as f64 / 1024.0 / 1024.0, z.mem as f64 / 1024.0 / 1024.0, rcolor, ratio, saved as f64 / 1024.0 / 1024.0);
            } else {
                println!(
                    "{BOLD}{0}ZRAM:{RESET} ratio={1}{2:.2}x{RESET}",
                    MAGENTA, rcolor, ratio
                );
            }
        }

        println!("\n{BOLD}{WHITE}Top processes:{RESET}");

        let mut procs: Vec<_> = sys
            .processes()
            .iter()
            .map(|(pid, p)| {
                (
                    *pid,
                    p.name().to_string_lossy().into_owned(),
                    p.cpu_usage(),
                    p.memory(),
                )
            })
            .collect();

        procs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        println!(
            "{:<6}  {:>6}  {:>8}  {:>30}",
            "PID", "CPU%", "MEM(GB)", "NAME"
        );
        for (pid, name, cpu, mem) in procs.into_iter().take(10) {
            let display_name: String = if name.len() > 30 {
                name.chars().take(30).collect()
            } else {
                name
            };
            println!(
                "{:<6}  {:>6}  {:>8}  {:>30}",
                pid,
                cpu.round() as u64,
                (mem as f64 / 1024.0 / 1024.0).round() as u64,
                display_name
            );
        }

        let advanced_marker = if advanced {
            format!(" {CYAN}[ADVANCED]{RESET}")
        } else {
            String::new()
        };
        let help_marker = if help {
            format!(" {CYAN}[HELP]{RESET}")
        } else {
            String::new()
        };
        println!("\n{BOLD}{WHITE}q=quit{RESET} | {BOLD}{CYAN}p=pause{RESET} | {BOLD}{CYAN}a=advanced{RESET} | {BOLD}{CYAN}h=help{RESET} | {BOLD}{CYAN}interval={}{}s{RESET}{}{}{}", CYAN, args.interval, advanced_marker, help_marker, pause_marker);

        std::thread::sleep(std::time::Duration::from_secs_f64(args.interval));
    }

    disable_raw_mode(&_original_termios);
    println!("\n{BOLD}{GREEN}Bye!{RESET}\n");
}
