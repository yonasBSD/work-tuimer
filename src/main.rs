mod models;
mod storage;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
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

        if let Event::Key(key) = event::read()? {
            handle_key_event(app, key, storage);
        }
    }

    Ok(())
}

fn handle_key_event(app: &mut AppState, key: KeyEvent, storage: &storage::Storage) {
    match app.mode {
        ui::AppMode::Browse => match key.code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Up | KeyCode::Char('k') => app.move_selection_up(),
            KeyCode::Down | KeyCode::Char('j') => app.move_selection_down(),
            KeyCode::Left | KeyCode::Char('h') => app.move_field_left(),
            KeyCode::Right | KeyCode::Char('l') => app.move_field_right(),
            KeyCode::Enter | KeyCode::Char('i') => app.enter_edit_mode(),
            KeyCode::Char('n') => app.add_new_record(),
            KeyCode::Char('b') => app.add_break(),
            KeyCode::Char('d') => app.delete_selected_record(),
            KeyCode::Char('v') => app.enter_visual_mode(),
            KeyCode::Char('T') => app.set_current_time_on_field(),
            KeyCode::Char('s') => {
                let _ = storage.save(&app.day_data);
            }
            _ => {}
        },
        ui::AppMode::Edit => match key.code {
            KeyCode::Esc => app.exit_edit_mode(),
            KeyCode::Tab => app.next_field(),
            KeyCode::Enter => {
                if let Err(_) = app.save_edit() {
                }
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
    }
}
