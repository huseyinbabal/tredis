use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub struct SplashState {
    pub current_step: usize,
    pub total_steps: usize,
    pub current_message: String,
    pub spinner_frame: usize,
}

impl SplashState {
    pub fn new() -> Self {
        Self {
            current_step: 0,
            total_steps: 3,
            current_message: "Initializing...".to_string(),
            spinner_frame: 0,
        }
    }

    pub fn set_message(&mut self, message: &str) {
        self.current_message = message.to_string();
        self.spinner_frame = (self.spinner_frame + 1) % 4;
    }

    pub fn complete_step(&mut self) {
        self.current_step += 1;
    }
}

pub fn render(f: &mut Frame, splash: &SplashState) {
    let area = f.area();

    // Center everything vertically
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Min(15),
            Constraint::Percentage(30),
        ])
        .split(area);

    let center_area = vertical[1];

    // Split center into logo and loading area
    let content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // Big logo
            Constraint::Length(2), // Spacer
            Constraint::Length(1), // Loading bar
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Status message
        ])
        .split(center_area);

    // Render big ASCII logo
    render_big_logo(f, content[0]);

    // Render loading bar
    render_loading_bar(f, splash, content[2]);

    // Render status message
    render_status(f, splash, content[4]);
}

fn render_big_logo(f: &mut Frame, area: Rect) {
    let logo_lines = vec![
        Line::from(Span::styled(
            r"████████╗██████╗ ██████╗██████╗ ██╗███████╗",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r"╚══██╔══╝██╔══██╗██╔════╝██╔══██╗██║██╔════╝",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r"   ██║   ██████╔╝█████╗  ██║  ██║██║███████╗",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r"   ██║   ██╔══██╗██╔══╝  ██║  ██║██║╚════██║",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r"   ██║   ██║  ██║██████╗ ██████╔╝██║███████║",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            r"   ╚═╝   ╚═╝  ╚═╝╚═════╝ ╚═════╝ ╚═╝╚══════╝",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Terminal UI for Redis",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            format!("v{}", crate::VERSION),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(logo_lines).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn render_loading_bar(f: &mut Frame, splash: &SplashState, area: Rect) {
    let progress = splash.current_step as f64 / splash.total_steps as f64;
    let bar_width = (area.width as usize).saturating_sub(20); // Leave some margin
    let filled = (bar_width as f64 * progress) as usize;
    let empty = bar_width.saturating_sub(filled);

    let bar = Line::from(vec![
        Span::styled("  [", Style::default().fg(Color::DarkGray)),
        Span::styled("█".repeat(filled), Style::default().fg(Color::Red)),
        Span::styled("░".repeat(empty), Style::default().fg(Color::DarkGray)),
        Span::styled("]", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" {}%", (progress * 100.0) as u8),
            Style::default().fg(Color::White),
        ),
    ]);

    let paragraph = Paragraph::new(bar).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn render_status(f: &mut Frame, splash: &SplashState, area: Rect) {
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸"];
    let spinner = spinner_chars[splash.spinner_frame % spinner_chars.len()];

    let status = Line::from(vec![
        Span::styled(format!("{} ", spinner), Style::default().fg(Color::Yellow)),
        Span::styled(&splash.current_message, Style::default().fg(Color::White)),
    ]);

    let paragraph = Paragraph::new(status).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}
