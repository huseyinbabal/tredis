use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table, TableState,
    },
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // If consumer is active, show messages view
    if app.stream_active {
        render_stream_messages(f, app, area);
        return;
    }

    let title = format!(" Redis Streams ({}) ", app.streams.len());

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

    if app.streams.is_empty() {
        let empty_msg = ratatui::widgets::Paragraph::new(
            "No streams found. Create one with: XADD mystream * field value",
        )
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
        f.render_widget(empty_msg, inner_area);
        return;
    }

    let header_cells = ["Stream Name", "Length", "First Entry ID", "Last Entry ID"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1);

    let rows = app.streams.iter().map(|item| {
        let cells = vec![
            Cell::from(item.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(item.length.to_string()).style(Style::default().fg(Color::Green)),
            Cell::from(item.first_entry_id.clone()).style(Style::default().fg(Color::White)),
            Cell::from(item.last_entry_id.clone()).style(Style::default().fg(Color::White)),
        ];
        Row::new(cells)
    });

    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(10),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
    ];

    let table = Table::new(rows, widths).header(header).row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    state.select(Some(app.selected_stream_index));

    f.render_stateful_widget(table, inner_area, &mut state);
}

fn render_stream_messages(f: &mut Frame, app: &App, area: Rect) {
    // Only log when there are messages (not every render)
    if !app.stream_messages.is_empty() {
        crate::log!(
            crate::LogLevel::Info,
            "[UI] Rendering {} stream messages - First: {} {:?}",
            app.stream_messages.len(),
            app.stream_messages[0].id,
            app.stream_messages[0].fields
        );
    }

    let stream_name = if !app.streams.is_empty() {
        &app.streams[app.selected_stream_index].name
    } else {
        "Unknown"
    };

    let title = format!(
        " Consuming: {} (Group: {}) - {} messages ",
        stream_name,
        app.stream_consumer_group,
        app.stream_messages.len()
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if app.stream_messages.is_empty() {
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string());

        let msg = format!(
            "Waiting for messages on stream: {}\n\n\
            Consumer Group: {}\n\
            Consumer Name: {}\n\n\
            Press Esc to stop consuming",
            stream_name,
            app.stream_consumer_group,
            format!("tredis_{}", hostname)
        );
        let empty_msg = Paragraph::new(msg)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(empty_msg, inner_area);
        return;
    }

    let visible_height = inner_area.height as usize;
    let total_messages = app.stream_messages.len();
    let scroll_offset = app.stream_scroll;

    let visible_messages = app
        .stream_messages
        .iter()
        .skip(scroll_offset)
        .take(visible_height);

    let mut lines = Vec::new();
    for msg in visible_messages {
        let fields_str: Vec<String> = msg
            .fields
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        let line = Line::from(vec![
            Span::styled(format!("[{}] ", msg.id), Style::default().fg(Color::Yellow)),
            Span::styled(fields_str.join(", "), Style::default().fg(Color::White)),
        ]);

        lines.push(line);
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner_area);

    // Render scrollbar
    if total_messages > visible_height {
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(total_messages)
            .position(scroll_offset);

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        f.render_stateful_widget(scrollbar, inner_area, &mut scrollbar_state);
    }
}
