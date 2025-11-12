use crate::models::{DayData, WorkRecord};
use crate::timer::TimerState;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use time::Date;

#[derive(Clone)]
pub struct Storage {
    data_dir: PathBuf,
}

/// High-level storage manager that provides transactional operations
/// and automatic file modification tracking
pub struct StorageManager {
    storage: Storage,
    file_modified_times: std::collections::HashMap<Date, Option<SystemTime>>,
}

impl StorageManager {
    /// Create a new StorageManager
    pub fn new() -> Result<Self> {
        Ok(StorageManager {
            storage: Storage::new()?,
            file_modified_times: std::collections::HashMap::new(),
        })
    }

    /// Create a new StorageManager with a custom directory (for testing)
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn new_with_dir(data_dir: PathBuf) -> Result<Self> {
        Ok(StorageManager {
            storage: Storage::new_with_dir(data_dir)?,
            file_modified_times: std::collections::HashMap::new(),
        })
    }

    /// Load day data with automatic file modification tracking
    /// Returns the loaded data and updates internal tracking
    pub fn load_with_tracking(&mut self, date: Date) -> Result<DayData> {
        let data = self.storage.load(&date)?;
        let modified_time = self.storage.get_file_modified_time(&date);
        self.file_modified_times.insert(date, modified_time);
        Ok(data)
    }

    /// Check if file has been modified externally and reload if needed
    /// Returns Some(DayData) if file was modified and reloaded, None if no change
    pub fn check_and_reload(&mut self, date: Date) -> Result<Option<DayData>> {
        let current_modified = self.storage.get_file_modified_time(&date);

        // Check if we've tracked this date before
        let is_tracked = self.file_modified_times.contains_key(&date);

        if !is_tracked {
            // First time checking this date - load it and start tracking
            let data = self.storage.load(&date)?;
            self.file_modified_times.insert(date, current_modified);
            Ok(Some(data))
        } else {
            let last_known = self.file_modified_times.get(&date).copied().flatten();

            // If modification times differ, reload the file
            if current_modified != last_known {
                let data = self.storage.load(&date)?;
                self.file_modified_times.insert(date, current_modified);
                Ok(Some(data))
            } else {
                Ok(None)
            }
        }
    }

    /// Add a new work record (transactional: load → add → save → track)
    #[allow(dead_code)]
    pub fn add_record(&mut self, date: Date, record: WorkRecord) -> Result<()> {
        let mut day_data = self.storage.load(&date)?;
        day_data.add_record(record);
        self.storage.save(&day_data)?;

        // Update tracking after successful save
        let modified_time = self.storage.get_file_modified_time(&date);
        self.file_modified_times.insert(date, modified_time);

        Ok(())
    }

    /// Update an existing work record (transactional: load → update → save → track)
    #[allow(dead_code)]
    pub fn update_record(&mut self, date: Date, record: WorkRecord) -> Result<()> {
        let mut day_data = self.storage.load(&date)?;

        // Update the record (will replace if ID exists)
        day_data.add_record(record);

        self.storage.save(&day_data)?;

        // Update tracking after successful save
        let modified_time = self.storage.get_file_modified_time(&date);
        self.file_modified_times.insert(date, modified_time);

        Ok(())
    }

    /// Remove a work record by ID (transactional: load → remove → save → track)
    /// Returns the removed record if found
    #[allow(dead_code)]
    pub fn remove_record(&mut self, date: Date, id: u32) -> Result<WorkRecord> {
        let mut day_data = self.storage.load(&date)?;

        let record = day_data
            .work_records
            .remove(&id)
            .context(format!("Record with ID {} not found", id))?;

        self.storage.save(&day_data)?;

        // Update tracking after successful save
        let modified_time = self.storage.get_file_modified_time(&date);
        self.file_modified_times.insert(date, modified_time);

        Ok(record)
    }

