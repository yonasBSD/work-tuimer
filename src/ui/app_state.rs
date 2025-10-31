use crate::models::{DayData, WorkRecord};

pub enum AppMode {
    Browse,
    Edit,
    Visual,
}

pub enum EditField {
    Name,
    Start,
    End,
}

pub struct AppState {
    pub day_data: DayData,
    pub mode: AppMode,
    pub selected_index: usize,
    pub edit_field: EditField,
    pub input_buffer: String,
    pub time_cursor: usize,
    pub should_quit: bool,
    pub visual_start: usize,
    pub visual_end: usize,
}

impl AppState {
    pub fn new(day_data: DayData) -> Self {
        AppState {
            day_data,
            mode: AppMode::Browse,
            selected_index: 0,
            edit_field: EditField::Name,
            input_buffer: String::new(),
            time_cursor: 0,
            should_quit: false,
            visual_start: 0,
            visual_end: 0,
        }
    }

    pub fn get_selected_record(&self) -> Option<&WorkRecord> {
        let records = self.day_data.get_sorted_records();
        records.get(self.selected_index).copied()
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
        if matches!(self.mode, AppMode::Visual) {
            self.visual_end = self.selected_index;
        }
    }

    pub fn move_selection_down(&mut self) {
        let record_count = self.day_data.work_records.len();
        if self.selected_index < record_count.saturating_sub(1) {
            self.selected_index += 1;
        }
        if matches!(self.mode, AppMode::Visual) {
            self.visual_end = self.selected_index;
        }
    }

    pub fn enter_edit_mode(&mut self) {
        if let Some(record) = self.get_selected_record() {
            let input_value = match self.edit_field {
                EditField::Name => record.name.clone(),
                EditField::Start => record.start.to_string(),
                EditField::End => record.end.to_string(),
            };
            self.mode = AppMode::Edit;
            self.input_buffer = input_value;
            self.time_cursor = 0;
        }
    }

    pub fn exit_edit_mode(&mut self) {
        self.mode = AppMode::Browse;
        self.input_buffer.clear();
        self.edit_field = EditField::Name;
        self.time_cursor = 0;
    }

    pub fn next_field(&mut self) {
        if let Some(record) = self.get_selected_record() {
            self.edit_field = match self.edit_field {
                EditField::Name => {
                    self.input_buffer = record.start.to_string();
                    self.time_cursor = 0;
                    EditField::Start
                }
                EditField::Start => {
                    self.input_buffer = record.end.to_string();
                    self.time_cursor = 0;
                    EditField::End
                }
                EditField::End => {
                    self.input_buffer = record.name.clone();
                    self.time_cursor = 0;
                    EditField::Name
                }
            };
        }
    }

