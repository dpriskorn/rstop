use crate::color::ColorScheme;
use crate::health::HealthFactors;
use crate::zram_stats::{ZramReader, ZramStats};
use ratatui::{
    style::Color,
    style::Style,
    text::{Line, Span, Text},
};

pub fn build_header_text(
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
) -> Text<'static> {
    let colors = ColorScheme::global();

    let zram_ratio = zram_stats
        .as_ref()
        .map(|z| zram_reader.compression_ratio(z))
        .unwrap_or(0.0);

    Text::from(vec![
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
                        format!("+{}", health_factors.zram_penalty)
                    } else {
                        health_factors.zram_penalty.to_string()
                    }
                ),
                Style::default().fg(if health_factors.zram_penalty >= 0 {
                    Color::Green
                } else {
                    Color::Red
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
    ])
}
