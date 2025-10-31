# Rust TUI Time Tracker - Implementation Plan

## Overview

Port of the worktimer2 web application to a terminal user interface (TUI) using Rust and ratatui.

---

## Core Features to Port

1. **Work Record Management**
   - Add/delete work records
   - Edit task name, start/end times (inline editing)
   - Auto-calculate duration
   - Add breaks ("PRZERWA")
   - Set current time for focused field

2. **Display Views**
   - Editable work records table (inline editing like web version)
   - Summed records (grouped by task name)
   - Total time display

3. **Persistence**
   - Auto-save on quit
   - Manual save capability
   - JSON file storage

---

## Rust Project Structure

```
worktimer-tui/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point, event loop
│   ├── app.rs            # App state & logic
│   ├── ui.rs             # Ratatui rendering
│   ├── models.rs         # WorkRecord, AppState structs
│   ├── storage.rs        # JSON file persistence (per-day files)
│   ├── input.rs          # Keyboard event handling
│   └── utils.rs          # Time calculations
└── docs/
    └── IMPLEMENTATION_PLAN.md

Runtime data location:
~/.local/share/worktimer/
├── 2024-10-31.json       # Today's records
├── 2024-10-30.json       # Previous days
└── config.json           # App preferences (future)
```

---

## Dependencies (Cargo.toml)

```toml
[dependencies]
ratatui = "0.26"
crossterm = "0.27"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
anyhow = "1.0"
```

---

## Data Models

```rust
#[derive(Serialize, Deserialize, Clone)]
struct TimePoint {
    hour: u8,    // 0-23
    minute: u8,  // 0-59
}

#[derive(Serialize, Deserialize, Clone)]
struct WorkRecord {
    id: u32,
    name: String,
    start: TimePoint,
    end: TimePoint,
    total_minutes: u32,
}

// Persisted to disk (YYYY-MM-DD.json)
#[derive(Serialize, Deserialize)]
struct DayData {
    date: String,  // "YYYY-MM-DD"
    last_id: u32,
    work_records: Vec<WorkRecord>,
}

// Runtime state (not all fields are persisted)
struct AppState {
    day_data: DayData,
    selected_row: usize,
    selected_field: FieldType,
    edit_mode: bool,
    edit_buffer: String,
    storage_path: PathBuf,  // e.g., ~/.local/share/worktimer/2024-10-31.json
}

enum FieldType {
    Name,
    StartHour,
    StartMinute,
    EndHour,
    EndMinute,
    DeleteButton,
}
```

---

## TUI Layout (Inline Editing)

```
┌─────────────────────────────────────────────────────────────────┐
│ WorkTimer TUI                              Total: 8h 30m        │
├─────────────────────────────────────────────────────────────────┤
│ WORK RECORDS                                                    │
│ ┌───┬──────────────┬───────────┬───────────┬──────────────┐   │
│ │ # │ Task Name    │ Start     │ End       │ Duration     │   │
│ ├───┼──────────────┼───────────┼───────────┼──────────────┤   │
│ │ 1 │ Coding       │ 10 : 00   │ 12 : 30   │ 2h 30m       │   │
│ │►2 │[Meeting___]  │ 13 : 00   │ 14 : 00   │ 1h 00m  [DEL]│   │
│ │ 3 │ PRZERWA      │ 14 : 00   │ 14 : 15   │ 15m          │   │
│ └───┴──────────────┴───────────┴───────────┴──────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│ GROUPED BY TASK                                                 │
│ Coding:     5h 30m    Meeting:   2h 00m    PRZERWA:   1h 00m  │
├─────────────────────────────────────────────────────────────────┤
│ Editing: Task Name                                              │
│ [Type to edit] [Tab] Next field  [Enter] Done  [Esc] Cancel   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Keyboard Controls

### Browse Mode (default)
- `↑/↓` - Navigate between records (rows)
- `←/→` - Navigate between fields (columns)
- `Enter` - Start editing current field
- `d` - Delete current record
- `n` - Add new work record (with default values)
- `b` - Add break (with default values)
- `T` - Set current time to focused time field
- `s` - Manual save
- `q` - Quit (with auto-save)

### Edit Mode (when editing a field)
- Type directly to edit text/numbers
- `Enter` - Finish editing, stay on same field
- `Tab` - Finish editing, move to next field
- `Esc` - Cancel editing, revert changes

---

## Visual States

### Selected but not editing
```
│ 2 │ Meeting      │ 13 : 00   │ 14 : 00   │ 1h 00m       │
    ^^^^^^^^^^^ highlighted
