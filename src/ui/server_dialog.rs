use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Which field is currently being edited in the server dialog
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerDialogField {
    Name,
    Uri,
}

/// State for the server connection dialog
#[derive(Debug, Clone)]
pub struct ServerDialogState {
    pub name: String,
    pub uri: String,
    pub active_field: ServerDialogField,
    pub error_message: Option<String>,
}

impl Default for ServerDialogState {
    fn default() -> Self {
        Self {
            name: String::new(),
            uri: "redis://localhost:6379/0".to_string(),
            active_field: ServerDialogField::Name,
            error_message: None,
        }
    }
}

impl ServerDialogState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_field(&mut self) {
        self.active_field = match self.active_field {
            ServerDialogField::Name => ServerDialogField::Uri,
            ServerDialogField::Uri => ServerDialogField::Name,
        };
    }

    pub fn current_input_mut(&mut self) -> &mut String {
        match self.active_field {
            ServerDialogField::Name => &mut self.name,
            ServerDialogField::Uri => &mut self.uri,
        }
    }

    pub fn push_char(&mut self, c: char) {
        self.current_input_mut().push(c);
        self.error_message = None;
    }

    pub fn pop_char(&mut self) {
        self.current_input_mut().pop();
        self.error_message = None;
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn set_error(&mut self, msg: String) {
        self.error_message = Some(msg);
    }

    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty() && !self.uri.trim().is_empty()
    }
}

pub fn render(f: &mut Frame, state: &ServerDialogState) {
    let area = centered_rect(60, 14, f.area());

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" New Server Connection ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Instruction
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Name label
            Constraint::Length(1), // Name input
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // URI label
            Constraint::Length(1), // URI input
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Error or buttons
            Constraint::Min(0),
        ])
        .split(inner);

    // Instructions
    let instructions = Paragraph::new(Line::from(vec![
        Span::styled("<Tab>", Style::default().fg(Color::Yellow)),
        Span::styled(" switch field  ", Style::default().fg(Color::DarkGray)),
        Span::styled("<Enter>", Style::default().fg(Color::Yellow)),
        Span::styled(" save  ", Style::default().fg(Color::DarkGray)),
        Span::styled("<Esc>", Style::default().fg(Color::Yellow)),
        Span::styled(" cancel", Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(instructions, chunks[0]);

    // Name label
    let name_label_style = if state.active_field == ServerDialogField::Name {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let name_label = Paragraph::new(Span::styled("Name:", name_label_style));
    f.render_widget(name_label, chunks[2]);

    // Name input
    let name_style = if state.active_field == ServerDialogField::Name {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };
    let name_display = if state.name.is_empty() && state.active_field != ServerDialogField::Name {
        "".to_string()
    } else {
        format!("{}_", state.name)
    };
    let name_text = if state.active_field == ServerDialogField::Name {
        format!(" {}", name_display)
    } else {
        format!(" {}", state.name)
    };
    let name_input = Paragraph::new(name_text).style(name_style);
    f.render_widget(name_input, chunks[3]);

    // URI label
    let uri_label_style = if state.active_field == ServerDialogField::Uri {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let uri_label = Paragraph::new(Span::styled("URI:", uri_label_style));
    f.render_widget(uri_label, chunks[5]);

    // URI input
    let uri_style = if state.active_field == ServerDialogField::Uri {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };
    let uri_text = if state.active_field == ServerDialogField::Uri {
        format!(" {}_", state.uri)
    } else {
        format!(" {}", state.uri)
    };
    let uri_input = Paragraph::new(uri_text).style(uri_style);
    f.render_widget(uri_input, chunks[6]);

    // Error message or help text
    if let Some(ref error) = state.error_message {
        let error_text = Paragraph::new(Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        ));
        f.render_widget(error_text, chunks[8]);
    } else {
        let help = Paragraph::new(Span::styled(
            "Example URI: redis://localhost:6379/0",
            Style::default().fg(Color::DarkGray),
        ));
        f.render_widget(help, chunks[8]);
    }
}

fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(height),
            Constraint::Percentage(30),
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
