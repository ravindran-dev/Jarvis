use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table,
        TableState,
    },
    Frame,
};

use crate::app::App;
use crate::utils::format;

/// Create storage status widget
pub fn create_storage_status(app: &App) -> Paragraph<'static> {
    let count = app.storage.get_results_count();
    let status = if app.storage.is_scanning() {
        format!("Scanning directories... ({} items found so far)", count)
    } else if count == 0 {
        "No large directories found. Press 'r' to scan.".to_string()
    } else {
        format!(
            "Scan complete. Found {} directories over 1MB. Press 'r' to rescan.",
            count
        )
    };

    Paragraph::new(status)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.primary))
                .border_type(BorderType::Rounded)
                .title(" Storage Status "),
        )
        .style(Style::default().fg(Color::White))
}

/// Create breadcrumb widget for storage navigation
pub fn create_breadcrumb(app: &App) -> Paragraph<'_> {
    let cursor = if app.storage_search_enabled && app.cursor_visible {
        "█"
    } else {
        ""
    };

    let text = if !app.storage_search_enabled {
        " SEARCH: Disabled (Press ? to enable)".to_string()
    } else if !app.storage_search_buffer.is_empty() {
        format!(" SEARCH: {}{}", app.storage_search_buffer, cursor)
    } else {
        format!(
            " SEARCH: {}{} (Start typing to filter)",
            app.storage_search_buffer, cursor
        )
    };

    let text_color = if !app.storage_search_enabled {
        Color::DarkGray
    } else if !app.storage_search_buffer.is_empty() {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    Paragraph::new(Line::from(vec![Span::styled(
        text,
        Style::default().fg(text_color).add_modifier(Modifier::BOLD),
    )]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                if app.storage_search_enabled {
                    " SEARCH MODE "
                } else {
                    " SEARCH "
                },
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )),
    )
}

/// Create storage table widget
pub fn create_storage_table(app: &App, available_height: usize) -> Table<'static> {
    let (results, title) = if let Some(current_path) = app.storage.get_current_path() {
        let subdirs = app
            .storage
            .get_subdirectories(current_path.to_string_lossy().as_ref());
        let path_display = current_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Directory");
        (subdirs, format!(" Subdirectories of {} ", path_display))
    } else {
        (
            app.storage.get_results(),
            " Directory Sizes (Largest First) ".to_string(),
        )
    };

    // Filter results based on search buffer
    let filtered_results: Vec<_> =
        if app.storage_search_enabled && !app.storage_search_buffer.is_empty() {
            results
                .into_iter()
                .filter(|item| {
                    item.path
                        .to_lowercase()
                        .contains(&app.storage_search_buffer.to_lowercase())
                })
                .collect()
        } else {
            results
        };

    let rows: Vec<Row> = if filtered_results.is_empty() {
        vec![Row::new(vec![
            Cell::from(if !app.storage_search_buffer.is_empty() {
                format!("No matches for '{}'", app.storage_search_buffer)
            } else if app.storage.get_current_path().is_some() {
                "No subdirectories found. Press Backspace to go back.".to_string()
            } else {
                "No data available - scanning in progress or press 'r' to scan".to_string()
            }),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        // Apply scroll offset to visible rows using actual available height
        let visible_start = app.scroll_offset;
        let visible_end = (app.scroll_offset + available_height).min(filtered_results.len());

        filtered_results
            .iter()
            .enumerate()
            .filter(|(idx, _)| *idx >= visible_start && *idx < visible_end)
            .map(|(idx, item)| {
                let style = if idx == app.selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(app.theme.primary)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                Row::new(vec![
                    Cell::from(item.path.clone()),
                    Cell::from(format::format_bytes(item.size)),
                    Cell::from(format!("{}", item.file_count)),
                ])
                .style(style)
            })
            .collect()
    };

    let widths = [
        Constraint::Percentage(60),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    Table::new(rows, widths)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.primary))
                .border_type(BorderType::Rounded)
                .title(title),
        )
        .header(
            Row::new(vec!["Path", "Size", "Files"])
                .style(
                    Style::default()
                        .fg(app.theme.warning)
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .column_spacing(1)
}

/// Create CPU usage widget
pub fn create_cpu_widget(app: &App) -> Paragraph<'static> {
    let cpu_data = app.metrics.get_cpu_info();

    let overall_color = get_usage_color(cpu_data.usage as f64);
    let overall_bar = create_unicode_bar(cpu_data.usage as f64, 50);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  CPU Usage:  ",
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:.1}%", cpu_data.usage),
                Style::default()
                    .fg(overall_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            overall_bar,
            Style::default()
                .fg(overall_color)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    for (i, core_usage) in cpu_data.per_core.iter().enumerate() {
        let simple_bar = create_simple_bar(*core_usage as f64, 14);
        let core_color = get_usage_color(*core_usage as f64);
        lines.push(Line::from(vec![
            Span::styled(
                format!("   Core {:2} :  ", i),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                simple_bar,
                Style::default().fg(core_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("   {:.1}%", core_usage),
                Style::default().fg(core_color),
            ),
        ]));
    }

    lines.push(Line::from(""));

    Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                "  CPU CORES ",
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )),
    )
}

