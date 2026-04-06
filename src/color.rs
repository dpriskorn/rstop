pub struct Colors;

impl Colors {
    pub const RED: &'static str = "\x1b[91m";
    pub const GREEN: &'static str = "\x1b[92m";
    pub const YELLOW: &'static str = "\x1b[93m";
    pub const BLUE: &'static str = "\x1b[94m";
    #[allow(dead_code)]
    pub const MAGENTA: &'static str = "\x1b[95m";
    pub const CYAN: &'static str = "\x1b[96m";
    pub const WHITE: &'static str = "\x1b[97m";
    pub const BOLD: &'static str = "\x1b[1m";
    pub const RESET: &'static str = "\x1b[0m";
}

pub struct ColorScheme {
    pub health_excellent: i32,
    pub health_good: i32,
    pub health_ok: i32,
    pub zram_excellent: f64,
    pub zram_good: f64,
    pub load_high: f64,
    pub load_medium: f64,
    pub cpu_excellent: f32,
    pub cpu_good: f32,
    pub swap_excellent: f32,
    pub swap_good: f32,
}

impl ColorScheme {
    pub fn global() -> &'static ColorScheme {
        static INSTANCE: ColorScheme = ColorScheme {
            health_excellent: 85,
            health_good: 70,
            health_ok: 50,
            zram_excellent: 3.0,
            zram_good: 2.0,
            load_high: 1.5,
            load_medium: 1.0,
            cpu_excellent: 50.0,
            cpu_good: 80.0,
            swap_excellent: 20.0,
            swap_good: 50.0,
        };
        &INSTANCE
    }

    pub fn color_for_cpu(&self, value: f32) -> &'static str {
        if value <= self.cpu_excellent {
            Colors::GREEN
        } else if value <= self.cpu_good {
            Colors::YELLOW
        } else {
            Colors::RED
        }
    }

    pub fn color_for_swap(&self, value: f32) -> &'static str {
        if value <= self.swap_excellent {
            Colors::GREEN
        } else if value <= self.swap_good {
            Colors::YELLOW
        } else {
            Colors::RED
        }
    }

    pub fn color_for_disk_swap(&self, value: f32) -> &'static str {
        if value == 0.0 {
            Colors::GREEN
        } else {
            Colors::RED
        }
    }

    pub fn color_for_health(&self, score: i32) -> &'static str {
        if score >= self.health_excellent {
            Colors::GREEN
        } else if score >= self.health_good {
            Colors::CYAN
        } else if score >= self.health_ok {
            Colors::YELLOW
        } else {
            Colors::RED
        }
    }

    pub fn color_for_zram(&self, ratio: f64) -> &'static str {
        if ratio >= self.zram_excellent {
            Colors::GREEN
        } else if ratio >= self.zram_good {
            Colors::YELLOW
        } else {
            Colors::RED
        }
    }

    pub fn color_for_load(&self, load: f64, cores: usize) -> &'static str {
        let high = cores as f64 * self.load_high;
        let medium = cores as f64 * self.load_medium;
        if load > high {
            Colors::RED
        } else if load > medium {
            Colors::YELLOW
        } else {
            Colors::GREEN
        }
    }

    #[allow(dead_code)]
    pub fn color_for_percent(&self, value: f32, threshold: f32) -> &'static str {
        if value > threshold {
            Colors::RED
        } else {
            Colors::WHITE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_for_load_green() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_load(1.0, 4), Colors::GREEN);
    }

    #[test]
    fn test_color_for_load_yellow() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_load(5.0, 4), Colors::YELLOW);
    }

    #[test]
    fn test_color_for_load_red() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_load(10.0, 4), Colors::RED);
    }

    #[test]
    fn test_color_for_percent_white() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_percent(50.0, 80.0), Colors::WHITE);
    }

    #[test]
    fn test_color_for_percent_red() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_percent(90.0, 80.0), Colors::RED);
    }

    #[test]
    fn test_color_for_health_excellent() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_health(90), Colors::GREEN);
    }

    #[test]
    fn test_color_for_health_good() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_health(75), Colors::CYAN);
    }

    #[test]
    fn test_color_for_health_ok() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_health(55), Colors::YELLOW);
    }

    #[test]
    fn test_color_for_health_stressed() {
        let colors = ColorScheme::global();
        assert_eq!(colors.color_for_health(30), Colors::RED);
    }
}
