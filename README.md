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
| `Enter` | Save changes and exit edit mode |
| `Esc` | Cancel and exit edit mode |
| `Backspace` | Delete character |
| Any char | Insert character |

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

WorkTimer supports automatic ticket detection from task names and browser integration for **any** issue tracker (JIRA, Linear, GitHub Issues, GitLab, Azure DevOps, etc.). **This feature is completely optional** - the application works perfectly without any configuration.

### Setup

**Note**: If you don't create a config file, the integration feature will be hidden (no `T`/`L` keybindings, no ticket badges). The app works perfectly without it.

To enable the integration, create a configuration file at the appropriate location for your platform:

- **Linux/macOS**: `~/.config/work-tuimer/config.toml` (or `$XDG_CONFIG_HOME/work-tuimer/config.toml` if set)
- **Windows**: `%APPDATA%\work-tuimer\config.toml`

You can configure multiple trackers with custom names:

**Example: JIRA tracker**

```toml
[integrations]
default_tracker = "my-jira"  # Default tracker when pattern is ambiguous

[integrations.trackers.my-jira]
enabled = true
base_url = "https://your-company.atlassian.net"
ticket_patterns = ["^PROJ-\\d+$", "^WORK-\\d+$"]  # Regex to match your tickets
browse_url = "{base_url}/browse/{ticket}"
worklog_url = "{base_url}/browse/{ticket}?focusedWorklogId=-1"
```

**Example: GitHub Issues tracker**

```toml
[integrations]
default_tracker = "github"

[integrations.trackers.github]
enabled = true
base_url = "https://github.com/yourorg/yourrepo"
ticket_patterns = ["^#\\d+$"]  # Matches #123
browse_url = "{base_url}/issues/{ticket}"
worklog_url = ""  # GitHub doesn't have worklogs
```

**Example: Multiple trackers (JIRA + Linear + GitHub)**

```toml
[integrations]
default_tracker = "work-jira"  # Fallback when patterns overlap

[integrations.trackers.work-jira]
enabled = true
base_url = "https://company.atlassian.net"
ticket_patterns = ["^PROJ-\\d+$", "^WORK-\\d+$"]  # Company JIRA projects
browse_url = "{base_url}/browse/{ticket}"
worklog_url = "{base_url}/browse/{ticket}?focusedWorklogId=-1"

[integrations.trackers.team-linear]
enabled = true
base_url = "https://linear.app/your-team"
ticket_patterns = ["^ENG-\\d+$", "^DESIGN-\\d+$"]  # Linear team patterns
browse_url = "{base_url}/issue/{ticket}"
worklog_url = ""

[integrations.trackers.oss-github]
enabled = true
base_url = "https://github.com/myorg/myrepo"
ticket_patterns = ["^#\\d+$"]  # GitHub issue numbers
browse_url = "{base_url}/issues/{ticket}"
worklog_url = ""
```

**Multiple Tracker Support**: The app automatically detects which tracker to use based on the `ticket_patterns` regex:
- Each tracker is checked in order until a pattern matches the ticket ID
- If a ticket matches multiple patterns, the **first matching tracker** in the config is used
- If **no pattern matches**, it falls back to the `default_tracker` (useful for catch-all scenarios or tickets that don't follow a strict pattern)
- You can name your trackers anything you want (e.g., `work-jira`, `my-company-tracker`, `team-issues`)

**Best Practice**: Define specific patterns for each tracker to avoid conflicts:
- ✅ Good: JIRA uses `^PROJ-\\d+$`, GitHub uses `^#\\d+$`, Linear uses `^ENG-\\d+$` (distinct patterns)
- ❌ Avoid: JIRA uses `^[A-Z]+-\\d+$`, Linear uses `^[A-Z]+-\\d+$` (overlapping - first one wins)

### Usage

1. **Include ticket IDs in task names**: When creating or editing tasks, include the ticket ID in the name:
   - JIRA: `"PROJ-123: Fix login bug"`
   - Linear: `"ENG-456: Add dark mode"`
   - GitHub: `"#789: Update documentation"`

2. **Visual indicator**: Tasks with detected tickets show a badge with a ticket icon: `🎫 Task Name [PROJ-123]`

3. **Open ticket in browser**: Press `T` (capital T) while a task with a detected ticket (🎫 icon visible) is selected to open the ticket in your default browser

4. **Open worklog URL**: Press `L` (capital L) while a task with a detected ticket (🎫 icon visible) is selected to open the worklog URL (if configured). Useful for JIRA users to quickly jump to the worklog entry form for a ticket

**Note**: The `T` and `L` keybindings only appear in the footer and only work when:
- Integrations are configured in `config.toml`
- The selected task has a ticket ID that matches one of your `ticket_patterns` (indicated by the 🎫 icon)

### Ticket Detection

The system uses regex patterns defined in `ticket_patterns` to detect ticket IDs from task names. Common patterns:
- **JIRA/Linear**: `^[A-Z]+-\\d+$` matches `PROJ-123`, `ENG-456`
- **GitHub Issues**: `^#\\d+$` matches `#123`, `#456`
- **Custom**: Define any regex pattern that matches your tracker's ticket format

Tickets are detected automatically from task names at runtime (no data model changes required).

### Supported Platforms

- **macOS**: Uses `open` command
- **Linux**: Uses `xdg-open` command  
- **Windows**: Uses `cmd /C start` command

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
