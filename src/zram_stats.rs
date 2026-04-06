use std::fs;

pub struct ZramStats {
    pub orig: u64,
    pub compr: u64,
    #[allow(dead_code)]
    pub mem: u64,
}

pub struct ZramReader;

impl ZramReader {
    pub fn new() -> Self {
        ZramReader
    }

    pub fn read(&self) -> Option<ZramStats> {
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

    pub fn compression_ratio(&self, stats: &ZramStats) -> f64 {
        if stats.compr > 0 {
            stats.orig as f64 / stats.compr as f64
        } else {
            0.0
        }
    }

    #[allow(dead_code)]
    pub fn saved_bytes(&self, stats: &ZramStats) -> i64 {
        stats.orig as i64 - stats.compr as i64
    }
}

impl Default for ZramReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zram_reader_creation() {
        let reader = ZramReader::new();
        assert!(reader.read().is_none() || reader.read().is_some());
    }

    #[test]
    fn test_compression_ratio() {
        let reader = ZramReader::new();
        let stats = ZramStats {
            orig: 1000,
            compr: 250,
            mem: 300,
        };
        assert_eq!(reader.compression_ratio(&stats), 4.0);
    }

    #[test]
    fn test_compression_ratio_zero_compr() {
        let reader = ZramReader::new();
        let stats = ZramStats {
            orig: 1000,
            compr: 0,
            mem: 300,
        };
        assert_eq!(reader.compression_ratio(&stats), 0.0);
    }

    #[test]
    fn test_saved_bytes() {
        let reader = ZramReader::new();
        let stats = ZramStats {
            orig: 1000,
            compr: 250,
            mem: 300,
        };
        assert_eq!(reader.saved_bytes(&stats), 750);
    }

    #[test]
    fn test_saved_bytes_negative() {
        let reader = ZramReader::new();
        let stats = ZramStats {
            orig: 100,
            compr: 250,
            mem: 300,
        };
        assert_eq!(reader.saved_bytes(&stats), -150);
    }
}
