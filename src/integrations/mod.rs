use crate::config::Config;
use anyhow::Result;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackerType {
    Jira,
    Linear,
}

/// Extract ticket ID from task name using regex pattern: "PROJ-123 - Task name" -> "PROJ-123"
pub fn extract_ticket_from_name(name: &str) -> Option<String> {
    // Match common ticket patterns: WORD-NUMBER (e.g., PROJ-123, WL-1, LIN-456)
    let re = Regex::new(r"\b([A-Z]{2,10}-\d+)\b").ok()?;

    re.captures(name)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Detect which tracker type a ticket belongs to based on config patterns
pub fn detect_tracker_type(ticket: &str, config: &Config) -> Option<TrackerType> {
    let jira_config = &config.integrations.jira;
    let linear_config = &config.integrations.linear;

    // Try JIRA patterns first
    if jira_config.enabled && matches_patterns(ticket, &jira_config.ticket_patterns) {
        return Some(TrackerType::Jira);
    }

    // Try Linear patterns
    if linear_config.enabled && matches_patterns(ticket, &linear_config.ticket_patterns) {
        return Some(TrackerType::Linear);
    }

    // Fallback to default if ambiguous
    match config.integrations.default_tracker.as_str() {
        "linear" => Some(TrackerType::Linear),
        _ => Some(TrackerType::Jira),
    }
}

/// Check if ticket matches any of the provided patterns
fn matches_patterns(ticket: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| {
        Regex::new(pattern)
            .ok()
            .map(|re| re.is_match(ticket))
            .unwrap_or(false)
    })
}

/// Build a URL for the given ticket and tracker type
pub fn build_url(
    ticket: &str,
    tracker: TrackerType,
    config: &Config,
    for_worklog: bool,
) -> Result<String> {
    let tracker_config = match tracker {
        TrackerType::Jira => &config.integrations.jira,
        TrackerType::Linear => &config.integrations.linear,
    };

    if !tracker_config.enabled {
        anyhow::bail!("Tracker is not enabled in config");
    }

    let template = if for_worklog && !tracker_config.worklog_url.is_empty() {
        &tracker_config.worklog_url
    } else {
        &tracker_config.browse_url
    };

    let url = template
        .replace("{base_url}", &tracker_config.base_url)
        .replace("{ticket}", ticket);

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ticket_simple() {
        let name = "PROJ-123 Fix login bug";
        let ticket = extract_ticket_from_name(name);
        assert_eq!(ticket, Some("PROJ-123".to_string()));
    }

    #[test]
    fn test_extract_ticket_wl_format() {
        let name = "WL-1 Morning standup";
        let ticket = extract_ticket_from_name(name);
        assert_eq!(ticket, Some("WL-1".to_string()));
    }

    #[test]
    fn test_extract_ticket_lin_format() {
        let name = "LIN-456 Code review";
        let ticket = extract_ticket_from_name(name);
        assert_eq!(ticket, Some("LIN-456".to_string()));
    }

    #[test]
    fn test_extract_ticket_bracketed() {
        let name = "[ABC-789] Task name";
        let ticket = extract_ticket_from_name(name);
        assert_eq!(ticket, Some("ABC-789".to_string()));
    }

    #[test]
    fn test_extract_ticket_in_middle() {
        let name = "Work on PROJ-456 - code cleanup";
        let ticket = extract_ticket_from_name(name);
        assert_eq!(ticket, Some("PROJ-456".to_string()));
    }

    #[test]
    fn test_extract_ticket_no_ticket() {
        let name = "Just a regular task";
        let ticket = extract_ticket_from_name(name);
        assert_eq!(ticket, None);
    }

    #[test]
    fn test_extract_ticket_invalid_format() {
        let name = "task-123 invalid";
        let ticket = extract_ticket_from_name(name);
        assert_eq!(ticket, None); // lowercase doesn't match
    }

    #[test]
    fn test_detect_tracker_jira() {
        let config = Config::default();
        let tracker = detect_tracker_type("PROJ-123", &config);
        assert_eq!(tracker, Some(TrackerType::Jira));
    }

    #[test]
    fn test_detect_tracker_wl() {
        let config = Config::default();
        let tracker = detect_tracker_type("WL-1", &config);
        assert_eq!(tracker, Some(TrackerType::Jira)); // WL matches JIRA pattern
    }

    #[test]
    fn test_detect_tracker_default_fallback() {
        let config = Config::default();
        let tracker = detect_tracker_type("UNKNOWN-999", &config);
        assert_eq!(tracker, Some(TrackerType::Jira)); // Falls back to default
    }

    #[test]
    fn test_build_url_jira_browse() {
        // Config with enabled JIRA integration
        let toml_str = r#"
[integrations]
default_tracker = "jira"

[integrations.jira]
enabled = true
base_url = "https://test.atlassian.net"
ticket_patterns = ["^[A-Z]+-\\d+$"]
browse_url = "{base_url}/browse/{ticket}"
worklog_url = "{base_url}/browse/{ticket}?focusedWorklogId=-1"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let url = build_url("WL-1", TrackerType::Jira, &config, false);
        assert!(url.is_ok());
        assert_eq!(url.unwrap(), "https://test.atlassian.net/browse/WL-1");
    }

    #[test]
    fn test_build_url_jira_worklog() {
        // Config with enabled JIRA integration
        let toml_str = r#"
[integrations]
default_tracker = "jira"

[integrations.jira]
enabled = true
base_url = "https://test.atlassian.net"
ticket_patterns = ["^[A-Z]+-\\d+$"]
browse_url = "{base_url}/browse/{ticket}"
worklog_url = "{base_url}/browse/{ticket}?focusedWorklogId=-1"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let url = build_url("WL-1", TrackerType::Jira, &config, true);
        assert!(url.is_ok());
        assert_eq!(
            url.unwrap(),
            "https://test.atlassian.net/browse/WL-1?focusedWorklogId=-1"
        );
    }

    #[test]
    fn test_build_url_with_custom_config() {
        let toml_str = r#"
[integrations]
default_tracker = "jira"

[integrations.jira]
enabled = true
base_url = "https://custom.atlassian.net"
ticket_patterns = ["^PROJ-\\d+$"]
browse_url = "{base_url}/browse/{ticket}"
        "#;

        let config: crate::config::Config = toml::from_str(toml_str).unwrap();
        let url = build_url("PROJ-456", TrackerType::Jira, &config, false);
        assert!(url.is_ok());
        assert_eq!(url.unwrap(), "https://custom.atlassian.net/browse/PROJ-456");
    }

    #[test]
    fn test_matches_patterns() {
        let patterns = vec!["^[A-Z]+-\\d+$".to_string()];
        assert!(matches_patterns("PROJ-123", &patterns));
        assert!(matches_patterns("WL-1", &patterns));
        assert!(!matches_patterns("invalid", &patterns));
    }

    #[test]
    fn test_extract_first_ticket_only() {
        // If there are multiple tickets, extract the first one
        let name = "PROJ-123 and WL-456 task";
        let ticket = extract_ticket_from_name(name);
        assert_eq!(ticket, Some("PROJ-123".to_string()));
    }
}
