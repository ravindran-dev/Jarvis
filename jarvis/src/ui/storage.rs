use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use std::path::PathBuf;

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Breadcrumb
            Constraint::Min(0),    // Table
        ])
        .split(area);

    let current_path = app.storage.get_current_path();

    // 1. Render Breadcrumb
    let path_str = match &current_path {
        Some(p) => p.to_string_lossy().to_string(),
        None => String::from("Mount Points"),
    };

    let search_indicator = if app.storage_search_enabled && !app.input_buffer.is_empty() {
        format!(" / Search: {} ", app.input_buffer)
    } else {
        String::new()
    };

    let breadcrumb = Paragraph::new(vec![Line::from(vec![
        Span::styled(" Location: ", Style::default().fg(app.theme.accent)),
        Span::raw(path_str),
        Span::styled(search_indicator, Style::default().fg(Color::Yellow)),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.theme.primary)),
    );

    f.render_widget(breadcrumb, chunks[0]);

    // 2. Render Table
    let table = if let Some(path) = &current_path {
        render_directory_table(app, path)
    } else {
        render_mounts_table(app)
    };

    let mut state = ratatui::widgets::TableState::default();
    state.select(Some(app.selected_index));
    f.render_stateful_widget(table, chunks[1], &mut state);
}

fn render_directory_table<'a>(app: &'a App, path: &PathBuf) -> Table<'a> {
    let mut items = app.storage.get_subdirectories(&path.to_string_lossy());

    if app.storage_search_enabled && !app.input_buffer.is_empty() {
        let search = app.input_buffer.to_lowercase();
        items.retain(|d| d.path.to_lowercase().contains(&search));
    }

    let header_cells = ["PATH", "SIZE", "FILES"].iter().map(|h| {
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

    let rows = items.iter().map(|item| {
        let size_str = format_size(item.size);
        let path_name = std::path::Path::new(&item.path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let cells = vec![
            Cell::from(if item.size == 0 {
                format!("{} (computing...)", path_name)
            } else {
                path_name
            }),
            Cell::from(size_str),
            Cell::from(item.file_count.to_string()),
        ];
        Row::new(cells)
    });

    Table::new(
        rows,
        [
            Constraint::Percentage(60),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
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
    )
}

fn render_mounts_table<'a>(app: &'a App) -> Table<'a> {
    let mut disks = app.metrics.get_disk_info();

    if app.storage_search_enabled && !app.input_buffer.is_empty() {
        let search = app.input_buffer.to_lowercase();
        disks.retain(|d| {
            d.name.to_lowercase().contains(&search)
                || d.mount_point.to_lowercase().contains(&search)
        });
    }

    let header_cells = [
        "Mount Point",
        "Device",
        "Type",
        "Total",
        "Used",
        "Free",
        "Usage",
    ]
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

    let rows = disks.iter().map(|disk| {
        let usage_pct = if disk.total > 0 {
            (disk.used as f64 / disk.total as f64) * 100.0
        } else {
            0.0
        };

        let color = if usage_pct > 90.0 {
            Color::Red
        } else if usage_pct > 75.0 {
            Color::Yellow
        } else {
            Color::Green
        };

        let bar_len = 12;
        let filled = ((usage_pct / 100.0) * bar_len as f64).round() as usize;
        let mut bar = String::new();
        for i in 0..bar_len {
            if i < filled {
                bar.push('█');
            } else {
                bar.push('░');
            }
        }

        let cells = vec![
            Cell::from(disk.mount_point.clone()),
            Cell::from(disk.name.clone()),
            Cell::from(disk.file_system.clone()),
            Cell::from(format_size(disk.total)),
            Cell::from(format_size(disk.used)),
            Cell::from(format_size(disk.total.saturating_sub(disk.used))),
            Cell::from(Line::from(vec![
                Span::styled(bar, Style::default().fg(color)),
                Span::styled(format!(" {:.0}%", usage_pct), Style::default().fg(color)),
            ])),
        ];
        Row::new(cells)
    });

    Table::new(
        rows,
        [
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
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
    )
}

fn format_size(bytes: u64) -> String {
    let kb = 1024_f64;
    let mb = kb * 1024.0;
    let gb = mb * 1024.0;
    let tb = gb * 1024.0;
    let bytes_f = bytes as f64;

    if bytes_f >= tb {
        format!("{:.1}T", bytes_f / tb)
    } else if bytes_f >= gb {
        format!("{:.1}G", bytes_f / gb)
    } else if bytes_f >= mb {
        format!("{:.1}M", bytes_f / mb)
    } else if bytes_f >= kb {
        format!("{:.1}K", bytes_f / kb)
    } else {
        format!("{}B", bytes)
    }
}
