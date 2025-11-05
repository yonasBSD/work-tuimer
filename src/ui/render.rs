use crate::ui::AppState;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table, TableState},
};

pub fn render(frame: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.size());

    let is_wide = frame.size().width >= 100;
    let middle_chunks = if is_wide {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(chunks[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(15)])
            .split(chunks[1])
    };

    render_header(frame, chunks[0], app);

    render_records(frame, middle_chunks[0], app);
    render_grouped_totals(frame, middle_chunks[1], app);

    render_footer(frame, chunks[2], app);

    // Render command palette overlay if active
    if matches!(app.mode, crate::ui::AppMode::CommandPalette) {
        render_command_palette(frame, app);
    }

    // Render calendar modal if active
    if matches!(app.mode, crate::ui::AppMode::Calendar) {
        render_calendar(frame, app);
    }

    // Render error modal if there's an error
    if app.last_error_message.is_some() {
        render_error_modal(frame, app);
    }
}

fn render_header(frame: &mut Frame, area: Rect, app: &AppState) {
    let date_str = format!("{}", app.current_date);

    let total_minutes: u32 = app
        .day_data
        .work_records
        .values()
        .map(|r| r.total_minutes)
        .sum();
    let total_hours = total_minutes / 60;
    let total_mins = total_minutes % 60;

    // Create a more visual header with sections
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let title_text = format!("⏱  WorkTimer - {} [←prev] [next→]", date_str);
    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    let total_text = format!("Total: {}h {:02}m", total_hours, total_mins);
    let total = Paragraph::new(total_text)
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        );

    frame.render_widget(title, chunks[0]);
    frame.render_widget(total, chunks[1]);
}

