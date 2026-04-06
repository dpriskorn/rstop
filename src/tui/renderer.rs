use crate::process_list::ProcessInfo;
use crate::tui::app::AppState;
use crate::tui::app::Mode;
use crate::tui::footer::render_footer;
use crate::tui::header::build_header_text;
use crate::zram_stats::ZramReader;
use crate::zram_stats::ZramStats;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::{cursor, terminal};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

extern crate libc;
use libc::isatty;

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
        health_factors: &crate::health::HealthFactors,
        zram_stats: Option<&ZramStats>,
        zram_reader: &ZramReader,
        processes: &[ProcessInfo],
        state: &AppState,
    ) {
        let header_text = build_header_text(
            cpu,
            zram_swap_percent,
            disk_swap_percent,
            load1,
            load5,
            load10,
            cores,
            health,
            health_label,
            health_factors,
            zram_stats,
            zram_reader,
        );

        let _ = self.terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(10),
                    Constraint::Min(6),
                    Constraint::Length(6),
                ])
                .split(area);

            // Header
            let header = Paragraph::new(header_text)
                .block(Block::bordered().title(" System Overview "))
                .style(Style::default().fg(Color::White));
            f.render_widget(header, chunks[0]);

            // Process table or Help
            if state.mode == Mode::Help {
                render_help_mode(f, chunks[1]);
            } else {
                render_process_table(f, chunks[1], processes, state);
            }

            // Footer
            render_footer(f, state, chunks[2]);
        });
    }
}

fn render_process_table(f: &mut Frame, area: Rect, processes: &[ProcessInfo], state: &AppState) {
    use crate::tui::app::Mode;

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

            let is_selected = (state.mode == Mode::Renice && state.selection == i)
                || (state.mode == Mode::Kill && state.selection == i);

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

    f.render_widget(table, area);
}

fn render_help_mode(f: &mut Frame, area: Rect) {
    use ratatui::widgets::Borders;

    let help_text = r#"
════════════════════════════════════════════════════════════════
                    RSTOP - HELP & HOTKEYS
════════════════════════════════════════════════════════════════

ZRAM RATIO:
  orig    = original data size before compression
  compr   = compressed size in zram
  ratio   = orig / compr (higher = better compression)
  saved   = orig - compr (actual RAM saved)
  Example: ratio 4.0x means 1000MB compresses to 250MB

HEALTH SCORE:
  Based on: SWAP%, LOAD, ZRAM ratio
  85+  = EXCELLENT   70-84 = GOOD   50-69 = OK   0-49 = STRESSED

HOTKEYS:
  [q]  Quit           Exit the application
  [p]  Pause          Toggle pause/resume display
  [h]  Help           Toggle this help screen
  [m]  Memory Sort    Sort by memory usage
  [r]  Renice Mode   Change process priority (nice value)
  [k]  Kill Mode     Send signal to process
  [↑]  Navigate Up   Select previous process
  [↓]  Navigate Down Select next process
  [Enter] Execute    Apply renice or kill to selected
  [Esc]  Cancel      Exit current mode
════════════════════════════════════════════════════════════════
"#;

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White))
        .block(Block::bordered().title(" Help ").borders(Borders::ALL));

    f.render_widget(help, area);
}

impl Drop for TuiRenderer {
    fn drop(&mut self) {
        let _ = execute!(
            self.terminal.backend_mut(),
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        );
        let _ = terminal::disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), cursor::Show);
    }
}

impl Default for TuiRenderer {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
