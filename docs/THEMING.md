# Theme Configuration

WorkTimer supports customizable color themes to personalize your UI experience. The application includes 8 pre-defined themes and supports custom theme definitions.

## Table of Contents

- [Quick Start](#quick-start)
- [Pre-defined Themes](#pre-defined-themes)
- [Custom Themes](#custom-themes)
- [Color Format Options](#color-format-options)
- [Theme Color Reference](#theme-color-reference)

## Quick Start

**Note**: Theming is completely optional. Without a config file, WorkTimer uses the default theme.

To enable theming, create a configuration file at the appropriate location for your platform:

- **Linux/macOS**: `~/.config/work-tuimer/config.toml` (or `$XDG_CONFIG_HOME/work-tuimer/config.toml` if set)
- **Windows**: `%APPDATA%\work-tuimer\config.toml`

Choose a pre-defined theme:

```toml
[theme]
active = "kanagawa"  # Options: default, kanagawa, catppuccin, gruvbox, monokai, dracula, everforest, terminal
```

## Pre-defined Themes

WorkTimer includes 8 carefully crafted themes:

### 1. **default**
The original WorkTimer color scheme with cyan highlights and dark backgrounds. Clean and professional.

```toml
[theme]
active = "default"
```

### 2. **kanagawa**
Dark navy blue aesthetic inspired by the Great Wave off Kanagawa. Deep blues with warm accents.

```toml
[theme]
active = "kanagawa"
```

### 3. **catppuccin**
Soothing pastel theme (Mocha variant) for comfortable viewing. Soft purples and blues.

```toml
[theme]
active = "catppuccin"
```

### 4. **gruvbox**
Retro groove warm color palette. Earth tones with high contrast.

```toml
[theme]
active = "gruvbox"
```

### 5. **monokai**
Classic editor theme with vibrant colors. Bright highlights on dark background.

```toml
[theme]
active = "monokai"
```

### 6. **dracula**
Dark theme with purple and pink accents. Modern and stylish.

```toml
[theme]
active = "dracula"
```

### 7. **everforest**
Comfortable green forest color scheme. Easy on the eyes.

```toml
[theme]
active = "everforest"
```

### 8. **terminal**
Uses your terminal's default colors. Inherits your terminal theme settings.

```toml
[theme]
active = "terminal"
```

## Custom Themes

Create your own theme with custom colors. Add a `[theme.custom.mytheme]` section to your config:

```toml
[theme]
active = "mytheme"  # Use your custom theme name

[theme.custom.mytheme]
# Border colors
active_border = "#00ffff"        # Cyan (can use hex)
inactive_border = "DarkGray"     # Can use named colors
searching_border = "yellow"      # Lowercase also works

# Background colors
selected_bg = "(40, 40, 60)"     # Can use RGB tuples
selected_inactive_bg = "#1e1e2d"
visual_bg = "#4682b4"
timer_active_bg = "#228b22"
row_alternate_bg = "#191923"
edit_bg = "#164e63"

# Text colors
primary_text = "White"
secondary_text = "Gray"
highlight_text = "Cyan"

# Status colors
success = "Green"
warning = "Yellow"
error = "LightRed"
info = "Cyan"

# Specific element colors
timer_text = "Yellow"
badge = "LightMagenta"
```

### Multiple Custom Themes

You can define multiple custom themes and switch between them:

```toml
[theme]
active = "work"  # Use 'work' during work hours

[theme.custom.work]
active_border = "#00ffff"
selected_bg = "#1e3a5f"
# ... other colors

[theme.custom.evening]
active_border = "#ff8c00"
selected_bg = "#2d1e1e"
# ... other colors
```

To switch themes, just change the `active` value and restart the app.

## Color Format Options

Colors can be specified in three formats:

### 1. Hex Colors
Standard hexadecimal RGB colors:
- **6-digit format**: `"#RRGGBB"` (e.g., `"#00ff00"` for green)
- **3-digit format**: `"#RGB"` (e.g., `"#0f0"` for green, expanded to `#00ff00`)

```toml
active_border = "#00ffff"      # Cyan
selected_bg = "#1e3a5f"        # Dark blue
primary_text = "#fff"          # White (shorthand)
```

### 2. RGB Tuples
RGB values as strings with parentheses:
- **Format**: `"(R, G, B)"` where R, G, B are 0-255
- Spaces are optional: `"(255,128,0)"` works too

```toml
selected_bg = "(40, 40, 60)"   # Dark purple-gray
warning = "(255, 165, 0)"      # Orange
timer_text = "(255,255,0)"     # Yellow (no spaces)
```

### 3. Named Colors
Standard terminal color names (case-insensitive):

**Basic Colors**:
- `Black`, `Red`, `Green`, `Yellow`, `Blue`, `Magenta`, `Cyan`, `White`

**Bright Colors**:
- `LightRed`, `LightGreen`, `LightYellow`, `LightBlue`, `LightMagenta`, `LightCyan`

**Grayscale**:
- `Gray`, `DarkGray`

```toml
primary_text = "White"
secondary_text = "Gray"
error = "LightRed"
success = "green"              # Case-insensitive
```

### Fallback Behavior

If a color value is invalid or cannot be parsed, it falls back to **White**. This ensures the UI remains functional even with configuration errors.

## Theme Color Reference

All themes use these semantic color names. Each color serves a specific purpose in the UI:

### Border Colors

| Color Name | Usage | Example Context |
|------------|-------|-----------------|
| `active_border` | Border color for focused elements | Selected table, active modal |
| `inactive_border` | Border color for unfocused elements | Unfocused panels, inactive windows |
| `searching_border` | Border color during search/filter | Task picker, search mode |

### Background Colors

| Color Name | Usage | Example Context |
|------------|-------|-----------------|
| `selected_bg` | Background for selected items | Currently selected row in table |
| `selected_inactive_bg` | Background for selected items when unfocused | Selected row when in different mode |
| `visual_bg` | Background in visual/multi-select mode | Multiple selected rows |
| `timer_active_bg` | Background for running timers | Row with active timer, timer bar |
| `row_alternate_bg` | Alternating row background color | Every other row for readability |
| `edit_bg` | Background for editable fields | Input fields in edit mode |

### Text Colors

| Color Name | Usage | Example Context |
|------------|-------|-----------------|
| `primary_text` | Main text color | Task names, times, main content |
| `secondary_text` | Dimmed/secondary text | Descriptions, labels, hints |
| `highlight_text` | Emphasized text | Titles, headers, important info |

### Status Colors

| Color Name | Usage | Example Context |
|------------|-------|-----------------|
| `success` | Success status (green actions) | Saved notification, completed actions |
| `warning` | Warning status (yellow alerts) | Validation warnings, cautionary messages |
| `error` | Error status (red errors) | Error messages, critical failures |
| `info` | Info status (blue information) | Help text, informational messages |

### Specific Element Colors

| Color Name | Usage | Example Context |
|------------|-------|-----------------|
| `timer_text` | Active timer text color | Timer duration display, elapsed time |
| `badge` | Badge/tag text color | Ticket badges (e.g., `[PROJ-123]`) |

## Examples

### Example: Minimalist Monochrome

```toml
[theme]
active = "mono"

[theme.custom.mono]
active_border = "White"
inactive_border = "DarkGray"
searching_border = "Gray"
selected_bg = "(50, 50, 50)"
selected_inactive_bg = "(30, 30, 30)"
visual_bg = "(70, 70, 70)"
timer_active_bg = "(80, 80, 80)"
row_alternate_bg = "(20, 20, 20)"
edit_bg = "(60, 60, 60)"
primary_text = "White"
secondary_text = "Gray"
highlight_text = "White"
success = "White"
warning = "White"
error = "White"
info = "White"
timer_text = "White"
badge = "Gray"
```

### Example: High Contrast

```toml
[theme]
active = "contrast"

[theme.custom.contrast]
active_border = "#ffff00"        # Bright yellow
inactive_border = "#808080"      # Gray
searching_border = "#00ffff"     # Cyan
selected_bg = "#000080"          # Navy blue
selected_inactive_bg = "#1a1a1a"
visual_bg = "#800080"            # Purple
timer_active_bg = "#006400"      # Dark green
row_alternate_bg = "#0a0a0a"
edit_bg = "#003366"
primary_text = "#ffffff"         # White
secondary_text = "#cccccc"       # Light gray
highlight_text = "#ffff00"       # Yellow
success = "#00ff00"              # Bright green
warning = "#ffa500"              # Orange
error = "#ff0000"                # Bright red
info = "#00ffff"                 # Cyan
timer_text = "#ffff00"           # Yellow
badge = "#ff00ff"                # Magenta
```

### Example: Soft Pastels

```toml
[theme]
active = "pastel"

[theme.custom.pastel]
active_border = "#b4befe"        # Lavender
inactive_border = "#6c7086"      # Gray
searching_border = "#f5c2e7"     # Pink
selected_bg = "(49, 50, 68)"     # Dark blue-gray
selected_inactive_bg = "(30, 30, 46)"
visual_bg = "(116, 199, 236)"    # Light blue
timer_active_bg = "(166, 227, 161)" # Light green
row_alternate_bg = "(24, 24, 37)"
edit_bg = "(69, 71, 90)"
primary_text = "#cdd6f4"         # Light blue-white
secondary_text = "#9399b2"       # Blue-gray
highlight_text = "#f5c2e7"       # Pink
success = "#a6e3a1"              # Light green
warning = "#f9e2af"              # Light yellow
error = "#f38ba8"                # Light red
info = "#89dceb"                 # Light cyan
timer_text = "#f9e2af"           # Light yellow
badge = "#cba6f7"                # Light purple
```

## Tips

1. **Test your theme**: Make changes and restart the app to see them immediately
2. **Start with a pre-defined theme**: Modify an existing theme instead of creating from scratch
3. **Use consistent color families**: Pick 2-3 main colors and use variations (lighter/darker)
4. **Consider contrast**: Ensure text is readable against backgrounds
5. **Check in different lighting**: Test your theme in both bright and dim environments
6. **Save backups**: Keep a copy of working configurations before major changes

## Troubleshooting

### Theme not loading
- Check the config file path is correct for your platform
- Verify TOML syntax is valid (use a TOML validator)
- Ensure the `active` theme name matches your custom theme name
- Check file permissions (must be readable)

### Colors look wrong
- Verify color format is correct (hex, RGB tuple, or named color)
- Check for typos in color names (case-insensitive but must be spelled correctly)
- Remember: invalid colors fall back to White
- Some terminals may not support true color (24-bit) - named colors are more compatible

### Theme not applying to all elements
- This is expected - WorkTimer currently uses a fixed set of semantic colors
- All UI elements are designed to use one of the 18 theme colors
- If you find an element that doesn't respect themes, please report it as a bug
