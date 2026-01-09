use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(" Access Control List ({}) ", app.acls.len());

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

    let header_cells = ["User", "Status", "Rules"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1);

    let rows = app.acls.iter().map(|item| {
        let status_style = if item.status == "on" {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        };

        let cells = vec![
            Cell::from(item.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(item.status.clone()).style(status_style),
            Cell::from(item.rules.clone()),
        ];
        Row::new(cells)
    });

    let widths = [
        Constraint::Percentage(20),
        Constraint::Percentage(10),
        Constraint::Percentage(70),
    ];

    let table = Table::new(rows, widths).header(header).row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    state.select(Some(app.selected_acl_index));

    f.render_stateful_widget(table, inner_area, &mut state);
}
