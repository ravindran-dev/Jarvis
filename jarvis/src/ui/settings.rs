use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Settings List
        ])
        .split(area);

    let header = Paragraph::new(vec![Line::from(vec![Span::styled(
        " JARVIS SETTINGS ",
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
    f.render_widget(header, chunks[0]);

    let mut lines = Vec::new();

    let settings: Vec<(&str, Vec<(&str, String)>)> = vec![
        (
            "APPEARANCE",
            vec![
                ("Theme", app.theme.name.clone()),
                ("Prompt Style", app.config.prompt_style.clone()),
                (
                    "Welcome Screen",
                    if app.config.welcome_screen {
                        "Enabled".to_string()
                    } else {
                        "Disabled".to_string()
                    },
                ),
            ],
        ),
        (
            "BEHAVIOR",
            vec![
                (
                    "Auto Refresh",
                    if app.config.auto_refresh {
                        "Enabled".to_string()
                    } else {
                        "Disabled".to_string()
                    },
                ),
                (
                    "Refresh Rate",
                    format!("{} ms", app.config.refresh_interval_ms),
                ),
            ],
        ),
        (
            "INTEGRATION",
            vec![
                ("Shell", app.config.shell.clone()),
                ("Terminal", app.config.terminal.clone()),
            ],
        ),
    ];

    let mut item_index = 0;

    for (section, items) in settings {
        lines.push(Line::from(Span::styled(
            section,
            Style::default()
                .fg(app.theme.warning)
                .add_modifier(Modifier::BOLD),
        )));

        for (key, val) in items {
            let mut style = Style::default();
            if item_index == app.settings_selected {
                style = style
                    .bg(app.theme.primary)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD);
            }

            let key_padded = format!("  {:<20}", key);
            lines.push(Line::from(vec![
                Span::styled(key_padded, style),
                Span::styled(
                    val,
                    style.fg(if item_index == app.settings_selected {
                        Color::Black
                    } else {
                        app.theme.text
                    }),
                ),
            ]));
            item_index += 1;
        }
        lines.push(Line::from(""));
    }

    let settings_block = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.theme.primary)),
    );

    f.render_widget(settings_block, chunks[1]);
}
