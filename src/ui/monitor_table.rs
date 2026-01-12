use crate::app::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(" Monitor ({} commands) ", app.monitor_entries.len());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if app.monitor_entries.is_empty() {
        let msg = if app.monitor_active {
            "Monitor is active but no commands captured yet.\n\nRun Redis commands in another terminal to see them here.\n\nExample: redis-cli SET mykey myvalue"
        } else {
            "Monitor not started. Switch to this view to begin monitoring."
        };
        let empty_msg = Paragraph::new(msg)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(empty_msg, inner_area);
        return;
    }

    let visible_height = inner_area.height as usize;
    let total_entries = app.monitor_entries.len();
    let scroll_offset = app.monitor_scroll;

    let visible_entries = app
        .monitor_entries
        .iter()
        .skip(scroll_offset)
        .take(visible_height)
        .enumerate();

    let mut lines = Vec::new();
    for (idx, entry) in visible_entries {
        let is_selected = scroll_offset + idx == app.selected_monitor_index;
        let style = if is_selected {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::styled(
                format!("[{}] ", entry.timestamp),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("DB:{} ", entry.db),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("{} ", entry.client),
                Style::default().fg(Color::Magenta),
            ),
            Span::styled(&entry.command, Style::default().fg(Color::Cyan)),
        ])
        .style(style);

        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner_area);

    // Render scrollbar
    if total_entries > visible_height {
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(total_entries)
            .position(scroll_offset);

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        f.render_stateful_widget(scrollbar, inner_area, &mut scrollbar_state);
    }
}
