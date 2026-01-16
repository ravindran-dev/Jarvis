use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, Screen};
use super::widgets;

/// Main render function - called every frame
pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Create main layout: title, header, content area, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(3),  // Header with tabs
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer with help
        ])
        .split(size);

    // Render title
    render_title(f, chunks[0], app);

    // Render header with navigation tabs
    render_header(f, app, chunks[1]);

    // Render main content based on current screen
    match app.current_screen {
        Screen::Storage => render_storage_screen(f, app, chunks[2]),
        Screen::Metrics => render_metrics_screen(f, app, chunks[2]),
        Screen::Commands => render_commands_screen(f, app, chunks[2]),
        Screen::Settings => render_settings_screen(f, app, chunks[2]),
    }

    // Render footer with help text
    render_footer(f, app, chunks[3]);
}

/// Render the title
fn render_title(f: &mut Frame, area: Rect, app: &App) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled("  J A R V I S  ", Style::default()
            .fg(app.theme.primary)
            .add_modifier(Modifier::BOLD | Modifier::ITALIC)),
        Span::styled("- System Monitor & Command Assistant", Style::default()
            .fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)));
    
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
    };

    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)))
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
            Constraint::Length(3),  // Status bar
            Constraint::Min(0),     // Storage list
        ])
        .split(area);

    // Status information
    let status = widgets::create_storage_status(app);
    f.render_widget(status, chunks[0]);

    // Storage results table
    let storage_table = widgets::create_storage_table(app);
    f.render_widget(storage_table, chunks[1]);
}

/// Render the system metrics screen
fn render_metrics_screen(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),  // Top half
            Constraint::Percentage(50),  // Bottom half
        ])
        .split(area);

    // Top section: CPU and Memory
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let cpu_widget = widgets::create_cpu_widget(app);
    f.render_widget(cpu_widget, top_chunks[0]);

    let memory_widget = widgets::create_memory_widget(app);
    f.render_widget(memory_widget, top_chunks[1]);

    // Bottom section: Disk and Network
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let disk_widget = widgets::create_disk_widget(app);
    f.render_widget(disk_widget, bottom_chunks[0]);

    let network_widget = widgets::create_network_widget(app);
    f.render_widget(network_widget, bottom_chunks[1]);
}

/// Render the command assistant screen
fn render_commands_screen(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Search bar
            Constraint::Min(0),     // Command list
        ])
        .split(area);

    // Search input
    let search_widget = widgets::create_search_input(app);
    f.render_widget(search_widget, chunks[0]);

    // Command list with state
    let (commands_widget, mut list_state) = widgets::create_commands_list(app);
    f.render_stateful_widget(commands_widget, chunks[1], &mut list_state);
}

/// Render the settings screen
fn render_settings_screen(f: &mut Frame, app: &App, area: Rect) {
    // Build dynamic settings lines
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

    // Plugins section
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
                .style(Style::default().fg(app.theme.primary)),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Render the footer with keyboard shortcuts
fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.current_screen {
        Screen::Storage => {
            "h l: Switch | j k: Navigate | r: Rescan | q: Quit"
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
    };

    let help = Paragraph::new(Line::from(vec![
        Span::styled(help_text, Style::default().fg(Color::DarkGray)),
    ]))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)))
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}
