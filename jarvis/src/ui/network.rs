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
            Constraint::Length(4), // Stats
            Constraint::Min(0),    // Table
        ])
        .split(area);

    // 1. Render Stats
    let stats = app.metrics.get_network_info();
    let stats_text = vec![
        Line::from(vec![
            Span::styled(
                " Total RX: ",
                Style::default()
                    .fg(app.theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format_bytes(stats.received)),
            Span::styled(
                "   RX Rate: ",
                Style::default()
                    .fg(app.theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}/s", format_bytes(stats.rx_rate))),
        ]),
        Line::from(vec![
            Span::styled(
                " Total TX: ",
                Style::default()
                    .fg(app.theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format_bytes(stats.sent)),
            Span::styled(
                "   TX Rate: ",
                Style::default()
                    .fg(app.theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}/s", format_bytes(stats.tx_rate))),
        ]),
    ];
    let stats_widget = Paragraph::new(stats_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" NETWORK METRICS ")
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.theme.primary)),
    );
    f.render_widget(stats_widget, chunks[0]);

    // 2. Render Table
    let mut connections = app.network_connections.clone();

    if app.input_mode && !app.input_buffer.is_empty() {
        let query = app.input_buffer.to_lowercase();
        connections.retain(|c| {
            c.local_addr.to_lowercase().contains(&query)
                || c.remote_addr.to_lowercase().contains(&query)
                || c.protocol.to_lowercase().contains(&query)
                || c.state.to_lowercase().contains(&query)
        });
    }

    let header_cells = ["PROTO", "LOCAL", "REMOTE", "STATE", "PID", "PROCESS NAME"]
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

    let rows = connections.iter().map(|c| {
        let state_color = match c.state.as_str() {
            "ESTABLISHED" => Color::Green,
            "LISTEN" => Color::Yellow,
            "TIME_WAIT" | "CLOSE_WAIT" => Color::Gray,
            _ => Color::White,
        };

        let proc_info = c.process_name.clone().unwrap_or_else(|| String::from("-"));
        let pid_info = c
            .pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| String::from("-"));

        let cells = vec![
            Cell::from(c.protocol.clone()),
            Cell::from(c.local_addr.clone()),
            Cell::from(c.remote_addr.clone()),
            Cell::from(c.state.clone()).style(Style::default().fg(state_color)),
            Cell::from(pid_info),
            Cell::from(proc_info),
        ];
        Row::new(cells)
    });

    let search_title = if app.input_mode {
        format!(" CONNECTIONS (Search: {}) ", app.input_buffer)
    } else {
        format!(" CONNECTIONS ")
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.primary))
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                search_title,
                Style::default().add_modifier(Modifier::BOLD),
            )),
    )
    .highlight_style(
        Style::default()
            .bg(app.theme.primary)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = ratatui::widgets::TableState::default();
    state.select(Some(app.network_scroll));
    f.render_stateful_widget(table, chunks[1], &mut state);
}

fn format_bytes(bytes: u64) -> String {
    let kb = 1024_f64;
    let mb = kb * 1024.0;
    let gb = mb * 1024.0;
    let b = bytes as f64;

    if b >= gb {
        format!("{:.2} GB", b / gb)
    } else if b >= mb {
        format!("{:.2} MB", b / mb)
    } else if b >= kb {
        format!("{:.2} KB", b / kb)
    } else {
        format!("{} B", bytes)
    }
}