/// Create memory usage widget
pub fn create_memory_widget(app: &App) -> Paragraph<'static> {
    let mem_data = app.metrics.get_memory_info();

    let usage_percent = (mem_data.used as f64 / mem_data.total as f64) * 100.0;
    let mem_bar = create_unicode_bar(usage_percent, 60);
    let mem_color = get_usage_color(usage_percent);

    let swap_percent = if mem_data.swap_total > 0 {
        (mem_data.swap_used as f64 / mem_data.swap_total as f64) * 100.0
    } else {
        0.0
    };
    let _swap_bar = create_unicode_bar(swap_percent, 60);
    let swap_color = get_usage_color(swap_percent);

    let lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " RAM Total:  ",
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "{:.1} GB",
                    mem_data.total as f64 / (1024.0 * 1024.0 * 1024.0)
                ),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " RAM Used:   ",
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "{:.2} GB",
                    mem_data.used as f64 / (1024.0 * 1024.0 * 1024.0)
                ),
                Style::default().fg(mem_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " RAM Avail:  ",
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "{:.2} GB",
                    mem_data.available as f64 / (1024.0 * 1024.0 * 1024.0)
                ),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                mem_bar,
                Style::default().fg(mem_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("   {:.1}%", usage_percent),
                Style::default().fg(mem_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Swap:  ",
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "{} / {:.2} GB",
                    format::format_bytes(mem_data.swap_used),
                    mem_data.swap_total as f64 / (1024.0 * 1024.0 * 1024.0)
                ),
                Style::default().fg(swap_color).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(""),
    ];

    Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " 󰨅 MEMORY ",
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )),
    )
}