    pub fn handle_char_input(&mut self, c: char) {
        match self.edit_field {
            EditField::Name => {
                self.input_buffer.push(c);
            }
            EditField::Start | EditField::End => {
                if !c.is_ascii_digit() {
                    return;
                }
                
                if self.input_buffer.len() != 5 {
                    return;
                }
                
                let positions = [0, 1, 3, 4];
                if self.time_cursor >= positions.len() {
                    return;
                }
                
                let pos = positions[self.time_cursor];
                let mut chars: Vec<char> = self.input_buffer.chars().collect();
                chars[pos] = c;
                self.input_buffer = chars.into_iter().collect();
                
                self.time_cursor += 1;
                
                if self.time_cursor >= positions.len() {
                    if self.save_current_field().is_ok() {
                        self.exit_edit_mode();
                    }
                }
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.edit_field {
            EditField::Name => {
                self.input_buffer.pop();
            }
            EditField::Start | EditField::End => {
                if self.time_cursor > 0 {
                    self.time_cursor -= 1;
                }
            }
        }
    }

    fn save_current_field(&mut self) -> Result<(), String> {
        let records = self.day_data.get_sorted_records();
        if let Some(&record) = records.get(self.selected_index) {
            let id = record.id;
            
            if let Some(record_mut) = self.day_data.work_records.get_mut(&id) {
                match self.edit_field {
                    EditField::Name => {
                        if self.input_buffer.trim().is_empty() {
                            return Err("Name cannot be empty".to_string());
                        }
                        record_mut.name = self.input_buffer.trim().to_string();
                    }
                    EditField::Start => {
                        record_mut.start = self.input_buffer.parse()
                            .map_err(|_| "Invalid start time format (use HH:MM)".to_string())?;
                        record_mut.update_duration();
                    }
                    EditField::End => {
                        record_mut.end = self.input_buffer.parse()
                            .map_err(|_| "Invalid end time format (use HH:MM)".to_string())?;
                        record_mut.update_duration();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn save_edit(&mut self) -> Result<(), String> {
        self.save_current_field()?;
        self.exit_edit_mode();
        Ok(())
    }

    pub fn add_new_record(&mut self) {
        use crate::models::{TimePoint, WorkRecord};
        
        let id = self.day_data.next_id();
        let default_start = TimePoint::new(9, 0).unwrap();
        let default_end = TimePoint::new(17, 0).unwrap();
        let record = WorkRecord::new(id, "New Task".to_string(), default_start, default_end);
        
        self.day_data.add_record(record);
        self.selected_index = self.day_data.work_records.len().saturating_sub(1);
    }

    pub fn add_break(&mut self) {
        use crate::models::{TimePoint, WorkRecord};
        
        let id = self.day_data.next_id();
        let default_start = TimePoint::new(12, 0).unwrap();
        let default_end = TimePoint::new(12, 15).unwrap();
        let record = WorkRecord::new(id, "Break".to_string(), default_start, default_end);
        
        self.day_data.add_record(record);
        self.selected_index = self.day_data.work_records.len().saturating_sub(1);
    }

    pub fn delete_selected_record(&mut self) {
        let records = self.day_data.get_sorted_records();
        if let Some(&record) = records.get(self.selected_index) {
            self.day_data.remove_record(record.id);
            
            if self.selected_index >= self.day_data.work_records.len() {
                self.selected_index = self.day_data.work_records.len().saturating_sub(1);
            }
        }
    }

    pub fn move_field_left(&mut self) {
        self.edit_field = match self.edit_field {
            EditField::Name => EditField::End,
            EditField::Start => EditField::Name,
            EditField::End => EditField::Start,
        };
    }

    pub fn move_field_right(&mut self) {
        self.edit_field = match self.edit_field {
            EditField::Name => EditField::Start,
            EditField::Start => EditField::End,
            EditField::End => EditField::Name,
        };
    }

    pub fn set_current_time_on_field(&mut self) {
        use time::{OffsetDateTime, UtcOffset};
        
        let local_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
        let now = OffsetDateTime::now_utc().to_offset(local_offset);
        let current_time = format!("{:02}:{:02}", now.hour(), now.minute());
        
        let records = self.day_data.get_sorted_records();
        if let Some(&record) = records.get(self.selected_index) {
            let id = record.id;
            
            if let Some(record_mut) = self.day_data.work_records.get_mut(&id) {
                match self.edit_field {
                    EditField::Start => {
                        if let Ok(time_point) = current_time.parse() {
                            record_mut.start = time_point;
                            record_mut.update_duration();
                        }
                    }
                    EditField::End => {
                        if let Ok(time_point) = current_time.parse() {
                            record_mut.end = time_point;
                            record_mut.update_duration();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn enter_visual_mode(&mut self) {
        self.mode = AppMode::Visual;
        self.visual_start = self.selected_index;
        self.visual_end = self.selected_index;
    }

    pub fn exit_visual_mode(&mut self) {
        self.mode = AppMode::Browse;
    }

    pub fn is_in_visual_selection(&self, index: usize) -> bool {
        let start = self.visual_start.min(self.visual_end);
        let end = self.visual_start.max(self.visual_end);
        index >= start && index <= end
    }

    pub fn delete_visual_selection(&mut self) {
        let records = self.day_data.get_sorted_records();
        let start = self.visual_start.min(self.visual_end);
        let end = self.visual_start.max(self.visual_end);
        
        let ids_to_delete: Vec<u32> = records
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= start && *i <= end)
            .map(|(_, record)| record.id)
            .collect();
        
        for id in ids_to_delete {
            self.day_data.remove_record(id);
        }
        
        if self.selected_index >= self.day_data.work_records.len() {
            self.selected_index = self.day_data.work_records.len().saturating_sub(1);
        }
        
        self.exit_visual_mode();
    }
}
