use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use super::widgets;
use crate::app::{App, Screen};

/// Main render function - called every frame
pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.size();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top Header
            Constraint::Min(0),    // Middle Content
            Constraint::Length(1), // Bottom Status Bar
        ])
        .split(size);

    widgets::render_top_header(f, app, main_chunks[0]);
    widgets::render_status_bar(f, app, main_chunks[2]);

    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // Sidebar width
            Constraint::Min(0),     // Main content
        ])
        .split(main_chunks[1]);

    widgets::render_sidebar(f, app, middle_chunks[0]);

    match app.current_screen {
        Screen::Overview => render_overview_screen(f, app, middle_chunks[1]),
        Screen::Cpu => render_cpu_screen(f, app, middle_chunks[1]),
        Screen::Memory => render_memory_screen(f, app, middle_chunks[1]),
        Screen::Storage => render_storage_screen(f, app, middle_chunks[1]),
        Screen::Processes => render_placeholder_screen(f, "PROCESSES", middle_chunks[1]),
        Screen::Network => render_network_screen(f, app, middle_chunks[1]),
        Screen::Services => render_placeholder_screen(f, "SERVICES", middle_chunks[1]),
        Screen::Users => render_placeholder_screen(f, "USERS", middle_chunks[1]),
        Screen::Commands => render_commands_screen(f, app, middle_chunks[1]),
        Screen::Events => render_events_screen(f, app, middle_chunks[1]),
        Screen::Settings => render_settings_screen(f, app, middle_chunks[1]),
        Screen::Help => render_help_screen(f, app, middle_chunks[1]),
    }
}

fn render_overview_screen(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let cpu_widget = widgets::create_cpu_widget(app);
    f.render_widget(cpu_widget, top_chunks[0]);

    let memory_widget = widgets::create_memory_widget(app);
    f.render_widget(memory_widget, top_chunks[1]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let disk_widget = widgets::create_disk_widget(app);
    f.render_widget(disk_widget, bottom_chunks[0]);

    widgets::render_network_pane(f, app, bottom_chunks[1]);
}

fn render_cpu_screen(f: &mut Frame, app: &mut App, area: Rect) {
    let cpu_widget = widgets::create_cpu_widget(app);
    f.render_widget(cpu_widget, area);
}

fn render_memory_screen(f: &mut Frame, app: &mut App, area: Rect) {
    let memory_widget = widgets::create_memory_widget(app);
    f.render_widget(memory_widget, area);
}

fn render_network_screen(f: &mut Frame, app: &mut App, area: Rect) {
    widgets::render_network_pane(f, app, area);
}

fn render_storage_screen(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let breadcrumb = widgets::create_breadcrumb(app);
    f.render_widget(breadcrumb, chunks[0]);

    let status = widgets::create_storage_status(app);
    f.render_widget(status, chunks[1]);

    let available_height = chunks[2].height.saturating_sub(3) as usize;
    let storage_table = widgets::create_storage_table(app, available_height);
    f.render_widget(storage_table, chunks[2]);
}

fn render_commands_screen(f: &mut Frame, app: &mut App, area: Rect) {
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

fn render_events_screen(f: &mut Frame, app: &mut App, area: Rect) {
    let event_log_widget = widgets::create_event_log_pane(app);
    f.render_widget(event_log_widget, area);
}

fn render_settings_screen(f: &mut Frame, app: &mut App, area: Rect) {
    let widget = widgets::create_settings_widget(app);
    f.render_widget(widget, area);
}

fn render_help_screen(f: &mut Frame, app: &mut App, area: Rect) {
    let widget = widgets::create_help_widget(app);
    f.render_widget(widget, area);
}

fn render_placeholder_screen(f: &mut Frame, title: &str, area: Rect) {
    use ratatui::layout::Alignment;
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::Span;
    use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

    let paragraph = Paragraph::new("Screen under development.")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" {} ", title),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Yellow),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
    f.render_widget(paragraph, area);
}
