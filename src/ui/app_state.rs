use crate::models::{DayData, WorkRecord};

pub enum AppMode {
    Browse,
    Edit,
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
    pub should_quit: bool,
}

impl AppState {
    pub fn new(day_data: DayData) -> Self {
        AppState {
            day_data,
            mode: AppMode::Browse,
            selected_index: 0,
            edit_field: EditField::Name,
            input_buffer: String::new(),
            should_quit: false,
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
    }

    pub fn move_selection_down(&mut self) {
        let record_count = self.day_data.work_records.len();
        if self.selected_index < record_count.saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn enter_edit_mode(&mut self) {
        if let Some(record) = self.get_selected_record() {
            let name = record.name.clone();
            self.mode = AppMode::Edit;
            self.edit_field = EditField::Name;
            self.input_buffer = name;
        }
    }

    pub fn exit_edit_mode(&mut self) {
        self.mode = AppMode::Browse;
        self.input_buffer.clear();
        self.edit_field = EditField::Name;
    }

    pub fn next_field(&mut self) {
        if let Some(record) = self.get_selected_record() {
            self.edit_field = match self.edit_field {
                EditField::Name => {
                    self.input_buffer = record.start.to_string();
                    EditField::Start
                }
                EditField::Start => {
                    self.input_buffer = record.end.to_string();
                    EditField::End
                }
                EditField::End => {
                    self.input_buffer = record.name.clone();
                    EditField::Name
                }
            };
        }
    }

    pub fn handle_char_input(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn handle_backspace(&mut self) {
        self.input_buffer.pop();
    }

    pub fn save_edit(&mut self) -> Result<(), String> {
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
        let record = WorkRecord::new(id, "PRZERWA".to_string(), default_start, default_end);
        
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
        use time::OffsetDateTime;
        
        let now = OffsetDateTime::now_utc();
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
}
