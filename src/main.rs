mod models;
mod storage;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use time::OffsetDateTime;
use ui::AppState;

fn main() -> Result<()> {
    let today = OffsetDateTime::now_utc().date();
    let storage = storage::Storage::new()?;
    let day_data = storage.load(&today)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new(day_data);

    let result = run_app(&mut terminal, &mut app, &storage);

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
    storage: &storage::Storage,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render::render(f, app))?;

        if app.should_quit {
            storage.save(&app.day_data)?;
            break;
        }

        if app.date_changed {
            storage.save(&app.day_data)?;
            let new_day_data = storage.load(&app.current_date)?;
            app.load_new_day_data(new_day_data);
            continue; // Force redraw with new data before waiting for next event
        }

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            handle_key_event(app, key, storage);
        }
    }

    Ok(())
}

fn handle_key_event(app: &mut AppState, key: KeyEvent, storage: &storage::Storage) {
    match app.mode {
        ui::AppMode::Browse => match key.code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('?') => app.open_command_palette(),
            KeyCode::Char('C') => app.open_calendar(),
            KeyCode::Up | KeyCode::Char('k') => app.move_selection_up(),
            KeyCode::Down | KeyCode::Char('j') => app.move_selection_down(),
            KeyCode::Left | KeyCode::Char('h') => app.move_field_left(),
            KeyCode::Right | KeyCode::Char('l') => app.move_field_right(),
            KeyCode::Enter | KeyCode::Char('i') => app.enter_edit_mode(),
            KeyCode::Char('c') => app.change_task_name(),
            KeyCode::Char('n') => app.add_new_record(),
            KeyCode::Char('b') => app.add_break(),
            KeyCode::Char('d') => app.delete_selected_record(),
            KeyCode::Char('v') => app.enter_visual_mode(),
            KeyCode::Char('t') => app.set_current_time_on_field(),
            KeyCode::Char('u') => app.undo(),
            KeyCode::Char('r') => app.redo(),
            KeyCode::Char('s') => {
                let _ = storage.save(&app.day_data);
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
        ui::AppMode::CommandPalette => {
            match key.code {
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
            }
        }
        ui::AppMode::Calendar => match key.code {
            KeyCode::Esc => app.close_calendar(),
            KeyCode::Enter => app.calendar_select_date(),
            KeyCode::Left | KeyCode::Char('h') => app.calendar_navigate_left(),
            KeyCode::Right | KeyCode::Char('l') => app.calendar_navigate_right(),
            KeyCode::Up | KeyCode::Char('k') => app.calendar_navigate_up(),
            KeyCode::Down | KeyCode::Char('j') => app.calendar_navigate_down(),
            KeyCode::Char('<') | KeyCode::Char(',') | KeyCode::Char('[') => app.calendar_previous_month(),
            KeyCode::Char('>') | KeyCode::Char('.') | KeyCode::Char(']') => app.calendar_next_month(),
            _ => {}
        },
    }
}

fn execute_command_action(app: &mut AppState, action: ui::app_state::CommandAction, storage: &storage::Storage) {
    use ui::app_state::CommandAction;
    
    match action {
        CommandAction::MoveUp => app.move_selection_up(),
        CommandAction::MoveDown => app.move_selection_down(),
        CommandAction::MoveLeft => app.move_field_left(),
        CommandAction::MoveRight => app.move_field_right(),
        CommandAction::Edit => app.enter_edit_mode(),
        CommandAction::Change => app.change_task_name(),
        CommandAction::New => app.add_new_record(),
        CommandAction::Break => app.add_break(),
        CommandAction::Delete => app.delete_selected_record(),
        CommandAction::Visual => app.enter_visual_mode(),
        CommandAction::SetNow => app.set_current_time_on_field(),
        CommandAction::Undo => app.undo(),
        CommandAction::Redo => app.redo(),
        CommandAction::Save => {
            let _ = storage.save(&app.day_data);
        }
        CommandAction::Quit => app.should_quit = true,
    }
}
