use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let show_filter = app.filter_active || !app.filter_text.is_empty();

    let (filter_area, table_area) = if show_filter {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, area)
    };

    if let Some(area) = filter_area {
        let filter_display = if app.filter_active {
            format!("/{}_", app.filter_text)
        } else {
            format!("/{}", app.filter_text)
        };

        let style = if app.filter_active {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let paragraph = Paragraph::new(Line::from(Span::styled(filter_display, style)));
        f.render_widget(paragraph, area);
    }

    let title = if app.selected_keys.is_empty() {
        format!(
            " Keys ({}/{}) [Page: {}] ",
            app.scan_result.len(),
            app.pagination.total_keys,
            app.pagination.cursor_stack.len() + 1
        )
    } else {
        format!(
            " Keys ({}/{}) [Page: {}] - {} selected ",
            app.scan_result.len(),
            app.pagination.total_keys,
            app.pagination.cursor_stack.len() + 1,
            app.selected_keys.len()
        )
    };

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

    let inner_area = block.inner(table_area);
    f.render_widget(block, table_area);

    let header_cells = ["Key", "Type", "TTL", "Memory"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1);

    let rows = app.scan_result.iter().map(|item| {
        let is_selected = app.selected_keys.contains(&item.key);

        let row_style = if is_selected {
            Style::default()
                .bg(Color::Green)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let cells = vec![
            Cell::from(item.key.clone()),
            Cell::from(item.key_type.clone()).style(if is_selected {
                Style::default().fg(Color::Black)
            } else {
                get_type_style(&item.key_type)
            }),
            Cell::from(item.ttl.to_string()),
            Cell::from(item.memory_usage.to_string()),
        ];
        Row::new(cells).style(row_style)
    });

    let widths = [
        Constraint::Percentage(50), // Key
        Constraint::Percentage(15), // Type
        Constraint::Percentage(15), // TTL
        Constraint::Percentage(20), // Memory
    ];

    let table = Table::new(rows, widths).header(header).row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    state.select(Some(app.selected_key_index));

    f.render_stateful_widget(table, inner_area, &mut state);
}

fn get_type_style(key_type: &str) -> Style {
    match key_type {
        "string" => Style::default().fg(Color::Cyan),
        "hash" => Style::default().fg(Color::Magenta),
        "list" => Style::default().fg(Color::Blue),
        "set" => Style::default().fg(Color::Green),
        "zset" => Style::default().fg(Color::Yellow),
        "stream" => Style::default().fg(Color::LightRed),
        _ => Style::default().fg(Color::White),
    }
}