/// Create disk usage widget
pub fn create_disk_widget(app: &App) -> Paragraph<'_> {
    let disks = app.metrics.get_disk_info();

    let mut lines = vec![];

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    let disk_list: Vec<_> = disks.to_vec();

    if disk_list.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "No disks found",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (idx, disk) in disk_list.iter().enumerate() {
            if idx > 0 {
                lines.push(Line::from(""));
                lines.push(Line::from(""));
            }

            let usage_percent = (disk.used as f64 / disk.total as f64) * 100.0;
            let bar = create_unicode_bar(usage_percent, 50);
            let disk_color = get_usage_color(usage_percent);

            lines.push(Line::from(vec![Span::styled(
                format!("  {}", disk.mount_point.clone()),
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(""));

            lines.push(Line::from(vec![
                Span::styled(
                    bar,
                    Style::default().fg(disk_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("   {:.1}%", usage_percent),
                    Style::default().fg(disk_color).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));

            lines.push(Line::from(vec![Span::styled(
                format!(
                    "  Used: {:.1} GB / Total: {:.1} GB",
                    disk.used as f64 / (1024.0 * 1024.0 * 1024.0),
                    disk.total as f64 / (1024.0 * 1024.0 * 1024.0)
                ),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(""));

    Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                "  DISK ",
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )),
    )
}

/// Render network pane with stats and active connections
pub fn render_network_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(app.theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            "   NETWORK ",
            Style::default()
                .fg(app.theme.primary)
                .add_modifier(Modifier::BOLD),
        ));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Split inner area into stats (top) and connections (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .split(inner_area);

    let net_data = app.metrics.get_network_info();

    // Render Stats
    let stats_lines = vec![
        Line::from(vec![
            Span::styled(
                "  RX Total:  ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format::format_bytes(net_data.received).to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "    RX Rate:   ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}/s", format::format_bytes(net_data.rx_rate)),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  TX Total:  ",
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format::format_bytes(net_data.sent).to_string(),
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "    TX Rate:   ",
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}/s", format::format_bytes(net_data.tx_rate)),
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    let stats_paragraph = Paragraph::new(stats_lines).alignment(Alignment::Left);
    f.render_widget(stats_paragraph, chunks[0]);

    // Render Connections
    let header_cells = ["PROTO", "LOCAL", "REMOTE", "STATE", "PID", "NAME"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells)
        .style(Style::default().bg(Color::DarkGray))
        .height(1)
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .network_connections
        .iter()
        .map(|conn| {
            let pid_str = conn
                .pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string());
            let name_str = conn.process_name.clone().unwrap_or_else(|| "-".to_string());
            let cells = vec![
                Cell::from(conn.protocol.clone()),
                Cell::from(conn.local_addr.clone()),
                Cell::from(conn.remote_addr.clone()),
                Cell::from(conn.state.clone()),
                Cell::from(pid_str),
                Cell::from(name_str),
            ];
            Row::new(cells).height(1).bottom_margin(0)
        })
        .collect();

    let widths = [
        Constraint::Length(6),
        Constraint::Length(22),
        Constraint::Length(22),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Min(10),
    ];
    let mut table = Table::new(rows, widths).header(header);

    // Highlight selection
    table = table.highlight_style(
        Style::default()
            .bg(app.theme.primary)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    if !app.network_connections.is_empty() {
        state.select(Some(app.network_scroll));
    }

    f.render_stateful_widget(table, chunks[1], &mut state);
}

/// Create search input widget
pub fn create_search_input(app: &App) -> Paragraph<'_> {
    let cursor = if app.input_mode && app.cursor_visible {
        "█"
    } else {
        ""
    };

    let text = if app.input_mode {
        format!(" SEARCH: {}{}", app.input_buffer, cursor)
    } else {
        " SEARCH: (Press / to search)".to_string()
    };

    let text_color = if app.input_mode {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    Paragraph::new(Line::from(vec![Span::styled(
        text,
        Style::default().fg(text_color).add_modifier(Modifier::BOLD),
    )]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                if app.input_mode {
                    " SEARCH MODE "
                } else {
                    " SEARCH "
                },
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD),
            )),
    )
}

/// Create commands list widget
pub fn create_commands_list(app: &App) -> (List<'_>, ListState) {
    let results = app.commands.get_results();
    let items: Vec<ListItem> = results
        .iter()
        .enumerate()
        .map(|(idx, cmd)| {
            let selected = idx == app.selected_index;

            let (cmd_style, desc_style, separator_style, _bg_color) = if selected {
                (
                    Style::default()
                        .fg(Color::Black)
                        .bg(app.theme.primary)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::Black).bg(app.theme.primary),
                    Style::default().fg(Color::Black).bg(app.theme.primary),
                    app.theme.primary,
                )
            } else {
                (
                    Style::default()
                        .fg(app.theme.primary)
                        .add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::DarkGray),
                    Style::default().fg(Color::DarkGray),
                    Color::Reset,
                )
            };

            let title = if cmd.dangerous {
                format!("    [DANGER] {}", cmd.command)
            } else {
                format!("   ✓  {}", cmd.command)
            };

            let separator_line = "─".repeat(300);

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(title, cmd_style),
                    Span::styled(" ".repeat(300), cmd_style),
                ]),
                Line::from(vec![
                    Span::styled(format!("     {}", cmd.description), desc_style),
                    Span::styled(" ".repeat(300), desc_style),
                ]),
                Line::from(vec![Span::styled(separator_line, separator_style)]),
            ])
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_index));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(app.theme.primary)
                        .add_modifier(Modifier::BOLD),
                )
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    format!(" COMMANDS ({}/{}) ", app.selected_index + 1, results.len()),
                    Style::default()
                        .fg(app.theme.primary)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default());

    (list, state)
}

