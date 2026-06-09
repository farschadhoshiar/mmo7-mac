use crate::tui::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Esc => app.quit(),
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') if ctrl => app.quit(),
        KeyCode::Char('c') => app.clear(),
        KeyCode::Char('p') | KeyCode::Char(' ') => app.toggle_pause(),
        KeyCode::Char('f') => app.toggle_follow(),
        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(1),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(1),
        KeyCode::PageUp => app.scroll_up(10),
        KeyCode::PageDown => app.scroll_down(10),
        KeyCode::Home | KeyCode::Char('g') => app.jump_top(),
        KeyCode::End | KeyCode::Char('G') => app.jump_bottom(),
        _ => {}
    }
}
