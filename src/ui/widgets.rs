use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table},
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
                .title(" Storage Status "),
        )
        .style(Style::default().fg(Color::White))
}

/// Create storage table widget
pub fn create_storage_table(app: &App) -> Table<'static> {
    let results = app.storage.get_results();

    let rows: Vec<Row> = if results.is_empty() {
        vec![Row::new(vec![
            Cell::from("No data available - scanning in progress or press 'r' to scan"),
            Cell::from(""),
            Cell::from(""),
        ])]
    } else {
        results
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let style = if idx == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
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
                .title(" Directory Sizes (Largest First) "),
        )
        .header(
            Row::new(vec!["Path", "Size", "Files"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        )
        .column_spacing(1)
}

/// Create CPU usage widget
pub fn create_cpu_widget(app: &App) -> Paragraph<'static> {
    let cpu_data = app.metrics.get_cpu_info();

    let overall_color = get_usage_color(cpu_data.usage as f64);
    let overall_bar = create_unicode_bar(cpu_data.usage as f64, 48);
    
    let mut lines = vec![
        Line::from(vec![
            Span::styled("▶ TOTAL ", Style::default().fg(Color::Rgb(255, 0, 255)).add_modifier(Modifier::BOLD)),
            Span::styled(overall_bar, Style::default().fg(overall_color)),
            Span::styled(
                format!(" {:>5.1}%", cpu_data.usage),
                Style::default().fg(overall_color).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
        ]),
    ];

    // Add per-core information with enhanced visualization
    for (i, core_usage) in cpu_data.per_core.iter().enumerate() {
        let bar = create_unicode_bar(*core_usage as f64, 45);
        let core_color = get_usage_color(*core_usage as f64);
        lines.push(Line::from(vec![
            Span::styled(format!("▸ {:02} ", i), Style::default().fg(Color::Rgb(100, 100, 255))),
            Span::styled(bar, Style::default().fg(core_color)),
            Span::styled(format!(" {:>5.1}%", core_usage), Style::default().fg(core_color)),
        ]));
    }

    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(0, 255, 255)))
            .title(Span::styled(" ⚡ CPU CORES ⚡ ", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)))
    )
}

/// Create memory usage widget
pub fn create_memory_widget(app: &App) -> Paragraph<'static> {
    let mem_data = app.metrics.get_memory_info();

    let usage_percent = (mem_data.used as f64 / mem_data.total as f64) * 100.0;
    let mem_bar = create_unicode_bar(usage_percent, 48);
    let mem_color = get_usage_color(usage_percent);
    
    let swap_percent = if mem_data.swap_total > 0 {
        (mem_data.swap_used as f64 / mem_data.swap_total as f64) * 100.0
    } else {
        0.0
    };
    let swap_bar = create_unicode_bar(swap_percent, 48);
    let swap_color = get_usage_color(swap_percent);

    let lines = vec![
        Line::from(vec![
            Span::styled("▶ RAM  ", Style::default().fg(Color::Rgb(255, 0, 255)).add_modifier(Modifier::BOLD)),
            Span::styled(mem_bar, Style::default().fg(mem_color)),
            Span::styled(format!(" {:.1}%", usage_percent), Style::default().fg(mem_color).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
        ]),
        Line::from(vec![
            Span::styled("       ", Style::default()),
            Span::styled(
                format!("{} / {}",
                    format::format_bytes(mem_data.used),
                    format::format_bytes(mem_data.total)
                ),
                Style::default().fg(Color::Rgb(150, 150, 255)),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("▶ SWAP ", Style::default().fg(Color::Rgb(255, 255, 0)).add_modifier(Modifier::BOLD)),
            Span::styled(swap_bar, Style::default().fg(swap_color)),
            Span::styled(format!(" {:.1}%", swap_percent), Style::default().fg(swap_color)),
        ]),
        Line::from(vec![
            Span::styled("       ", Style::default()),
            Span::styled(
                format!("{} / {}",
                    format::format_bytes(mem_data.swap_used),
                    format::format_bytes(mem_data.swap_total)
                ),
                Style::default().fg(Color::Rgb(200, 200, 100)),
            ),
        ]),
    ];

    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(255, 0, 255)))
            .title(Span::styled(" 💾 MEMORY BANKS 💾 ", Style::default().fg(Color::Rgb(255, 0, 255)).add_modifier(Modifier::BOLD)))
    )
}

/// Create disk usage widget
pub fn create_disk_widget(app: &App) -> Paragraph<'_> {
    let disks = app.metrics.get_disk_info();

    let mut lines = vec![];

    // Make a copy we can iterate through
    let disk_list: Vec<_> = disks.iter().cloned().collect();

    for (idx, disk) in disk_list.iter().enumerate() {
        if idx > 0 {
            lines.push(Line::from(""));
            lines.push(Line::from(""));
        }
        
        let usage_percent = (disk.used as f64 / disk.total as f64) * 100.0;
        let bar = create_unicode_bar(usage_percent, 40);
        let disk_color = get_usage_color(usage_percent);

        lines.push(Line::from(vec![
            Span::styled(
                format!("▶ {:<6}", disk.mount_point.clone()),
                Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD),
            ),
            Span::styled(bar, Style::default().fg(disk_color)),
            Span::styled(format!(" {:>5.1}%", usage_percent), Style::default().fg(disk_color).add_modifier(Modifier::BOLD)),
        ]));

        lines.push(Line::from(vec![
            Span::styled("        ", Style::default()),
            Span::styled(
                format!("{} / {}",
                    format::format_bytes(disk.used),
                    format::format_bytes(disk.total)
                ),
                Style::default().fg(Color::Rgb(150, 150, 200)),
            ),
        ]));
    }

    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(0, 255, 255)))
            .title(Span::styled(" 💿 STORAGE DRIVES 💿 ", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)))
    )
}

