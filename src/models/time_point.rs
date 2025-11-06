use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimePoint {
    pub hour: u8,
    pub minute: u8,
}

impl TimePoint {
    pub fn new(hour: u8, minute: u8) -> Result<Self, String> {
        if hour >= 24 {
            return Err(format!("Hour must be 0-23, got {}", hour));
        }
        if minute >= 60 {
            return Err(format!("Minute must be 0-59, got {}", minute));
        }
        Ok(TimePoint { hour, minute })
    }

    pub fn from_minutes_since_midnight(minutes: u32) -> Result<Self, String> {
        if minutes >= 24 * 60 {
            return Err(format!("Minutes must be < 1440, got {}", minutes));
        }
        Ok(TimePoint {
            hour: (minutes / 60) as u8,
            minute: (minutes % 60) as u8,
        })
    }

    pub fn to_minutes_since_midnight(self) -> u32 {
        (self.hour as u32) * 60 + (self.minute as u32)
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid time format: {}", s));
        }

        let hour = parts[0]
            .parse::<u8>()
            .map_err(|_| format!("Invalid hour: {}", parts[0]))?;
        let minute = parts[1]
            .parse::<u8>()
            .map_err(|_| format!("Invalid minute: {}", parts[1]))?;

        Self::new(hour, minute)
    }
}

impl fmt::Display for TimePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

impl FromStr for TimePoint {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TimePoint::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid_time() {
        let time = TimePoint::new(14, 30).unwrap();
        assert_eq!(time.hour, 14);
        assert_eq!(time.minute, 30);
    }

    #[test]
    fn test_new_boundary_values() {
        assert!(TimePoint::new(0, 0).is_ok());
        assert!(TimePoint::new(23, 59).is_ok());
    }

    #[test]
    fn test_new_invalid_hour() {
        assert!(TimePoint::new(24, 0).is_err());
        assert!(TimePoint::new(25, 30).is_err());
    }

    #[test]
    fn test_new_invalid_minute() {
        assert!(TimePoint::new(12, 60).is_err());
        assert!(TimePoint::new(12, 99).is_err());
    }

    #[test]
    fn test_parse_valid_time() {
        let time = TimePoint::parse("14:30").unwrap();
        assert_eq!(time.hour, 14);
        assert_eq!(time.minute, 30);
    }

    #[test]
    fn test_parse_with_leading_zeros() {
        let time = TimePoint::parse("09:05").unwrap();
        assert_eq!(time.hour, 9);
        assert_eq!(time.minute, 5);
    }

    #[test]
    fn test_parse_without_leading_zeros() {
        let time = TimePoint::parse("9:5").unwrap();
        assert_eq!(time.hour, 9);
        assert_eq!(time.minute, 5);
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!(TimePoint::parse("14").is_err());
        assert!(TimePoint::parse("14:30:00").is_err());
        assert!(TimePoint::parse("not a time").is_err());
        assert!(TimePoint::parse("").is_err());
    }

    #[test]
    fn test_parse_invalid_values() {
        assert!(TimePoint::parse("24:00").is_err());
        assert!(TimePoint::parse("12:60").is_err());
        assert!(TimePoint::parse("-1:30").is_err());
    }

    #[test]
    fn test_to_minutes_since_midnight() {
        assert_eq!(TimePoint::new(0, 0).unwrap().to_minutes_since_midnight(), 0);
        assert_eq!(
            TimePoint::new(1, 0).unwrap().to_minutes_since_midnight(),
            60
        );
        assert_eq!(
            TimePoint::new(14, 30).unwrap().to_minutes_since_midnight(),
            870
        );
        assert_eq!(
            TimePoint::new(23, 59).unwrap().to_minutes_since_midnight(),
            1439
        );
    }

    #[test]
    fn test_from_minutes_since_midnight() {
        let time = TimePoint::from_minutes_since_midnight(0).unwrap();
        assert_eq!(time, TimePoint::new(0, 0).unwrap());

        let time = TimePoint::from_minutes_since_midnight(60).unwrap();
        assert_eq!(time, TimePoint::new(1, 0).unwrap());

        let time = TimePoint::from_minutes_since_midnight(870).unwrap();
        assert_eq!(time, TimePoint::new(14, 30).unwrap());

        let time = TimePoint::from_minutes_since_midnight(1439).unwrap();
        assert_eq!(time, TimePoint::new(23, 59).unwrap());
    }

    #[test]
    fn test_from_minutes_invalid() {
        assert!(TimePoint::from_minutes_since_midnight(1440).is_err());
        assert!(TimePoint::from_minutes_since_midnight(9999).is_err());
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original = TimePoint::new(14, 30).unwrap();
        let minutes = original.to_minutes_since_midnight();
        let converted = TimePoint::from_minutes_since_midnight(minutes).unwrap();
        assert_eq!(original, converted);
    }

    #[test]
    fn test_display_format() {
        assert_eq!(TimePoint::new(9, 5).unwrap().to_string(), "09:05");
        assert_eq!(TimePoint::new(14, 30).unwrap().to_string(), "14:30");
        assert_eq!(TimePoint::new(0, 0).unwrap().to_string(), "00:00");
        assert_eq!(TimePoint::new(23, 59).unwrap().to_string(), "23:59");
    }

    #[test]
    fn test_from_str_trait() {
        let time: TimePoint = "14:30".parse().unwrap();
        assert_eq!(time.hour, 14);
        assert_eq!(time.minute, 30);
    }

    #[test]
    fn test_ordering() {
        let time1 = TimePoint::new(9, 0).unwrap();
        let time2 = TimePoint::new(14, 30).unwrap();
        let time3 = TimePoint::new(14, 30).unwrap();

        assert!(time1 < time2);
        assert!(time2 > time1);
        assert_eq!(time2, time3);
    }

    #[test]
    fn test_clone_and_copy() {
        let time1 = TimePoint::new(14, 30).unwrap();
        let time2 = time1;
        assert_eq!(time1, time2);
    }
}
