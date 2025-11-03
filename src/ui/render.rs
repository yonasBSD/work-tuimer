use crate::ui::AppState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table, TableState},
    Frame,
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

    // Split the middle section horizontally for records and summary on wide screens,
    // vertically on narrow screens
    let is_wide = frame.size().width >= 100;
    let middle_chunks = if is_wide {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(chunks[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),
                Constraint::Length(15),
            ])
            .split(chunks[1])
    };

    render_header(frame, chunks[0], app);
    render_records(frame, middle_chunks[0], app);
    render_grouped_totals(frame, middle_chunks[1], app);
    render_footer(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &AppState) {
    let date_str = format!("{}", app.day_data.date);
    
    let total_minutes: u32 = app.day_data.work_records.values()
        .map(|r| r.total_minutes)
        .sum();
    let total_hours = total_minutes / 60;
    let total_mins = total_minutes % 60;
    
    // Create a more visual header with sections
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    let title_text = format!("⏱  WorkTimer - {}", date_str);
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
        );
    
    let total_text = format!("Total: {}h {:02}m", total_hours, total_mins);
    let total = Paragraph::new(total_text)
        .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green))
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
            let is_in_visual = matches!(app.mode, crate::ui::AppMode::Visual) && app.is_in_visual_selection(i);
            
            // Enhanced styling with more vibrant colors
            let style = if is_in_visual {
                Style::default()
                    .bg(Color::Rgb(70, 130, 180))  // Steel blue background
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .bg(Color::Rgb(40, 40, 60))    // Dark blue-gray background
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if i % 2 == 0 {
                Style::default().bg(Color::Rgb(25, 25, 35))  // Subtle alternating rows
            } else {
                Style::default()
            };

            // Add icon/emoji based on task type
            let icon = if record.name.to_lowercase().contains("break") {
                "☕"
            } else if record.name.to_lowercase().contains("meeting") {
                "👥"
            } else if record.name.to_lowercase().contains("code") || record.name.to_lowercase().contains("dev") {
                "💻"
            } else {
                "📋"
            };

            let (name_display, start_display, end_display) = if is_editing {
                match app.edit_field {
                    crate::ui::EditField::Name => (
                        format!("{} [{}]", icon, app.input_buffer),
                        record.start.to_string(),
                        record.end.to_string(),
                    ),
                    crate::ui::EditField::Start | crate::ui::EditField::End => {
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
                        
                        match app.edit_field {
                            crate::ui::EditField::Start => (
                                format!("{} {}", icon, record.name),
                                display,
                                record.end.to_string(),
                            ),
                            crate::ui::EditField::End => (
                                format!("{} {}", icon, record.name),
                                record.start.to_string(),
                                display,
                            ),
                            _ => unreachable!(),
                        }
                    }
                }
            } else if is_selected {
                match app.edit_field {
                    crate::ui::EditField::Name => (
                        format!("{} <{}>", icon, record.name),
                        record.start.to_string(),
                        record.end.to_string(),
                    ),
                    crate::ui::EditField::Start => (
                        format!("{} {}", icon, record.name),
                        format!("<{}>", record.start),
                        record.end.to_string(),
                    ),
                    crate::ui::EditField::End => (
                        format!("{} {}", icon, record.name),
                        record.start.to_string(),
                        format!("<{}>", record.end),
                    ),
                }
            } else {
                (format!("{} {}", icon, record.name), record.start.to_string(), record.end.to_string())
            };

            Row::new(vec![
                Cell::from(name_display),
                Cell::from(start_display).style(Style::default().fg(Color::LightGreen)),
                Cell::from(end_display).style(Style::default().fg(Color::LightRed)),
                Cell::from(record.format_duration()).style(Style::default().fg(Color::LightMagenta)),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
        ],
    )
    .header(
        Row::new(vec![
            Cell::from("📝 Task Name"),
            Cell::from("🕐 Start"),
            Cell::from("🕐 End"),
            Cell::from("⏱  Duration"),
        ])
        .style(Style::default()
            .fg(Color::Rgb(255, 215, 0))  // Gold color
            .add_modifier(Modifier::BOLD))
        .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Magenta))
            .title("📊 Work Records")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
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
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
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
                Cell::from(format!("{}h {:02}m", hours, mins))
                    .style(Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(65),
            Constraint::Percentage(35),
        ],
    )
    .header(
        Row::new(vec![
            Cell::from("Task"),
            Cell::from("Total"),
        ])
        .style(Style::default()
            .fg(Color::Rgb(255, 215, 0))
            .add_modifier(Modifier::BOLD))
        .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .title("📈 Summary")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    );

    frame.render_widget(table, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &AppState) {
    let (help_text, mode_color, mode_label) = match app.mode {
        crate::ui::AppMode::Browse => (
            "↑/↓: Row | ←/→: Field | Enter: Edit | c: Change | n: New | b: Break | d: Delete | v: Visual | T: Now | q: Quit",
            Color::Cyan,
            "BROWSE"
        ),
        crate::ui::AppMode::Edit => (
            "Tab: Next field | Enter: Save | Esc: Cancel",
            Color::Yellow,
            "EDIT"
        ),
        crate::ui::AppMode::Visual => (
            "↑/↓: Extend selection | d: Delete | Esc: Exit visual",
            Color::Magenta,
            "VISUAL"
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
                .padding(Padding::horizontal(1))
        );

    frame.render_widget(footer, area);
}
