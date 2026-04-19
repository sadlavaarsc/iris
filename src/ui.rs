use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use ratatui_image::{Resize, StatefulImage};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let term_area = f.area();

    // Top info bar: filename + dimensions
    let top_area = ratatui::layout::Rect::new(0, 0, term_area.width, 1);

    // Bottom hint bar: scale/offset + controls
    let bottom_y = term_area.height.saturating_sub(1);
    let bottom_area = ratatui::layout::Rect::new(0, bottom_y, term_area.width, 1);

    // Image area comes from app (already avoids top and bottom bars)
    let image_area = app.image_area;

    if let Some(ref mut state) = app.image_state {
        let image = StatefulImage::default().resize(Resize::Fit(None));
        f.render_stateful_widget(image, image_area, state);
    } else {
        let msg = Paragraph::new("Loading image...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(msg, image_area);
    }

    // --- Top bar: filename + original size ---
    let name = if app.filename.len() > 24 {
        format!("{}…", &app.filename[..23])
    } else {
        app.filename.clone()
    };
    let top_spans = vec![
        Span::styled(
            format!(" {} ", name),
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::styled(
            format!(" {}x{} ", app.original_width, app.original_height),
            Style::default().fg(Color::Black).bg(Color::Green),
        ),
    ];
    let top_bar = Paragraph::new(Line::from(top_spans));
    f.render_widget(top_bar, top_area);

    // --- Bottom bar: status (left) + controls (right-aligned) ---
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(46)])
        .split(bottom_area);

    let status = Paragraph::new(Line::from(vec![Span::styled(
        format!(" {} ", app.status_message),
        Style::default().fg(Color::Black).bg(Color::Blue),
    )]));
    f.render_widget(status, bottom_chunks[0]);

    let controls = Paragraph::new(Line::from(vec![Span::styled(
        " +/- zoom  ←↑↓→ pan  o open  r reset  q quit ",
        Style::default().fg(Color::Gray),
    )]))
    .alignment(Alignment::Right);
    f.render_widget(controls, bottom_chunks[1]);
}