```

### Editing name field
```
│ 2 │[Meeting___]  │ 13 : 00   │ 14 : 00   │ 1h 00m       │
    ^^^^^^^^^^^^ text cursor visible
```

### Editing hour field
```
│ 2 │ Meeting      │[13]: 00   │ 14 : 00   │ 1h 00m       │
                    ^^^^ editing
```

---

## Implementation Phases

### Phase 1: Core Data & Storage
- Define models (`WorkRecord`, `AppState`, `TimePoint`)
- Implement JSON save/load (`storage.rs`)
- Time calculation utils (`utils.rs`)
- Test with sample data

### Phase 2: Browse Mode UI
- Render work records table
- Navigate rows with ↑/↓
- Navigate fields with ←/→
- Highlight selected field
- Total time display

### Phase 3: Inline Editing
- Enter edit mode on selected field
- Text/number input handling
- Save changes on Enter/Tab
- Cancel on Esc
- Auto-calculate duration on time changes

### Phase 4: CRUD Operations
- Add new record (`n`) with default values
- Delete current record (`d`)
- Add break (`b`) with default values
- "Set current time" with `T` key for time fields

### Phase 5: Grouping Panel
- Calculate summed records (group by task name)
- Display in bottom panel
- Optional: Tab to switch focus between panels

### Phase 6: Persistence
- Auto-save on quit
- Manual save with `s`
- Load state on startup
- Handle missing/corrupted state file gracefully

---

## Key Differences from Web Version

| Feature | Web | TUI |
|---------|-----|-----|
| Input | Text input fields | Inline editing with Enter |
| "Set current time" | Button click | `T` key on time field |
| Navigation | Mouse | Arrow keys |
| Layout | Collapsible sections | Fixed split panels |
| Delete | Button per row | `d` key on selected row |
| Persistence | localStorage (browser) | JSON file (~/.worktimer.json or ./data/state.json) |
| Add record/break | Buttons at bottom | `n`/`b` keyboard shortcuts |

---

## File Storage Strategy

### Recommended Approach
Use **per-day JSON files** to match the web app's localStorage pattern where each day's data is separate.

### Storage Location
`~/.local/share/worktimer/` (follows XDG Base Directory spec on Linux/macOS)

### File Structure
```
~/.local/share/worktimer/
├── 2024-10-31.json    # Today's work records
├── 2024-10-30.json    # Yesterday's records
├── 2024-10-29.json
└── config.json        # Optional: app preferences
```

### Data File Format (YYYY-MM-DD.json)
```json
{
  "date": "2024-10-31",
  "last_id": 5,
  "work_records": [
    {
      "id": 1,
      "name": "Coding",
      "start": { "hour": 10, "minute": 0 },
      "end": { "hour": 12, "minute": 30 },
      "total_minutes": 150
    },
    {
      "id": 2,
      "name": "Meeting",
      "start": { "hour": 13, "minute": 0 },
      "end": { "hour": 14, "minute": 0 },
      "total_minutes": 60
    }
  ]
}
```

### Benefits
- **Matches web app behavior**: Web version stores data per-day in localStorage
- **Easy to browse**: Each day is a separate file
- **Natural backups**: Copy entire directory to backup all history
- **No database needed**: Simple JSON files
- **Future-proof**: Easy to add date picker to view past days

### Implementation Notes
- On startup, load `<today's date>.json`
- Auto-save writes to current day's file
- Create directory if it doesn't exist
- Fallback to `./data/` if `~/.local/share/` is not writable

---

## Notes

- No complex modal dialogs needed - everything is inline editing like the web version
- Duration is always calculated automatically, never edited directly
- Auto-save prevents data loss on unexpected quit
- Polish language labels ("PRZERWA") preserved from original
