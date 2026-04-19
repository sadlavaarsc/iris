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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let main_area = chunks[0];
    let status_area = chunks[1];

    if let Some(ref mut state) = app.image_state {
        let image = StatefulImage::default().resize(Resize::Fit(None));
        f.render_stateful_widget(image, main_area, state);
    } else {
        let msg = Paragraph::new("Loading image...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(msg, main_area);
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
