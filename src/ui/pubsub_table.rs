use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    if app.pubsub_subscribe_mode {
        // Subscribe mode - show channel input or messages
        if app.pubsub_subscribe_channel.is_empty() {
            render_subscribe_input(f, app, area);
        } else {
            render_subscribe_messages(f, app, area);
        }
    } else {
        // Normal mode - show channels list
        render_channels(f, app, area);
    }
}

fn render_channels(f: &mut Frame, app: &App, area: Rect) {
    let title = format!(" PubSub Channels ({}) ", app.pubsub_channels.len());

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

    if app.pubsub_channels.is_empty() {
        let msg = "No active PubSub channels.\n\n\
                   Redis only shows channels with active subscribers.\n\
                   To see a channel here, run in another terminal:\n\
                   redis-cli SUBSCRIBE <channel>\n\n\
                   Press 's' to subscribe to a test channel\n\
                   Press 'R' to refresh";
        let empty_msg = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(empty_msg, inner_area);
        return;
    }

    let header_cells = ["Channel", "Subscribers"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    });
    let header = Row::new(header_cells).height(1);

    let rows = app.pubsub_channels.iter().enumerate().map(|(idx, item)| {
        let is_selected = idx == app.selected_pubsub_index;
        let style = if is_selected {
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let cells = vec![
            Cell::from(item.name.clone()).style(Style::default().fg(Color::Cyan)),
            Cell::from(item.subscribers.to_string()).style(Style::default().fg(Color::Green)),
        ];
        Row::new(cells).style(style)
    });

    let widths = [Constraint::Percentage(70), Constraint::Percentage(30)];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = TableState::default();
    state.select(Some(app.selected_pubsub_index));
    f.render_stateful_widget(table, inner_area, &mut state);
}

fn render_subscribe_input(f: &mut Frame, app: &App, area: Rect) {
    // Dark background
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Subscribe to Channel ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    f.render_widget(block, area);

    // Center dialog
    let dialog_width = 50;
    let dialog_height = 7;
    let dialog_x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

    f.render_widget(Clear, dialog_area);

    let dialog_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Enter Channel Name ")
        .title_style(Style::default().fg(Color::Yellow));

    let inner = dialog_block.inner(dialog_area);
    f.render_widget(dialog_block, dialog_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    // Input field
    let input_text = format!("> {}_", app.pubsub_subscribe_input);
    let input = Paragraph::new(input_text).style(Style::default().fg(Color::White));
    f.render_widget(input, chunks[1]);

    // Help text
    let help = Paragraph::new("Enter: Subscribe | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[3]);
}

fn render_subscribe_messages(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(area);

    // Top: Info box with command
    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title(Span::styled(
            format!(" Subscribed to: {} ", app.pubsub_subscribe_channel),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner_info = info_block.inner(chunks[0]);
    f.render_widget(info_block, chunks[0]);

    let cmd = format!(
        "redis-cli PUBLISH {} \"your message\"",
        app.pubsub_subscribe_channel
    );
    let info_lines = vec![
        Line::from(vec![
            Span::styled("Publish with: ", Style::default().fg(Color::Yellow)),
            Span::styled(cmd, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc or q to stop",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let info_para = Paragraph::new(info_lines).alignment(Alignment::Center);
    f.render_widget(info_para, inner_info);

    // Bottom: Messages
    let msg_title = format!(" Messages ({}) ", app.pubsub_messages.len());
    let msg_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            msg_title,
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);

    let inner_msg = msg_block.inner(chunks[1]);
    f.render_widget(msg_block, chunks[1]);

    if app.pubsub_messages.is_empty() {
        let waiting = Paragraph::new("Waiting for messages...")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(waiting, inner_msg);
        return;
    }

    let visible_height = inner_msg.height as usize;
    let lines: Vec<Line> = app
        .pubsub_messages
        .iter()
        .take(visible_height)
        .map(|msg| {
            Line::from(vec![
                Span::styled(
                    format!("[{}] ", msg.timestamp),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(&msg.message, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, inner_msg);
}
