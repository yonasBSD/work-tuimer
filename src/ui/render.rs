use crate::ui::AppState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(frame.size());

    render_header(frame, chunks[0], app);
    render_records(frame, chunks[1], app);
    render_grouped_totals(frame, chunks[2], app);
    render_footer(frame, chunks[3], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &AppState) {
    let date_str = format!("{}", app.day_data.date);
    
    let total_minutes: u32 = app.day_data.work_records.values()
        .map(|r| r.total_minutes)
        .sum();
    let total_hours = total_minutes / 60;
    let total_mins = total_minutes % 60;
    
    let header_text = format!(
        "WorkTimer - {}                                                   Total: {}h {:02}m",
        date_str, total_hours, total_mins
    );
    
    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    
    frame.render_widget(header, area);
}

fn render_records(frame: &mut Frame, area: Rect, app: &AppState) {
    let records = app.day_data.get_sorted_records();
    
    let items: Vec<ListItem> = records
        .iter()
        .enumerate()
        .map(|(i, record)| {
            let is_selected = i == app.selected_index;
            let is_editing = matches!(app.mode, crate::ui::AppMode::Edit) && is_selected;
            let is_in_visual = matches!(app.mode, crate::ui::AppMode::Visual) && app.is_in_visual_selection(i);
            
            let style = if is_in_visual {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else if is_selected {
                Style::default().fg(Color::Gray)
            } else {
                Style::default()
            };

            let (name_display, start_display, end_display) = if is_editing {
                match app.edit_field {
                    crate::ui::EditField::Name => (
                        format!("[{}]", app.input_buffer),
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
                                record.name.clone(),
                                display,
                                record.end.to_string(),
                            ),
                            crate::ui::EditField::End => (
                                record.name.clone(),
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
                        format!("<{}>", record.name),
                        record.start.to_string(),
                        record.end.to_string(),
                    ),
                    crate::ui::EditField::Start => (
                        record.name.clone(),
                        format!("<{}>", record.start),
                        record.end.to_string(),
                    ),
                    crate::ui::EditField::End => (
                        record.name.clone(),
                        record.start.to_string(),
                        format!("<{}>", record.end),
                    ),
                }
            } else {
                (record.name.clone(), record.start.to_string(), record.end.to_string())
            };

            let content = format!(
                "{:20} | {} - {} | {}",
                name_display,
                start_display,
                end_display,
                record.format_duration()
            );

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Work Records"));

    frame.render_widget(list, area);
}

fn render_grouped_totals(frame: &mut Frame, area: Rect, app: &AppState) {
    let grouped = app.day_data.get_grouped_totals();
    
    let content: String = if grouped.is_empty() {
        "No records".to_string()
    } else {
        grouped
            .iter()
            .map(|(name, minutes)| {
                let hours = minutes / 60;
                let mins = minutes % 60;
                format!("{}: {}h {:02}m", name, hours, mins)
            })
            .collect::<Vec<_>>()
            .join("  |  ")
    };

    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Grouped by Task"));

    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &AppState) {
    let help_text = match app.mode {
        crate::ui::AppMode::Browse => {
            "↑/↓: Row | ←/→: Field | Enter: Edit | n: New | b: Break | d: Delete | v: Visual | T: Now | q: Quit"
        }
        crate::ui::AppMode::Edit => {
            "Tab: Next field | Enter: Save | Esc: Cancel"
        }
        crate::ui::AppMode::Visual => {
            "↑/↓: Extend selection | d: Delete | Esc: Exit visual"
        }
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}
