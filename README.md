# WorkTimer TUI

![WorkTimer TUI Example](work_tuimer_example.png)
Live demo: https://x.com/KsenKamil/status/1985423210859368716

A terminal user interface (TUI) for tracking work time entries with inline editing capabilities. Built with Rust and ratatui for efficient time management.

## Features

- **Browse Mode**: View all work entries with Vi-style navigation
- **Calendar View**: Visual month calendar for quick date navigation with `Shift+C`
- **Day Navigation**: Navigate between days with `[` (previous) and `]` (next)
- **Edit Mode**: Inline editing of task names and time fields
- **Visual Mode**: Select and delete multiple records at once with `v` keybind
- **Smart Breaks**: Add break entries that calculate duration automatically with `b` keybind
- **Task Picker**: Quick select from existing task names with `/` keybind while editing
- **Undo/Redo**: Recover from mistakes with `u` / `r` keybinds (max 50 levels)
- **Auto-save**: Automatically saves changes on quit and when switching days
- **Persistent Storage**: JSON file per day in `~/.local/share/work-tuimer/` (or `./data/` fallback)
- **Switch days and even whole months** via `[` and `]` keybind (+ "C" (capital c) for running calendar)
- **Issue Tracker Integration**: Automatic ticket detection from task names with browser shortcuts (supports JIRA, Linear, GitHub, GitLab, and any custom tracker)

## Installation

### Pre-built Binaries

Download the latest pre-built binary for your platform from [GitHub Releases](https://github.com/Kamyil/work-tuimer/releases):

- **Linux (x86_64)**: `work-tuimer-linux-x86_64`
- **macOS (Intel)**: `work-tuimer-macos-x86_64`
- **macOS (Apple Silicon)**: `work-tuimer-macos-aarch64`
- **Windows**: `work-tuimer-windows-x86_64.exe`

After downloading, make the binary executable and run it:

```bash
# Linux / macOS
chmod +x work-tuimer-linux-x86_64
./work-tuimer-linux-x86_64

# Windows
work-tuimer-windows-x86_64.exe
```

### Build from Source

If you prefer to build from source or don't see a binary for your platform:

```bash
cargo build --release
./target/release/work-tuimer
```

## Usage

### Browse Mode

| Key | Action |
|-----|--------|
| `↑/k` | Move selection up |
| `↓/j` | Move selection down |
| `←/h` | Move field left (Name → Start → End) |
| `→/l` | Move field right (Name → Start → End) |
| `[` | Navigate to previous day (auto-saves) |
| `]` | Navigate to next day (auto-saves) |
| `Shift+C` | Open calendar view for date navigation |
| `Enter/i` | Enter edit mode on selected field |
| `c` | Change task name (quick edit) |
| `n` | Add new work record |
| `b` | Add break (uses selected record's end time as start) |
| `d` | Delete selected record |
| `v` | Enter visual mode (multi-select) |
| `t` | Set current time on selected field |
| `T` | Open ticket in browser (only visible if config exists) |
| `L` | Open worklog URL in browser (only visible if config exists) |
| `u` | Undo last change |
| `r` | Redo undone change |
| `s` | Save to file |
| `q` | Quit (auto-saves) |

### Edit Mode

| Key | Action |
|-----|--------|
| `Tab` | Next field (Name → Start → End → Description → Name) |
| `/` | Open task picker (quick select from existing tasks) |
| `Enter` | Save changes and exit edit mode |
| `Esc` | Cancel and exit edit mode |
| `Backspace` | Delete character |
| Any char | Insert character |

### Task Picker

When editing a task name, press `/` to open a quick picker showing all unique task names from the current day. This helps you quickly reuse task names without retyping.

| Key | Action |
|-----|--------|
| `↑/k` | Move selection up |
| `↓/j` | Move selection down |
| `Enter` | Select task name |
| `Esc` | Cancel and return to editing |

### Visual Mode

| Key | Action |
|-----|--------|
| `↑/k` | Extend selection up |
| `↓/j` | Extend selection down |
| `d` | Delete selected records |
| `Esc` | Exit visual mode |

### Calendar View

| Key | Action |
|-----|--------|
| `↑/k` | Move selection up (1 week) |
| `↓/j` | Move selection down (1 week) |
| `←/h` | Move selection left (1 day) |
| `→/l` | Move selection right (1 day) |
| `[/</,` | Previous month |
| `]/>/.` | Next month |
| `Enter` | Jump to selected date |
| `Esc` | Close calendar view |

## Issue Tracker Integration (Optional)

WorkTimer supports automatic ticket detection from task names and browser integration for **any** issue tracker (JIRA, Linear, GitHub Issues, GitLab, Azure DevOps, etc.). 

**This feature is completely optional** - the application works perfectly without any configuration.

### Quick Start

1. **Include ticket IDs in task names**: `"PROJ-123: Fix login bug"` or `"#456: Update docs"`
2. **See the ticket badge**: Tasks with detected tickets show `🎫 Task Name [PROJ-123]`
3. **Open in browser**: Press `T` to open the ticket or `L` to open the worklog

### Configuration

Create a config file at `~/.config/work-tuimer/config.toml`:

```toml
[integrations]
default_tracker = "my-jira"

[integrations.trackers.my-jira]
enabled = true
base_url = "https://your-company.atlassian.net"
ticket_patterns = ["^PROJ-\\d+$", "^WORK-\\d+$"]
browse_url = "{base_url}/browse/{ticket}"
worklog_url = "{base_url}/browse/{ticket}?focusedWorklogId=-1"
```

**📖 For detailed documentation, configuration examples, and multiple tracker setup, see [Issue Tracker Integration Guide](docs/ISSUE_TRACKER_INTEGRATION.md)**

## Data Format

Data is stored per day in JSON format:

```json
{
  "date": "2025-10-31",
  "work_records": [
    {
      "id": 1,
      "name": "Task name",
      "start": "09:00",
      "end": "12:00",
      "total_minutes": 180,
      "description": "Optional description"
    }
  ]
}
```

Storage locations (checked in order):
1. `~/.local/share/work-tuimer/YYYY-MM-DD.json`
2. `./data/YYYY-MM-DD.json` (fallback)

## Project Structure

```
src/
├── models/         # Core data models
│   ├── time_point.rs   - Time representation (HH:MM format)
│   ├── work_record.rs  - Individual work entry
│   └── day_data.rs     - Daily collection of records
├── storage/        # File I/O
│   └── storage.rs      - JSON persistence
├── ui/             # Terminal interface
│   ├── app_state.rs    - State management & event handlers
│   └── render.rs       - UI rendering with ratatui
└── main.rs         # Entry point & event loop
```

## Development

```bash
cargo check
cargo build
cargo test
cargo clippy
```

### Creating a Release

This project uses GitHub Actions to automatically build and publish pre-built binaries. To create a new release:

```bash
just release v0.2.0
```

This will:
1. Create a git tag for the version
2. Push the tag to GitHub
3. Trigger GitHub Actions to build binaries for all platforms
4. Automatically upload the binaries to a GitHub Release

You can track the build progress in the [Actions tab](https://github.com/sst/work-tuimer/actions).

## License

MIT
