use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use iris::{
    app::App,
    cli::Args,
    events,
    ui,
    viewer::ImageViewer,
    worker::{ImageResult, ImageWorker},
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

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        f.render_widget(image, f.area());
    })?;

    Ok(())
}

fn run_interactive(path: &std::path::Path) -> Result<()> {
    let viewer = ImageViewer::new(path)?;
    let (orig_w, orig_h) = viewer.original_size();

    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let viewer = viewer.query_stdio();

    eprintln!("[IRIS-BENCH] Protocol: {}", viewer.protocol_name());
    eprintln!("[IRIS-BENCH] Image: {}x{}", orig_w, orig_h);
    eprintln!("[IRIS-BENCH] FontSize: {:?}", viewer.font_size());
    eprintln!("[IRIS-BENCH] === START ===");

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(orig_w, orig_h);
    app.set_status(format!("{}x{}", orig_w, orig_h));

    let size = terminal.size()?;
    let term_area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
    let area = compute_image_area(term_area);
    app.image_area = area;

    let worker = ImageWorker::new(viewer);
    worker.request(app.scale, app.offset_x, app.offset_y, area);

    let res = run_app(&mut terminal, &mut app, &worker);

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

    eprintln!("[IRIS-BENCH] === END ===");

    Ok(())
}

/// Compute the image rendering area as a centered subset of the terminal.
/// This limits the Kitty/Sixel transmission size while keeping the image visible.
fn compute_image_area(term: Rect) -> Rect {
    // Use 60% of terminal width and 75% of terminal height (minus status bar)
    let avail_h = term.height.saturating_sub(1);
    let w = ((term.width as f32 * 0.6).ceil() as u16).max(10);
    let h = ((avail_h as f32 * 0.75).ceil() as u16).max(5);
    let x = (term.width.saturating_sub(w)) / 2;
    let y = (avail_h.saturating_sub(h)) / 2;
    Rect::new(x, y, w, h)
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    worker: &ImageWorker,
) -> Result<()> {
    let size = terminal.size()?;
    let term_area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
    let mut last_image_area = compute_image_area(term_area);
    app.image_area = last_image_area;

    let debounce = Duration::from_millis(75);
    let mut pending_update: Option<Instant> = None;
    let mut request_sent_at: Option<Instant> = None;
    let mut frame_count: u64 = 0;

    loop {
        frame_count += 1;
        let size = terminal.size()?;
        let term_area = ratatui::layout::Rect::new(0, 0, size.width, size.height);
        let image_area = compute_image_area(term_area);
        let area_changed = image_area != last_image_area;
        app.image_area = image_area;

        // Receive completed background work
        while let Some(result) = worker.try_recv() {
            let recv_elapsed = request_sent_at.map(|t| t.elapsed().as_secs_f64() * 1000.0);
            app.has_pending_work = false;
            match result {
                ImageResult::Ready(protocol) => {
                    app.image_state = Some(protocol);
                    if let Some(ms) = recv_elapsed {
                        eprintln!("[IRIS-BENCH] worker-ready  e2e={:.2}ms", ms);
                    }
                }
                ImageResult::Error(e) => {
                    if let Some(ms) = recv_elapsed {
                        eprintln!("[IRIS-BENCH] worker-error  e2e={:.2}ms err={}", ms, e);
                    }
                    app.set_status(format!("Error: {}", e));
                }
            }
            request_sent_at = None;
        }

        // Time the draw call
        let draw_start = Instant::now();
        terminal.draw(|f| ui::draw(f, app))?;
        let draw_ms = draw_start.elapsed().as_secs_f64() * 1000.0;
        if draw_ms > 1.0 {
            eprintln!(
                "[IRIS-BENCH] draw         area={}x{}  took={:.2}ms",
                image_area.width, image_area.height, draw_ms
            );
        }

        let now = Instant::now();

        // Process pending update if debounce elapsed
        if let Some(deadline) = pending_update {
            if now >= deadline {
                pending_update = None;
                if !app.has_pending_work {
                    let params_changed = app.scale != app.last_scale
                        || app.offset_x != app.last_offset_x
                        || app.offset_y != app.last_offset_y
                        || image_area != app.last_area;
                    if params_changed {
                        app.has_pending_work = true;
                        app.last_scale = app.scale;
                        app.last_offset_x = app.offset_x;
                        app.last_offset_y = app.offset_y;
                        app.last_area = image_area;
                        request_sent_at = Some(Instant::now());
                        worker.request(app.scale, app.offset_x, app.offset_y, image_area);
                    }
                }
            }
        }

        // Dynamic poll timeout
        let poll_timeout = pending_update
            .map(|d| d.saturating_duration_since(now).min(Duration::from_millis(50)))
            .unwrap_or(Duration::from_millis(50));

        if !event::poll(poll_timeout)? {
            continue;
        }

        // Drain events
        let drain_start = Instant::now();
        let mut changed = false;
        let mut event_count = 0;
        while event::poll(Duration::from_millis(0))? {
            let evt = event::read()?;
            event_count += 1;
            if events::handle_event(app, evt) {
                changed = true;
            }
            if app.should_quit {
                break;
            }
        }
        let drain_ms = drain_start.elapsed().as_secs_f64() * 1000.0;

        if app.should_quit {
            break;
        }

        if changed || area_changed {
            last_image_area = image_area;
            pending_update = Some(Instant::now() + debounce);
            if event_count > 1 || drain_ms > 1.0 {
                eprintln!(
                    "[IRIS-BENCH] events       count={}  drain={:.2}ms",
                    event_count, drain_ms
                );
            }
        }
    }

    eprintln!("[IRIS-BENCH] frames={}", frame_count);

    Ok(())
}
