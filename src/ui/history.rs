use crate::models::DayData;

const MAX_HISTORY_DEPTH: usize = 50;

#[derive(Debug, Default)]
pub struct History {
    undo_stack: Vec<DayData>,
    redo_stack: Vec<DayData>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, state: DayData) {
        if self.undo_stack.len() >= MAX_HISTORY_DEPTH {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(state);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current_state: DayData) -> Option<DayData> {
        if let Some(previous_state) = self.undo_stack.pop() {
            self.redo_stack.push(current_state);
            Some(previous_state)
        } else {
            None
        }
    }

    pub fn redo(&mut self, current_state: DayData) -> Option<DayData> {
        if let Some(next_state) = self.redo_stack.pop() {
            self.undo_stack.push(current_state);
            Some(next_state)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TimePoint, WorkRecord};
    use time::Date;

    fn create_test_date() -> Date {
        Date::from_calendar_date(2025, time::Month::November, 6).unwrap()
    }

    fn create_day_with_record(id: u32, name: &str) -> DayData {
        let mut day = DayData::new(create_test_date());
        let start = TimePoint::new(9, 0).unwrap();
        let end = TimePoint::new(17, 0).unwrap();
        let record = WorkRecord::new(id, name.to_string(), start, end);
        day.add_record(record);
        day
    }

    #[test]
    fn test_new_history() {
        let history = History::new();
        assert_eq!(history.undo_stack.len(), 0);
        assert_eq!(history.redo_stack.len(), 0);
    }

    #[test]
    fn test_push_single_state() {
        let mut history = History::new();
        let day = create_day_with_record(1, "Task1");

        history.push(day);

        assert_eq!(history.undo_stack.len(), 1);
        assert_eq!(history.redo_stack.len(), 0);
    }

    #[test]
    fn test_push_multiple_states() {
        let mut history = History::new();

        history.push(create_day_with_record(1, "Task1"));
        history.push(create_day_with_record(2, "Task2"));
        history.push(create_day_with_record(3, "Task3"));

        assert_eq!(history.undo_stack.len(), 3);
        assert_eq!(history.redo_stack.len(), 0);
    }

    #[test]
    fn test_push_clears_redo_stack() {
        let mut history = History::new();
        let day1 = create_day_with_record(1, "Task1");
        let day2 = create_day_with_record(2, "Task2");
        let day3 = create_day_with_record(3, "Task3");

        history.push(day1.clone());
        history.push(day2.clone());

        // Undo once to populate redo stack
        history.undo(day3.clone());
        assert_eq!(history.redo_stack.len(), 1);

        // Push new state should clear redo stack
        history.push(create_day_with_record(4, "Task4"));
        assert_eq!(history.redo_stack.len(), 0);
    }

    #[test]
    fn test_push_respects_max_depth() {
        let mut history = History::new();

        // Push more than MAX_HISTORY_DEPTH states
        for i in 0..55 {
            history.push(create_day_with_record(i, &format!("Task{}", i)));
        }

        // Should not exceed MAX_HISTORY_DEPTH
        assert_eq!(history.undo_stack.len(), MAX_HISTORY_DEPTH);
    }

    #[test]
    fn test_push_max_depth_removes_oldest() {
        let mut history = History::new();

        // Push MAX_HISTORY_DEPTH states
        for i in 0..MAX_HISTORY_DEPTH {
            history.push(create_day_with_record(i as u32, &format!("Task{}", i)));
        }

        // Push one more
        history.push(create_day_with_record(999, "NewTask"));

        assert_eq!(history.undo_stack.len(), MAX_HISTORY_DEPTH);
        // The last one should be the newest
        assert_eq!(
            history
                .undo_stack
                .last()
                .unwrap()
                .work_records
                .get(&999)
                .unwrap()
                .name,
            "NewTask"
        );
    }

    #[test]
    fn test_undo_empty_history() {
        let mut history = History::new();
        let current = create_day_with_record(1, "Current");

        let result = history.undo(current);

        assert!(result.is_none());
        assert_eq!(history.undo_stack.len(), 0);
        assert_eq!(history.redo_stack.len(), 0);
    }

    #[test]
    fn test_undo_single_state() {
        let mut history = History::new();
        let day1 = create_day_with_record(1, "Task1");
        let day2 = create_day_with_record(2, "Task2");

        history.push(day1.clone());

        let result = history.undo(day2.clone());

        assert!(result.is_some());
        let previous = result.unwrap();
        assert_eq!(previous.work_records.get(&1).unwrap().name, "Task1");
        assert_eq!(history.undo_stack.len(), 0);
        assert_eq!(history.redo_stack.len(), 1);
    }

    #[test]
    fn test_undo_multiple_times() {
        let mut history = History::new();
        let day1 = create_day_with_record(1, "Task1");
        let day2 = create_day_with_record(2, "Task2");
        let day3 = create_day_with_record(3, "Task3");

        history.push(day1.clone());
        history.push(day2.clone());

        // First undo
        let result1 = history.undo(day3.clone());
        assert!(result1.is_some());
        assert_eq!(result1.unwrap().work_records.get(&2).unwrap().name, "Task2");

        // Second undo
        let result2 = history.undo(day2.clone());
        assert!(result2.is_some());
        assert_eq!(result2.unwrap().work_records.get(&1).unwrap().name, "Task1");

        // Third undo should return None
        let result3 = history.undo(day1.clone());
        assert!(result3.is_none());
    }

    #[test]
    fn test_undo_moves_to_redo_stack() {
        let mut history = History::new();
        let day1 = create_day_with_record(1, "Task1");
        let day2 = create_day_with_record(2, "Task2");

        history.push(day1.clone());
        history.undo(day2.clone());

        assert_eq!(history.redo_stack.len(), 1);
        assert_eq!(
            history
                .redo_stack
                .last()
                .unwrap()
                .work_records
                .get(&2)
                .unwrap()
                .name,
            "Task2"
        );
    }

    #[test]
    fn test_redo_empty_redo_stack() {
        let mut history = History::new();
        let current = create_day_with_record(1, "Current");

        let result = history.redo(current);

        assert!(result.is_none());
    }

    #[test]
    fn test_redo_after_undo() {
        let mut history = History::new();
        let day1 = create_day_with_record(1, "Task1");
        let day2 = create_day_with_record(2, "Task2");
        let day3 = create_day_with_record(3, "Task3");

        history.push(day1.clone());
        history.push(day2.clone());

        // Undo
        history.undo(day3.clone());

        // Redo
        let result = history.redo(day2.clone());
        assert!(result.is_some());
        assert_eq!(result.unwrap().work_records.get(&3).unwrap().name, "Task3");
    }

    #[test]
    fn test_undo_redo_cycle() {
        let mut history = History::new();
        let day1 = create_day_with_record(1, "Task1");
        let day2 = create_day_with_record(2, "Task2");
        let day3 = create_day_with_record(3, "Task3");

        history.push(day1.clone());
        history.push(day2.clone());

        // Undo twice
        let undo1 = history.undo(day3.clone()).unwrap();
        let undo2 = history.undo(undo1.clone()).unwrap();

        assert_eq!(undo2.work_records.get(&1).unwrap().name, "Task1");

        // Redo twice
        let redo1 = history.redo(undo2.clone()).unwrap();
        let redo2 = history.redo(redo1.clone()).unwrap();

        assert_eq!(redo2.work_records.get(&3).unwrap().name, "Task3");
    }

    #[test]
    fn test_redo_moves_to_undo_stack() {
        let mut history = History::new();
        let day1 = create_day_with_record(1, "Task1");
        let day2 = create_day_with_record(2, "Task2");

        history.push(day1.clone());
        history.undo(day2.clone());

        assert_eq!(history.undo_stack.len(), 0);

        history.redo(day1.clone());

        assert_eq!(history.undo_stack.len(), 1);
        assert_eq!(
            history
                .undo_stack
                .last()
                .unwrap()
                .work_records
                .get(&1)
                .unwrap()
                .name,
            "Task1"
        );
    }

    #[test]
    fn test_multiple_undos_and_redos() {
        let mut history = History::new();

        for i in 1..=5 {
            history.push(create_day_with_record(i, &format!("Task{}", i)));
        }

        let mut current = create_day_with_record(6, "Task6");

        // Undo 3 times
        current = history.undo(current).unwrap();
        current = history.undo(current).unwrap();
        current = history.undo(current).unwrap();

        assert_eq!(history.undo_stack.len(), 2);
        assert_eq!(history.redo_stack.len(), 3);

        // Redo 2 times
        current = history.redo(current).unwrap();
        let _final_state = history.redo(current).unwrap();

        assert_eq!(history.undo_stack.len(), 4);
        assert_eq!(history.redo_stack.len(), 1);
    }
}
