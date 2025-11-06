use crate::models::DayData;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use time::Date;

pub struct Storage {
    data_dir: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let data_dir = Self::get_data_directory()?;
        fs::create_dir_all(&data_dir).context("Failed to create data directory")?;

        Ok(Storage { data_dir })
    }

    #[cfg(test)]
    pub fn new_with_dir(data_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&data_dir).context("Failed to create data directory")?;
        Ok(Storage { data_dir })
    }

    fn get_data_directory() -> Result<PathBuf> {
        if let Some(data_dir) = dirs::data_local_dir() {
            let app_dir = data_dir.join("work-tuimer");
            if fs::create_dir_all(&app_dir).is_ok() {
                return Ok(app_dir);
            }
        }

        let fallback = PathBuf::from("./data");
        fs::create_dir_all(&fallback).context("Failed to create fallback data directory")?;
        Ok(fallback)
    }

    fn get_file_path(&self, date: &Date) -> PathBuf {
        self.data_dir.join(format!(
            "{}-{:02}-{:02}.json",
            date.year(),
            date.month() as u8,
            date.day()
        ))
    }

    pub fn load(&self, date: &Date) -> Result<DayData> {
        let path = self.get_file_path(date);

        if !path.exists() {
            return Ok(DayData::new(*date));
        }

        let contents =
            fs::read_to_string(&path).context(format!("Failed to read file: {:?}", path))?;

        let day_data: DayData = serde_json::from_str(&contents).context("Failed to parse JSON")?;

        Ok(day_data)
    }

    pub fn save(&self, day_data: &DayData) -> Result<()> {
        let path = self.get_file_path(&day_data.date);

        let json = serde_json::to_string_pretty(day_data).context("Failed to serialize data")?;

        fs::write(&path, json).context(format!("Failed to write file: {:?}", path))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TimePoint, WorkRecord};
    use tempfile::TempDir;
    use time::Date;

    fn create_test_date() -> Date {
        Date::from_calendar_date(2025, time::Month::November, 6).unwrap()
    }

    fn create_test_record(id: u32, name: &str) -> WorkRecord {
        let start = TimePoint::new(9, 0).unwrap();
        let end = TimePoint::new(17, 0).unwrap();
        WorkRecord::new(id, name.to_string(), start, end)
    }

    #[test]
    fn test_new_storage_with_temp_dir() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf());

        assert!(storage.is_ok());
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_get_file_path_format() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        let file_path = storage.get_file_path(&date);

        assert_eq!(file_path.file_name().unwrap(), "2025-11-06.json");
    }

    #[test]
    fn test_load_nonexistent_file_returns_empty_day() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        let result = storage.load(&date);

        assert!(result.is_ok());
        let day_data = result.unwrap();
        assert_eq!(day_data.date, date);
        assert_eq!(day_data.work_records.len(), 0);
        assert_eq!(day_data.last_id, 0);
    }

    #[test]
    fn test_save_and_load_empty_day() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();
        let day_data = DayData::new(date);

        // Save
        let save_result = storage.save(&day_data);
        assert!(save_result.is_ok());

        // Load
        let load_result = storage.load(&date);
        assert!(load_result.is_ok());
        let loaded_data = load_result.unwrap();

        assert_eq!(loaded_data.date, date);
        assert_eq!(loaded_data.work_records.len(), 0);
    }

    #[test]
    fn test_save_and_load_with_records() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        let mut day_data = DayData::new(date);
        day_data.add_record(create_test_record(1, "Coding"));
        day_data.add_record(create_test_record(2, "Meeting"));

        // Save
        storage.save(&day_data).unwrap();

        // Load
        let loaded_data = storage.load(&date).unwrap();

        assert_eq!(loaded_data.date, date);
        assert_eq!(loaded_data.work_records.len(), 2);
        assert_eq!(loaded_data.last_id, 2);
        assert!(loaded_data.work_records.contains_key(&1));
        assert!(loaded_data.work_records.contains_key(&2));
    }

    #[test]
    fn test_save_overwrites_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        // Save first version
        let mut day_data1 = DayData::new(date);
        day_data1.add_record(create_test_record(1, "Task1"));
        storage.save(&day_data1).unwrap();

        // Save second version (overwrite)
        let mut day_data2 = DayData::new(date);
        day_data2.add_record(create_test_record(2, "Task2"));
        storage.save(&day_data2).unwrap();

        // Load should return second version
        let loaded_data = storage.load(&date).unwrap();
        assert_eq!(loaded_data.work_records.len(), 1);
        assert!(loaded_data.work_records.contains_key(&2));
        assert!(!loaded_data.work_records.contains_key(&1));
    }

    #[test]
    fn test_save_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();
        let day_data = DayData::new(date);

        let file_path = storage.get_file_path(&date);
        assert!(!file_path.exists());

        storage.save(&day_data).unwrap();

        assert!(file_path.exists());
    }

    #[test]
    fn test_load_preserves_record_details() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        let mut day_data = DayData::new(date);
        let mut record = create_test_record(1, "Important Task");
        record.description = "This is a description".to_string();
        day_data.add_record(record);

        storage.save(&day_data).unwrap();
        let loaded_data = storage.load(&date).unwrap();

        let loaded_record = loaded_data.work_records.get(&1).unwrap();
        assert_eq!(loaded_record.name, "Important Task");
        assert_eq!(loaded_record.description, "This is a description");
        assert_eq!(loaded_record.total_minutes, 480);
    }

    #[test]
    fn test_multiple_dates() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();

        let date1 = Date::from_calendar_date(2025, time::Month::November, 5).unwrap();
        let date2 = Date::from_calendar_date(2025, time::Month::November, 6).unwrap();

        let mut day1 = DayData::new(date1);
        day1.add_record(create_test_record(1, "Day1Task"));

        let mut day2 = DayData::new(date2);
        day2.add_record(create_test_record(1, "Day2Task"));

        storage.save(&day1).unwrap();
        storage.save(&day2).unwrap();

        let loaded_day1 = storage.load(&date1).unwrap();
        let loaded_day2 = storage.load(&date2).unwrap();

        assert_eq!(loaded_day1.work_records.get(&1).unwrap().name, "Day1Task");
        assert_eq!(loaded_day2.work_records.get(&1).unwrap().name, "Day2Task");
    }

    #[test]
    fn test_json_format_is_pretty() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        let mut day_data = DayData::new(date);
        day_data.add_record(create_test_record(1, "Task"));

        storage.save(&day_data).unwrap();

        let file_path = storage.get_file_path(&date);
        let contents = fs::read_to_string(file_path).unwrap();

        // Pretty JSON should have newlines
        assert!(contents.contains('\n'));
        assert!(contents.contains("  ")); // Indentation
    }
}
