use crate::tui::app::{App, View};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        KeyCode::Esc => return app.quit(),
        KeyCode::Char('q') => return app.quit(),
        KeyCode::Char('c') if ctrl => return app.quit(),
        KeyCode::Tab => return app.toggle_view(),
        _ => {}
    }

    match app.view {
        View::Wizard => handle_wizard_key(app, key),
        View::Sniffer => handle_sniffer_key(app, key),
    }
}

fn handle_wizard_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(' ') => app.wizard.on_space(),
        KeyCode::Enter => app.wizard.on_accept(),
        KeyCode::Char('r') => app.wizard.on_retry(),
        KeyCode::Char('n') => app.wizard.on_skip(),
        KeyCode::Char('b') => app.wizard.on_rebaseline(),
        _ => {}
    }
}

fn handle_sniffer_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('c') => app.clear(),
        KeyCode::Char('p') | KeyCode::Char(' ') => app.toggle_pause(),
        KeyCode::Char('f') => app.toggle_follow(),
        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(1),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(1),
        KeyCode::PageUp => app.scroll_up(10),
        KeyCode::PageDown => app.scroll_down(10),
        KeyCode::Home | KeyCode::Char('g') => app.jump_top(),
        KeyCode::End | KeyCode::Char('G') => app.jump_bottom(),
        KeyCode::Char(d @ '1'..='9') => {
            let id = (d as u8) - b'1';
            app.toggle_iface_visibility(id);
        }
        _ => {}
    }
}
