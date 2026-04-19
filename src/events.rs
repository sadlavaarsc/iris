use std::path::Path;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

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
        KeyCode::Char('o') | KeyCode::Char('O') => {
            if let Err(e) = open_with_default(&app.image_path) {
                app.set_status(format!("Open failed: {}", e));
            } else {
                app.set_status("Opened with default viewer".to_string());
            }
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
        MouseEventKind::Down(MouseButton::Left) => {
            app.drag_last_col = Some(mouse.column);
            app.drag_last_row = Some(mouse.row);
            false
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if let (Some(last_col), Some(last_row)) = (app.drag_last_col, app.drag_last_row) {
                let dx = mouse.column as i32 - last_col as i32;
                let dy = mouse.row as i32 - last_row as i32;
                app.pan(-dx, -dy);
                app.drag_last_col = Some(mouse.column);
                app.drag_last_row = Some(mouse.row);
                true
            } else {
                false
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            app.drag_last_col = None;
            app.drag_last_row = None;
            false
        }
        MouseEventKind::ScrollUp => {
            if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                app.pan(-3, 0);
            } else {
                app.zoom_in();
            }
            true
        }
        MouseEventKind::ScrollDown => {
            if mouse.modifiers.contains(KeyModifiers::SHIFT) {
                app.pan(3, 0);
            } else {
                app.zoom_out();
            }
            true
        }
        MouseEventKind::ScrollLeft => {
            app.pan(-3, 0);
            true
        }
        MouseEventKind::ScrollRight => {
            app.pan(3, 0);
            true
        }
        _ => false,
    }
}

/// Open a file with the system's default application.
fn open_with_default(path: &Path) -> std::io::Result<std::process::Child> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(path)
            .spawn()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "opening with default viewer is not supported on this platform",
        ))
    }
}
