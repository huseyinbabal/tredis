use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Highlight search matches within text, returning spans with highlighted portions
fn highlight_matches(
    text: &str,
    search: &str,
    base_color: Color,
    is_current: bool,
) -> Vec<Span<'static>> {
    if search.is_empty() {
        return vec![Span::styled(
            text.to_string(),
            Style::default().fg(base_color),
        )];
    }

    let text_lower = text.to_lowercase();
    let search_lower = search.to_lowercase();
    let mut spans = Vec::new();
    let mut last_end = 0;

    for (start, _) in text_lower.match_indices(&search_lower) {
        // Add text before match
        if start > last_end {
            spans.push(Span::styled(
                text[last_end..start].to_string(),
                Style::default().fg(base_color),
            ));
        }
        // Add highlighted match
        let highlight_style = if is_current {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        };
        spans.push(Span::styled(
            text[start..start + search.len()].to_string(),
            highlight_style,
        ));
        last_end = start + search.len();
    }

    // Add remaining text
    if last_end < text.len() {
        spans.push(Span::styled(
            text[last_end..].to_string(),
            Style::default().fg(base_color),
        ));
    }

    if spans.is_empty() {
        vec![Span::styled(
            text.to_string(),
            Style::default().fg(base_color),
        )]
    } else {
        spans
    }
}

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Split area for search input if active
    let (content_area, search_area) = if app.info_search_active {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(3)])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    // Build title with search info
    let title = if !app.info_search_text.is_empty() && !app.info_search_matches.is_empty() {
        format!(
            " Server Information [{}/{}] ",
            app.info_search_current + 1,
            app.info_search_matches.len()
        )
    } else if !app.info_search_text.is_empty() {
        " Server Information [No matches] ".to_string()
    } else {
        " Server Information ".to_string()
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

    let inner_area = block.inner(content_area);
    f.render_widget(block, content_area);

    let search_text = &app.info_search_text;
    let has_search = !search_text.is_empty();

    let lines: Vec<Line> = app
        .info_data
        .iter()
        .enumerate()
        .map(|(idx, (k, v))| {
            let is_current = !app.info_search_matches.is_empty()
                && app.info_search_current < app.info_search_matches.len()
                && app.info_search_matches[app.info_search_current] == idx;

            if v.is_empty() {
                // Section header
                if has_search {
                    let mut spans = vec![Span::raw("\n")];
                    spans.extend(highlight_matches(k, search_text, Color::Yellow, is_current));
                    Line::from(spans)
                } else {
                    Line::from(vec![
                        Span::raw("\n"),
                        Span::styled(
                            k,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ])
                }
            } else {
                // Regular key-value line
                if has_search {
                    let key_formatted = format!("{:<30}", k);
                    let mut spans =
                        highlight_matches(&key_formatted, search_text, Color::DarkGray, is_current);
                    spans.extend(highlight_matches(v, search_text, Color::White, is_current));
                    Line::from(spans)
                } else {
                    Line::from(vec![
                        Span::styled(format!("{:<30}", k), Style::default().fg(Color::DarkGray)),
                        Span::styled(v, Style::default().fg(Color::White)),
                    ])
                }
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines).scroll((app.info_scroll as u16, 0));
    f.render_widget(paragraph, inner_area);

    // Render search input if active
    if let Some(search_rect) = search_area {
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Span::styled(
                " Search (n: next, p: prev, Esc: close) ",
                Style::default().fg(Color::Yellow),
            ));

        let search_input = Paragraph::new(Line::from(vec![
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::styled(&app.info_search_text, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]))
        .block(search_block);

        f.render_widget(search_input, search_rect);
    }
}
