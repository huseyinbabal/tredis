use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Context info
            Constraint::Percentage(25), // Stats
            Constraint::Percentage(20), // Keybindings 1
            Constraint::Percentage(20), // Keybindings 2
            Constraint::Percentage(15), // Logo
        ])
        .split(area);

    render_context_column(f, app, columns[0]);
    render_stats_column(f, app, columns[1]);
    render_keybindings_col1(f, app, columns[2]);
    render_keybindings_col2(f, columns[3]);
    render_logo(f, columns[4]);
}

fn render_context_column(f: &mut Frame, app: &App, area: Rect) {
    let lines = vec![
        Line::from(vec![
            Span::styled("Server:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.current_server_name(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Resource:", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!(" {}", app.active_resource),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}

fn render_stats_column(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![
        Line::from(vec![
            Span::styled("DB:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.connection_config.db.to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Keys:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.pagination.total_keys.to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    // Page indicator
    lines.push(Line::from(vec![
        Span::styled("Page:    ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", app.pagination.cursor_stack.len() + 1),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        if app.pagination.next_cursor != 0 {
            Span::styled("+", Style::default().fg(Color::Yellow))
        } else {
            Span::raw("")
        },
    ]));

    lines.push(Line::from(vec![
        Span::styled("Version: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "7.0.0",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}

fn render_keybindings_col1(f: &mut Frame, app: &App, area: Rect) {
    let bindings = match app.active_resource.as_str() {
        "servers" => vec![
            ("<c>", "Connect"),
            ("<d>", "Describe"),
            ("<a>", "Add"),
            ("<C-d>", "Delete"),
        ],
        "keys" => vec![
            ("<d>", "Describe"),
            ("<]>", "Next Page"),
            ("<[>", "Prev Page"),
            ("</>", "Filter"),
        ],
        "streams" => vec![
            ("<d>", "Describe"),
            ("<c>", "Consume"),
            ("<R>", "Refresh"),
            ("", ""),
        ],
        "monitor" => vec![("<j/k>", "Scroll"), ("<R>", "Clear"), ("", ""), ("", "")],
        "info" => vec![
            ("<j/k>", "Scroll"),
            ("</>", "Search"),
            ("<n/N>", "Next/Prev"),
            ("<R>", "Refresh"),
        ],
        "pubsub" => vec![
            ("<s>", "Test Subscribe"),
            ("<R>", "Refresh"),
            ("<Esc>", "Stop"),
            ("", ""),
        ],
        _ => vec![
            ("<j/k>", "Navigate"),
            ("<R>", "Refresh"),
            ("", ""),
            ("", ""),
        ],
    };

    let lines: Vec<Line> = bindings
        .iter()
        .map(|(key, desc)| {
            if key.is_empty() {
                Line::from("")
            } else {
                Line::from(vec![
                    Span::styled(format!("{:<9}", key), Style::default().fg(Color::Yellow)),
                    Span::styled(*desc, Style::default().fg(Color::DarkGray)),
                ])
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}

fn render_keybindings_col2(f: &mut Frame, area: Rect) {
    let bindings = vec![
        ("<:>", "Command"),
        ("<q>", "Quit"),
        ("<ctrl-c>", "Force Quit"),
    ];

    let lines: Vec<Line> = bindings
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(format!("{:<10}", key), Style::default().fg(Color::Yellow)),
                Span::styled(*desc, Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}

fn render_logo(f: &mut Frame, area: Rect) {
    let logo = vec![
        Line::from(Span::styled(
            "▀█▀ █▀▄ █▀▀ █▀▄ █ █▀",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            " █  █▀▄ ██▄ █▄▀ █ ▄█",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "         Redis TUI",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            format!("           v{}", crate::VERSION),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(logo).alignment(Alignment::Left);
    f.render_widget(paragraph, area);
}
