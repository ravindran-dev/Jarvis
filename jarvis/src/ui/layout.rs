use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use super::widgets;
use crate::app::{App, Screen};

pub fn render(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Create a global block with cyan borders
    use ratatui::style::{Color, Style};
    use ratatui::widgets::{Block, BorderType, Borders};

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.primary));

    let inner_area = main_block.inner(size);
    f.render_widget(main_block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top Header
            Constraint::Min(0),    // Main Content
            Constraint::Length(1), // Bottom Status Bar
        ])
        .split(inner_area);

    widgets::render_top_header(f, app, chunks[0]);

    match app.current_screen {
        Screen::Overview => render_overview_screen(f, app, chunks[1]),
        Screen::Cpu => render_cpu_screen(f, app, chunks[1]),
        Screen::Memory => render_memory_screen(f, app, chunks[1]),
        Screen::Storage => crate::ui::storage::render(f, app, chunks[1]),
        Screen::Processes => crate::ui::processes::render(f, app, chunks[1]),
        Screen::Network => crate::ui::network::render(f, app, chunks[1]),
        Screen::Services => crate::ui::services::render(f, app, chunks[1]),
        Screen::Users => crate::ui::users::render(f, app, chunks[1]),
        Screen::Settings => crate::ui::settings::render(f, app, chunks[1]),
        _ => render_placeholder_screen(f, "NOT IMPLEMENTED", chunks[1]),
    }

    widgets::render_status_bar(f, app, chunks[2]);

    if app.confirm_action != crate::app::ConfirmAction::None {
        render_confirm_popup(f, app, size);
    }
}

fn render_overview_screen(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(chunks[0]);
    let cpu_widget = widgets::create_cpu_widget(app);
    f.render_widget(cpu_widget, left_chunks[0]);

    let center_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    let memory_widget = widgets::create_memory_widget(app);
    f.render_widget(memory_widget, center_chunks[0]);
    let disk_widget = widgets::create_disk_widget(app);
    f.render_widget(disk_widget, center_chunks[1]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(chunks[2]);
    widgets::render_network_pane(f, app, right_chunks[0]);
}

fn render_cpu_screen(f: &mut Frame, app: &mut App, area: Rect) {
    let cpu_widget = widgets::create_cpu_widget(app);
    f.render_widget(cpu_widget, area);
}

fn render_memory_screen(f: &mut Frame, app: &mut App, area: Rect) {
    let memory_widget = widgets::create_memory_widget(app);
    f.render_widget(memory_widget, area);
}

fn render_placeholder_screen(f: &mut Frame, title: &str, area: Rect) {
    use ratatui::layout::Alignment;
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::Span;
    use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

    let paragraph = Paragraph::new("Screen under development or removed.")
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
                .border_type(BorderType::Plain),
        );
    f.render_widget(paragraph, area);
}

fn render_confirm_popup(f: &mut Frame, app: &App, area: Rect) {
    use crate::app::ConfirmAction;
    use ratatui::layout::{Alignment, Constraint, Direction, Layout};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(6),
            Constraint::Percentage(40),
        ])
        .split(area);

    let popup_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(popup_layout[1])[1];

    let (title_text, action_text, target_name) = match &app.confirm_action {
        ConfirmAction::KillProcess(p, n) => (
            " CONFIRM TERMINATION ",
            "Kill process",
            format!("{} (PID {})", n, p),
        ),
        ConfirmAction::ServiceAction(action, name) => {
            let title = match action.as_str() {
                "start" => " START SERVICE ",
                "stop" => " STOP SERVICE ",
                "restart" => " RESTART SERVICE ",
                "enable" => " ENABLE SERVICE ",
                "disable" => " DISABLE SERVICE ",
                _ => " SERVICE ACTION ",
            };
            (title, action.as_str(), name.clone())
        }
        _ => return,
    };

    let text = vec![
        Line::from(vec![
            Span::styled(format!("{} ", action_text), Style::default()),
            Span::styled(
                target_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("?", Style::default()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "[y] Yes",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(
                "[n] No",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .title(Span::styled(
                title_text,
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Red)),
    );

    f.render_widget(Clear, popup_chunk);
    f.render_widget(paragraph, popup_chunk);
}
