pub struct HealthFactors {
    pub mem_penalty: i32,
    pub swap_penalty: i32,
    pub load_penalty: i32,
    pub zram_penalty: i32,
}

impl HealthFactors {
    pub fn calculate(
        disk_swap_percent: f32,
        load1: f64,
        zram_ratio: f64,
        cores: usize,
    ) -> HealthFactors {
        let mut mem_penalty = 0;
        let mut swap_penalty = 0;
        let mut load_penalty = 0;
        let mut zram_penalty = 0;

        if disk_swap_percent > 1.0 {
            swap_penalty = 50;
        }

        let cores = cores as f64;
        if load1 > cores * 1.5 {
            load_penalty = 25;
        } else if load1 > cores {
            load_penalty = 10;
        }

        if zram_ratio > 0.0 && zram_ratio < 1.5 {
            zram_penalty = -10;
        } else if zram_ratio >= 1.5 && zram_ratio < 2.0 {
            zram_penalty = -5;
        } else if zram_ratio > 3.0 {
            zram_penalty = 5;
        }

        HealthFactors {
            mem_penalty,
            swap_penalty,
            load_penalty,
            zram_penalty,
        }
    }
}

pub struct HealthCalculator;

impl HealthCalculator {
    pub fn calculate(
        disk_swap_percent: f32,
        load1: f64,
        zram_ratio: f64,
        cores: usize,
    ) -> (i32, &'static str) {
        let mut score = 100;

        if disk_swap_percent > 1.0 {
            score -= 50;
        }

        let cores = cores as f64;
        if load1 > cores * 1.5 {
            score -= 25;
        } else if load1 > cores {
            score -= 10;
        }

        if zram_ratio > 0.0 {
            if zram_ratio < 1.5 {
                score -= 10;
            } else if zram_ratio < 2.0 {
                score -= 5;
            } else if zram_ratio > 3.0 {
                score += 5;
            }
        }

        let score = score.clamp(0, 100);
        let label = if score >= 85 {
            "EXCELLENT"
        } else if score >= 70 {
            "GOOD"
        } else if score >= 50 {
            "OK"
        } else {
            "STRESSED"
        };

        (score, label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_calculator_excellent() {
        let (score, label) = HealthCalculator::calculate(0.0, 1.0, 3.5, 4);
        assert!(score >= 85, "Score: {}", score);
        assert_eq!(label, "EXCELLENT");
    }

    #[test]
    fn test_health_calculator_stressed() {
        let (score, label) = HealthCalculator::calculate(10.0, 10.0, 1.2, 2);
        assert!(score < 50);
    }

    #[test]
    fn test_health_calculator_good() {
        let (score, label) = HealthCalculator::calculate(0.0, 5.0, 1.4, 4);
        assert!(score >= 70, "Score should be >= 70, got {}", score);
        assert!(score <= 84, "Score should be <= 84, got {}", score);
    }

    #[test]
    fn test_health_calculator_ok() {
        let (score, label) = HealthCalculator::calculate(0.0, 7.0, 1.4, 4);
        assert!(score >= 50, "Score should be >= 50, got {}", score);
        assert!(score < 70, "Score should be < 70, got {}", score);
    }

    #[test]
    fn test_health_factors_disk_swap() {
        let factors = HealthFactors::calculate(10.0, 1.0, 2.0, 4);
        assert_eq!(factors.swap_penalty, 50);
    }

    #[test]
    fn test_health_factors_high_load() {
        let factors = HealthFactors::calculate(0.0, 6.5, 2.0, 4);
        assert_eq!(factors.load_penalty, 25);
    }
}
