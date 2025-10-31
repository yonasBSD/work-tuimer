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
        fs::create_dir_all(&data_dir)
            .context("Failed to create data directory")?;
        
        Ok(Storage { data_dir })
    }

    fn get_data_directory() -> Result<PathBuf> {
        if let Some(data_dir) = dirs::data_local_dir() {
            let app_dir = data_dir.join("worktimer");
            if fs::create_dir_all(&app_dir).is_ok() {
                return Ok(app_dir);
            }
        }

        let fallback = PathBuf::from("./data");
        fs::create_dir_all(&fallback)
            .context("Failed to create fallback data directory")?;
        Ok(fallback)
    }

    fn get_file_path(&self, date: &Date) -> PathBuf {
        self.data_dir.join(format!("{}-{:02}-{:02}.json", date.year(), date.month() as u8, date.day()))
    }

    pub fn load(&self, date: &Date) -> Result<DayData> {
        let path = self.get_file_path(date);

        if !path.exists() {
            return Ok(DayData::new(*date));
        }

        let contents = fs::read_to_string(&path)
            .context(format!("Failed to read file: {:?}", path))?;

        let day_data: DayData = serde_json::from_str(&contents)
            .context("Failed to parse JSON")?;

        Ok(day_data)
    }

    pub fn save(&self, day_data: &DayData) -> Result<()> {
        let path = self.get_file_path(&day_data.date);

        let json = serde_json::to_string_pretty(day_data)
            .context("Failed to serialize data")?;

        fs::write(&path, json)
            .context(format!("Failed to write file: {:?}", path))?;

        Ok(())
    }
}
