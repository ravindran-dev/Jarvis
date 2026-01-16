use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::error;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

mod app;
mod config;
mod async_loop;
mod commands;
mod plugins;
mod system;
mod theme;
mod ui;
mod utils;

use app::App;
use plugins::loader::PluginLoader;
use theme::Theme;

/// Initialize the terminal for TUI rendering
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore terminal to normal mode
fn restore_terminal(mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Error)
        .init();

    let _theme = Theme::dark();

    let mut plugin_loader = PluginLoader::default();
    plugin_loader.validate_plugins_dir()?;
    let _ = plugin_loader.discover();

    let mut terminal = setup_terminal()?;

    let mut app = App::new()?;

    let result = app.run(&mut terminal);

    if let Err(e) = restore_terminal(terminal) {
        error!("Failed to restore terminal: {}", e);
    }

    if let Err(e) = result {
        error!("Application error: {}", e);
        return Err(e);
    }

    Ok(())
}
