use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(" Clients ({}) ", app.clients.len());

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

    let header_cells = ["ID", "Address", "Name", "Age", "Idle", "Flags", "DB", "Cmd"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1);

    let rows = app.clients.iter().map(|item| {
        let cells = vec![
            Cell::from(item.id.clone()),
            Cell::from(item.addr.clone()),
            Cell::from(item.name.clone()),
            Cell::from(item.age.clone()),
            Cell::from(item.idle.clone()),
            Cell::from(item.flags.clone()),
            Cell::from(item.db.clone()),
            Cell::from(item.cmd.clone()),
        ];
        Row::new(cells)
    });

    let widths = [
        Constraint::Length(5),
        Constraint::Length(25),
        Constraint::Length(15),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(5),
        Constraint::Length(15),
    ];

    let table = Table::new(rows, widths).header(header).row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    state.select(Some(app.selected_client_index));

    f.render_stateful_widget(table, inner_area, &mut state);
}
