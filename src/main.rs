mod cli;
mod config;
mod integrations;
mod models;
mod storage;
mod timer;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use time::OffsetDateTime;
use ui::AppState;

fn main() -> Result<()> {
    // Try to parse CLI arguments
    let args: Vec<String> = std::env::args().collect();

    // If there are CLI arguments (beyond program name), run in CLI mode
    if args.len() > 1 {
        return run_cli();
    }

    // Otherwise, run TUI
    run_tui()
}

/// Run in CLI mode
fn run_cli() -> Result<()> {
    let cli = cli::Cli::parse();
    let storage = storage::Storage::new()?;
    cli::handle_command(cli.command, storage)
}

/// Run in TUI mode
fn run_tui() -> Result<()> {
    let today = OffsetDateTime::now_local()
        .context("Failed to get local time")?
        .date();
    let mut storage = storage::StorageManager::new()?;
    let day_data = storage.load_with_tracking(today)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new(day_data);

    // Load active timer if one exists
    if let Ok(Some(timer)) = storage.load_active_timer() {
        app.active_timer = Some(timer);
    }

    // Initialize last_file_modified with tracked time
    app.last_file_modified = storage.get_last_modified(&today);

    let result = run_app(&mut terminal, &mut app, &mut storage);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
    storage: &mut storage::StorageManager,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render::render(f, app))?;

        if app.should_quit {
            storage.save(&app.day_data)?;
            app.last_file_modified = storage.get_last_modified(&app.current_date);
            break;
        }

        if app.date_changed {
            storage.save(&app.day_data)?;
            let new_day_data = storage.load_with_tracking(app.current_date)?;
            app.load_new_day_data(new_day_data);
            app.last_file_modified = storage.get_last_modified(&app.current_date);
            continue; // Force redraw with new data before waiting for next event
        }

        // Poll for events with timeout to update timer display
        if event::poll(std::time::Duration::from_millis(500))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            handle_key_event(app, key, storage);
        }
        // If no event (timeout), check for external file changes and redraw with updated timer
        else {
            // Check if the file has been modified externally (e.g., by CLI)
            app.check_and_reload_if_modified(storage);
        }
    }

    Ok(())
}

