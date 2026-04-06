use ratatui::style::Color;

pub struct ColorScheme {
    pub health_excellent: i32,
    pub health_good: i32,
    pub health_ok: i32,
    pub zram_excellent: f64,
    pub zram_good: f64,
    pub cpu_excellent: f32,
    pub cpu_good: f32,
}

impl ColorScheme {
    pub fn global() -> &'static ColorScheme {
        static INSTANCE: ColorScheme = ColorScheme {
            health_excellent: 85,
            health_good: 70,
            health_ok: 50,
            zram_excellent: 2.0,
            zram_good: 1.5,
            cpu_excellent: 50.0,
            cpu_good: 80.0,
        };
        &INSTANCE
    }

    pub fn color_for_cpu(&self, value: f32) -> Color {
        if value <= self.cpu_excellent {
            Color::Green
        } else if value <= self.cpu_good {
            Color::Yellow
        } else {
            Color::Red
        }
    }

    pub fn color_for_health(&self, score: i32) -> Color {
        if score >= self.health_excellent {
            Color::Green
        } else if score >= self.health_good {
            Color::Cyan
        } else if score >= self.health_ok {
            Color::Yellow
        } else {
            Color::Red
        }
    }

    pub fn color_for_zram(&self, ratio: f64) -> Color {
        if ratio >= self.zram_excellent {
            Color::Green
        } else if ratio >= self.zram_good {
            Color::Yellow
        } else {
            Color::Red
        }
    }

    pub fn color_for_load(&self, load: f64, cores: usize) -> Color {
        let high = cores as f64 * 1.5;
        let medium = cores as f64 * 1.0;
        if load > high {
            Color::Red
        } else if load > medium {
            Color::Yellow
        } else {
            Color::Green
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_for_health_excellent() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_health(90), Color::Green);
    }

    #[test]
    fn test_color_for_health_good() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_health(75), Color::Cyan);
    }

    #[test]
    fn test_color_for_health_ok() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_health(55), Color::Yellow);
    }

    #[test]
    fn test_color_for_health_stressed() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_health(30), Color::Red);
    }
}
