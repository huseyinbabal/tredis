use crate::app::{App, Mode, PendingActionType};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    if app.mode == Mode::Confirm {
        render_confirm_dialog(f, app);
    }
}

fn render_confirm_dialog(f: &mut Frame, app: &App) {
    let Some(pending) = &app.pending_action else {
        return;
    };

    let area = centered_rect(60, 9, f.area());

    f.render_widget(Clear, area);

    // Different title and message based on action type
    let (title, message) = match pending.action_type {
        PendingActionType::DeleteKey => (
            "Delete Key",
            format!("Are you sure you want to delete key '{}'?", pending.key),
        ),
        PendingActionType::DeleteServer => (
            "Delete Server",
            format!("Are you sure you want to delete server '{}'?", pending.key),
        ),
    };

    let title_color = Color::Red;

    // Build Cancel/OK buttons with selection indicator
    let cancel_style = if !pending.selected_yes {
        Style::default().fg(Color::Black).bg(Color::Magenta)
    } else {
        Style::default().fg(Color::White)
    };

    let ok_style = if pending.selected_yes {
        Style::default().fg(Color::Black).bg(Color::Magenta)
    } else {
        Style::default().fg(Color::White)
    };

    let text = vec![
        Line::from(Span::styled(
            format!("<{}>", title),
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Cancel ", cancel_style),
            Span::raw("    "),
            Span::styled(" OK ", ok_style),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(height),
            Constraint::Percentage(40),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