/// Create command output pane widget
pub fn create_output_pane(app: &App) -> Paragraph<'_> {
    let mut lines: Vec<Line> = Vec::new();

    if app.command_output.is_empty() {
        lines.push(Line::from(Span::styled(
            "No command output yet. Press Enter to run.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for l in app.command_output.iter() {
            lines.push(Line::from(l.as_str()));
        }
    }

    Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(app.theme.primary)
                        .add_modifier(Modifier::BOLD),
                )
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    " OUTPUT ",
                    Style::default()
                        .fg(app.theme.primary)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().fg(Color::White))
}

/// Create event log pane widget
pub fn create_event_log_pane(app: &App) -> Paragraph<'_> {
    let mut lines: Vec<Line> = Vec::new();

    if app.event_log.is_empty() {
        lines.push(Line::from(Span::styled(
            "Waiting for system events...",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        // Show newest events at the bottom, so we iterate normally.
        // We might want to just show the last N events that fit, but Paragraph scrolls/truncates.
        for e in app.event_log.iter() {
            lines.push(Line::from(vec![
                Span::styled(
                    "[EVENT] ",
                    Style::default()
                        .fg(app.theme.warning)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(e.to_string()),
            ]));
        }
    }

    Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(
                    Style::default()
                        .fg(app.theme.primary)
                        .add_modifier(Modifier::BOLD),
                )
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    " EVENT LOG ",
                    Style::default()
                        .fg(app.theme.primary)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().fg(Color::White))
}
/// Create a text-based progress bar
/// Create a Unicode block-based progress bar (more visual)
fn create_unicode_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64) as usize;
    let partial_index = (((percentage / 100.0) * width as f64) - filled as f64) * 8.0;
    let empty = width
        .saturating_sub(filled)
        .saturating_sub(if partial_index > 0.0 { 1 } else { 0 });

    let partial_char = match partial_index as usize {
        0 => "",
        1 => "▏",
        2 => "▎",
        3 => "▍",
        4 => "▌",
        5 => "▋",
        6 => "▊",
        7 => "▉",
        _ => "",
    };

    format!(
        "{}{}{}",
        "█".repeat(filled),
        partial_char,
        "░".repeat(empty)
    )
}

/// Create a simple bar for core usage
fn create_simple_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64) as usize;
    let empty = width.saturating_sub(filled);

    format!("{}{}", "▮".repeat(filled), "▯".repeat(empty))
}

/// Get color based on usage percentage
fn get_usage_color(percent: f64) -> Color {
    match percent {
        p if p < 25.0 => Color::Rgb(0, 255, 100),   // Neon green
        p if p < 50.0 => Color::Rgb(0, 255, 255),   // Cyan glow
        p if p < 75.0 => Color::Rgb(255, 100, 255), // Magenta glow
        _ => Color::Rgb(255, 0, 100),               // Hot pink/red
    }
}