fn render_records(frame: &mut Frame, area: Rect, app: &AppState) {
    let records = app.day_data.get_sorted_records();

    // Calculate how many rows can fit in the visible area
    // Account for: borders (2) + header (2) + margin (1) = 5 lines
    let available_height = area.height.saturating_sub(5) as usize;

    // Calculate scroll offset to keep selected item visible
    let scroll_offset = if records.len() > available_height {
        if app.selected_index >= available_height {
            app.selected_index.saturating_sub(available_height - 1)
        } else {
            0
        }
    } else {
        0
    };

    let rows: Vec<Row> = records
        .iter()
        .enumerate()
        .map(|(i, record)| {
            let is_selected = i == app.selected_index;
            let is_editing = matches!(app.mode, crate::ui::AppMode::Edit) && is_selected;
            let is_in_visual =
                matches!(app.mode, crate::ui::AppMode::Visual) && app.is_in_visual_selection(i);

            // Enhanced styling with more vibrant colors
            let style = if is_in_visual {
                Style::default()
                    .bg(Color::Rgb(70, 130, 180)) // Steel blue background
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .bg(Color::Rgb(40, 40, 60)) // Dark blue-gray background
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if i % 2 == 0 {
                Style::default().bg(Color::Rgb(25, 25, 35)) // Subtle alternating rows
            } else {
                Style::default()
            };

            // Add icon/emoji based on task type
            let icon = if record.name.to_lowercase().contains("break") {
                "☕"
            } else if record.name.to_lowercase().contains("meeting") {
                "👥"
            } else if record.name.to_lowercase().contains("code")
                || record.name.to_lowercase().contains("dev")
            {
                "💻"
            } else {
                "📋"
            };

            // Determine display text and styles for each field
            let (name_display, start_display, end_display, description_display) = if is_editing {
                match app.edit_field {
                    crate::ui::EditField::Name => {
                        // Extract and display ticket badge if present
                        let display = if let Some(ticket) = crate::integrations::extract_ticket_from_name(&app.input_buffer) {
                            format!("{} {} [{}]", icon, app.input_buffer, ticket)
                        } else {
                            format!("{} {}", icon, app.input_buffer)
                        };
                        (
                            display,
                            record.start.to_string(),
                            record.end.to_string(),
                            record.description.clone(),
                        )
                    }
                    crate::ui::EditField::Description => {
                        // Extract and display ticket badge if present
                        let display = if let Some(ticket) = crate::integrations::extract_ticket_from_name(&record.name) {
                            format!("{} {} [{}]", icon, record.name, ticket)
                        } else {
                            format!("{} {}", icon, record.name)
                        };
                        (
                            display,
                            record.start.to_string(),
                            record.end.to_string(),
                            app.input_buffer.clone(),
                        )
                    }
                    crate::ui::EditField::Start | crate::ui::EditField::End => {
                        // Add cursor position indicator for time fields
                        let time_str = &app.input_buffer;
                        let positions = [0, 1, 3, 4];
                        let cursor_pos = if app.time_cursor < positions.len() {
                            positions[app.time_cursor]
                        } else {
                            positions[positions.len() - 1]
                        };

                        let mut display = String::new();
                        for (i, ch) in time_str.chars().enumerate() {
                            if i == cursor_pos {
                                display.push('[');
                                display.push(ch);
                                display.push(']');
                            } else {
                                display.push(ch);
                            }
                        }

                        // Extract and display ticket badge if present
                        let name_with_badge = if let Some(ticket) = crate::integrations::extract_ticket_from_name(&record.name) {
                            format!("{} {} [{}]", icon, record.name, ticket)
                        } else {
                            format!("{} {}", icon, record.name)
                        };

                        match app.edit_field {
                            crate::ui::EditField::Start => (
                                name_with_badge,
                                display,
                                record.end.to_string(),
                                record.description.clone(),
                            ),
                            crate::ui::EditField::End => (
                                name_with_badge,
                                record.start.to_string(),
                                display,
                                record.description.clone(),
                            ),
                            _ => unreachable!(),
                        }
                    }
                }
            } else {
                // Extract and display ticket badge if present (non-editing mode)
                let name_with_badge = if let Some(ticket) = crate::integrations::extract_ticket_from_name(&record.name) {
                    format!("{} {} [{}]", icon, record.name, ticket)
                } else {
                    format!("{} {}", icon, record.name)
                };
                (
                    name_with_badge,
                    record.start.to_string(),
                    record.end.to_string(),
                    record.description.clone(),
                )
            };

            // Define toned down background colors for focused and edit states
            let focus_bg = Color::Rgb(88, 28, 135); // Dark purple (similar to Tailwind purple-900)
            let edit_bg = Color::Rgb(22, 78, 99); // Dark cyan (similar to Tailwind cyan-900)

            // Apply styles based on focus and edit state
            let name_style = if is_editing && matches!(app.edit_field, crate::ui::EditField::Name) {
                Style::default()
                    .bg(edit_bg)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected && matches!(app.edit_field, crate::ui::EditField::Name) {
                Style::default()
                    .bg(focus_bg)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let start_style = if is_editing && matches!(app.edit_field, crate::ui::EditField::Start)
            {
                Style::default()
                    .bg(edit_bg)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected && matches!(app.edit_field, crate::ui::EditField::Start) {
                Style::default()
                    .bg(focus_bg)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::LightGreen)
            };

            let end_style = if is_editing && matches!(app.edit_field, crate::ui::EditField::End) {
                Style::default()
                    .bg(edit_bg)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected && matches!(app.edit_field, crate::ui::EditField::End) {
                Style::default()
                    .bg(focus_bg)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::LightRed)
            };

            let description_style = if is_editing
                && matches!(app.edit_field, crate::ui::EditField::Description)
            {
                Style::default()
                    .bg(edit_bg)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected && matches!(app.edit_field, crate::ui::EditField::Description) {
                Style::default()
                    .bg(focus_bg)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White) // White for description text
            };

            Row::new(vec![
                Cell::from(name_display).style(name_style),
                Cell::from(start_display).style(start_style),
                Cell::from(end_display).style(end_style),
                Cell::from(record.format_duration())
                    .style(Style::default().fg(Color::LightMagenta)),
                Cell::from(description_display).style(description_style),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Percentage(30),
        ],
    )
     .header(
         Row::new(vec![
             Cell::from("📝 Task Name"),
             Cell::from("🕐 Start"),
             Cell::from("🕐 End"),
             Cell::from("⏱  Duration"),
             Cell::from("📄 Description"),
         ])
        .style(
            Style::default()
                .fg(Color::Rgb(255, 215, 0)) // Gold color
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Magenta))
            .title("📊 Work Records")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
    );

    // Use stateful rendering to handle scrolling
    let mut table_state = TableState::default()
        .with_selected(Some(app.selected_index))
        .with_offset(scroll_offset);

    frame.render_stateful_widget(table, area, &mut table_state);
}

fn render_grouped_totals(frame: &mut Frame, area: Rect, app: &AppState) {
    let grouped = app.day_data.get_grouped_totals();

    if grouped.is_empty() {
        let paragraph = Paragraph::new("No records yet")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title("📈 Summary")
                    .title_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let rows: Vec<Row> = grouped
        .iter()
        .map(|(name, minutes)| {
            let hours = minutes / 60;
            let mins = minutes % 60;

            // Choose icon based on task type
            let icon = if name.to_lowercase().contains("break") {
                "☕"
            } else if name.to_lowercase().contains("meeting") {
                "👥"
            } else if name.to_lowercase().contains("code") || name.to_lowercase().contains("dev") {
                "💻"
            } else {
                "📋"
            };

            Row::new(vec![
                Cell::from(format!("{} {}", icon, name)),
                Cell::from(format!("{}h {:02}m", hours, mins)).style(
                    Style::default()
                        .fg(Color::LightMagenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Percentage(65), Constraint::Percentage(35)],
    )
    .header(
        Row::new(vec![Cell::from("Task"), Cell::from("Total")])
            .style(
                Style::default()
                    .fg(Color::Rgb(255, 215, 0))
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .title("📈 Summary")
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
    );

    frame.render_widget(table, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &AppState) {
    let (help_text, mode_color, mode_label) = match app.mode {
         crate::ui::AppMode::Browse => (
             "↑/↓: Row | ←/→: Field | [/]: Day | C: Calendar | Enter: Edit | c: Change | n: New | b: Break | d: Delete | v: Visual | t: Now | T: Ticket | ?: Help | q: Quit",
             Color::Cyan,
             "BROWSE",
         ),
        crate::ui::AppMode::Edit => (
            "Tab: Next field | Enter: Save | Esc: Cancel",
            Color::Yellow,
            "EDIT",
        ),
        crate::ui::AppMode::Visual => (
            "↑/↓: Extend selection | d: Delete | Esc: Exit visual",
            Color::Magenta,
            "VISUAL",
        ),
        crate::ui::AppMode::CommandPalette => (
            "↑/↓: Navigate | Enter: Execute | Esc: Cancel",
            Color::LightGreen,
            "COMMAND PALETTE",
        ),
        crate::ui::AppMode::Calendar => (
            "hjkl/arrows: Navigate | </>: Month | Enter: Select | Esc: Cancel",
            Color::Magenta,
            "CALENDAR",
        ),
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(mode_color))
                .title(format!("⌨  {} MODE", mode_label))
                .title_style(Style::default().fg(mode_color).add_modifier(Modifier::BOLD))
                .padding(Padding::horizontal(1)),
        );

    frame.render_widget(footer, area);
}

fn render_command_palette(frame: &mut Frame, app: &AppState) {
    use ratatui::widgets::Clear;

    // Create a centered modal
    let area = frame.size();
    let width = area.width.min(80);
    let height = area.height.min(20);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let modal_area = Rect {
        x,
        y,
        width,
        height,
    };

    // Clear the background
    frame.render_widget(Clear, modal_area);

    // Add a background block for the entire modal
    let bg_block = Block::default().style(Style::default().bg(Color::Rgb(20, 20, 30)));
    frame.render_widget(bg_block, modal_area);

    // Split modal into input and results
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(modal_area);

    // Render search input
    let input_text = if app.command_palette_input.is_empty() {
        "Type to search commands...".to_string()
    } else {
        app.command_palette_input.clone()
    };

    let input_style = if app.command_palette_input.is_empty() {
        Style::default()
            .fg(Color::DarkGray)
            .bg(Color::Rgb(30, 30, 45))
    } else {
        Style::default().fg(Color::White).bg(Color::Rgb(30, 30, 45))
    };

    let input = Paragraph::new(input_text).style(input_style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightGreen))
            .title("🔍 Search Commands")
            .title_style(
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(Color::Rgb(30, 30, 45))),
    );

    frame.render_widget(input, chunks[0]);

    // Render filtered commands
    let filtered = app.get_filtered_commands();

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, (_, score, cmd))| {
            let is_selected = i == app.command_palette_selected;

            let style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(70, 130, 180))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().bg(Color::Rgb(25, 25, 38))
            };

            let key_display = format!("  {}  ", cmd.key);
            let score_display = if *score > 0 {
                format!(" ({})", score)
            } else {
                String::new()
            };

            Row::new(vec![
                Cell::from(key_display).style(
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                ),
                Cell::from(cmd.description).style(Style::default().fg(Color::White)),
                Cell::from(score_display).style(Style::default().fg(Color::DarkGray)),
            ])
            .style(style)
        })
        .collect();

    let results_table = Table::new(
        rows,
        [
            Constraint::Length(15),
            Constraint::Min(30),
            Constraint::Length(10),
        ],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightGreen))
            .title(format!("📋 Commands ({} found)", filtered.len()))
            .title_style(
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(Color::Rgb(25, 25, 38))),
    );

    frame.render_widget(results_table, chunks[1]);
}

fn render_calendar(frame: &mut Frame, app: &AppState) {
    use ratatui::widgets::Clear;
    use time::{Date, Month, Weekday};

    // Create a centered modal
    let area = frame.size();
    let width = area.width.min(60);
    let height = area.height.min(25);
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let modal_area = Rect {
        x,
        y,
        width,
        height,
    };

    // Clear the background
    frame.render_widget(Clear, modal_area);

    // Add a background block for the entire modal
    let bg_block = Block::default().style(Style::default().bg(Color::Rgb(20, 20, 30)));
    frame.render_widget(bg_block, modal_area);

    // Create the calendar layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Month/Year header
            Constraint::Min(15),   // Calendar grid
        ])
        .split(modal_area);

    // Render month/year header
    let month_name = match app.calendar_view_month {
        Month::January => "January",
        Month::February => "February",
        Month::March => "March",
        Month::April => "April",
        Month::May => "May",
        Month::June => "June",
        Month::July => "July",
        Month::August => "August",
        Month::September => "September",
        Month::October => "October",
        Month::November => "November",
        Month::December => "December",
    };

    let header_text = format!(
        "📅  {} {}  [< prev] [next >]",
        month_name, app.calendar_view_year
    );
    let header = Paragraph::new(header_text)
        .style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Magenta))
                .style(Style::default().bg(Color::Rgb(30, 30, 45))),
        );

    frame.render_widget(header, chunks[0]);

    // Build calendar grid
    let first_day =
        Date::from_calendar_date(app.calendar_view_year, app.calendar_view_month, 1).unwrap();

    let days_in_month = get_days_in_month(app.calendar_view_month, app.calendar_view_year);
    let first_weekday = first_day.weekday();

    // Calculate starting offset (0 = Monday, 6 = Sunday)
    let offset = match first_weekday {
        Weekday::Monday => 0,
        Weekday::Tuesday => 1,
        Weekday::Wednesday => 2,
        Weekday::Thursday => 3,
        Weekday::Friday => 4,
        Weekday::Saturday => 5,
        Weekday::Sunday => 6,
    };

    // Create calendar rows
    let mut rows = Vec::new();

    // Header row with weekday names
    rows.push(Row::new(vec![
        Cell::from("Mon").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Tue").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Wed").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Thu").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Fri").style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Sat").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from("Sun").style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // Calendar days
    let mut current_day = 1;
    let mut week_row = Vec::new();

    // Fill in the offset days with empty cells
    for _ in 0..offset {
        week_row.push(Cell::from("  "));
    }

    // Fill in the actual days
    for day_of_week in offset..7 {
        if current_day <= days_in_month {
            let date = Date::from_calendar_date(
                app.calendar_view_year,
                app.calendar_view_month,
                current_day,
            )
            .unwrap();

            let is_selected = date == app.calendar_selected_date;
            let is_today = date == time::OffsetDateTime::now_utc().date();
            let is_current_view = date == app.current_date;

            let day_str = format!("{:2}", current_day);

            let style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(147, 51, 234)) // Purple highlight
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_current_view {
                Style::default()
                    .bg(Color::Rgb(30, 80, 150)) // Blue for current viewed day
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_today {
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::White)
            };

            week_row.push(Cell::from(day_str).style(style));
            current_day += 1;
        } else {
            week_row.push(Cell::from("  "));
        }

        if day_of_week == 6 {
            rows.push(Row::new(week_row.clone()));
            week_row.clear();
        }
    }

    // Continue filling remaining weeks
    while current_day <= days_in_month {
        for _ in 0..7 {
            if current_day <= days_in_month {
                let date = Date::from_calendar_date(
                    app.calendar_view_year,
                    app.calendar_view_month,
                    current_day,
                )
                .unwrap();

                let is_selected = date == app.calendar_selected_date;
                let is_today = date == time::OffsetDateTime::now_utc().date();
                let is_current_view = date == app.current_date;

                let day_str = format!("{:2}", current_day);

                let style = if is_selected {
                    Style::default()
                        .bg(Color::Rgb(147, 51, 234))
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else if is_current_view {
                    Style::default()
                        .bg(Color::Rgb(30, 80, 150))
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else if is_today {
                    Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else {
                    Style::default().fg(Color::White)
                };

                week_row.push(Cell::from(day_str).style(style));
                current_day += 1;
            } else {
                week_row.push(Cell::from("  "));
            }
        }
        rows.push(Row::new(week_row.clone()));
        week_row.clear();
    }

    let calendar_table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
        ],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Magenta))
            .title("📆 Select Date")
            .title_style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(Color::Rgb(25, 25, 38))),
    );

    frame.render_widget(calendar_table, chunks[1]);
}

