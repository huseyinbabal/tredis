use crate::app::App;
use crate::model::ServerType;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["", "Name", "Type", "Version", "Status"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells).style(Style::default()).height(1);

    let current_server_name = app.current_server.as_ref().map(|s| s.name.as_str());

    let rows = app
        .tredis_config
        .servers
        .iter()
        .enumerate()
        .map(|(idx, server)| {
            let is_current = current_server_name == Some(server.name.as_str());
            let is_selected = idx == app.selected_server_index;

            let indicator = if is_current { "●" } else { "" };
            let status = if is_current { "Connected" } else { "" };

            // Get server type and version from info
            let (server_type, version) = if let Some(ref info) = server.info {
                let type_str = match info.server_type {
                    ServerType::Standalone => "Standalone",
                    ServerType::Cluster => {
                        if let Some(size) = info.cluster_size {
                            // We'll just show "Cluster" - size shown in describe
                            "Cluster"
                        } else {
                            "Cluster"
                        }
                    }
                    ServerType::Sentinel => "Sentinel",
                };
                (type_str, info.redis_version.as_str())
            } else {
                ("Unknown", "-")
            };

            // Color for server type
            let type_color = match server_type {
                "Standalone" => Color::Blue,
                "Cluster" => Color::Magenta,
                "Sentinel" => Color::Yellow,
                _ => Color::DarkGray,
            };

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(indicator).style(Style::default().fg(Color::Green)),
                Cell::from(server.name.clone()),
                Cell::from(server_type).style(Style::default().fg(type_color)),
                Cell::from(version).style(Style::default().fg(Color::Cyan)),
                Cell::from(status).style(Style::default().fg(Color::Green)),
            ])
            .style(style)
            .height(1)
        });

    let widths = [
        Constraint::Length(2),
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
    ];

    let server_count = app.tredis_config.servers.len();
    let title = format!(" Servers ({}) ", server_count);

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(title)
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = TableState::default();
    state.select(Some(app.selected_server_index));

    f.render_stateful_widget(table, area, &mut state);
}
