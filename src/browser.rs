use std::path::{Path, PathBuf};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{List, ListItem, ListState, Paragraph},
    Terminal,
};

pub struct BrowserState {
    pub entries: Vec<PathBuf>,
    pub list_state: ListState,
    pub should_quit: bool,
    pub selected: Option<PathBuf>,
}

impl BrowserState {
    pub fn new(dir: &Path) -> Result<Self> {
        let mut entries = collect_images(dir)?;
        entries.sort();

        let mut list_state = ListState::default();
        if !entries.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            entries,
            list_state,
            should_quit: false,
            selected: None,
        })
    }

    pub fn select_next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.entries.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_prev(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.entries.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn confirm(&mut self) {
        if let Some(i) = self.list_state.selected() {
            self.selected = self.entries.get(i).cloned();
        }
        self.should_quit = true;
    }
}

fn is_image(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    matches!(
        ext.as_deref(),
        Some(
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "tif" | "tga" | "ico"
                | "avif" | "heic" | "heif" | "pnm" | "ppm" | "pgm" | "pbm"
        )
    )
}

fn collect_images(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && is_image(&path) {
            entries.push(path);
        }
    }
    Ok(entries)
}

/// Run the file browser. Returns the selected image path, or None if user quit.
pub fn run_browser(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, dir: &Path) -> Result<Option<PathBuf>> {
    let mut state = BrowserState::new(dir)?;

    loop {
        terminal.draw(|f| draw_browser(f, &mut state, dir))?;

        if !event::poll(std::time::Duration::from_millis(50))? {
            continue;
        }

        while event::poll(std::time::Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => handle_key(&mut state, key),
                Event::Mouse(mouse) => handle_mouse(&mut state, mouse),
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        if state.should_quit {
            break;
        }
    }

    Ok(state.selected)
}

fn handle_key(state: &mut BrowserState, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            state.should_quit = true;
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
            state.select_next();
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
            state.select_prev();
        }
        KeyCode::Enter => {
            state.confirm();
        }
        _ => {}
    }
}

fn handle_mouse(state: &mut BrowserState, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollDown => state.select_next(),
        MouseEventKind::ScrollUp => state.select_prev(),
        _ => {}
    }
}

fn draw_browser(f: &mut ratatui::Frame, state: &mut BrowserState, dir: &Path) {
    let area = f.area();

    // Title bar
    let title_text = format!(" Iris  —  {}  ({} images) ", dir.display(), state.entries.len());
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan));
    f.render_widget(title, Rect::new(0, 0, area.width, 1));

    // Main list area
    let list_area = Rect::new(0, 1, area.width, area.height.saturating_sub(2));

    let items: Vec<ListItem> = state
        .entries
        .iter()
        .map(|path| {
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            ListItem::new(name)
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, list_area, &mut state.list_state);

    // Hint bar
    let hint_y = area.height.saturating_sub(1);
    let hint = Paragraph::new(" ↑/↓ or j/k: navigate  Enter: open  q: quit ")
        .style(Style::default().fg(Color::Gray));
    f.render_widget(hint, Rect::new(0, hint_y, area.width, 1));
}
