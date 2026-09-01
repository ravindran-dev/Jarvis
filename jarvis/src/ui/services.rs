use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search / Header
            Constraint::Min(0),    // Table
        ])
        .split(area);

    let search_indicator = if app.input_mode {
        format!(" SEARCH: {}█ ", app.input_buffer)
    } else if !app.input_buffer.is_empty() {
        format!(" SEARCH: {} ", app.input_buffer)
    } else {
        String::from(" SYSTEM SERVICES ")
    };

    let breadcrumb = Paragraph::new(vec![Line::from(vec![Span::styled(
        search_indicator,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.theme.primary)),
    );

    f.render_widget(breadcrumb, chunks[0]);

    // Table
    let mut services = app.service_tracker.get_services();

    if !app.input_buffer.is_empty() {
        let search = app.input_buffer.to_lowercase();
        services.retain(|s| s.name.to_lowercase().contains(&search));
    }

    let header_cells = ["SERVICE", "STATUS", "ENABLED"].iter().map(|h| {
        Cell::from(*h).style(
            Style::default()
                .fg(app.theme.warning)
                .add_modifier(Modifier::BOLD),
        )
    });

    let header = Row::new(header_cells)
        .style(Style::default().bg(app.theme.background))
        .height(1)
        .bottom_margin(1);

    let rows = services.iter().map(|s| {
        let status_color = match s.status.as_str() {
            "Running" => Color::Green,
            "Dead" | "Exited" => Color::DarkGray,
            "failed" => Color::Red,
            _ => Color::Yellow,
        };

        let enabled_color = match s.enabled.as_str() {
            "enabled" => Color::Green,
            "disabled" => Color::Red,
            _ => Color::DarkGray,
        };

        Row::new(vec![
            Cell::from(s.name.clone()),
            Cell::from(Span::styled(
                s.status.clone(),
                Style::default().fg(status_color),
            )),
            Cell::from(Span::styled(
                s.enabled.clone(),
                Style::default().fg(enabled_color),
            )),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.theme.primary)),
    )
    .highlight_style(
        Style::default()
            .bg(app.theme.primary)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = ratatui::widgets::TableState::default();
    state.select(Some(app.selected_index));
    f.render_stateful_widget(table, chunks[1], &mut state);
}
