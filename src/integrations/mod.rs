use crate::config::Config;
use anyhow::Result;
use regex::Regex;

/// Extract ticket ID from task name using regex pattern: "PROJ-123 - Task name" -> "PROJ-123"
pub fn extract_ticket_from_name(name: &str) -> Option<String> {
    // Match common ticket patterns: WORD-NUMBER (e.g., PROJ-123, WL-1, LIN-456)
    let re = Regex::new(r"\b([A-Z]{2,10}-\d+)\b").ok()?;

    re.captures(name)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

/// Detect which tracker a ticket belongs to based on config patterns
/// Returns the tracker name if a match is found
pub fn detect_tracker(ticket: &str, config: &Config) -> Option<String> {
    // Try each enabled tracker's patterns
    for (name, tracker_config) in &config.integrations.trackers {
        if tracker_config.enabled && matches_patterns(ticket, &tracker_config.ticket_patterns) {
            return Some(name.clone());
        }
    }

    // Fallback to default tracker if configured
    config.integrations.default_tracker.clone()
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

/// Build a URL for the given ticket and tracker name
pub fn build_url(
    ticket: &str,
    tracker_name: &str,
    config: &Config,
    for_worklog: bool,
) -> Result<String> {
    let tracker_config = config
        .integrations
        .trackers
        .get(tracker_name)
        .ok_or_else(|| anyhow::anyhow!("Tracker '{}' not found in config", tracker_name))?;

    if !tracker_config.enabled {
        anyhow::bail!("Tracker '{}' is not enabled in config", tracker_name);
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
    fn test_detect_tracker_by_pattern() {
        let toml_str = r#"
[integrations]
default_tracker = "my-jira"

[integrations.trackers.my-jira]
enabled = true
base_url = "https://test.atlassian.net"
ticket_patterns = ["^PROJ-\\d+$", "^WL-\\d+$"]
browse_url = "{base_url}/browse/{ticket}"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();

        let tracker = detect_tracker("PROJ-123", &config);
        assert_eq!(tracker, Some("my-jira".to_string()));

        let tracker = detect_tracker("WL-1", &config);
        assert_eq!(tracker, Some("my-jira".to_string()));
    }

    #[test]
    fn test_detect_tracker_default_fallback() {
        let toml_str = r#"
[integrations]
default_tracker = "my-jira"

[integrations.trackers.my-jira]
enabled = true
base_url = "https://test.atlassian.net"
ticket_patterns = ["^PROJ-\\d+$"]
browse_url = "{base_url}/browse/{ticket}"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();

        // UNKNOWN-999 doesn't match pattern, falls back to default
        let tracker = detect_tracker("UNKNOWN-999", &config);
        assert_eq!(tracker, Some("my-jira".to_string()));
    }

    #[test]
    fn test_detect_tracker_multiple_trackers() {
        let toml_str = r#"
[integrations]
default_tracker = "my-jira"

[integrations.trackers.my-jira]
enabled = true
base_url = "https://test.atlassian.net"
ticket_patterns = ["^PROJ-\\d+$"]
browse_url = "{base_url}/browse/{ticket}"

[integrations.trackers.github]
enabled = true
base_url = "https://github.com/user/repo"
ticket_patterns = ["^#\\d+$"]
browse_url = "{base_url}/issues/{ticket}"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();

        let tracker = detect_tracker("PROJ-123", &config);
        assert_eq!(tracker, Some("my-jira".to_string()));

        let tracker = detect_tracker("#456", &config);
        assert_eq!(tracker, Some("github".to_string()));
    }

    #[test]
    fn test_detect_tracker_overlapping_patterns_first_wins() {
        // Test that when multiple trackers match, the first one in iteration order wins
        let toml_str = r#"
[integrations]
default_tracker = "fallback"

[integrations.trackers.jira]
enabled = true
base_url = "https://jira.example.com"
ticket_patterns = ["^[A-Z]+-\\d+$"]
browse_url = "{base_url}/browse/{ticket}"

[integrations.trackers.linear]
enabled = true
base_url = "https://linear.app/team"
ticket_patterns = ["^[A-Z]+-\\d+$"]
browse_url = "{base_url}/issue/{ticket}"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();

        // PROJ-123 matches both patterns - should use whichever tracker appears first
        let tracker = detect_tracker("PROJ-123", &config);
        assert!(tracker.is_some());

        // The result should be deterministic (either "jira" or "linear")
        // Note: HashMap iteration order is not guaranteed in Rust, but it should be consistent
        let tracker_name = tracker.unwrap();
        assert!(tracker_name == "jira" || tracker_name == "linear");

        // Verify the same ticket always resolves to the same tracker
        let tracker2 = detect_tracker("PROJ-123", &config);
        assert_eq!(tracker2, Some(tracker_name));
    }

    #[test]
    fn test_detect_tracker_no_match_no_default() {
        // Test that when no patterns match and no default is set, returns None
        let toml_str = r#"
[integrations]

[integrations.trackers.jira]
enabled = true
base_url = "https://jira.example.com"
ticket_patterns = ["^PROJ-\\d+$"]
browse_url = "{base_url}/browse/{ticket}"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();

        // UNKNOWN-999 doesn't match and there's no default_tracker
        let tracker = detect_tracker("UNKNOWN-999", &config);
        assert_eq!(tracker, None);
    }

    #[test]
    fn test_build_url_browse() {
        let toml_str = r#"
[integrations]
default_tracker = "my-jira"

[integrations.trackers.my-jira]
enabled = true
base_url = "https://test.atlassian.net"
ticket_patterns = ["^[A-Z]+-\\d+$"]
browse_url = "{base_url}/browse/{ticket}"
worklog_url = "{base_url}/browse/{ticket}?focusedWorklogId=-1"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let url = build_url("WL-1", "my-jira", &config, false);
        assert!(url.is_ok());
        assert_eq!(url.unwrap(), "https://test.atlassian.net/browse/WL-1");
    }

    #[test]
    fn test_build_url_worklog() {
        let toml_str = r#"
[integrations]
default_tracker = "my-jira"

[integrations.trackers.my-jira]
enabled = true
base_url = "https://test.atlassian.net"
ticket_patterns = ["^[A-Z]+-\\d+$"]
browse_url = "{base_url}/browse/{ticket}"
worklog_url = "{base_url}/browse/{ticket}?focusedWorklogId=-1"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let url = build_url("WL-1", "my-jira", &config, true);
        assert!(url.is_ok());
        assert_eq!(
            url.unwrap(),
            "https://test.atlassian.net/browse/WL-1?focusedWorklogId=-1"
        );
    }

    #[test]
    fn test_build_url_github() {
        let toml_str = r#"
[integrations]

[integrations.trackers.github]
enabled = true
base_url = "https://github.com/user/repo"
ticket_patterns = ["^#\\d+$"]
browse_url = "{base_url}/issues/{ticket}"
worklog_url = ""
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        let url = build_url("#456", "github", &config, false);
        assert!(url.is_ok());
        assert_eq!(url.unwrap(), "https://github.com/user/repo/issues/#456");
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

    #[test]
    fn test_build_url_with_query_params_in_browse_url() {
        // Issue #42: URLs with query parameters in browse_url template
        // e.g., Zentao-style URLs: {base_url}?m=my&f=work&mode=bug
        let toml_str = r#"
[integrations]
default_tracker = "zentao"

[integrations.trackers.zentao]
enabled = true
base_url = "http://domain/index.php"
ticket_patterns = ["^BUG-\\d+$"]
browse_url = "{base_url}?m=my&f=work&mode=bug&type=assignedTo"
worklog_url = "{base_url}?m=bug&f=view&bugID={ticket}"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();

        // Test browse URL with query params (no ticket placeholder in browse_url)
        let url = build_url("BUG-123", "zentao", &config, false);
        assert!(url.is_ok());
        let url_str = url.unwrap();
        assert_eq!(
            url_str,
            "http://domain/index.php?m=my&f=work&mode=bug&type=assignedTo"
        );
        // Verify URL contains all query parameters
        assert!(url_str.contains("m=my"));
        assert!(url_str.contains("f=work"));
        assert!(url_str.contains("mode=bug"));
        assert!(url_str.contains("type=assignedTo"));
    }

    #[test]
    fn test_build_url_with_ticket_in_query_params() {
        // Issue #42: worklog URLs that put ticket ID in query parameter
        let toml_str = r#"
[integrations]
default_tracker = "zentao"

[integrations.trackers.zentao]
enabled = true
base_url = "http://domain/index.php"
ticket_patterns = ["^BUG-\\d+$"]
browse_url = "{base_url}?m=my&f=work&mode=bug"
worklog_url = "{base_url}?m=bug&f=view&bugID={ticket}"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();

        // Test worklog URL with ticket in query string
        let url = build_url("BUG-456", "zentao", &config, true);
        assert!(url.is_ok());
        let url_str = url.unwrap();
        assert_eq!(
            url_str,
            "http://domain/index.php?m=bug&f=view&bugID=BUG-456"
        );
        // Verify ticket was substituted correctly
        assert!(url_str.contains("bugID=BUG-456"));
        assert!(url_str.contains("m=bug"));
        assert!(url_str.contains("f=view"));
    }

    #[test]
    fn test_build_url_complex_query_string() {
        // Test URL with many & characters that could break Windows cmd
        let toml_str = r#"
[integrations]
default_tracker = "tracker"

[integrations.trackers.tracker]
enabled = true
base_url = "https://tracker.example.com"
ticket_patterns = ["^ISSUE-\\d+$"]
browse_url = "{base_url}/view?id={ticket}&action=show&tab=details&expand=true"
worklog_url = ""
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();

        let url = build_url("ISSUE-789", "tracker", &config, false);
        assert!(url.is_ok());
        let url_str = url.unwrap();
        assert_eq!(
            url_str,
            "https://tracker.example.com/view?id=ISSUE-789&action=show&tab=details&expand=true"
        );
        // Verify all parts are present
        assert!(url_str.contains("id=ISSUE-789"));
        assert!(url_str.contains("action=show"));
        assert!(url_str.contains("tab=details"));
        assert!(url_str.contains("expand=true"));
    }
}
