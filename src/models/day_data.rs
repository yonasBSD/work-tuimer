use super::WorkRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::Date;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayData {
    pub date: Date,
    pub last_id: u32,
    pub work_records: HashMap<u32, WorkRecord>,
}

impl DayData {
    pub fn new(date: Date) -> Self {
        DayData {
            date,
            last_id: 0,
            work_records: HashMap::new(),
        }
    }

    pub fn add_record(&mut self, record: WorkRecord) {
        if record.id > self.last_id {
            self.last_id = record.id;
        }
        self.work_records.insert(record.id, record);
    }

    pub fn remove_record(&mut self, id: u32) -> Option<WorkRecord> {
        self.work_records.remove(&id)
    }

    pub fn next_id(&mut self) -> u32 {
        self.last_id += 1;
        self.last_id
    }

    pub fn get_sorted_records(&self) -> Vec<&WorkRecord> {
        let mut records: Vec<&WorkRecord> = self.work_records.values().collect();
        records.sort_by_key(|r| r.start);
        records
    }

    pub fn get_grouped_totals(&self) -> Vec<(String, u32)> {
        let mut totals: HashMap<String, u32> = HashMap::new();

        for record in self.work_records.values() {
            *totals.entry(record.name.clone()).or_insert(0) += record.total_minutes;
        }

        let mut result: Vec<(String, u32)> = totals.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TimePoint;

    fn create_test_date() -> Date {
        Date::from_calendar_date(2025, time::Month::November, 6).unwrap()
    }

    fn create_test_record(id: u32, name: &str, start_hour: u8, end_hour: u8) -> WorkRecord {
        let start = TimePoint::new(start_hour, 0).unwrap();
        let end = TimePoint::new(end_hour, 0).unwrap();
        WorkRecord::new(id, name.to_string(), start, end)
    }

    #[test]
    fn test_new_day_data() {
        let date = create_test_date();
        let day = DayData::new(date);

        assert_eq!(day.date, date);
        assert_eq!(day.last_id, 0);
        assert_eq!(day.work_records.len(), 0);
    }

    #[test]
    fn test_add_record() {
        let mut day = DayData::new(create_test_date());
        let record = create_test_record(1, "Coding", 9, 17);

        day.add_record(record.clone());

        assert_eq!(day.work_records.len(), 1);
        assert_eq!(day.last_id, 1);
        assert!(day.work_records.contains_key(&1));
    }

    #[test]
    fn test_add_multiple_records() {
        let mut day = DayData::new(create_test_date());

        day.add_record(create_test_record(1, "Coding", 9, 12));
        day.add_record(create_test_record(2, "Meeting", 13, 14));
        day.add_record(create_test_record(3, "Code Review", 14, 16));

        assert_eq!(day.work_records.len(), 3);
        assert_eq!(day.last_id, 3);
    }

    #[test]
    fn test_add_record_updates_last_id() {
        let mut day = DayData::new(create_test_date());

        day.add_record(create_test_record(5, "Task", 9, 10));
        assert_eq!(day.last_id, 5);

        day.add_record(create_test_record(2, "Task2", 10, 11));
        assert_eq!(day.last_id, 5); // Should not decrease

        day.add_record(create_test_record(10, "Task3", 11, 12));
        assert_eq!(day.last_id, 10);
    }

    #[test]
    fn test_remove_record() {
        let mut day = DayData::new(create_test_date());
        day.add_record(create_test_record(1, "Coding", 9, 17));

        let removed = day.remove_record(1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "Coding");
        assert_eq!(day.work_records.len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_record() {
        let mut day = DayData::new(create_test_date());
        day.add_record(create_test_record(1, "Coding", 9, 17));

        let removed = day.remove_record(999);
        assert!(removed.is_none());
        assert_eq!(day.work_records.len(), 1);
    }

    #[test]
    fn test_next_id() {
        let mut day = DayData::new(create_test_date());

        assert_eq!(day.next_id(), 1);
        assert_eq!(day.next_id(), 2);
        assert_eq!(day.next_id(), 3);
        assert_eq!(day.last_id, 3);
    }

    #[test]
    fn test_next_id_after_add_record() {
        let mut day = DayData::new(create_test_date());
        day.add_record(create_test_record(5, "Task", 9, 10));

        assert_eq!(day.last_id, 5);
        assert_eq!(day.next_id(), 6);
        assert_eq!(day.next_id(), 7);
    }

    #[test]
    fn test_get_sorted_records_empty() {
        let day = DayData::new(create_test_date());
        let sorted = day.get_sorted_records();
        assert_eq!(sorted.len(), 0);
    }

    #[test]
    fn test_get_sorted_records_single() {
        let mut day = DayData::new(create_test_date());
        day.add_record(create_test_record(1, "Coding", 9, 17));

        let sorted = day.get_sorted_records();
        assert_eq!(sorted.len(), 1);
        assert_eq!(sorted[0].name, "Coding");
    }

    #[test]
    fn test_get_sorted_records_already_sorted() {
        let mut day = DayData::new(create_test_date());
        day.add_record(create_test_record(1, "Morning", 9, 12));
        day.add_record(create_test_record(2, "Afternoon", 13, 17));

        let sorted = day.get_sorted_records();
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].name, "Morning");
        assert_eq!(sorted[1].name, "Afternoon");
    }

    #[test]
    fn test_get_sorted_records_unsorted() {
        let mut day = DayData::new(create_test_date());
        day.add_record(create_test_record(1, "Afternoon", 13, 17));
        day.add_record(create_test_record(2, "Morning", 9, 12));
        day.add_record(create_test_record(3, "Evening", 18, 20));

        let sorted = day.get_sorted_records();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].name, "Morning");
        assert_eq!(sorted[1].name, "Afternoon");
        assert_eq!(sorted[2].name, "Evening");
    }

    #[test]
    fn test_get_sorted_records_same_start_time() {
        let mut day = DayData::new(create_test_date());
        let start = TimePoint::new(9, 0).unwrap();
        let end1 = TimePoint::new(10, 0).unwrap();
        let end2 = TimePoint::new(11, 0).unwrap();

        day.add_record(WorkRecord::new(1, "Task1".to_string(), start, end1));
        day.add_record(WorkRecord::new(2, "Task2".to_string(), start, end2));

        let sorted = day.get_sorted_records();
        assert_eq!(sorted.len(), 2);
        // Both start at 9:00, order doesn't matter but both should be present
        assert!(sorted.iter().any(|r| r.name == "Task1"));
        assert!(sorted.iter().any(|r| r.name == "Task2"));
    }

    #[test]
    fn test_get_grouped_totals_empty() {
        let day = DayData::new(create_test_date());
        let totals = day.get_grouped_totals();
        assert_eq!(totals.len(), 0);
    }

    #[test]
    fn test_get_grouped_totals_single_task() {
        let mut day = DayData::new(create_test_date());
        day.add_record(create_test_record(1, "Coding", 9, 17)); // 8 hours

        let totals = day.get_grouped_totals();
        assert_eq!(totals.len(), 1);
        assert_eq!(totals[0].0, "Coding");
        assert_eq!(totals[0].1, 480); // 8 * 60 minutes
    }

    #[test]
    fn test_get_grouped_totals_multiple_different_tasks() {
        let mut day = DayData::new(create_test_date());
        day.add_record(create_test_record(1, "Coding", 9, 12)); // 3 hours
        day.add_record(create_test_record(2, "Meeting", 13, 14)); // 1 hour
        day.add_record(create_test_record(3, "Code Review", 14, 16)); // 2 hours

        let totals = day.get_grouped_totals();
        assert_eq!(totals.len(), 3);

        // Should be sorted by duration (descending)
        assert_eq!(totals[0].0, "Coding");
        assert_eq!(totals[0].1, 180);
        assert_eq!(totals[1].0, "Code Review");
        assert_eq!(totals[1].1, 120);
        assert_eq!(totals[2].0, "Meeting");
        assert_eq!(totals[2].1, 60);
    }

    #[test]
    fn test_get_grouped_totals_same_task_multiple_times() {
        let mut day = DayData::new(create_test_date());
        day.add_record(create_test_record(1, "Coding", 9, 11)); // 2 hours
        day.add_record(create_test_record(2, "Meeting", 11, 12)); // 1 hour
        day.add_record(create_test_record(3, "Coding", 13, 16)); // 3 hours
        day.add_record(create_test_record(4, "Coding", 16, 17)); // 1 hour

        let totals = day.get_grouped_totals();
        assert_eq!(totals.len(), 2);

        // Coding should be grouped: 2 + 3 + 1 = 6 hours
        assert_eq!(totals[0].0, "Coding");
        assert_eq!(totals[0].1, 360);
        assert_eq!(totals[1].0, "Meeting");
        assert_eq!(totals[1].1, 60);
    }

    #[test]
    fn test_get_grouped_totals_sorted_by_duration() {
        let mut day = DayData::new(create_test_date());
        day.add_record(create_test_record(1, "Short", 9, 10)); // 1 hour
        day.add_record(create_test_record(2, "Long", 10, 15)); // 5 hours
        day.add_record(create_test_record(3, "Medium", 15, 17)); // 2 hours

        let totals = day.get_grouped_totals();

        // Should be sorted by duration descending
        assert_eq!(totals[0].0, "Long");
        assert_eq!(totals[1].0, "Medium");
        assert_eq!(totals[2].0, "Short");
    }

    #[test]
    fn test_clone() {
        let mut day1 = DayData::new(create_test_date());
        day1.add_record(create_test_record(1, "Coding", 9, 17));

        let day2 = day1.clone();

        assert_eq!(day1.date, day2.date);
        assert_eq!(day1.last_id, day2.last_id);
        assert_eq!(day1.work_records.len(), day2.work_records.len());
    }
}
