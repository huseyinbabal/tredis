use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(" PubSub Channels ({}) ", app.pubsub_channels.len());

    let info = Span::styled(
        " Read-only view • Shows active channels with subscriber counts • Press <R> to refresh ",
        Style::default()
            .fg(Color::DarkGray)
            .bg(Color::Rgb(40, 40, 40)),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .title_bottom(info)
        .title_alignment(Alignment::Center);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    if app.pubsub_channels.is_empty() {
        let empty_msg = Paragraph::new(
            "No active PubSub channels found.\n\n\
            Channels appear when they have active subscribers.\n\
            Publish to create: redis-cli PUBLISH mychannel \"hello\"\n\n\
            Press <R> to refresh",
        )
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
        f.render_widget(empty_msg, inner_area);
        return;
    }

    let header_cells = ["Channel Name", "Subscribers"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1);

    let rows = app.pubsub_channels.iter().map(|channel| {
        let cells = vec![
            Cell::from(channel.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(channel.subscribers.to_string()).style(Style::default().fg(Color::Green)),
        ];
        Row::new(cells)
    });

    let widths = [Constraint::Percentage(70), Constraint::Percentage(30)];

    let table = Table::new(rows, widths).header(header).row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    state.select(Some(app.selected_pubsub_index));

    f.render_stateful_widget(table, inner_area, &mut state);
}
