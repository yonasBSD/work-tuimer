# WorkTimer TUI - Task Tracker

This file tracks active development tasks for the WorkTimer project. Tasks are managed by the development assistant and updated throughout work sessions.

## Status Legend
- `[ ]` Pending
- `[~]` In Progress
- `[x]` Completed
- `[-]` Cancelled/Blocked

---

## Current Sprint

### Bug Fixes

#### Issue #2: Double Key Events on WezTerm/Windows
- [x] Update Cargo.toml to use Rust edition 2024
- [x] Import KeyEventKind from crossterm::event
- [x] Filter key events to only handle KeyEventKind::Press
- [ ] Test fix on multiple terminals
- **Context**: On WezTerm/Windows, key events fire for both Press and Release, causing doubled actions
- **Solution**: Filter events using `if let` with `&& key.kind == KeyEventKind::Press` pattern

### High Priority Features (Quick Wins)

#### Daily Summary
- [ ] Design summary layout (total hours, breaks, effective work time)
- [ ] Add calculation methods for daily statistics
- [ ] Integrate summary display in UI header/footer
- [ ] Add task count and average duration metrics

#### Day Navigation
- [x] Add `[` and `]` shortcuts for previous/next day navigation
- [x] Implement date change and data reload logic
- [x] Show current date in UI header with navigation hints
- [ ] Add "go to today" shortcut (e.g., `g` + `t`)

#### Status Bar
- [ ] Design status bar layout (mode, help text, save status)
- [ ] Display current mode and selected field
- [ ] Show contextual help based on current mode
- [ ] Add "modified" indicator for unsaved changes
- [ ] Display undo/redo availability indicators

### Medium Priority Features (High Value)

#### Search/Filter
- [ ] Add search mode with `/` shortcut
- [ ] Implement real-time task name filtering
- [ ] Add search result highlighting
- [ ] Support filtering by duration or time range

#### Color Coding
- [ ] Define color scheme for different task types
- [ ] Add visual distinction for breaks vs work tasks
- [ ] Highlight long-duration tasks (>4 hours)
- [ ] Add color customization support

#### Export Reports
- [ ] Create CSV export module
- [ ] Add export command (e.g., `e` shortcut)
- [ ] Support date range selection for exports
- [ ] Include summary statistics in exports

#### Validation
- [ ] Detect overlapping time entries
- [ ] Show warnings for invalid time ranges
- [ ] Validate end time > start time
- [ ] Suggest corrections before saving

### Low Priority Features (Nice to Have)

#### Auto-complete
- [ ] Track frequency of task names
- [ ] Implement task name suggestions during typing
- [ ] Add quick-select for recent tasks
- [ ] Store task history across sessions

#### Configuration File
- [ ] Define config file structure (TOML)
- [ ] Add support for `~/.config/work-tuimer/config.toml`
- [ ] Allow customizing default times, storage location
- [ ] Support theme and color customization

#### Timer Mode
- [ ] Add active timer display in UI
- [ ] Implement start/stop/pause functionality
- [ ] Auto-update end time while timer is running
- [ ] Add timer notification/alert

#### Task Templates
- [ ] Create template management system
- [ ] Add save/load template shortcuts
- [ ] Allow editing template library
- [ ] Support template with default durations

#### Themes
- [ ] Implement theme switching system
- [ ] Create dark/light theme variants
- [ ] Auto-detect system theme preference
- [ ] Allow custom color palette configuration

### Future/Research Tasks

#### Recurring Tasks
- [ ] Design recurring task data model
- [ ] Add UI for setting recurrence rules
- [ ] Implement auto-generation logic
- [ ] Support modifying individual occurrences

#### Data Import
- [ ] Design import data format specification
- [ ] Add CSV import functionality
- [ ] Support preview before import
- [ ] Handle duplicate/conflict resolution

#### Backup System
- [ ] Implement automatic backup scheduling
- [ ] Add manual backup command
- [ ] Support configurable backup location
- [ ] Add backup restoration functionality

#### Cloud Sync (Optional)
- [ ] Research cloud sync solutions
- [ ] Design encryption for sensitive data
- [ ] Implement conflict resolution
- [ ] Add sync status indicators

---

## Completed Tasks

### New Task Default Times (2025-11-04)
- [x] Update `add_new_record()` to use current time as start time
- [x] Set end time to one hour after current time by default
- **Notes**: New tasks now start at the current time instead of using the end time of the selected record or fixed 9:00-17:00 defaults.

### Day Navigation Feature (2025-11-03)
- [x] Add `current_date` field to AppState to track viewed date
- [x] Implement `navigate_to_previous_day()` and `navigate_to_next_day()` methods
- [x] Add automatic save/load when switching days
- [x] Wire up `[` and `]` keyboard shortcuts for day navigation
- [x] Update header to show current date with navigation hints
- [x] Add day navigation shortcuts to footer help text
- [x] Clear undo/redo history when switching days
- **Notes**: Auto-saves current day data before loading new day. Undo/redo stacks are reset per day.

### Undo/Redo Feature (2025-11-03)
- [x] Create history module for undo/redo state management
- [x] Add history stack to AppState with max depth configuration (50 levels)
- [x] Implement snapshot capture before mutations
- [x] Add undo/redo methods to AppState
- [x] Wire up 'u' and 'r' keyboard shortcuts in main.rs
- [x] Test undo/redo with various operations
- [x] Update README documentation
- **Commit**: a06047c - "Add undo/redo functionality with 'u' and 'r' shortcuts"

---

## Notes
- Tasks are added when starting new features or bug fixes
- Progress is tracked in real-time during implementation
- Completed tasks are moved to the "Completed Tasks" section
- This file is managed by the development assistant
