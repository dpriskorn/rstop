use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::{cursor, terminal};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Color,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Row, Table},
    Terminal,
};
use std::io;
use std::time::Duration;

extern crate libc;
use libc::isatty;

use crate::color::ColorScheme;
use crate::health::HealthFactors;
use crate::process_list::ProcessInfo;
use crate::zram_stats::{ZramReader, ZramStats};

pub struct AppState {
    pub sort_by_mem: bool,
    pub renice_active: bool,
    pub renice_selection: usize,
    pub renice_nice_value: i32,
    pub kill_active: bool,
    pub kill_selection: usize,
    pub kill_signal: i32,
    pub help_active: bool,
    pub pause_active: bool,
    pub advanced: bool,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            sort_by_mem: false,
            renice_active: false,
            renice_selection: 0,
            renice_nice_value: 19,
            kill_active: false,
            kill_selection: 0,
            kill_signal: 15,
            help_active: false,
            pause_active: false,
            advanced: false,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

fn is_terminal() -> bool {
    unsafe { isatty(libc::STDIN_FILENO) != 0 }
}

pub struct TuiRenderer {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TuiRenderer {
    pub fn new() -> io::Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;

        execute!(
            terminal.backend_mut(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Hide
        )?;

        let _ = terminal::enable_raw_mode();

        Ok(TuiRenderer { terminal })
    }

    pub fn poll_event(&mut self) -> Option<u8> {
        if !is_terminal() {
            return None;
        }

        loop {
            match event::poll(Duration::from_millis(50)) {
                Ok(true) => {
                    if let Ok(Event::Key(key)) = event::read() {
                        if key.kind == KeyEventKind::Press {
                            return Some(match key.code {
                                KeyCode::Char(c) => c as u8,
                                KeyCode::Enter => b'\n',
                                KeyCode::Esc => 0x1b,
                                KeyCode::Up => 0xF0,
                                KeyCode::Down => 0xF1,
                                KeyCode::Right => 0xF2,
                                KeyCode::Left => 0xF3,
                                _ => return None,
                            });
                        }
                    }
                }
                Ok(false) => return None,
                Err(_) => return None,
            }
        }
    }

    pub fn draw(
        &mut self,
        cpu: f32,
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
        processes: &[ProcessInfo],
        state: &AppState,
    ) {
        let colors = ColorScheme::global();

        let zram_ratio = zram_stats
            .as_ref()
            .map(|z| zram_reader.compression_ratio(z))
            .unwrap_or(0.0);

        let header_text = Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::raw(" HEALTH      "),
                Span::styled(
                    format!("{}/100 [{}]", health, health_label),
                    Style::default().fg(colors.color_for_health(health)),
                ),
            ]),
            Line::from(vec![
                Span::raw(" PENALTIES   "),
                Span::styled(
                    format!(
                        "disk_swap={}",
                        if health_factors.swap_penalty > 0 {
                            -health_factors.swap_penalty
                        } else {
                            health_factors.swap_penalty
                        }
                    ),
                    Style::default().fg(if health_factors.swap_penalty > 0 {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                ),
                Span::raw(" "),
                Span::styled(
                    format!(
                        "load={}",
                        if health_factors.load_penalty > 0 {
                            -health_factors.load_penalty
                        } else {
                            health_factors.load_penalty
                        }
                    ),
                    Style::default().fg(if health_factors.load_penalty > 0 {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                ),
                Span::raw(" "),
                Span::styled(
                    format!(
                        "zram_ratio={}",
                        if health_factors.zram_penalty > 0 {
                            -health_factors.zram_penalty
                        } else {
                            health_factors.zram_penalty
                        }
                    ),
                    Style::default().fg(if health_factors.zram_penalty > 0 {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                ),
            ]),
            Line::from(vec![
                Span::raw(" CPU         "),
                Span::styled(
                    format!("{:.0}%", cpu),
                    Style::default().fg(colors.color_for_cpu(cpu)),
                ),
            ]),
            Line::from(vec![
                Span::raw(" ZRAM        "),
                Span::styled(
                    format!("{:.1}x", zram_ratio),
                    Style::default().fg(colors.color_for_zram(zram_ratio)),
                ),
            ]),
            Line::from(vec![
                Span::raw(" SWAP(ZRAM)  "),
                Span::styled(
                    format!("{:.0}%", zram_swap_percent),
                    Style::default().fg(if zram_swap_percent <= 50.0 {
                        Color::Green
                    } else if zram_swap_percent <= 80.0 {
                        Color::Yellow
                    } else {
                        Color::Red
                    }),
                ),
            ]),
            Line::from(vec![
                Span::raw(" SWAP(SSD)   "),
                Span::styled(
                    format!("{:.0}%", disk_swap_percent),
                    Style::default().fg(if disk_swap_percent <= 1.0 {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
            ]),
            Line::from(vec![
                Span::raw(" LOAD        "),
                Span::styled(
                    format!("{:.2}", load1),
                    Style::default().fg(colors.color_for_load(load1, cores)),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:.2}", load5),
                    Style::default().fg(colors.color_for_load(load5, cores)),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:.2}", load10),
                    Style::default().fg(colors.color_for_load(load10, cores)),
                ),
            ]),
        ]);

        let _ = self.terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(10),
                    Constraint::Min(6),
                    Constraint::Length(3),
                ])
                .split(area);

            let header = Paragraph::new(header_text)
                .block(Block::bordered().title(" System Overview "))
                .style(Style::default().fg(Color::White));

            f.render_widget(header, chunks[0]);

            let rows: Vec<Row> = processes
                .iter()
                .take(10)
                .enumerate()
                .map(|(i, p)| {
                    let mem_mb = (p.mem as f64 / 1024.0 / 1024.0).round() as u64;
                    let time_min = p.time / 60;
                    let name = if p.name.len() > 20 {
                        p.name.chars().take(20).collect()
                    } else {
                        p.name.clone()
                    };

                    let is_selected = (state.renice_active && state.renice_selection == i)
                        || (state.kill_active && state.kill_selection == i);

                    let m = if is_selected { ">" } else { " " };
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    Row::new(vec![
                        m.to_string(),
                        p.pid.as_u32().to_string(),
                        p.user.clone(),
                        p.nice.to_string(),
                        format!("{:.0}", p.cpu),
                        mem_mb.to_string(),
                        time_min.to_string(),
                        name,
                    ])
                    .style(style)
                })
                .collect();

            let widths = [
                Constraint::Length(2),
                Constraint::Length(8),
                Constraint::Length(8),
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(6),
                Constraint::Min(10),
            ];

            let header_style = Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD);

            let table = Table::new(rows, widths)
                .header(
                    Row::new(vec!["M", "PID", "USER", "NI", "CPU", "MEM", "TIME", "NAME"])
                        .style(header_style)
                        .height(1),
                )
                .block(Block::bordered().title(" Processes "))
                .widths(widths);

            f.render_widget(table, chunks[1]);

            let mut text = String::from("q=quit | p=pause | a=advanced | h=help | m=mem");

            if !state.help_active && !state.renice_active && !state.kill_active {
                text.push_str(" | r=renice | k=kill");
            }

            text.push_str(" | interval=2.0s");

            if state.renice_active {
                text.push_str(&format!(" | RENICE MODE: nice={}", state.renice_nice_value));
            }
            if state.kill_active {
                text.push_str(&format!(" | KILL MODE: signal={}", state.kill_signal));
            }

            let footer = Paragraph::new(text).style(Style::default().fg(Color::White));

            f.render_widget(footer, chunks[2]);
        });
    }
}

impl Default for TuiRenderer {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
