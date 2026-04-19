use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use crate::app::App;

pub fn handle_event(app: &mut App, event: Event) -> bool {
    match event {
        Event::Key(key) => handle_key(app, key),
        Event::Mouse(mouse) => handle_mouse(app, mouse),
        Event::Resize(_, _) => false,
        _ => false,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.should_quit = true;
            true
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            app.zoom_in();
            true
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            app.zoom_out();
            true
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.reset_view();
            true
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
            app.pan(0, -1);
            true
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
            app.pan(0, 1);
            true
        }
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('a') => {
            app.pan(-1, 0);
            true
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('d') => {
            app.pan(1, 0);
            true
        }
        KeyCode::Char('K') | KeyCode::Char('W') => {
            app.pan(0, -5);
            true
        }
        KeyCode::Char('J') | KeyCode::Char('S') => {
            app.pan(0, 5);
            true
        }
        KeyCode::Char('H') | KeyCode::Char('A') => {
            app.pan(-5, 0);
            true
        }
        KeyCode::Char('L') | KeyCode::Char('D') => {
            app.pan(5, 0);
            true
        }
        KeyCode::Char('0') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.reset_view();
            true
        }
        _ => false,
    }
}

fn handle_mouse(app: &mut App, mouse: MouseEvent) -> bool {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            app.zoom_in();
            true
        }
        MouseEventKind::ScrollDown => {
            app.zoom_out();
            true
        }
        _ => false,
    }
}
