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

    // Create main layout: header, content area, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with tabs
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer with help
        ])
        .split(size);

    // Render header with navigation tabs
    render_header(f, app, chunks[0]);

    // Render main content based on current screen
    match app.current_screen {
        Screen::Storage => render_storage_screen(f, app, chunks[1]),
        Screen::Metrics => render_metrics_screen(f, app, chunks[1]),
        Screen::Commands => render_commands_screen(f, app, chunks[1]),
        Screen::Settings => render_settings_screen(f, app, chunks[1]),
    }

    // Render footer with help text
    render_footer(f, app, chunks[2]);
}

/// Render the header with navigation tabs
fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Storage", "Metrics", "Commands", "Settings"];
    
    let selected_index = match app.current_screen {
        Screen::Storage => 0,
        Screen::Metrics => 1,
        Screen::Commands => 2,
        Screen::Settings => 3,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Jarvis "))
        .select(selected_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
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

    // Command list
    let commands_widget = widgets::create_commands_list(app);
    f.render_widget(commands_widget, chunks[1]);
}

/// Render the settings screen
fn render_settings_screen(f: &mut Frame, _app: &App, area: Rect) {
    let settings_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Settings",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Update Interval: 1 second"),
        Line::from("Storage Scan Threads: Auto"),
        Line::from("Log Level: Info"),
        Line::from(""),
        Line::from(Span::styled(
            "Plugin Configuration",
            Style::default().fg(Color::Yellow),
        )),
        Line::from("No plugins loaded"),
        Line::from(""),
        Line::from(Span::styled(
            "Future Features:",
            Style::default().fg(Color::Green),
        )),
        Line::from("• Customizable refresh intervals"),
        Line::from("• Theme selection"),
        Line::from("• Export reports"),
        Line::from("• Plugin management"),
    ];

    let paragraph = Paragraph::new(settings_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Settings ")
                .style(Style::default().fg(Color::White)),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Render the footer with keyboard shortcuts
fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.current_screen {
        Screen::Storage => {
            "← → / h l: Switch tabs | ↑ ↓ / j k: Navigate | r: Rescan | q: Quit"
        }
        Screen::Metrics => {
            "← → / h l: Switch tabs | r: Refresh | q: Quit"
        }
        Screen::Commands => {
            "← → / h l: Switch tabs | ↑ ↓ / j k: Navigate | /: Search | Enter: Execute | q: Quit"
        }
        Screen::Settings => {
            "← → / h l: Switch tabs | q: Quit"
        }
    };

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);

    f.render_widget(help, area);
}
