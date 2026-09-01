use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    let mut procs = app.process_tracker.get_processes();

    // Apply search filter
    if !app.input_buffer.is_empty() {
        let query = app.input_buffer.to_lowercase();
        procs.retain(|p| {
            p.name.to_lowercase().contains(&query)
                || p.user.to_lowercase().contains(&query)
                || p.cmd.to_lowercase().contains(&query)
                || p.pid.to_string().contains(&query)
        });
    }

    // Sort by CPU desc by default
    procs.sort_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let header_cells = ["PID", "NAME", "CPU", "MEMORY", "STATUS", "USER"]
        .iter()
        .map(|h| {
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

    let rows = procs.iter().map(|p| {
        let cpu_color = if p.cpu_usage > 50.0 {
            Color::Red
        } else if p.cpu_usage > 20.0 {
            Color::Yellow
        } else {
            Color::Green
        };
        let mem_color = if p.mem_usage_percent > 50.0 {
            Color::Red
        } else if p.mem_usage_percent > 20.0 {
            Color::Yellow
        } else {
            Color::Green
        };

        let cells = vec![
            Cell::from(p.pid.to_string()),
            Cell::from(p.name.clone()),
            Cell::from(format!("{:>5.1}%", p.cpu_usage)).style(Style::default().fg(cpu_color)),
            Cell::from(format!("{:>5.1} MB", p.mem_bytes as f64 / 1_048_576.0))
                .style(Style::default().fg(mem_color)),
            Cell::from(p.state.clone()),
            Cell::from(p.user.clone()),
        ];
        Row::new(cells)
    });

    let search_title = if app.input_mode {
        format!(" PROCESSES (Search: {}) ", app.input_buffer)
    } else {
        format!(" PROCESSES ({} running) ", procs.len())
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(25),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.theme.primary))
            .title(vec![Span::styled(
                search_title,
                Style::default().add_modifier(Modifier::BOLD),
            )]),
    )
    .highlight_style(
        Style::default()
            .bg(app.theme.primary)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = ratatui::widgets::TableState::default();

    // Ensure selected_index is within bounds
    if app.selected_index >= procs.len() && !procs.is_empty() {
        app.selected_index = procs.len() - 1;
    }

    state.select(Some(app.selected_index));
    f.render_stateful_widget(table, chunks[0], &mut state);
}
