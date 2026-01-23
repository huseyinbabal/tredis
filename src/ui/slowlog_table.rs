use crate::app::App;
use chrono::{TimeZone, Utc};
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(" Slowlog ({}) ", app.slowlogs.len());

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

    let header_cells = ["ID", "Time", "Duration (μs)", "Command"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1);

    let rows = app.slowlogs.iter().map(|item| {
        let dt = Utc.timestamp_opt(item.timestamp, 0).unwrap();
        let time_str = dt.format("%H:%M:%S").to_string();

        let cells = vec![
            Cell::from(item.id.to_string()),
            Cell::from(time_str),
            Cell::from(item.duration.to_string()),
            Cell::from(item.command.clone()),
        ];
        Row::new(cells)
    });

    let widths = [
        Constraint::Length(5),
        Constraint::Length(10),
        Constraint::Length(15),
        Constraint::Min(20),
    ];

    let table = Table::new(rows, widths).header(header).row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    state.select(Some(app.selected_slowlog_index));

    f.render_stateful_widget(table, inner_area, &mut state);
}
