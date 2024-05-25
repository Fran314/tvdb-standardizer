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
        _ => {}
    }
    Ok(())
}
pub fn handle_movie_selection_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc | KeyCode::Enter => app.state = State::FileExplorer,
        KeyCode::Backspace => app.pop_search_string(),
        KeyCode::Char(c) => app.append_to_search_string(c),
        KeyCode::Up => app.prev_movie(),
        KeyCode::Down => app.next_movie(),
        // Counter handlers
        KeyCode::Right => {}
        KeyCode::Left => {}
        // Other handlers you could add here.
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
