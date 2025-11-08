# WorkTimer TUI - Task Tracker

This file tracks active development tasks for the WorkTimer project. Tasks are managed by the development assistant and updated throughout work sessions.

## Status Legend
- `[ ]` Pending
- `[~]` In Progress
- `[x]` Completed
- `[-]` Cancelled/Blocked

---

## Current Sprint

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
- [x] Add quick-select for recent tasks (Task Picker - Issue #14)
- [ ] Track frequency of task names
- [ ] Implement task name suggestions during typing
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

### Bug Fix: Timer Stop Creates Duplicate Records + Keybind Change (2025-11-08)
- [x] Change timer stop keybind from `X` to `S` (toggle behavior: starts/stops timer)
- [x] Fix critical bug where stopping timer created new record instead of updating existing one
- [x] Add `source_record_id: Option<u32>` field to `TimerState` struct
- [x] Update `TimerManager::start()` to accept `source_record_id` parameter
- [x] Rewrite `TimerManager::stop()` to update existing record's end time when source_record_id present
- [x] Update TUI `start_timer_for_selected()` to pass record ID when starting timer
- [x] Update CLI timer start to pass `None` (CLI timers create new records as before)
- [x] Update all 20+ test cases to include new parameter
- [x] All 125 tests passing, no clippy warnings
- **Context**: Fixed critical bug where pressing `S` to stop a timer would create a NEW work record instead of updating the selected record's end time. The timer system didn't track which record it was started from.
- **Root Cause**: Timer had no link back to source record, always created new records on stop
- **Solution**: Added optional `source_record_id` field with `#[serde(default)]` for backward compatibility. TUI passes record ID when starting timer from existing record, CLI passes None to create new records.
- **Keybind Change**: Changed stop from `X` to `S` for toggle behavior (S = start/stop). When timer running, S stops it. When no timer, S starts from selected record.
- **Testing**: All 125 tests pass, clippy clean, ready for manual verification
- **Files Modified**: src/timer/mod.rs, src/ui/app_state.rs, src/cli/mod.rs, src/storage/mod.rs, src/main.rs
- **Commits**: 
  - 352de58 - "Change timer stop keybind from X to S with toggle behavior"
  - 14934ef - "Fix timer stop not saving work record to day data"
  - 0857f83 - "Fix timer stop to update existing work record end time instead of creating duplicates"

### Feature: Phase 4 - TUI Timer Integration (2025-11-08)
- [x] Completed 14/14 tasks for full TUI integration
- [x] Timer display bar with status, task name, and elapsed time (H:MM:SS)
- [x] Dynamic layout adjustment (1-line timer bar at top when active)
- [x] Timer keybindings in Browse mode: S (Start), P (Pause/Resume toggle), X (Stop)
- [x] Visual distinction: Active timer rows highlighted in green with ⏱ icon
- [x] Footer help text updated to show timer commands
- [x] All 125 tests passing (120 existing + 5 CLI tests from Phase 3)
- [x] No clippy warnings, properly formatted code
- **Context**: Completed Phase 4 of the 4-phase timer implementation. TUI now shows active timer with full visual feedback and keyboard control. Timer status is persisted across sessions and loads automatically on startup.
- **Design**: Timer bar appears at top (1 line), selected record with active timer shown with green background + ⏱ icon, keybindings follow convention (S=Start, P=Pause, X=Stop, capital letters to avoid conflicts)
- **Testing**: All 125 tests pass, manual verification of keybindings working correctly
- **Files Modified**: src/main.rs (timer load on startup, keybindings), src/ui/app_state.rs (timer methods already implemented in P3), src/ui/render.rs (timer bar, dynamic layout, visual distinction)
- **Commit**: 1093622 - "Phase 4: TUI Timer Integration - Active timer display, keybindings, and visual distinction"

### Feature: Task Picker (Issue #14) (2025-11-06)
- [x] Add `TaskPicker` mode to AppMode enum
- [x] Implement `open_task_picker()` and `close_task_picker()` methods
- [x] Add `get_unique_task_names()` to extract unique tasks from current day
- [x] Add navigation methods: `move_task_picker_up()`, `move_task_picker_down()`
- [x] Implement `select_task_from_picker()` to apply selected task to input buffer
- [x] Add `/` keybind in Edit mode (when editing Name field) to open picker
- [x] Add TaskPicker keyboard handling (↑/↓/k/j for navigation, Enter to select, Esc to cancel)
- [x] Create `render_task_picker()` with centered modal display
- [x] Add task type icons (☕ break, 👥 meeting, 💻 code, 📋 other)
- [x] Update footer help text to show `/: Task Picker` in Edit mode
- [x] All 21 tests passing, no clippy warnings
- [x] Update README.md with Task Picker documentation
- **Context**: Implemented quick task picker feature requested in GitHub Issue #14. When editing a task name, users can press `/` to open a popup showing all unique task names from the current day, allowing quick reuse without retyping.
- **Design**: Mini-picker style (centered popup), day-scoped (current day only), context-aware (only in Name edit field)
- **Files Modified**: src/ui/app_state.rs, src/main.rs, src/ui/render.rs, README.md, TASKS.md
- **Branch**: feature/task-picker
- **Issue**: https://github.com/Kamyil/work-tuimer/issues/14

### Bug Fix: Remove Hardcoded Config Defaults (2025-11-05)
- [x] Remove hardcoded JIRA URL (`mccomprojects.atlassian.net`) from `IntegrationConfig::default()`
- [x] Add `has_integrations()` method to Config to check if any tracker is properly configured
- [x] Hide `T` keybinding in main.rs when `!app.config.has_integrations()`
- [x] Hide ticket badges `[PROJ-123]` in render.rs when no config exists (4 locations)
- [x] Update README.md to clarify integration feature is completely optional
- [x] Fix integration tests to use explicit TOML config instead of relying on defaults
- [x] All 19 tests pass, no clippy warnings
- **Context**: User discovered hardcoded default JIRA URL would cause first-time users to hit wrong instance. Decision: Remove all defaults and hide feature completely when not configured for clean UX.
- **Design Philosophy**: Feature should be invisible until configured. No hardcoded URLs, no visible keybinds, no badges.
- **Testing**: Verified with `cargo test`, `cargo clippy`, and `cargo fmt --check` in nix-shell
- **Commit**: 93d23e4 - "Remove hardcoded config defaults and hide feature when not configured"
- **Files Modified**: src/config/mod.rs, src/main.rs, src/ui/render.rs, src/integrations/mod.rs (tests), README.md

### Bug Fix: Error Modal and Keybinding Fix (2025-11-05)
- [x] Add error modal rendering with red theme and centered display
- [x] Integrate error modal into main render pipeline (shows on top of everything)
- [x] Add `clear_error()` method to AppState for auto-dismissing errors
- [x] Change keybinding from `Shift+J` to `T` (capital T) for better reliability
- [x] Update help text in footer from "Shift+J: Ticket" to "T: Ticket"
- [x] Update README.md with new `T` keybinding
- **Context**: Fixed issue where `Shift+J` keybinding wasn't working - modifier key detection was unreliable across terminals. Changed to simple `T` key which works perfectly. Also added prominent error modal (red, centered overlay) to make debugging easier.
- **Root Cause**: Crossterm's `KeyModifiers::SHIFT` detection with `Char('j')` wasn't reliable on macOS/terminal combinations
- **Solution**: Use `KeyCode::Char('T')` which naturally requires Shift to type, avoiding modifier detection issues
- **Testing**: Manually tested - `T` key successfully opens tickets in browser, error modal displays for invalid cases
- **Files Modified**: src/ui/render.rs (error modal), src/main.rs (keybinding), README.md

### Feature: JIRA/Linear Integration - Phase 1 Auto-detection (2025-11-05)
- [x] Restore JIRA/Linear ticket functionality with auto-detection from task names
- [x] Add `open_ticket_in_browser()` method to AppState with platform-specific browser commands
- [x] Add Shift+J keybinding to open tickets in browser from Browse mode
- [x] Update UI to display ticket badges (`📋 Task Name [PROJ-123]`) in all edit states
- [x] Update footer help text to show "Shift+J: Ticket" in Browse mode
- [x] Extract ticket IDs using regex pattern `[A-Z]{2,10}-\d+` from task names
- [x] Detect tracker type (JIRA/Linear) using existing config patterns
- [x] Build URLs and open browser using `std::process::Command` (macOS: `open`, Windows: `cmd /C start`, Linux: `xdg-open`)
- [x] Update README.md with ticket detection documentation
- [x] Update TASKS.md to document Phase 1 completion
- **Context**: Implemented Phase 1 (Auto-detection MVP) of JIRA/Linear integration. Tickets are detected automatically from task names at runtime without data model changes. Users can include ticket IDs like "WL-1: Task" or "PROJ-123: Feature" and press Shift+J to open in browser.
- **Design**: Task-name level detection (not per-record), no persistent storage, config-based JIRA vs Linear detection
- **Testing**: All 19 tests pass via `cargo test` in nix-shell
- **Future**: Phase 2 (manual mapping via `T` key), Phase 3 (JIRA worklog export via `W` command)
- **Files Modified**: src/ui/app_state.rs, src/main.rs, src/ui/render.rs, README.md, TASKS.md

### Feature: JIRA/Linear Integration Refactoring (2025-11-05)
- [x] Revert ticket field from per-record to design for task-level association
- [x] Remove `ticket: Option<String>` field from WorkRecord struct
- [x] Update EditField enum to use Description instead of Ticket
- [x] Fix save_current_field() to save description instead of ticket
- [x] Remove open_ticket_in_browser() method from AppState
- [x] Remove Shift+J keybinding from main.rs
- [x] Update render.rs: Replace ticket display with description display
- [x] Update table header from "🎫 Ticket" to "📄 Description"
- [x] Update footer help text to remove Shift+J reference
- [x] Run cargo check and cargo test - all 19 tests pass
- **Context**: Completed refactoring of the JIRA/Linear integration. The initial design incorrectly placed ticket tracking at the per-record level, but users need to associate tickets with task names instead (since the same ticket is worked on multiple times per day). Design decision: Move ticket association to task-name/summary level for future implementation.
- **Testing**: All 19 unit tests pass (config serialization, URL building, ticket pattern matching)
- **Build**: Successfully compiles via `nix-shell --run 'cargo build'`
- **Files Modified**: src/models/work_record.rs, src/ui/app_state.rs, src/ui/render.rs, src/main.rs

### Feature: JIRA/Linear Integration (2025-11-05)
- [x] Add dependencies: `toml`, `regex` crates (removed `open` crate, using `std::process::Command` instead)
- [x] Create config system (`src/config/mod.rs`) with TOML support
- [x] Build integrations module (`src/integrations/mod.rs`) with ticket extraction and URL building
- [x] Update WorkRecord model to include optional `ticket` field
- [x] Update AppState to support Ticket field in edit mode (Name → Start → End → Ticket cycling)
- [x] Add Shift+J keybinding to open tickets in browser
- [x] Update UI rendering to display tickets with blue badge styling
- [x] Update README with JIRA/Linear integration documentation
- **Context**: Implemented full JIRA and Linear issue tracker integration allowing users to open tickets directly from work records. Features auto-detection of tracker type and customizable configuration via `~/.config/work-tuimer/config.toml`
- **Testing**: All 19 unit tests pass (config serialization, URL building, ticket pattern matching)
- **Build**: Successfully compiles via `nix-shell --run 'cargo build'` (resolves libiconv linking issue)
- **Files Modified**: Cargo.toml, src/main.rs, src/config/mod.rs (NEW), src/integrations/mod.rs (NEW), src/models/work_record.rs, src/ui/app_state.rs, src/ui/render.rs, README.md

### Bug Fix: New Task Placement (2025-11-05)
- [x] Fix `add_new_record()` to place new tasks after selected record instead of at current time
- **Context**: The "n" keybind was creating tasks at random positions (wherever current time fell in the sorted list), while "b" for breaks worked correctly
- **Solution**: Changed `add_new_record()` to use the same logic as `add_break()` - start time is set to the selected record's end time, with 1 hour duration
- **File**: src/ui/app_state.rs:335-359

### Issue #2: Double Key Events on WezTerm/Windows (2025-11-05)
- [x] Update Cargo.toml to use Rust edition 2024
- [x] Import KeyEventKind from crossterm::event
- [x] Filter key events to only handle KeyEventKind::Press
- **Context**: On WezTerm/Windows, key events fire for both Press and Release, causing doubled actions
- **Solution**: Filter events using `if let` with `&& key.kind == KeyEventKind::Press` pattern
- **Commit**: 0e49371 - "Fix doubled keyboard actions on WezTerm/Windows by filtering for Press events only"
- **Issue**: https://github.com/Kamyil/work-tuimer/issues/2

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
