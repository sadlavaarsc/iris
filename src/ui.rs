use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use ratatui_image::{Resize, StatefulImage};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let term_area = f.area();

    // Status bar at the bottom, full width
    let status_y = term_area.height.saturating_sub(1);
    let status_area = ratatui::layout::Rect::new(0, status_y, term_area.width, 1);

    // Use the constrained image area from app
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

    let status_spans = vec![
        Span::styled(
            format!(" {} ", app.status_message),
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::raw(" | "),
        Span::styled(
            " +/-: zoom  ←↑↓→: pan  r: reset  q: quit ",
            Style::default().fg(Color::Gray),
        ),
    ];
    let status = Paragraph::new(Line::from(status_spans));
    f.render_widget(status, status_area);
}
