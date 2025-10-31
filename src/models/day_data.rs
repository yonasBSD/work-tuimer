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

    pub fn get_record(&self, id: u32) -> Option<&WorkRecord> {
        self.work_records.get(&id)
    }

    pub fn get_record_mut(&mut self, id: u32) -> Option<&mut WorkRecord> {
        self.work_records.get_mut(&id)
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
