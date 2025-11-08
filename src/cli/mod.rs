use crate::storage::Storage;
use crate::timer::TimerManager;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::time::Duration;

/// WorkTimer CLI - Automatic time tracking
#[derive(Parser)]
#[command(name = "work-tuimer")]
#[command(about = "Automatic time tracking with CLI commands and TUI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Start a new timer
    Start {
        /// Task name
        task: String,

        /// Optional task description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Stop the running timer
    Stop,

    /// Pause the running timer
    Pause,

    /// Resume the paused timer
    Resume,

    /// Show status of running timer
    Status,
}

/// Handle CLI command execution
pub fn handle_command(cmd: Commands, storage: Storage) -> Result<()> {
    match cmd {
        Commands::Start { task, description } => handle_start(task, description, storage),
        Commands::Stop => handle_stop(storage),
        Commands::Pause => handle_pause(storage),
        Commands::Resume => handle_resume(storage),
        Commands::Status => handle_status(storage),
    }
}

/// Start a new timer
fn handle_start(task: String, description: Option<String>, storage: Storage) -> Result<()> {
    let timer_manager = TimerManager::new(storage);

    // Trim task name
    let task = task.trim().to_string();
    if task.is_empty() {
        return Err(anyhow::anyhow!("Task name cannot be empty"));
    }

    let timer = timer_manager.start(task, description, None, None)?;

    let start_time = format_time(timer.start_time);
    println!("✓ Timer started");
    println!("  Task: {}", timer.task_name);
    if let Some(desc) = &timer.description {
        println!("  Description: {}", desc);
    }
    println!("  Started at: {}", start_time);

    Ok(())
}

/// Stop the running timer
fn handle_stop(storage: Storage) -> Result<()> {
    let timer_manager = TimerManager::new(storage);

    // Load and validate timer exists
    let timer = timer_manager
        .status()?
        .ok_or_else(|| anyhow::anyhow!("No timer is running"))?;

    let elapsed = timer_manager.get_elapsed_duration(&timer);
    let formatted_duration = format_duration(elapsed);

    // Stop the timer and get the work record
    let _record = timer_manager.stop()?;

    let start_time = format_time(timer.start_time);
    let end_time = format_time(timer.end_time.unwrap_or_else(time::OffsetDateTime::now_utc));

    println!("✓ Timer stopped");
    println!("  Task: {}", timer.task_name);
    println!("  Duration: {}", formatted_duration);
    println!("  Started at: {}", start_time);
    println!("  Ended at: {}", end_time);

    Ok(())
}

/// Pause the running timer
fn handle_pause(storage: Storage) -> Result<()> {
    let timer_manager = TimerManager::new(storage);

    let timer = timer_manager
        .status()?
        .ok_or_else(|| anyhow::anyhow!("No timer is running"))?;

    let _paused_timer = timer_manager.pause()?;
    let elapsed = timer_manager.get_elapsed_duration(&timer);
    let formatted_duration = format_duration(elapsed);

    println!("⏸ Timer paused");
    println!("  Task: {}", timer.task_name);
    println!("  Elapsed: {}", formatted_duration);

    Ok(())
}

/// Resume the paused timer
fn handle_resume(storage: Storage) -> Result<()> {
    let timer_manager = TimerManager::new(storage);

    let timer = timer_manager
        .status()?
        .ok_or_else(|| anyhow::anyhow!("No timer is running"))?;

    let _resumed_timer = timer_manager.resume()?;
    let elapsed = timer_manager.get_elapsed_duration(&timer);
    let formatted_duration = format_duration(elapsed);

    println!("▶ Timer resumed");
    println!("  Task: {}", timer.task_name);
    println!("  Total elapsed (before pause): {}", formatted_duration);

    Ok(())
}

/// Show status of running timer
fn handle_status(storage: Storage) -> Result<()> {
    let timer_manager = TimerManager::new(storage);

    match timer_manager.status()? {
        Some(timer) => {
            let elapsed = timer_manager.get_elapsed_duration(&timer);
            let formatted_duration = format_duration(elapsed);
            let start_time = format_time(timer.start_time);

            println!("⏱ Timer Status");
            println!("  Task: {}", timer.task_name);
            println!(
                "  Status: {}",
                match timer.status {
                    crate::timer::TimerStatus::Running => "Running",
                    crate::timer::TimerStatus::Paused => "Paused",
                    crate::timer::TimerStatus::Stopped => "Stopped",
                }
            );
            println!("  Elapsed: {}", formatted_duration);
            println!("  Started at: {}", start_time);
            if let Some(desc) = &timer.description {
                println!("  Description: {}", desc);
            }
        }
        None => {
            println!("No timer is currently running");
        }
    }

    Ok(())
}

/// Format time::OffsetDateTime for display (HH:MM:SS)
fn format_time(dt: time::OffsetDateTime) -> String {
    format!("{:02}:{:02}:{:02}", dt.hour(), dt.minute(), dt.second())
}

/// Format Duration for display (h:mm:ss or mm:ss)
fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}h {:02}m {:02}s", hours, minutes, seconds)
    } else {
        format!("{}m {:02}s", minutes, seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_hours_minutes_seconds() {
        let duration = Duration::from_secs(3661); // 1h 1m 1s
        assert_eq!(format_duration(duration), "1h 01m 01s");
    }

    #[test]
    fn test_format_duration_minutes_seconds() {
        let duration = Duration::from_secs(125); // 2m 5s
        assert_eq!(format_duration(duration), "2m 05s");
    }

    #[test]
    fn test_format_duration_seconds_only() {
        let duration = Duration::from_secs(45);
        assert_eq!(format_duration(duration), "0m 45s");
    }

    #[test]
    fn test_format_duration_zero() {
        let duration = Duration::from_secs(0);
        assert_eq!(format_duration(duration), "0m 00s");
    }

    #[test]
    fn test_format_time() {
        use time::macros::datetime;
        let dt = datetime!(2025-01-15 14:30:45 UTC);
        assert_eq!(format_time(dt), "14:30:45");
    }
}
