use super::TimePoint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkRecord {
    pub id: u32,
    pub name: String,
    pub start: TimePoint,
    pub end: TimePoint,
    pub total_minutes: u32,
    #[serde(default)]
    pub description: String,
}

impl WorkRecord {
    pub fn new(id: u32, name: String, start: TimePoint, end: TimePoint) -> Self {
        let total_minutes = Self::calculate_duration(&start, &end);
        WorkRecord {
            id,
            name,
            start,
            end,
            total_minutes,
            description: String::new(),
        }
    }

    pub fn calculate_duration(start: &TimePoint, end: &TimePoint) -> u32 {
        let start_mins = start.to_minutes_since_midnight();
        let end_mins = end.to_minutes_since_midnight();

        if end_mins >= start_mins {
            end_mins - start_mins
        } else {
            (24 * 60 - start_mins) + end_mins
        }
    }

    pub fn update_duration(&mut self) {
        self.total_minutes = Self::calculate_duration(&self.start, &self.end);
    }

    pub fn format_duration(&self) -> String {
        let hours = self.total_minutes / 60;
        let minutes = self.total_minutes % 60;
        format!("{}h {:02}m", hours, minutes)
    }
}
