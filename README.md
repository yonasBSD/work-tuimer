# WorkTimer TUI

A terminal user interface (TUI) version of WorkTimer built with Rust and ratatui.

## Status: Phase 2 Complete ✅

### Completed Features

**Phase 1: Core Data & Storage**
- ✅ Data models (TimePoint, WorkRecord, DayData)
- ✅ JSON file-based storage (~/.local/share/worktimer/ or ./data/ fallback)
- ✅ Per-day data files (YYYY-MM-DD.json format)

**Phase 2: Browse Mode UI**
- ✅ Ratatui-based terminal interface
- ✅ Display work records in a list
- ✅ Navigation with arrow keys (↑/↓)
- ✅ Header showing current date
- ✅ Footer with keyboard shortcuts
- ✅ Selection highlighting

### Running

```bash
cd tui
cargo run
```

### Testing with Sample Data

Sample data file is provided in `./data/2025-10-31.json`

### Controls (Browse Mode)

- `↑/↓` - Navigate records
- `q` - Quit

### Architecture

```
tui/
├── src/
│   ├── models/         # Data structures
│   │   ├── time_point.rs   - Time representation (HH:MM)
│   │   ├── work_record.rs  - Individual work entries
│   │   └── day_data.rs     - Daily collection of records
│   ├── storage/        # File I/O
│   │   └── storage.rs      - JSON load/save
│   ├── ui/             # Terminal UI
│   │   ├── app_state.rs    - App state & mode management
│   │   └── render.rs       - UI rendering
│   └── main.rs         # App entry & event loop
└── .cargo/
    └── config.toml     # Fix for macOS libiconv linking
```

### Next Steps (Phase 3+)

- [ ] Inline editing mode (Enter to edit, Tab to switch fields)
- [ ] Add/Delete records
- [ ] Break tracking
- [ ] Grouped totals panel
- [ ] Auto-save on changes
- [ ] Date navigation (previous/next day)
