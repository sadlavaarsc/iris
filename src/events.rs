use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use std::time::Duration;

use crate::app::App;

pub fn handle_events(app: &mut App) -> anyhow::Result<bool> {
    if event::poll(Duration::from_millis(50))? {
        match event::read()? {
            Event::Key(key) => return Ok(handle_key(app, key)),
            Event::Mouse(mouse) => {
                handle_mouse(app, mouse);
                return Ok(false);
            }
            Event::Resize(_, _) => return Ok(false),
            _ => return Ok(false),
        }
    }
    Ok(false)
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
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') | KeyCode::Char('w') | KeyCode::Char('W') => {
            app.pan(0, -10);
            true
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Char('s') | KeyCode::Char('S') => {
            app.pan(0, 10);
            true
        }
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::Char('a') | KeyCode::Char('A') => {
            app.pan(-10, 0);
            true
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') | KeyCode::Char('d') | KeyCode::Char('D') => {
            app.pan(10, 0);
            true
        }
        KeyCode::Char('0') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.reset_view();
            true
        }
        _ => false,
    }
}

fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            app.zoom_in();
        }
        MouseEventKind::ScrollDown => {
            app.zoom_out();
        }
        _ => {}
    }
}
