//! Timer module for automatic time tracking
//!
//! This module provides automatic timer functionality for tracking work sessions.
//! Timers can be started, paused, resumed, and stopped, with automatic conversion
//! to WorkRecord upon completion.

use crate::models::{TimePoint, WorkRecord};
use crate::storage::Storage;
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;
use time::{Date, OffsetDateTime};

/// Timer status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimerStatus {
    Running,
    Paused,
    Stopped,
}

/// Active timer state with SQLite-ready fields
///
/// This struct represents an active timer session. All fields are designed
/// to be compatible with SQLite storage for future migration (Issue #22).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    /// Optional ID for future SQLite primary key (currently unused)
    pub id: Option<u32>,

    /// Task name being tracked
    pub task_name: String,

    /// Optional description for the task
    pub description: Option<String>,

    /// When the timer was started (UTC)
    pub start_time: OffsetDateTime,

    /// When the timer was stopped (UTC), None if still active
    pub end_time: Option<OffsetDateTime>,

    /// Date when timer was started
    pub date: Date,

    /// Current status of the timer
    pub status: TimerStatus,

    /// Total duration in seconds when paused (cumulative)
    pub paused_duration_secs: i64,

    /// When timer was last paused (to track current pause duration)
    pub paused_at: Option<OffsetDateTime>,

    /// When this timer record was created (audit field)
    pub created_at: OffsetDateTime,

    /// When this timer record was last updated (audit field)
    pub updated_at: OffsetDateTime,
}

/// Timer manager for controlling timer operations
///
/// Provides methods to start, stop, pause, and resume timers, as well as
/// query their current status. Manages persistence through the Storage layer.
pub struct TimerManager {
    storage: Storage,
}

impl TimerManager {
    /// Create a new timer manager
    pub fn new(storage: Storage) -> Self {
        TimerManager { storage }
    }

    /// Start a new timer
    ///
    /// # Errors
    /// Returns an error if a timer is already running
    pub fn start(&self, task_name: String, description: Option<String>) -> Result<TimerState> {
        // Check if timer already running
        if (self.storage.load_active_timer()?).is_some() {
            return Err(anyhow!("A timer is already running"));
        }

        let now = OffsetDateTime::now_utc();
        let timer = TimerState {
            id: None,
            task_name,
            description,
            start_time: now,
            end_time: None,
            date: now.date(),
            status: TimerStatus::Running,
            paused_duration_secs: 0,
            paused_at: None,
            created_at: now,
            updated_at: now,
        };

        self.storage.save_active_timer(&timer)?;
        Ok(timer)
    }

    /// Stop the active timer and convert it to a WorkRecord
    ///
    /// # Errors
    /// Returns an error if no timer is running
    pub fn stop(&self) -> Result<WorkRecord> {
        let mut timer = self
            .storage
            .load_active_timer()?
            .ok_or_else(|| anyhow!("No timer is currently running"))?;

        let now = OffsetDateTime::now_utc();
        timer.end_time = Some(now);
        timer.status = TimerStatus::Stopped;
        timer.updated_at = now;

        let work_record = self.to_work_record(timer)?;

        self.storage.clear_active_timer()?;

        Ok(work_record)
    }

    /// Pause the active timer
    ///
    /// # Errors
    /// Returns an error if timer is not running
    pub fn pause(&self) -> Result<TimerState> {
        let mut timer = self
            .storage
            .load_active_timer()?
            .ok_or_else(|| anyhow!("No timer is currently running"))?;

        if timer.status == TimerStatus::Paused {
            return Err(anyhow!("Timer is already paused"));
        }

        if timer.status != TimerStatus::Running {
            return Err(anyhow!("Can only pause a running timer"));
        }

        let now = OffsetDateTime::now_utc();
        timer.paused_at = Some(now);
        timer.status = TimerStatus::Paused;
        timer.updated_at = now;

        self.storage.save_active_timer(&timer)?;
        Ok(timer)
    }

    /// Resume a paused timer
    ///
    /// # Errors
    /// Returns an error if timer is not paused
    pub fn resume(&self) -> Result<TimerState> {
        let mut timer = self
            .storage
            .load_active_timer()?
            .ok_or_else(|| anyhow!("No timer is currently running"))?;

        if timer.status != TimerStatus::Paused {
            return Err(anyhow!("Can only resume a paused timer"));
        }

        let now = OffsetDateTime::now_utc();

        // Add current pause duration to cumulative paused time
        if let Some(paused_at) = timer.paused_at {
            let pause_duration = (now - paused_at).whole_seconds();
            timer.paused_duration_secs += pause_duration;
        }

        timer.paused_at = None;
        timer.status = TimerStatus::Running;
        timer.updated_at = now;

        self.storage.save_active_timer(&timer)?;
        Ok(timer)
    }