/// Create network usage widget
pub fn create_network_widget(app: &mut App) -> Paragraph<'_> {
    let net_data = app.metrics.get_network_info();

    let mut lines = vec![
        Line::from(vec![
            Span::styled("▼ RX  ", Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:>12}", format::format_bytes(net_data.received)),
                Style::default().fg(Color::Rgb(200, 255, 200)),
            ),
            Span::styled(
                format!("  {:>10}/s", format::format_bytes(net_data.rx_rate)),
                Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("▲ TX  ", Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:>12}", format::format_bytes(net_data.sent)),
                Style::default().fg(Color::Rgb(200, 255, 255)),
            ),
            Span::styled(
                format!("  {:>10}/s", format::format_bytes(net_data.tx_rate)),
                Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    // Add temperature if available
    if let Some(temp) = app.metrics.get_temperature() {
        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("◈ TEMP ", Style::default().fg(Color::Rgb(255, 100, 0)).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("{:.1}°C", temp),
                Style::default().fg(get_temp_color(temp as f64)).add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ),
        ]));
    }

    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(0, 255, 0)))
            .title(Span::styled(" 🌐 NETWORK I/O 🌐 ", Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD)))
    )
}

/// Create search input widget
pub fn create_search_input(app: &App) -> Paragraph<'_> {
    let text = if app.input_mode {
        format!("SEARCH: {}_", app.input_buffer)
    } else {
        "SEARCH: (Press / to search)".to_string()
    };
    
    let text_color = if app.input_mode {
        Color::Yellow
    } else {
        Color::Gray
    };

    Paragraph::new(Line::from(vec![
        Span::styled(text, Style::default().fg(text_color).add_modifier(Modifier::BOLD))
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                if app.input_mode { " SEARCH MODE " } else { " SEARCH " },
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
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

            let (cmd_style, desc_style) = if selected {
                (
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan),
                )
            } else {
                (
                    Style::default().fg(Color::Cyan),
                    Style::default().fg(Color::Gray),
                )
            };

            let title = if cmd.dangerous {
                format!("[DANGER] {}", cmd.command)
            } else {
                cmd.command.clone()
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(title, cmd_style),
                ]),
                Line::from(vec![
                    Span::styled(
                        format!("  {}", cmd.description),
                        desc_style,
                    ),
                ]),
            ])
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.selected_index));
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(
                    format!(" COMMANDS ({}/{}) ", app.selected_index + 1, results.len()),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                ))
        );

    (list, state)
}

/// Settings widget
#[allow(dead_code)]
pub fn create_settings_widget() -> Paragraph<'static> {
    let lines = vec![
        Line::from(""),
        Line::from("Settings - System Monitoring Configuration"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Refresh Rate: ", Style::default().fg(Color::Yellow)),
            Span::raw("500ms"),
        ]),
        Line::from(vec![
            Span::styled("Storage Scan Threshold: ", Style::default().fg(Color::Yellow)),
            Span::raw("1 MB"),
        ]),
        Line::from(vec![
            Span::styled("Display Format: ", Style::default().fg(Color::Yellow)),
            Span::raw("Binary (1024 bytes)"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Keyboard Shortcuts:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  Left/Right (or j/k) - Switch tabs"),
        Line::from("  Up/Down (or i/o)    - Navigate lists"),
        Line::from("  /                   - Search commands"),
        Line::from("  r                   - Rescan storage"),
        Line::from("  q                   - Quit"),
    ];

    Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title_alignment(Alignment::Center)
            .title(" Settings ")
    )
}

/// Create a text-based progress bar
#[allow(dead_code)]
fn create_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64) as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
}

/// Create a Unicode block-based progress bar (more visual)
fn create_unicode_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64) as usize;
    let partial_index = (((percentage / 100.0) * width as f64) - filled as f64) * 8.0;
    let empty = width.saturating_sub(filled).saturating_sub(if partial_index > 0.0 { 1 } else { 0 });
    
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
    
    format!("{}{}{}",
        "█".repeat(filled),
        partial_char,
        "░".repeat(empty)
    )
}

/// Get color based on usage percentage
fn get_usage_color(percent: f64) -> Color {
    match percent {
        p if p < 25.0 => Color::Rgb(0, 255, 100),      // Neon green
        p if p < 50.0 => Color::Rgb(0, 255, 255),      // Cyan glow
        p if p < 75.0 => Color::Rgb(255, 100, 255),    // Magenta glow
        _ => Color::Rgb(255, 0, 100),                   // Hot pink/red
    }
}

/// Get color based on temperature
fn get_temp_color(temp: f64) -> Color {
    match temp {
        t if t < 40.0 => Color::Rgb(0, 255, 200),      // Cool cyan
        t if t < 60.0 => Color::Rgb(255, 255, 0),      // Yellow
        t if t < 80.0 => Color::Rgb(255, 100, 0),      // Orange glow
        _ => Color::Rgb(255, 0, 0),                     // Danger red
    }
}
