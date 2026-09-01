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
        String::from(" SYSTEM USERS ")
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
    let mut users = app.user_tracker.get_users();

    if !app.input_buffer.is_empty() {
        let search = app.input_buffer.to_lowercase();
        users.retain(|u| u.username.to_lowercase().contains(&search));
    }

    let header_cells = ["USER", "UID", "HOME", "SHELL"].iter().map(|h| {
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

    let rows = users.iter().map(|u| {
        Row::new(vec![
            Cell::from(Span::styled(
                u.username.clone(),
                Style::default().fg(Color::Green),
            )),
            Cell::from(u.uid.clone()),
            Cell::from(u.home.clone()),
            Cell::from(u.shell.clone()),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
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
