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

    pub fn to_minutes_since_midnight(&self) -> u32 {
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
