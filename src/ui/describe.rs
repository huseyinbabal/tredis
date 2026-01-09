use crate::app::App;
use crate::model::KeyValue;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let key_info = if !app.scan_result.is_empty() {
        Some(&app.scan_result[app.selected_key_index])
    } else {
        None
    };

    let title = if let Some(info) = key_info {
        format!(" Describe: {} ({}) ", info.key, info.key_type)
    } else {
        " Describe ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let content_text = match &app.describe_data {
        KeyValue::String(s) => s.clone(),
        KeyValue::List(l) => serde_json::to_string_pretty(l).unwrap_or_default(),
        KeyValue::Set(s) => serde_json::to_string_pretty(s).unwrap_or_default(),
        KeyValue::ZSet(z) => serde_json::to_string_pretty(z).unwrap_or_default(),
        KeyValue::Hash(h) => serde_json::to_string_pretty(h).unwrap_or_default(),
        KeyValue::Stream(_) => "Stream data...".to_string(),
        KeyValue::None => "No data loaded.".to_string(),
        KeyValue::Error(e) => format!("Error: {}", e),
    };

    let lines: Vec<Line> = content_text
        .lines()
        .map(|l| Line::from(Span::styled(l, Style::default().fg(Color::White))))
        .collect();

    let scroll = app.describe_scroll as u16;
    let paragraph = Paragraph::new(lines).scroll((scroll, 0));

    f.render_widget(paragraph, inner_area);
}
