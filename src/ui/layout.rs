use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, Screen};
use super::widgets;

/// Main render function - called every frame
pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(size);

    render_title(f, chunks[0], app);

    render_header(f, app, chunks[1]);

    match app.current_screen {
        Screen::Storage => render_storage_screen(f, app, chunks[2]),
        Screen::Metrics => render_metrics_screen(f, app, chunks[2]),
        Screen::Commands => render_commands_screen(f, app, chunks[2]),
        Screen::Settings => render_settings_screen(f, app, chunks[2]),
        Screen::Help => render_help_screen(f, app, chunks[2]),
    }

    render_footer(f, app, chunks[3]);
}

/// Render the title
fn render_title(f: &mut Frame, area: Rect, app: &App) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("  J A R V I S  ", Style::default()
            .fg(app.theme.primary)
            .add_modifier(Modifier::BOLD | Modifier::ITALIC)),
        Span::styled("- System Monitor & Command Assistant", Style::default()
            .fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD))
        .border_type(BorderType::Rounded));
    
    f.render_widget(title, area);
}

/// Render the header with navigation tabs
fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec![" STORAGE", "  METRICS", " COMMANDS", " SETTINGS"];
    
    let selected_index = match app.current_screen {
        Screen::Storage => 0,
        Screen::Metrics => 1,
        Screen::Commands => 2,
        Screen::Settings => 3,

        Screen::Help => 0, // Help doesn't have its own tab, default to Storage
    };
    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD))
            .border_type(BorderType::Rounded))
        .select(selected_index)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(app.theme.primary)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

/// Render the storage analysis screen
fn render_storage_screen(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

    let status = widgets::create_storage_status(app);
    f.render_widget(status, chunks[0]);

    let breadcrumb = widgets::create_breadcrumb(app);
    f.render_widget(breadcrumb, chunks[1]);

    let available_height = chunks[2].height.saturating_sub(3) as usize; // Subtract border/header
    let storage_table = widgets::create_storage_table(app, available_height);
    f.render_widget(storage_table, chunks[2]);
}

/// Render the system metrics screen
fn render_metrics_screen(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let cpu_widget = widgets::create_cpu_widget(app);
    let cpu_area = centered_rect(96, 92, top_chunks[0]);
    f.render_widget(cpu_widget, cpu_area);

    let memory_widget = widgets::create_memory_widget(app);
    let mem_area = centered_rect(96, 92, top_chunks[1]);
    f.render_widget(memory_widget, mem_area);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let disk_widget = widgets::create_disk_widget(app);
    let disk_area = centered_rect(96, 92, bottom_chunks[0]);
    f.render_widget(disk_widget, disk_area);

    let network_widget = widgets::create_network_widget(app);
    let net_area = centered_rect(96, 92, bottom_chunks[1]);
    f.render_widget(network_widget, net_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

/// Render the command assistant screen
fn render_commands_screen(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(10),
        ])
        .split(area);

    let search_widget = widgets::create_search_input(app);
    f.render_widget(search_widget, chunks[0]);

    let (commands_widget, mut list_state) = widgets::create_commands_list(app);
    f.render_stateful_widget(commands_widget, chunks[1], &mut list_state);

    let output_widget = widgets::create_output_pane(app);
    f.render_widget(output_widget, chunks[2]);
}

/// Render the settings screen
fn render_settings_screen(f: &mut Frame, app: &App, area: Rect) {
    let themes = app.get_available_themes();
    let theme_name = themes.get(app.get_current_theme_index()).map(|t| t.name.as_str()).unwrap_or("dark");
    let interval = format!("{} ms", app.config.refresh_interval_ms);
    let log_level = app.config.log_level.to_uppercase();
    let threshold = format!("{} MB", app.config.storage_min_threshold_mb);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "Settings",
        Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    let items = vec![
        ("Theme", theme_name.to_string()),
        ("Refresh Interval", interval),
        ("Log Level", log_level),
        ("Storage Min Threshold", threshold),
    ];

    for (i, (label, value)) in items.iter().enumerate() {
        let selected = i == app.settings_selected;
        let style = if selected {
            Style::default().fg(Color::Black).bg(app.theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{:<24}", label), style),
            Span::styled(value.clone(), style),
            Span::raw(" ".repeat(120)),
        ]));
        lines.push(Line::from(Span::styled("─".repeat(120), Style::default().fg(Color::DarkGray))));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Plugins",
        Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
    )));
    let plugins = app.plugins.list_with_status();
    if plugins.is_empty() {
        lines.push(Line::from("No plugins loaded"));
    } else {
        for (name, enabled) in plugins {
            lines.push(Line::from(format!("{} [{}]", name, if enabled { "Enabled" } else { "Disabled" })));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Settings ")
                .style(Style::default().fg(app.theme.primary))
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Render the footer with keyboard shortcuts
fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.current_screen {
        Screen::Storage => {
            if app.storage.get_current_path().is_some() {
                "Type: Filter | Backspace: Go Back | ?: Toggle Search | Enter: Drill Down/Open | r: Rescan | q: Quit"
            } else {
                "h l: Switch | Type: Filter | ?: Toggle Search | Enter: Drill Down | r: Rescan | q: Quit"
            }
        }
        Screen::Metrics => {
            "h l: Switch | r: Refresh | q: Quit"
        }
        Screen::Commands => {
            "h l: Switch | j k: Navigate | /: Search | Enter: Execute | q: Quit"
        }
        Screen::Settings => {
            "Tab: Switch | j k: Navigate | ←→ / -+: Adjust | t: Toggle Theme | q: Quit"
        }
        Screen::Help => {
            "? / q: Close Help | q: Quit"
        }
    };

    let help = Paragraph::new(Line::from(vec![
        Span::styled(help_text, Style::default().fg(Color::DarkGray)),
    ]))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD))
            .border_type(BorderType::Rounded))
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}

/// Render the help screen with all keymaps
fn render_help_screen(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    lines.push(Line::from(Span::styled(
        " KEYBOARD KEYMAPS - Press ? or q to Close",
        Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "Global Keys:",
        Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from("  Tab / Shift+Tab    Switch between Storage, Metrics, Commands, Settings"));
    lines.push(Line::from("  h / l              Navigate left / right between screens"));
    lines.push(Line::from("  t / T              Cycle through available themes"));
    lines.push(Line::from("  Ctrl+H / Ctrl+L    Previous / Next theme"));
    lines.push(Line::from("  ?                  Show this help screen"));
    lines.push(Line::from("  q                  Quit application"));
    lines.push(Line::from("  Ctrl+C             Force quit"));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "Storage Screen:",
        Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from("  j / k               Navigate up / down in directory list"));
    lines.push(Line::from("  Enter               Drill into subdirectories or open folder"));
    lines.push(Line::from("  Backspace           Go back to parent directory"));
    lines.push(Line::from("  r                   Rescan storage"));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "Metrics Screen:",
        Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from("  r                   Refresh metrics"));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "Commands Screen:",
        Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from("  j / k               Navigate up / down in command list"));
    lines.push(Line::from("  /                   Activate search mode"));
    lines.push(Line::from("  Enter               Execute selected command"));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "Settings Screen:",
        Style::default().fg(app.theme.warning).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from("  j / k               Navigate down / up through settings"));
    lines.push(Line::from("  ← → / - +           Adjust selected setting"));
    lines.push(Line::from(""));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .style(Style::default().fg(app.theme.primary))
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);

}