    /// Save day data and update tracking
    pub fn save(&mut self, day_data: &DayData) -> Result<()> {
        self.storage.save(day_data)?;

        // Update tracking after successful save
        let modified_time = self.storage.get_file_modified_time(&day_data.date);
        self.file_modified_times
            .insert(day_data.date, modified_time);

        Ok(())
    }

    /// Get the last known modification time for a date
    pub fn get_last_modified(&self, date: &Date) -> Option<SystemTime> {
        self.file_modified_times.get(date).copied().flatten()
    }

    /// Pass-through methods for timer operations (these don't need tracking)
    #[allow(dead_code)]
    pub fn save_active_timer(&self, timer: &TimerState) -> Result<()> {
        self.storage.save_active_timer(timer)
    }

    pub fn load_active_timer(&self) -> Result<Option<TimerState>> {
        self.storage.load_active_timer()
    }

    #[allow(dead_code)]
    pub fn clear_active_timer(&self) -> Result<()> {
        self.storage.clear_active_timer()
    }

    /// Create a TimerManager using the internal storage
    /// This allows timer operations while keeping storage abstraction
    fn create_timer_manager(&self) -> crate::timer::TimerManager {
        // Clone the storage for timer operations
        // This is safe because timer operations are independent
        crate::timer::TimerManager::new(self.storage.clone())
    }

    /// Start a new timer with the given task name and optional description
    pub fn start_timer(
        &self,
        task_name: String,
        description: Option<String>,
        source_record_id: Option<u32>,
        source_record_date: Option<time::Date>,
    ) -> Result<TimerState> {
        let timer_manager = self.create_timer_manager();
        timer_manager.start(task_name, description, source_record_id, source_record_date)
    }

    /// Stop the active timer and return the work record
    pub fn stop_timer(&self) -> Result<crate::models::WorkRecord> {
        let timer_manager = self.create_timer_manager();
        timer_manager.stop()
    }

    /// Pause the active timer
    pub fn pause_timer(&self) -> Result<TimerState> {
        let timer_manager = self.create_timer_manager();
        timer_manager.pause()
    }

    /// Resume a paused timer
    pub fn resume_timer(&self) -> Result<TimerState> {
        let timer_manager = self.create_timer_manager();
        timer_manager.resume()
    }

    /// Get elapsed duration for a timer
    #[allow(dead_code)]
    pub fn get_timer_elapsed(&self, timer: &TimerState) -> std::time::Duration {
        let timer_manager = self.create_timer_manager();
        timer_manager.get_elapsed_duration(timer)
    }
}

impl Storage {
    pub fn new() -> Result<Self> {
        let data_dir = Self::get_data_directory()?;
        fs::create_dir_all(&data_dir).context("Failed to create data directory")?;

        Ok(Storage { data_dir })
    }

