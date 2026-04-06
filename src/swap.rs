pub struct SwapGroup {
    #[allow(dead_code)]
    pub size: u64,
    #[allow(dead_code)]
    pub used: u64,
    pub percent: f32,
}

impl SwapGroup {
    pub fn from_size_used(size: u64, used: u64) -> Self {
        SwapGroup {
            percent: if size > 0 {
                (used as f32 / size as f32) * 100.0
            } else {
                0.0
            },
            size,
            used,
        }
    }
}

pub struct SwapStats {
    pub zram: SwapGroup,
    pub disk: SwapGroup,
}

impl SwapStats {
    pub fn read() -> Self {
        let mut zram_size: u64 = 0;
        let mut zram_used: u64 = 0;
        let mut disk_size: u64 = 0;
        let mut disk_used: u64 = 0;

        let content = match std::fs::read_to_string("/proc/swaps") {
            Ok(c) => c,
            Err(_) => {
                return SwapStats {
                    zram: SwapGroup::from_size_used(0, 0),
                    disk: SwapGroup::from_size_used(0, 0),
                };
            }
        };

        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line
                .split(|c: char| c.is_whitespace())
                .filter(|s| !s.is_empty())
                .collect();
            if parts.len() < 4 {
                continue;
            }

            let dev = parts[0];
            let size: u64 = match parts[2].parse() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let used: u64 = match parts[3].parse() {
                Ok(u) => u,
                Err(_) => continue,
            };

            if dev.starts_with("/dev/zram") {
                zram_size += size;
                zram_used += used;
            } else {
                disk_size += size;
                disk_used += used;
            }
        }

        SwapStats {
            zram: SwapGroup::from_size_used(zram_size, zram_used),
            disk: SwapGroup::from_size_used(disk_size, disk_used),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_group_zero() {
        let group = SwapGroup::from_size_used(0, 0);
        assert_eq!(group.percent, 0.0);
    }

    #[test]
    fn test_swap_group_full() {
        let group = SwapGroup::from_size_used(1000, 1000);
        assert_eq!(group.percent, 100.0);
    }

    #[test]
    fn test_swap_group_half() {
        let group = SwapGroup::from_size_used(1000, 500);
        assert_eq!(group.percent, 50.0);
    }

    #[test]
    fn test_swap_stats_read() {
        let _stats = SwapStats::read();
    }
}