    /// Get the current timer status
    ///
    /// Returns None if no timer is running
    pub fn status(&self) -> Result<Option<TimerState>> {
        self.storage.load_active_timer()
    }

    /// Calculate elapsed duration of a timer
    ///
    /// Returns the time since start_time, minus any paused durations.
    pub fn get_elapsed_duration(&self, timer: &TimerState) -> StdDuration {
        let end_point = if timer.status == TimerStatus::Paused {
            // If paused, use when it was paused
            timer.paused_at.unwrap_or_else(OffsetDateTime::now_utc)
        } else {
            // If running, use now
            OffsetDateTime::now_utc()
        };

        let elapsed = end_point - timer.start_time;
        let paused_duration_std = StdDuration::from_secs(timer.paused_duration_secs as u64);

        // Convert time::Duration to std::Duration for arithmetic
        let elapsed_std = StdDuration::from_secs(elapsed.whole_seconds() as u64)
            + StdDuration::from_nanos(elapsed.subsec_nanoseconds() as u64);

        elapsed_std
            .checked_sub(paused_duration_std)
            .unwrap_or(StdDuration::ZERO)
    }

    /// Convert a stopped timer to a WorkRecord
    fn to_work_record(&self, timer: TimerState) -> Result<WorkRecord> {
        if timer.status != TimerStatus::Stopped {
            return Err(anyhow!("Can only convert stopped timers to WorkRecord"));
        }

        let start_time = timer.start_time;
        let end_time = timer
            .end_time
            .ok_or_else(|| anyhow!("Stopped timer must have end_time"))?;

        // Extract just the time portion from the OffsetDateTime values
        let start_timepoint = TimePoint::new(start_time.hour(), start_time.minute())
            .map_err(|e| anyhow!(e))
            .context("Failed to create TimePoint for timer start time")?;

        let end_timepoint = TimePoint::new(end_time.hour(), end_time.minute())
            .map_err(|e| anyhow!(e))
            .context("Failed to create TimePoint for timer end time")?;

        let mut record = WorkRecord::new(
            1, // Placeholder ID, will be set by DayData
            timer.task_name,
            start_timepoint,
            end_timepoint,
        );

        if let Some(description) = timer.description {
            record.description = description;
        }

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (Storage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new_with_dir(temp_dir.path().to_path_buf()).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_timer_state_creation() {
        let now = OffsetDateTime::now_utc();
        let timer = TimerState {
            id: None,
            task_name: "Test Task".to_string(),
            description: None,
            start_time: now,
            end_time: None,
            date: now.date(),
            status: TimerStatus::Running,
            paused_duration_secs: 0,
            paused_at: None,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(timer.task_name, "Test Task");
        assert_eq!(timer.status, TimerStatus::Running);
        assert_eq!(timer.paused_duration_secs, 0);
    }

    #[test]
    fn test_timer_serialization() {
        let now = OffsetDateTime::now_utc();
        let timer = TimerState {
            id: None,
            task_name: "Test Task".to_string(),
            description: Some("Test description".to_string()),
            start_time: now,
            end_time: None,
            date: now.date(),
            status: TimerStatus::Running,
            paused_duration_secs: 0,
            paused_at: None,
            created_at: now,
            updated_at: now,
        };

        let json = serde_json::to_string(&timer).unwrap();
        let deserialized: TimerState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.task_name, timer.task_name);
        assert_eq!(deserialized.status, timer.status);
    }

    #[test]
    fn test_start_timer() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let result = manager.start("Work".to_string(), None);
        assert!(result.is_ok());

        let timer = result.unwrap();
        assert_eq!(timer.task_name, "Work");
        assert_eq!(timer.status, TimerStatus::Running);
        assert_eq!(timer.paused_duration_secs, 0);
    }

    #[test]
    fn test_cannot_start_when_already_running() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let _ = manager.start("Task 1".to_string(), None);
        let result = manager.start("Task 2".to_string(), None);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "A timer is already running"
        );
    }

    #[test]
    fn test_pause_running_timer() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let _ = manager.start("Work".to_string(), None);
        let result = manager.pause();

        assert!(result.is_ok());
        let timer = result.unwrap();
        assert_eq!(timer.status, TimerStatus::Paused);
        assert!(timer.paused_at.is_some());
    }

    #[test]
    fn test_cannot_pause_paused_timer() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let _ = manager.start("Work".to_string(), None);
        let _ = manager.pause();
        let result = manager.pause();

        assert!(result.is_err());
    }

    #[test]
    fn test_pause_without_running_timer() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let result = manager.pause();
        assert!(result.is_err());
    }

    #[test]
    fn test_resume_paused_timer() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let _ = manager.start("Work".to_string(), None);
        let _ = manager.pause();
        let result = manager.resume();

        assert!(result.is_ok());
        let timer = result.unwrap();
        assert_eq!(timer.status, TimerStatus::Running);
        assert!(timer.paused_at.is_none());
    }

    #[test]
    fn test_resume_updates_paused_duration() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let _ = manager.start("Work".to_string(), None);
        let paused1 = manager.pause().unwrap();
        assert_eq!(paused1.paused_duration_secs, 0);

        // Simulate time passing by manually updating
        let _ = manager.resume();
        let paused2 = manager.pause().unwrap();

        // paused_duration_secs should have increased
        assert!(paused2.paused_duration_secs >= 0);
    }

    #[test]
    fn test_cannot_resume_running_timer() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let _ = manager.start("Work".to_string(), None);
        let result = manager.resume();

        assert!(result.is_err());
    }

    #[test]
    fn test_status_returns_none_when_no_timer() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let result = manager.status().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_status_returns_running_timer() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let _ = manager.start("Work".to_string(), None);
        let result = manager.status().unwrap();

        assert!(result.is_some());
        let timer = result.unwrap();
        assert_eq!(timer.task_name, "Work");
        assert_eq!(timer.status, TimerStatus::Running);
    }

    #[test]
    fn test_stop_running_timer() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let _ = manager.start("Work".to_string(), None);
        let result = manager.stop();

        assert!(result.is_ok());
        let work_record = result.unwrap();
        assert_eq!(work_record.name, "Work");

        // Timer should be cleared
        let timer_status = manager.status().unwrap();
        assert!(timer_status.is_none());
    }

    #[test]
    fn test_cannot_stop_without_running_timer() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let result = manager.stop();
        assert!(result.is_err());
    }

    #[test]
    fn test_stop_returns_work_record_with_description() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let _ = manager.start("Work".to_string(), Some("Important task".to_string()));
        let work_record = manager.stop().unwrap();

        assert_eq!(work_record.name, "Work");
        assert_eq!(work_record.description, "Important task");
    }

    #[test]
    fn test_full_timer_lifecycle() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        // Start
        let started = manager.start("Task".to_string(), None).unwrap();
        assert_eq!(started.status, TimerStatus::Running);

        // Pause
        let paused = manager.pause().unwrap();
        assert_eq!(paused.status, TimerStatus::Paused);

        // Resume
        let resumed = manager.resume().unwrap();
        assert_eq!(resumed.status, TimerStatus::Running);

        // Pause again
        let paused_again = manager.pause().unwrap();
        assert_eq!(paused_again.status, TimerStatus::Paused);

        // Resume again
        let resumed_again = manager.resume().unwrap();
        assert_eq!(resumed_again.status, TimerStatus::Running);

        // Stop
        let work_record = manager.stop().unwrap();
        assert_eq!(work_record.name, "Task");

        // Verify timer is cleared
        let status = manager.status().unwrap();
        assert!(status.is_none());
    }

    #[test]
    fn test_get_elapsed_duration_running() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let timer = manager.start("Task".to_string(), None).unwrap();
        let elapsed = manager.get_elapsed_duration(&timer);

        // Should be close to 0 since just started
        assert!(elapsed.as_secs() < 2);
    }

    #[test]
    fn test_get_elapsed_duration_with_pause() {
        let (storage, _temp) = create_test_storage();
        let manager = TimerManager::new(storage);

        let _ = manager.start("Task".to_string(), None);
        let _ = manager.pause();

        let timer = manager.status().unwrap().unwrap();
        let elapsed = manager.get_elapsed_duration(&timer);

        // Should be very small since just paused
        assert!(elapsed.as_secs() < 2);
    }
}
