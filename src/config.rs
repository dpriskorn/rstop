use crate::logger::Logger;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterConfig {
    pub min_cpu: Option<f32>,
    pub exclude_names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub filter: Option<FilterConfig>,
    pub interval: Option<f64>,
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();

        if let Some(path) = config_path {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                match serde_yaml::from_str::<Config>(&contents) {
                    Ok(config) => {
                        let logger = Logger::new();
                        logger.debug(&format!("Loaded config from {:?}", path));
                        return config;
                    }
                    Err(e) => {
                        let logger = Logger::new();
                        logger.debug(&format!("Failed to parse config: {}", e));
                    }
                }
            }
        }

        Self::default()
    }

    fn config_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join("rtop").join("config.yaml"))
    }

    pub fn merge_with_args(
        &self,
        min_cpu: Option<f64>,
        exclude_names: Vec<String>,
        interval: f64,
    ) -> (f32, Vec<String>, f64) {
        let min_cpu = min_cpu
            .map(|v| v as f32)
            .or(self.filter.as_ref().and_then(|f| f.min_cpu))
            .unwrap_or(10.0);

        let exclude_names = if exclude_names.is_empty() {
            self.filter
                .as_ref()
                .and_then(|f| f.exclude_names.clone())
                .unwrap_or_else(|| vec!["HeapHelper".to_string()])
        } else {
            exclude_names
        };

        let interval = self.interval.unwrap_or(interval);

        (min_cpu, exclude_names, interval)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            filter: Some(FilterConfig {
                min_cpu: Some(10.0),
                exclude_names: Some(vec!["HeapHelper".to_string()]),
            }),
            interval: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.filter.is_some());
        assert_eq!(config.filter.as_ref().unwrap().min_cpu, Some(10.0));
        assert_eq!(config.interval, None);
    }

    #[test]
    fn test_merge_with_args() {
        let config = Config::default();
        let (min_cpu, exclude_names, interval) =
            config.merge_with_args(Some(5.0), vec!["test".to_string()], 1.0);
        assert_eq!(min_cpu, 5.0);
        assert_eq!(exclude_names, vec!["test"]);
        assert_eq!(interval, 1.0);
    }

    #[test]
    fn test_merge_without_args() {
        let config = Config::default();
        let (min_cpu, exclude_names, interval) = config.merge_with_args(None, vec![], 2.0);
        assert_eq!(min_cpu, 10.0);
        assert!(exclude_names.contains(&"HeapHelper".to_string()));
        assert_eq!(interval, 2.0);
    }
}
