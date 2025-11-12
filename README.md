# WorkTUImer
![work-tuimer](https://github.com/user-attachments/assets/207f9b66-0b08-4e97-a471-a9f413a7369c)

Live demo: https://x.com/KsenKamil/status/1985423210859368716

Simple, keyboard-driven TUI for time-tracking that allows you to quickly add time blocks and automatically group time if same task was done in different sessions
Built with Rust and ratatui for efficient time management.

## Features

- **Fully keyboard-driven**: No mouse required - everything accessible via keybinds
- **Active timer tracking**: Start/stop/pause timers that automatically update work records with actual time spent
- **Time as PIN-Inputs**: Easly type time with 4 clicks, since all time inputs are PIN-input alike
- **Log tasks and breaks, get totals automatically**: Add work entries with start/end times - durations are calculated and summed
- **Task picker with history**: Quickly select from previously used task names or create new ones
- **Calendar navigation**: Jump between days, weeks, and months
- **Arrow keys or Vim motions**: Navigate with arrow keys + Enter, or use h/j/k/l + i for Vim-style workflow
- **Inline editing with undo/redo**: Fix mistakes in place, up to 50 levels of history
- **Auto-saves locally per day**: Data stored as JSON files, for each day, on your machine (`~/.local/share/work-tuimer/`)
- **Optional ticket integration**: Detect and link to JIRA, Linear, GitHub issues from task names - open ticket URLs directly in your browser from the app

## Installation

### FreeBSD

```sh
pkg install work-tuimer
```

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
| `C` | Open calendar view for date navigation |
| `Enter/i` | Enter edit mode on selected field |
| `c` | Change task name (opens picker to select/filter/create) |
| `n` | Add new work record |
| `b` | Add break (uses selected record's end time as start) |
| `d` | Delete selected record |
| `v` | Enter visual mode (multi-select) |
| `S` | Start/Stop timer for selected record |
| `P` | Pause/Resume active timer |
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
| `Enter` | Save changes and exit edit mode |
| `Esc` | Cancel and exit edit mode |
| `Backspace` | Delete character |
| Any char | Insert character |

### Task Picker (accessed via `c` in Browse mode)

Press `c` on the Name field to open the task picker:
- Shows all unique task names from the current day
- Type to filter the list
- Press Enter to select a task or create a new one

| Key | Action |
|-----|--------|
| Any char | Type to filter tasks or create new name (including h/j/k/l) |
| `↑` | Move selection up in filtered list |
| `↓` | Move selection down in filtered list |
| `Enter` | Select highlighted task or create typed name |
| `Backspace` | Delete character from filter |
| `Esc` | Cancel and return to browse mode |

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

## Timer Sessions

WorkTimer includes a built-in timer system for real-time time tracking. Sessions allow you to track time as you work, with automatic updates, pause/resume support, and seamless CLI/TUI integration.

### Quick Start

**In the TUI:**
1. Select a work record and press `S` to start a session
2. See the timer status bar at the top with elapsed time
3. Press `P` to pause/resume, `S` to stop

**From the CLI:**
```bash
# Start a session
work-tuimer session start "My Task"

# Check status
work-tuimer session status

# Pause/resume
work-tuimer session pause
work-tuimer session resume

# Stop and save
work-tuimer session stop
```

### Key Features

- **Automatic time updates**: End time is set when you stop the session
- **Pause support**: Only active time is counted, paused duration tracked separately
- **Cross-session persistence**: Sessions survive app restarts
- **CLI + TUI integration**: Start in CLI, stop in TUI, or vice versa
- **Visual indicators**: Active sessions highlighted with ⏱ icon

**📖 For detailed documentation, workflows, and troubleshooting, see [Timer Sessions Guide](docs/SESSIONS.md)**

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

## Theme Configuration (Optional)

WorkTimer supports customizable color themes to personalize your UI experience. The application includes 8 pre-defined themes and supports custom theme definitions.

**This feature is completely optional** - the application works perfectly without any configuration.

### Quick Start

1. **Choose a pre-defined theme**: Pick from 8 built-in themes (kanagawa, catppuccin, gruvbox, etc.)
2. **Add to config**: Create `~/.config/work-tuimer/config.toml` with `[theme]` section
3. **Restart the app**: Your new theme will be applied immediately

```toml
[theme]
active = "kanagawa"  # Options: default, kanagawa, catppuccin, gruvbox, monokai, dracula, everforest, terminal
```

### Available Themes

- **default**: Original cyan highlights with dark backgrounds
- **kanagawa**: Dark navy blue aesthetic inspired by the Great Wave
- **catppuccin**: Soothing pastel theme for comfortable viewing
- **gruvbox**: Retro groove warm color palette
- **monokai**: Classic editor theme with vibrant colors
- **dracula**: Dark theme with purple and pink accents
- **everforest**: Comfortable green forest color scheme
- **terminal**: Uses your terminal's default colors

**📖 For detailed documentation, custom themes, color formats, and examples, see [Theme Configuration Guide](docs/THEMING.md)**

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