    /// Create a new Storage with a custom directory (for testing)
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn new_with_dir(data_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&data_dir).context("Failed to create data directory")?;
        Ok(Storage { data_dir })
    }

    fn get_data_directory() -> Result<PathBuf> {
        // Primary: Use system data directory (~/.local/share on Linux, ~/Library/Application Support on macOS)
        if let Some(data_dir) = dirs::data_local_dir() {
            let app_dir = data_dir.join("work-tuimer");
            if fs::create_dir_all(&app_dir).is_ok() {
                return Ok(app_dir);
            }
        }

        // Fallback: Use ./data for development/testing if system location fails
        let local_data = PathBuf::from("./data");
        if fs::create_dir_all(&local_data).is_ok() {
            return Ok(local_data);
        }

        anyhow::bail!("Failed to create data directory in system location or ./data")
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

    /// Get the modification time of a day data file
    /// Returns None if the file doesn't exist
    pub fn get_file_modified_time(&self, date: &Date) -> Option<std::time::SystemTime> {
        let path = self.get_file_path(date);
        if path.exists() {
            fs::metadata(&path).ok().and_then(|m| m.modified().ok())
        } else {
            None
        }
    }

    /// Get the path to the running timer file
    fn get_timer_file_path(&self) -> PathBuf {
        self.data_dir.join("running_timer.json")
    }

    /// Save an active timer to running_timer.json
    pub fn save_active_timer(&self, timer: &TimerState) -> Result<()> {
        let path = self.get_timer_file_path();
        let json = serde_json::to_string_pretty(timer).context("Failed to serialize timer")?;
        fs::write(&path, json).context(format!("Failed to write timer file: {:?}", path))?;
        Ok(())
    }

    /// Load the active timer from running_timer.json
    ///
    /// Returns None if no timer file exists (no active timer)
    pub fn load_active_timer(&self) -> Result<Option<TimerState>> {
        let path = self.get_timer_file_path();

        if !path.exists() {
            return Ok(None);
        }

        let contents =
            fs::read_to_string(&path).context(format!("Failed to read timer file: {:?}", path))?;
        let timer: TimerState =
            serde_json::from_str(&contents).context("Failed to parse timer JSON")?;

        Ok(Some(timer))
    }

    /// Clear the active timer by deleting running_timer.json
    pub fn clear_active_timer(&self) -> Result<()> {
        let path = self.get_timer_file_path();

        if path.exists() {
            fs::remove_file(&path).context(format!("Failed to delete timer file: {:?}", path))?;
        }

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

    #[test]
    fn test_load_active_timer_returns_none_when_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();

        let result = storage.load_active_timer().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_save_and_load_active_timer() {
        use crate::timer::{TimerState, TimerStatus};
        use time::OffsetDateTime;

        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();

        let now = OffsetDateTime::now_utc();
        let timer = TimerState {
            id: None,
            task_name: "Work".to_string(),
            description: Some("Important".to_string()),
            start_time: now,
            end_time: None,
            date: now.date(),
            status: TimerStatus::Running,
            paused_duration_secs: 0,
            paused_at: None,
            created_at: now,
            updated_at: now,
            source_record_id: None,
            source_record_date: None,
        };

        // Save
        storage.save_active_timer(&timer).unwrap();

        // Load
        let loaded = storage.load_active_timer().unwrap();
        assert!(loaded.is_some());

        let loaded_timer = loaded.unwrap();
        assert_eq!(loaded_timer.task_name, "Work");
        assert_eq!(loaded_timer.description, Some("Important".to_string()));
        assert_eq!(loaded_timer.status, TimerStatus::Running);
    }

    #[test]
    fn test_clear_active_timer() {
        use crate::timer::{TimerState, TimerStatus};
        use time::OffsetDateTime;

        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();

        let now = OffsetDateTime::now_utc();
        let timer = TimerState {
            id: None,
            task_name: "Work".to_string(),
            description: None,
            start_time: now,
            end_time: None,
            date: now.date(),
            status: TimerStatus::Running,
            paused_duration_secs: 0,
            paused_at: None,
            created_at: now,
            updated_at: now,
            source_record_id: None,
            source_record_date: None,
        };

        // Save
        storage.save_active_timer(&timer).unwrap();
        assert!(storage.load_active_timer().unwrap().is_some());

        // Clear
        storage.clear_active_timer().unwrap();

        // Verify it's gone
        assert!(storage.load_active_timer().unwrap().is_none());
    }

    #[test]
    fn test_clear_active_timer_when_none_exists() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();

        // Should not error even if file doesn't exist
        let result = storage.clear_active_timer();
        assert!(result.is_ok());
    }

    // StorageManager tests
    #[test]
    fn test_storage_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_storage_manager_load_with_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        let result = manager.load_with_tracking(date);
        assert!(result.is_ok());

        // Should have tracking info now
        assert!(manager.file_modified_times.contains_key(&date));
    }

    #[test]
    fn test_storage_manager_add_record_transactional() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        let record = create_test_record(1, "Test Task");

        // Add record (should load, add, save, track)
        let result = manager.add_record(date, record);
        assert!(result.is_ok());

        // Verify it was saved
        let day_data = manager.load_with_tracking(date).unwrap();
        assert_eq!(day_data.work_records.len(), 1);
        assert!(day_data.work_records.contains_key(&1));

        // Verify tracking was updated
        assert!(manager.get_last_modified(&date).is_some());
    }

    #[test]
    fn test_storage_manager_update_record_transactional() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        // Add initial record
        let record1 = create_test_record(1, "Original Task");
        manager.add_record(date, record1).unwrap();

        // Update the record
        let record2 = create_test_record(1, "Updated Task");
        let result = manager.update_record(date, record2);
        assert!(result.is_ok());

        // Verify update
        let day_data = manager.load_with_tracking(date).unwrap();
        assert_eq!(day_data.work_records.len(), 1);
        assert_eq!(day_data.work_records.get(&1).unwrap().name, "Updated Task");
    }

    #[test]
    fn test_storage_manager_remove_record_transactional() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        // Add record
        let record = create_test_record(1, "To Be Removed");
        manager.add_record(date, record).unwrap();

        // Remove it
        let result = manager.remove_record(date, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "To Be Removed");

        // Verify it's gone
        let day_data = manager.load_with_tracking(date).unwrap();
        assert_eq!(day_data.work_records.len(), 0);
    }

    #[test]
    fn test_storage_manager_remove_nonexistent_record_fails() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        // Try to remove record that doesn't exist
        let result = manager.remove_record(date, 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_storage_manager_check_and_reload_no_change() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        // Load initially
        manager.load_with_tracking(date).unwrap();

        // Check for reload (no external changes)
        let result = manager.check_and_reload(date);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // No reload needed
    }

    #[test]
    fn test_storage_manager_check_and_reload_with_external_change() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        // Load initially (empty)
        manager.load_with_tracking(date).unwrap();

        // Simulate external change by using a different storage instance
        let mut external_manager =
            StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let record = create_test_record(1, "External Change");
        external_manager.add_record(date, record).unwrap();

        // Check for reload - should detect change
        let result = manager.check_and_reload(date);
        assert!(result.is_ok());
        let reloaded_data = result.unwrap();
        assert!(reloaded_data.is_some()); // Reload happened
        assert_eq!(reloaded_data.unwrap().work_records.len(), 1);
    }

    #[test]
    fn test_storage_manager_save_updates_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        let mut day_data = DayData::new(date);
        day_data.add_record(create_test_record(1, "Task"));

        // Save should update tracking
        manager.save(&day_data).unwrap();

        assert!(manager.get_last_modified(&date).is_some());
    }

    // Additional Storage tests
    #[test]
    fn test_get_file_modified_time_returns_none_for_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        let modified_time = storage.get_file_modified_time(&date);
        assert!(modified_time.is_none());
    }

    #[test]
    fn test_get_file_modified_time_returns_some_after_save() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();
        let day_data = DayData::new(date);

        // Before save - no modification time
        assert!(storage.get_file_modified_time(&date).is_none());

        // After save - should have modification time
        storage.save(&day_data).unwrap();
        assert!(storage.get_file_modified_time(&date).is_some());
    }

    #[test]
    fn test_get_timer_file_path_format() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();

        let timer_path = storage.get_timer_file_path();
        assert_eq!(timer_path.file_name().unwrap(), "running_timer.json");
    }

    #[test]
    fn test_file_path_format_with_single_digit_month_and_day() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = Date::from_calendar_date(2025, time::Month::January, 5).unwrap();

        let file_path = storage.get_file_path(&date);
        assert_eq!(file_path.file_name().unwrap(), "2025-01-05.json");
    }

    #[test]
    fn test_file_path_format_with_december() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = Date::from_calendar_date(2025, time::Month::December, 31).unwrap();

        let file_path = storage.get_file_path(&date);
        assert_eq!(file_path.file_name().unwrap(), "2025-12-31.json");
    }

    #[test]
    fn test_save_and_load_timer_with_paused_status() {
        use crate::timer::{TimerState, TimerStatus};
        use time::OffsetDateTime;

        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();

        let now = OffsetDateTime::now_utc();
        let timer = TimerState {
            id: None,
            task_name: "Paused Work".to_string(),
            description: None,
            start_time: now,
            end_time: None,
            date: now.date(),
            status: TimerStatus::Paused,
            paused_duration_secs: 120,
            paused_at: Some(now),
            created_at: now,
            updated_at: now,
            source_record_id: None,
            source_record_date: None,
        };

        storage.save_active_timer(&timer).unwrap();
        let loaded = storage.load_active_timer().unwrap().unwrap();

        assert_eq!(loaded.status, TimerStatus::Paused);
        assert_eq!(loaded.paused_duration_secs, 120);
        assert!(loaded.paused_at.is_some());
    }

    #[test]
    fn test_save_and_load_timer_with_source_record() {
        use crate::timer::{TimerState, TimerStatus};
        use time::OffsetDateTime;

        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();

        let now = OffsetDateTime::now_utc();
        let source_date = Date::from_calendar_date(2025, time::Month::November, 5).unwrap();
        let timer = TimerState {
            id: None,
            task_name: "Continued Work".to_string(),
            description: Some("From record 5".to_string()),
            start_time: now,
            end_time: None,
            date: now.date(),
            status: TimerStatus::Running,
            paused_duration_secs: 0,
            paused_at: None,
            created_at: now,
            updated_at: now,
            source_record_id: Some(5),
            source_record_date: Some(source_date),
        };

        storage.save_active_timer(&timer).unwrap();
        let loaded = storage.load_active_timer().unwrap().unwrap();

        assert_eq!(loaded.source_record_id, Some(5));
        assert_eq!(loaded.source_record_date, Some(source_date));
    }

    #[test]
    fn test_save_multiple_records_preserves_order() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        let mut day_data = DayData::new(date);
        for i in 1..=5 {
            day_data.add_record(create_test_record(i, &format!("Task{}", i)));
        }

        storage.save(&day_data).unwrap();
        let loaded = storage.load(&date).unwrap();

        assert_eq!(loaded.work_records.len(), 5);
        assert_eq!(loaded.last_id, 5);
        for i in 1..=5 {
            assert!(loaded.work_records.contains_key(&i));
        }
    }

    // Additional StorageManager tests
    #[test]
    fn test_storage_manager_timer_passthrough_save_and_load() {
        use crate::timer::{TimerState, TimerStatus};
        use time::OffsetDateTime;

        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();

        let now = OffsetDateTime::now_utc();
        let timer = TimerState {
            id: None,
            task_name: "Test".to_string(),
            description: None,
            start_time: now,
            end_time: None,
            date: now.date(),
            status: TimerStatus::Running,
            paused_duration_secs: 0,
            paused_at: None,
            created_at: now,
            updated_at: now,
            source_record_id: None,
            source_record_date: None,
        };

        // Test passthrough methods
        manager.save_active_timer(&timer).unwrap();
        let loaded = manager.load_active_timer().unwrap();
        assert!(loaded.is_some());

        manager.clear_active_timer().unwrap();
        let cleared = manager.load_active_timer().unwrap();
        assert!(cleared.is_none());
    }

    #[test]
    fn test_storage_manager_tracks_multiple_dates() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();

        let date1 = Date::from_calendar_date(2025, time::Month::November, 1).unwrap();
        let date2 = Date::from_calendar_date(2025, time::Month::November, 2).unwrap();
        let date3 = Date::from_calendar_date(2025, time::Month::November, 3).unwrap();

        // Load multiple dates
        manager.load_with_tracking(date1).unwrap();
        manager.load_with_tracking(date2).unwrap();
        manager.load_with_tracking(date3).unwrap();

        // All should be tracked
        assert!(manager.file_modified_times.contains_key(&date1));
        assert!(manager.file_modified_times.contains_key(&date2));
        assert!(manager.file_modified_times.contains_key(&date3));
    }

    #[test]
    fn test_storage_manager_get_last_modified_returns_none_for_untracked_date() {
        let temp_dir = TempDir::new().unwrap();
        let manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        // Date not loaded yet - no tracking
        assert!(manager.get_last_modified(&date).is_none());
    }

    #[test]
    fn test_storage_manager_check_and_reload_before_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        // Check without loading first - should load as new
        let result = manager.check_and_reload(date).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_storage_manager_add_multiple_records_incrementally() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        // Add records one by one
        manager
            .add_record(date, create_test_record(1, "Task1"))
            .unwrap();
        manager
            .add_record(date, create_test_record(2, "Task2"))
            .unwrap();
        manager
            .add_record(date, create_test_record(3, "Task3"))
            .unwrap();

        // Verify all are saved
        let day_data = manager.load_with_tracking(date).unwrap();
        assert_eq!(day_data.work_records.len(), 3);
    }

    #[test]
    fn test_storage_manager_update_multiple_times() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        // Add initial record
        manager
            .add_record(date, create_test_record(1, "Version1"))
            .unwrap();

        // Update multiple times
        manager
            .update_record(date, create_test_record(1, "Version2"))
            .unwrap();
        manager
            .update_record(date, create_test_record(1, "Version3"))
            .unwrap();

        // Final version should be loaded
        let day_data = manager.load_with_tracking(date).unwrap();
        assert_eq!(day_data.work_records.get(&1).unwrap().name, "Version3");
    }

    #[test]
    fn test_storage_manager_remove_from_multiple_records() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        // Add multiple records
        manager
            .add_record(date, create_test_record(1, "Task1"))
            .unwrap();
        manager
            .add_record(date, create_test_record(2, "Task2"))
            .unwrap();
        manager
            .add_record(date, create_test_record(3, "Task3"))
            .unwrap();

        // Remove middle one
        let removed = manager.remove_record(date, 2).unwrap();
        assert_eq!(removed.name, "Task2");

        // Verify remaining
        let day_data = manager.load_with_tracking(date).unwrap();
        assert_eq!(day_data.work_records.len(), 2);
        assert!(day_data.work_records.contains_key(&1));
        assert!(!day_data.work_records.contains_key(&2));
        assert!(day_data.work_records.contains_key(&3));
    }

    #[test]
    fn test_storage_manager_save_empty_day_data() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        let day_data = DayData::new(date);
        manager.save(&day_data).unwrap();

        // Should be able to load empty data
        let loaded = manager.load_with_tracking(date).unwrap();
        assert_eq!(loaded.work_records.len(), 0);
        assert_eq!(loaded.last_id, 0);
    }

    #[test]
    fn test_storage_manager_save_overwrites_with_tracking_update() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = StorageManager::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        let date = create_test_date();

        // First save
        let mut day_data1 = DayData::new(date);
        day_data1.add_record(create_test_record(1, "Task1"));
        manager.save(&day_data1).unwrap();
        let first_modified = manager.get_last_modified(&date);

        // Give time for modification time to change
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Second save - overwrite
        let mut day_data2 = DayData::new(date);
        day_data2.add_record(create_test_record(2, "Task2"));
        manager.save(&day_data2).unwrap();
        let second_modified = manager.get_last_modified(&date);

        // Tracking should be updated
        assert_ne!(first_modified, second_modified);

        // Data should be overwritten
        let loaded = manager.load_with_tracking(date).unwrap();
        assert_eq!(loaded.work_records.len(), 1);
        assert!(loaded.work_records.contains_key(&2));
        assert!(!loaded.work_records.contains_key(&1));
    }
}
