use crate::tui::app::AppState;
use crate::tui::app::Mode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph},
    Frame,
};

pub fn render_footer(f: &mut Frame, state: &AppState, footer_area: Rect) {
    // Split into info area (top) and bottom row area
    let footer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(footer_area);

    // Info box (full width)
    let info_color = if state.info.is_empty() {
        Color::DarkGray
    } else {
        Color::White
    };
    let info = Paragraph::new(state.info.as_str())
        .style(Style::default().fg(info_color))
        .block(Block::bordered().title(" Info "));
    f.render_widget(info, footer_chunks[0]);

    // Bottom row: Commands, Sort, and Mode
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(30),
            Constraint::Length(18),
            Constraint::Length(20),
        ])
        .split(footer_chunks[1]);

    // Mode box (right side)
    let mode_text = match state.mode {
        Mode::Normal => " Normal ",
        Mode::Pause => " Paused ",
        Mode::Renice => " Renice ",
        Mode::Kill => " Kill ",
        Mode::Help => " Help ",
    };
    let mode_color = match state.mode {
        Mode::Normal => Color::Green,
        Mode::Pause => Color::Red,
        Mode::Renice => Color::Yellow,
        Mode::Kill => Color::Red,
        Mode::Help => Color::Cyan,
    };
    let mode = Paragraph::new(mode_text)
        .style(Style::default().fg(mode_color).add_modifier(Modifier::BOLD))
        .block(Block::bordered().title(" Mode "));
    f.render_widget(mode, bottom_chunks[2]);

    // Sort box (center)
    let sort_text = if state.sort_by_mem { " MEM " } else { " CPU " };
    let sort_color = if state.sort_by_mem {
        Color::Yellow
    } else {
        Color::Cyan
    };
    let sort = Paragraph::new(sort_text)
        .style(Style::default().fg(sort_color).add_modifier(Modifier::BOLD))
        .block(Block::bordered().title(" Sort "));
    f.render_widget(sort, bottom_chunks[1]);

    // Commands box (left side)
    let footer = Paragraph::new("[q][p][h][m][r][k]")
        .style(Style::default().fg(Color::White))
        .block(Block::bordered().title(" Commands "));
    f.render_widget(footer, bottom_chunks[0]);
}
