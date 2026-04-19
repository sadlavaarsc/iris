mod app;
mod cli;
mod events;
mod ui;
mod viewer;

use std::io;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{
    app::App,
    cli::Args,
    viewer::ImageViewer,
};

fn main() -> Result<()> {
    let args = Args::parse();

    if args.no_interactive {
        run_static(&args.path)?;
    } else {
        run_interactive(&args.path)?;
    }

    Ok(())
}

fn run_static(path: &std::path::Path) -> Result<()> {
    let viewer = ImageViewer::new(path)?.query_stdio();
    let (_width, _height) = viewer.original_size();

    let area = ratatui::layout::Rect {
        x: 0,
        y: 0,
        width: u16::MAX,
        height: u16::MAX,
    };

    let mut protocol = viewer.create_static(1.0, 0, 0, area)?;
    let image = ratatui_image::Image::new(&mut protocol);

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        f.render_widget(image, f.area());
    })?;

    Ok(())
}

fn run_interactive(path: &std::path::Path) -> Result<()> {
    let mut viewer = ImageViewer::new(path)?;
    let (orig_w, orig_h) = viewer.original_size();

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Query terminal capabilities after entering alternate screen
    viewer = viewer.query_stdio();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(orig_w, orig_h);
    app.set_status(format!("{}x{}", orig_w, orig_h));

    let size = terminal.size()?;
    let area = ratatui::layout::Rect::new(0, 0, size.width, size.height);

    // Create initial image state
    if let Ok(state) = viewer.create_stateful(app.scale, app.offset_x, app.offset_y, area) {
        app.image_state = Some(state);
    }

    let res = run_app(
        &mut terminal,
        &mut app,
        &viewer,
    );

    terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    viewer: &ImageViewer,
) -> Result<()> {
    let size = terminal.size()?;
    let mut last_area = ratatui::layout::Rect::new(0, 0, size.width, size.height);

    loop {
        let size = terminal.size()?;
        let current_area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
        let area_changed = current_area != last_area;

        terminal.draw(|f| ui::draw(f, app))?;

        let changed = events::handle_events(app)?;

        if app.should_quit {
            break;
        }

        if changed || area_changed {
            last_area = current_area;
            match viewer.create_stateful(app.scale, app.offset_x, app.offset_y, current_area) {
                Ok(state) => app.image_state = Some(state),
                Err(e) => app.set_status(format!("Error: {}", e)),
            }
        }
    }

    Ok(())
}
