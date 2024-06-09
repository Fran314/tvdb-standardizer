use crate::app::{App, AppResult, State};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].

pub fn handle_file_explorer_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => app.quit(),
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::Enter => app.process_selection(),
        KeyCode::Char('h') | KeyCode::Left => app.exit_dir(),
        KeyCode::Char('j') | KeyCode::Down => app.next_dir_entry(),
        KeyCode::Char('k') | KeyCode::Up => app.prev_dir_entry(),
        KeyCode::Char('l') | KeyCode::Right => app.enter_dir(),
        KeyCode::Char('s') => app.state = State::MovieSelection,
        KeyCode::Char('m') => app.change_mode(),
        KeyCode::Char('t') => app.change_target(),
        _ => {}
    }
    Ok(())
}
pub fn handle_movie_selection_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc | KeyCode::Enter => app.state = State::FileExplorer,
        KeyCode::Tab => app.ui_next(),
        KeyCode::BackTab => app.ui_prev(),
        KeyCode::Char(c) => app.ui_add(c),
        KeyCode::Backspace => app.ui_remove(),
        KeyCode::Left => app.ui_left(),
        KeyCode::Right => app.ui_right(),
        KeyCode::Up => app.select_prev(),
        KeyCode::Down => app.select_next(),
        _ => {}
    }
    Ok(())
}
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.state {
        State::FileExplorer => handle_file_explorer_key_events(key_event, app),
        State::MovieSelection => handle_movie_selection_key_events(key_event, app),
    }
}