fn get_days_in_month(month: time::Month, year: i32) -> u8 {
    use time::Month;
    match month {
        Month::January
        | Month::March
        | Month::May
        | Month::July
        | Month::August
        | Month::October
        | Month::December => 31,
        Month::April | Month::June | Month::September | Month::November => 30,
        Month::February => {
            if is_leap_year_render(year) {
                29
            } else {
                28
            }
        }
    }
}

fn is_leap_year_render(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn render_error_modal(frame: &mut Frame, app: &AppState) {
    use ratatui::widgets::Clear;
    use ratatui::text::Line;

    // Create a centered modal
    let area = frame.size();
    let width = area.width.min(70);
    let height = 10;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let modal_area = Rect {
        x,
        y,
        width,
        height,
    };

    // Clear the background
    frame.render_widget(Clear, modal_area);

    // Add a background block for the entire modal
    let bg_block = Block::default().style(Style::default().bg(Color::Rgb(40, 20, 20)));
    frame.render_widget(bg_block, modal_area);

    // Get error message
    let error_text = if let Some(ref error_msg) = app.last_error_message {
        error_msg.clone()
    } else {
        "Unknown error".to_string()
    };

    // Split modal into message and footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(modal_area);

    // Render error message
    let lines = vec![
        Line::from(""),
        Line::from(format!("  {}", error_text)).style(Style::default().fg(Color::White)),
        Line::from(""),
    ];

    let error_msg = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Red))
                .title("❌ ERROR")
                .title_style(
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(Color::Rgb(40, 20, 20))),
        );

    frame.render_widget(error_msg, chunks[0]);

    // Render help text
    let help = Paragraph::new("Press any key to dismiss")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(help, chunks[1]);
}
