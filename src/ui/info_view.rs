use crate::app::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Server Information ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let lines: Vec<Line> = app
        .info_data
        .iter()
        .map(|(k, v)| {
            if v.is_empty() {
                // Section header
                Line::from(vec![
                    Span::raw("\n"),
                    Span::styled(
                        k,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(format!("{:<30}", k), Style::default().fg(Color::DarkGray)),
                    Span::styled(v, Style::default().fg(Color::White)),
                ])
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines).scroll((app.info_scroll as u16, 0));
    f.render_widget(paragraph, inner_area);
}