fn handle_key_event(app: &mut AppState, key: KeyEvent, storage: &mut storage::StorageManager) {
    // Clear any previous error messages on new key press
    app.clear_error();

    match app.mode {
        ui::AppMode::Browse => match key.code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('?') => app.open_command_palette(),
            KeyCode::Char('C') => app.open_calendar(),
            KeyCode::Char('T') if app.config.has_integrations() => app.open_ticket_in_browser(),
            KeyCode::Char('L') if app.config.has_integrations() => app.open_worklog_in_browser(),
            // Timer keybindings
            KeyCode::Char('S') => {
                // Start/Stop toggle - Start if no timer active, Stop if timer is running
                if let Some(timer) = app.get_timer_status() {
                    use crate::timer::TimerStatus;
                    if matches!(timer.status, TimerStatus::Running | TimerStatus::Paused) {
                        if let Err(e) = app.stop_active_timer(storage) {
                            app.last_error_message = Some(e);
                        }
                    } else if let Err(e) = app.start_timer_for_selected(storage) {
                        app.last_error_message = Some(e);
                    }
                } else if let Err(e) = app.start_timer_for_selected(storage) {
                    app.last_error_message = Some(e);
                }
            }
            KeyCode::Char('P') => {
                // Pause/Resume toggle
                if let Some(timer) = app.get_timer_status() {
                    use crate::timer::TimerStatus;
                    match timer.status {
                        TimerStatus::Running => {
                            if let Err(e) = app.pause_active_timer(storage) {
                                app.last_error_message = Some(e);
                            }
                        }
                        TimerStatus::Paused => {
                            if let Err(e) = app.resume_active_timer(storage) {
                                app.last_error_message = Some(e);
                            }
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => app.move_selection_up(),
            KeyCode::Down | KeyCode::Char('j') => app.move_selection_down(),
            KeyCode::Left | KeyCode::Char('h') => app.move_field_left(),
            KeyCode::Right | KeyCode::Char('l') => app.move_field_right(),
            KeyCode::Enter | KeyCode::Char('i') => app.enter_edit_mode(),
            KeyCode::Char('c') => app.change_task_name(),
            KeyCode::Char('n') => {
                app.add_new_record();
                let _ = storage.save(&app.day_data);
                app.last_file_modified = storage.get_last_modified(&app.current_date);
            }
            KeyCode::Char('b') => {
                app.add_break();
                let _ = storage.save(&app.day_data);
                app.last_file_modified = storage.get_last_modified(&app.current_date);
            }
            KeyCode::Char('d') => {
                app.delete_selected_record();
                let _ = storage.save(&app.day_data);
                app.last_file_modified = storage.get_last_modified(&app.current_date);
            }
            KeyCode::Char('v') => app.enter_visual_mode(),
            KeyCode::Char('t') => app.set_current_time_on_field(),
            KeyCode::Char('u') => {
                app.undo();
                let _ = storage.save(&app.day_data);
                app.last_file_modified = storage.get_last_modified(&app.current_date);
            }
            KeyCode::Char('r') => {
                app.redo();
                let _ = storage.save(&app.day_data);
                app.last_file_modified = storage.get_last_modified(&app.current_date);
            }
            KeyCode::Char('s') => {
                let _ = storage.save(&app.day_data);
                app.last_file_modified = storage.get_last_modified(&app.current_date);
            }
            KeyCode::Char('[') => app.navigate_to_previous_day(),
            KeyCode::Char(']') => app.navigate_to_next_day(),
            _ => {}
        },
        ui::AppMode::Edit => match key.code {
            KeyCode::Esc => app.exit_edit_mode(),
            KeyCode::Tab => app.next_field(),
            KeyCode::Enter => {
                let _ = app.save_edit();
                let _ = storage.save(&app.day_data);
                app.last_file_modified = storage.get_last_modified(&app.current_date);
            }
            KeyCode::Backspace => app.handle_backspace(),
            KeyCode::Char(c) => app.handle_char_input(c),
            _ => {}
        },
        ui::AppMode::Visual => match key.code {
            KeyCode::Esc => app.exit_visual_mode(),
            KeyCode::Up | KeyCode::Char('k') => app.move_selection_up(),
            KeyCode::Down | KeyCode::Char('j') => app.move_selection_down(),
            KeyCode::Char('d') => app.delete_visual_selection(),
            _ => {}
        },
        ui::AppMode::CommandPalette => match key.code {
            KeyCode::Esc => app.close_command_palette(),
            KeyCode::Up => app.move_command_palette_up(),
            KeyCode::Down => {
                let filtered_count = app.get_filtered_commands().len();
                app.move_command_palette_down(filtered_count);
            }
            KeyCode::Enter => {
                if let Some(action) = app.execute_selected_command() {
                    execute_command_action(app, action, storage);
                }
            }
            KeyCode::Backspace => app.handle_command_palette_backspace(),
            KeyCode::Char(c) => app.handle_command_palette_char(c),
            _ => {}
        },
        ui::AppMode::Calendar => match key.code {
            KeyCode::Esc => app.close_calendar(),
            KeyCode::Enter => app.calendar_select_date(),
            KeyCode::Left | KeyCode::Char('h') => app.calendar_navigate_left(),
            KeyCode::Right | KeyCode::Char('l') => app.calendar_navigate_right(),
            KeyCode::Up | KeyCode::Char('k') => app.calendar_navigate_up(),
            KeyCode::Down | KeyCode::Char('j') => app.calendar_navigate_down(),
            KeyCode::Char('<') | KeyCode::Char(',') | KeyCode::Char('[') => {
                app.calendar_previous_month()
            }
            KeyCode::Char('>') | KeyCode::Char('.') | KeyCode::Char(']') => {
                app.calendar_next_month()
            }
            _ => {}
        },
        ui::AppMode::TaskPicker => match key.code {
            KeyCode::Esc => app.close_task_picker(),
            KeyCode::Up => app.move_task_picker_up(),
            KeyCode::Down => {
                let filtered_tasks = app.get_filtered_task_names();
                app.move_task_picker_down(filtered_tasks.len());
            }
            KeyCode::Enter => app.select_task_from_picker(),
            KeyCode::Backspace => app.handle_task_picker_backspace(),
            KeyCode::Char(c) => app.handle_task_picker_char(c),
            _ => {}
        },
    }
}

fn execute_command_action(
    app: &mut AppState,
    action: ui::app_state::CommandAction,
    storage: &mut storage::StorageManager,
) {
    use ui::app_state::CommandAction;

    match action {
        CommandAction::MoveUp => app.move_selection_up(),
        CommandAction::MoveDown => app.move_selection_down(),
        CommandAction::MoveLeft => app.move_field_left(),
        CommandAction::MoveRight => app.move_field_right(),
        CommandAction::Edit => app.enter_edit_mode(),
        CommandAction::Change => app.change_task_name(),
        CommandAction::New => {
            app.add_new_record();
            let _ = storage.save(&app.day_data);
            app.last_file_modified = storage.get_last_modified(&app.current_date);
        }
        CommandAction::Break => {
            app.add_break();
            let _ = storage.save(&app.day_data);
            app.last_file_modified = storage.get_last_modified(&app.current_date);
        }
        CommandAction::Delete => {
            app.delete_selected_record();
            let _ = storage.save(&app.day_data);
            app.last_file_modified = storage.get_last_modified(&app.current_date);
        }
        CommandAction::Visual => app.enter_visual_mode(),
        CommandAction::SetNow => app.set_current_time_on_field(),
        CommandAction::Undo => {
            app.undo();
            let _ = storage.save(&app.day_data);
            app.last_file_modified = storage.get_last_modified(&app.current_date);
        }
        CommandAction::Redo => {
            app.redo();
            let _ = storage.save(&app.day_data);
            app.last_file_modified = storage.get_last_modified(&app.current_date);
        }
        CommandAction::Save => {
            let _ = storage.save(&app.day_data);
            app.last_file_modified = storage.get_last_modified(&app.current_date);
        }
        CommandAction::StartTimer => {
            if let Err(e) = app.start_timer_for_selected(storage) {
                app.last_error_message = Some(format!("Failed to start timer: {}", e));
            }
        }
        CommandAction::PauseTimer => {
            #[allow(clippy::collapsible_if)]
            if app
                .active_timer
                .as_ref()
                .is_some_and(|t| matches!(t.status, crate::timer::TimerStatus::Running))
            {
                if let Err(e) = app.pause_active_timer(storage) {
                    app.last_error_message = Some(format!("Failed to pause timer: {}", e));
                }
            } else if app
                .active_timer
                .as_ref()
                .is_some_and(|t| matches!(t.status, crate::timer::TimerStatus::Paused))
            {
                if let Err(e) = app.resume_active_timer(storage) {
                    app.last_error_message = Some(format!("Failed to resume timer: {}", e));
                }
            }
        }
        CommandAction::Quit => app.should_quit = true,
    }
}
