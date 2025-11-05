use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration for issue tracker integrations (JIRA, Linear, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub integrations: IntegrationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Default tracker type when auto-detection is ambiguous
    #[serde(default = "default_tracker")]
    pub default_tracker: String,

    /// JIRA configuration
    #[serde(default)]
    pub jira: TrackerConfig,

    /// Linear configuration
    #[serde(default)]
    pub linear: TrackerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrackerConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub base_url: String,
    /// Regex patterns to match ticket IDs for this tracker
    #[serde(default)]
    pub ticket_patterns: Vec<String>,
    /// URL template for browsing tickets: {base_url}, {ticket}
    #[serde(default)]
    pub browse_url: String,
    /// URL template for worklog page: {base_url}, {ticket}
    #[serde(default)]
    pub worklog_url: String,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            default_tracker: "jira".to_string(),
            jira: TrackerConfig::default(),
            linear: TrackerConfig::default(),
        }
    }
}

fn default_tracker() -> String {
    "jira".to_string()
}

impl Config {
    /// Load config from file, or return defaults if file doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path();

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .context(format!("Failed to read config file: {:?}", config_path))?;
            let config: Config =
                toml::from_str(&contents).context("Failed to parse config TOML")?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Get config file path (~/.config/work-tuimer/config.toml)
    fn get_config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("work-tuimer").join("config.toml")
        } else {
            PathBuf::from("./config.toml")
        }
    }

    /// Check if any tracker integration is properly configured
    pub fn has_integrations(&self) -> bool {
        (self.integrations.jira.enabled && !self.integrations.jira.base_url.is_empty())
            || (self.integrations.linear.enabled && !self.integrations.linear.base_url.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.integrations.default_tracker, "jira");
        assert!(!config.integrations.jira.enabled);
        assert_eq!(config.integrations.jira.base_url, "");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize");
        assert!(toml_str.contains("jira"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[integrations]
default_tracker = "jira"

[integrations.jira]
enabled = true
base_url = "https://test.atlassian.net"
ticket_patterns = ["^PROJ-\\d+$"]
browse_url = "{base_url}/browse/{ticket}"
worklog_url = "{base_url}/browse/{ticket}?focusedWorklogId=-1"
        "#;

        let config: Config = toml::from_str(toml_str).expect("Failed to deserialize");
        assert_eq!(
            config.integrations.jira.base_url,
            "https://test.atlassian.net"
        );
        assert_eq!(config.integrations.jira.ticket_patterns[0], "^PROJ-\\d+$");
    }

    #[test]
    fn test_tracker_config_defaults() {
        let tracker = TrackerConfig::default();
        assert!(!tracker.enabled);
        assert!(tracker.base_url.is_empty());
        assert!(tracker.ticket_patterns.is_empty());
    }
}
